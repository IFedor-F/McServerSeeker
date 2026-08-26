use crate::scan_jobs::WorkerManagerService;
use crate::scan_jobs::worker_manager::WorkerManagerError;
use axum::Json;
use chrono::Utc;
use data_core::manager_api::{
    ManagerJobReq, RescanRequest, RescanTarget, Schedule, ScheduleData, ScheduleJobData,
    WorkerJobReq,
};
use reqwest::StatusCode;
use serde::Serialize;
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

#[derive(thiserror::Error, Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleManagerError {
    #[error("can't find task with this name: {0}")]
    ScheduleNameNotFound(String),
    #[error(transparent)]
    #[serde(untagged)]
    WorkerManagerError(#[from] WorkerManagerError),
}
impl axum::response::IntoResponse for ScheduleManagerError {
    fn into_response(self) -> axum::response::Response {
        let payload = json!({
            "message": self.to_string(),
            "error_data": self,
        });
        let status_code = match self {
            ScheduleManagerError::ScheduleNameNotFound(_) => StatusCode::NOT_FOUND,
            ScheduleManagerError::WorkerManagerError(e) => e.into_response().status(),
        };
        (status_code, Json(payload)).into_response()
    }
}

struct ScheduleJob {
    info: ScheduleData,
    handle: Option<JoinHandle<()>>,
}
impl ScheduleJob {
    fn new(info: ScheduleData) -> Self {
        Self { info, handle: None }
    }
    fn stop_if_running(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
        self.handle = None;
    }
    fn set_handle(&mut self, handle: JoinHandle<()>) {
        self.stop_if_running();
        self.handle = Some(handle)
    }
}
impl Drop for ScheduleJob {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}
pub struct ScheduleService {
    manager: Arc<WorkerManagerService>,
    schedules: Mutex<HashMap<String, ScheduleJob>>,
    db_pool: PgPool,
}

impl ScheduleService {
    pub fn new(db_pool: PgPool, manager: Arc<WorkerManagerService>) -> Self {
        Self {
            manager,
            schedules: Mutex::new(HashMap::new()),
            db_pool,
        }
    }
    pub async fn get_schedule(&self, name: String) -> Result<ScheduleData, ScheduleManagerError> {
        let schedule = self.schedules.lock().await;
        schedule
            .get(&name)
            .ok_or(ScheduleManagerError::ScheduleNameNotFound(name))
            .map(|v| v.info.clone())
    }
    pub async fn get_all_schedules(&self) -> Vec<ScheduleData> {
        let schedule = self.schedules.lock().await;
        schedule.values().map(|v| v.info.clone()).collect()
    }
    pub async fn upsert_schedule(
        &self,
        schedule_data: ScheduleData,
    ) -> Result<(), ScheduleManagerError> {
        self.manager.validate_executor(&schedule_data.executor)?;
        let job_name = schedule_data.name.clone();
        _ = self.remove_schedule(&job_name).await;
        log::info!("added new schedule {job_name}");
        self.schedules
            .lock()
            .await
            .insert(job_name, ScheduleJob::new(schedule_data));
        Ok(())
    }
    pub async fn run_schedule(&self, name: String) -> Result<(), ScheduleManagerError> {
        self.stop_schedule(&name).await?;
        let mut schedules = self.schedules.lock().await;
        let schedule_job = schedules
            .get_mut(&name)
            .ok_or(ScheduleManagerError::ScheduleNameNotFound(name))?;

        let info = schedule_job.info.clone();
        let job_request = match info.job_data {
            ScheduleJobData::Discover(data) => WorkerJobReq::Discover(data),
            ScheduleJobData::RescanDb { method, rate } => {
                let rows = match sqlx::query!("SELECT ip, port, last_used_nick FROM data.servers")
                    .fetch_all(&self.db_pool)
                    .await
                {
                    Ok(rows) => rows,
                    Err(e) => {
                        log::error!("database error while trying to rescan database: {e}");
                        return Ok(());
                    }
                };
                let targets = rows
                    .into_iter()
                    .map(|r| RescanTarget {
                        ip: r.ip.ip(),
                        port: r.port as u16,
                        player_name: r.last_used_nick,
                    })
                    .collect();
                WorkerJobReq::Rescan(RescanRequest {
                    method,
                    rate,
                    targets,
                })
            }
        };
        let manager_job_req = ManagerJobReq {
            name: info.name.clone(),
            executor: info.executor.clone(),
            job_request,
        };

        let handle = match info.schedule.clone() {
            Schedule::Always => {
                let manager = self.manager.clone();
                tokio::spawn(async move {
                    loop {
                        log::info!("[{}] starting job", info.name);
                        match manager.run_job(manager_job_req.clone()).await {
                            Ok(job_id) => {
                                while manager.is_job_active(job_id).await {
                                    tokio::time::sleep(Duration::from_secs(5)).await;
                                }
                            }
                            Err(e) => {
                                log::error!("[{}] failed to start job: {}", info.name, e);
                                tokio::time::sleep(Duration::from_secs(5)).await; // Prevent spamming errors
                            }
                        }
                        tokio::time::sleep(Duration::from_secs(info.wait_secs)).await;
                    }
                })
            }

            Schedule::Cron(cron_schedule) => {
                let manager = self.manager.clone();

                tokio::spawn(async move {
                    loop {
                        let now = Utc::now();
                        if let Some(next_run) = cron_schedule.upcoming(Utc).next() {
                            let delay = (next_run - now).to_std().unwrap_or(Duration::ZERO);
                            log::info!("[{}] next job will be started at {}", info.name, next_run,);
                            tokio::time::sleep(delay).await;

                            log::info!("[{}] running job", info.name);
                            if let Err(e) = manager.run_job(manager_job_req.clone()).await {
                                log::error!("[{}] failed to spawn cron job: {}", info.name, e);
                            }
                        } else {
                            break;
                        }
                    }
                })
            }
        };
        schedule_job.set_handle(handle);
        Ok(())
    }
    pub async fn remove_schedule(&self, job_name: &str) -> Result<(), ScheduleManagerError> {
        match self.schedules.lock().await.remove(job_name) {
            None => Err(ScheduleManagerError::ScheduleNameNotFound(
                job_name.to_string(),
            )),
            Some(_) => {
                log::info!("removed schedule {job_name}");
                Ok(())
            }
        }
    }
    pub async fn stop_schedule(&self, job_name: &str) -> Result<(), ScheduleManagerError> {
        if let Some(job) = self.schedules.lock().await.get_mut(job_name) {
            log::info!("stop schedule {job_name}");
            job.stop_if_running();
            Ok(())
        } else {
            Err(ScheduleManagerError::ScheduleNameNotFound(
                job_name.to_string(),
            ))
        }
    }
}
