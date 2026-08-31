use crate::scan_jobs::WorkerManagerService;
use crate::scan_jobs::worker_manager::WorkerManagerError;
use axum::Json;
use chrono::Utc;
use data_core::api::manager::{ManagerJobReq, Schedule, ScheduleData};
use reqwest::StatusCode;
use serde::Serialize;
use serde_json::json;
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
    #[error("task with name '{0}' already exists")]
    DuplicateScheduleName(String),
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
            ScheduleManagerError::DuplicateScheduleName(_) => StatusCode::CONFLICT,
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
            log::info!("stopping schedule '{}'", self.info.name);
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
}

impl ScheduleService {
    pub fn new(manager: Arc<WorkerManagerService>) -> Self {
        Self {
            manager,
            schedules: Mutex::new(HashMap::new()),
        }
    }
    pub async fn get_schedule(&self, name: &str) -> Result<ScheduleData, ScheduleManagerError> {
        let schedule = self.schedules.lock().await;
        schedule
            .get(name)
            .ok_or(ScheduleManagerError::ScheduleNameNotFound(name.to_string()))
            .map(|v| v.info.clone())
    }
    pub async fn get_all_schedules(&self) -> Vec<ScheduleData> {
        let schedule = self.schedules.lock().await;
        schedule.values().map(|v| v.info.clone()).collect()
    }
    pub async fn add_schedule(
        &self,
        schedule_data: ScheduleData,
    ) -> Result<(), ScheduleManagerError> {
        let job_name = schedule_data.name.clone();
        if self.schedules.lock().await.contains_key(&job_name) {
            return Err(ScheduleManagerError::DuplicateScheduleName(job_name));
        }
        self.manager.validate_executor(&schedule_data.executor)?;
        log::info!("added new schedule '{job_name}'");
        self.schedules
            .lock()
            .await
            .insert(job_name, ScheduleJob::new(schedule_data));
        Ok(())
    }
    pub async fn run_schedule(&self, name: &str) -> Result<(), ScheduleManagerError> {
        self.stop_schedule(name).await?;
        let mut schedules = self.schedules.lock().await;
        let schedule_job = schedules
            .get_mut(name)
            .ok_or(ScheduleManagerError::ScheduleNameNotFound(name.to_string()))?;

        let info = schedule_job.info.clone();
        let manager_job_req = ManagerJobReq {
            name: info.name.clone(),
            executor: info.executor.clone(),
            task: info.task,
        };

        let handle = match info.schedule.clone() {
            Schedule::Always => {
                let manager = self.manager.clone();
                tokio::spawn(async move {
                    loop {
                        log::info!("starting job '{}'", info.name);
                        match manager.run_job(manager_job_req.clone()).await {
                            Ok(job_id) => {
                                while manager.is_job_active(job_id).await {
                                    tokio::time::sleep(Duration::from_secs(5)).await;
                                }
                            }
                            Err(e) => {
                                log::error!("failed to start job '{}': {}", info.name, e);
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
                            log::info!("job '{}' will be started at {}", info.name, next_run,);
                            tokio::time::sleep(delay).await;

                            log::info!("running job '{}'", info.name);
                            if let Err(e) = manager.run_job(manager_job_req.clone()).await {
                                log::error!("failed to spawn cron job '{}': {}", info.name, e);
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
            job.stop_if_running();
            Ok(())
        } else {
            Err(ScheduleManagerError::ScheduleNameNotFound(
                job_name.to_string(),
            ))
        }
    }
}
