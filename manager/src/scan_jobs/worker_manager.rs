use super::{ManagerTask, Worker};
use crate::database::{DbQueueWorker, ParsedForSqlServerData};
use crate::scan_jobs::balancing::Balancer;
use crate::scan_jobs::worker::WorkerError;
use axum::Json;
use axum::http::StatusCode;
use data_core::api::manager::{
    JobExecutor, JobId, JobProgress, ManagerJobInfo, ManagerJobReq, WorkerStatus,
};
use futures::FutureExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{RwLock, mpsc};

#[derive(thiserror::Error, Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "type", content = "detail")]
#[serde(rename_all = "snake_case")]
pub enum WorkerManagerError {
    #[error("worker '{0}' was not found in the manager")]
    WorkerNotFound(String),
}

impl axum::response::IntoResponse for WorkerManagerError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            WorkerManagerError::WorkerNotFound(_) => StatusCode::NOT_FOUND,
        };
        let payload = json!({
            "message": self.to_string(),
            "error_data": self,
        });
        (status, Json(payload)).into_response()
    }
}

#[derive(Debug)]
pub struct WorkerManagerService {
    next_job_id: AtomicU64,
    tasks: Arc<RwLock<HashMap<JobId, ManagerTask>>>,
    workers: HashMap<String, Arc<Worker>>,
    db_queue: mpsc::Sender<ParsedForSqlServerData>,
}

// helpers and init
impl WorkerManagerService {
    pub fn new(db_queue_worker: DbQueueWorker) -> Self {
        let (tx, rx) = mpsc::channel(256);
        tokio::spawn(async move {
            db_queue_worker.run(rx).await;
        });
        Self {
            next_job_id: AtomicU64::new(1),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            workers: HashMap::new(),
            db_queue: tx,
        }
    }
    pub fn add_worker(&mut self, worker: Worker) {
        self.workers
            .insert(worker.worker_info.name.clone(), Arc::new(worker));
    }

    async fn insert_job(&self, job_task: ManagerTask) -> JobId {
        let job_id = JobId(self.next_job_id.fetch_add(1, Ordering::SeqCst));
        self.tasks.write().await.insert(job_id, job_task);
        job_id
    }
    fn find_worker_by_name(&self, name: String) -> Result<Arc<Worker>, WorkerManagerError> {
        self.workers
            .get(&name)
            .cloned()
            .ok_or(WorkerManagerError::WorkerNotFound(name))
    }
    fn get_specified_workers(
        &self,
        worker_names: HashSet<String>,
    ) -> Result<Vec<Arc<Worker>>, WorkerManagerError> {
        let founded_workers: Vec<_> = self
            .workers
            .iter()
            .filter(|(name, _)| worker_names.contains(*name))
            .collect();

        if worker_names.len() != founded_workers.len() {
            let missing_worker = worker_names
                .into_iter()
                .find(|name| !founded_workers.iter().any(|(s, _)| **s == *name))
                .unwrap(); // expect at least one missing worker because lengths are different

            return Err(WorkerManagerError::WorkerNotFound(missing_worker));
        }
        Ok(founded_workers
            .into_iter()
            .map(|(_, w)| w.clone())
            .collect())
    }
}

// API
impl WorkerManagerService {
    pub fn validate_executor(&self, executor: &JobExecutor) -> Result<(), WorkerManagerError> {
        match executor {
            JobExecutor::Worker { name } => {
                if self.workers.contains_key(name) {
                    Ok(())
                } else {
                    Err(WorkerManagerError::WorkerNotFound(name.to_string()))
                }
            }
            JobExecutor::LeastLoadedSpecified { worker_names }
            | JobExecutor::BalanceSpecified { worker_names } => {
                for name in worker_names {
                    if !self.workers.contains_key(name) {
                        return Err(WorkerManagerError::WorkerNotFound(name.to_string()));
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    pub async fn run_job(&self, job_req: ManagerJobReq) -> Result<JobId, WorkerManagerError> {
        let job_task = ManagerTask::new(job_req);

        let future = match job_task.req.executor.clone() {
            JobExecutor::Worker { name } => {
                let worker = self.find_worker_by_name(name)?;
                execute_one_worker(job_task.clone(), worker, self.db_queue.clone()).boxed()
            }
            JobExecutor::LeastLoadedSpecified { worker_names } => {
                let founded_workers = self.get_specified_workers(worker_names)?;
                let worker = select_less_loaded_worker(founded_workers);
                execute_one_worker(job_task.clone(), worker, self.db_queue.clone()).boxed()
            }
            JobExecutor::LeastLoadedAll => {
                let worker = select_less_loaded_worker(self.workers.clone().into_values());
                execute_one_worker(job_task.clone(), worker, self.db_queue.clone()).boxed()
            }
            JobExecutor::BalanceSpecified { worker_names } => {
                let founded_workers = self.get_specified_workers(worker_names)?;
                let balancer = Balancer::new(founded_workers);
                balancer
                    .run_work(job_task.clone(), self.db_queue.clone())
                    .boxed()
            }
            JobExecutor::BalanceAll => {
                let balancer = Balancer::new(self.workers.clone().into_values().collect());
                balancer
                    .run_work(job_task.clone(), self.db_queue.clone())
                    .boxed()
            }
        };
        let job_id = self.insert_job(job_task).await;
        let shared_jobs = self.tasks.clone();

        let job_id_cloned = job_id.clone();
        tokio::spawn(async move {
            match future.await {
                Ok(_) => {}
                Err(e) => {
                    log::error!("error while running job: {e}")
                }
            };
            shared_jobs.write().await.remove(&job_id_cloned);
        });

        Ok(job_id)
    }
    pub async fn cancel_job(&self, job_id: JobId) {
        if let Some(task) = self.tasks.read().await.get(&job_id) {
            log::info!("cancel job with id {}", job_id.0);
            task.cancellation_token.cancel()
        }
    }

    pub async fn get_job_progress(&self, job_id: JobId) -> Option<JobProgress> {
        if let Some(task) = self.tasks.read().await.get(&job_id) {
            let progress = task.progress.read().await;
            Some(progress.clone())
        } else {
            None
        }
    }
    pub async fn get_job_info(&self, job_id: JobId) -> Option<ManagerJobInfo> {
        let tasks = self.tasks.read().await;
        let task = tasks.get(&job_id);
        if let Some(task) = task {
            Some(ManagerJobInfo {
                id: job_id,
                req: task.req.clone(),
                progress: task.progress.read().await.clone(),
            })
        } else {
            None
        }
    }
    pub async fn get_all_jobs_info(&self) -> Vec<ManagerJobInfo> {
        let tasks = self.tasks.read().await;
        let mut results = Vec::with_capacity(tasks.len());

        for (&id, task) in tasks.iter() {
            let progress = task.progress.read().await.clone();
            results.push(ManagerJobInfo {
                id,
                req: task.req.clone(),
                progress,
            });
        }
        results
    }

    pub async fn is_job_active(&self, job_id: JobId) -> bool {
        self.tasks.read().await.get(&job_id).is_some()
    }
    pub fn get_workers(&self) -> Vec<WorkerStatus> {
        self.workers
            .iter()
            .map(|(_, worker)| WorkerStatus {
                info: worker.worker_info.clone(),
                loading: worker.load.get_load(),
            })
            .collect()
    }
}

fn select_less_loaded_worker<I>(workers: I) -> Arc<Worker>
where
    I: IntoIterator<Item = Arc<Worker>>,
{
    workers
        .into_iter()
        .min_by(|w1, w2| w1.load.get_load().total_cmp(&w2.load.get_load()))
        .expect("expect at least one worker")
}

async fn execute_one_worker(
    job_task: ManagerTask,
    worker: Arc<Worker>,
    db_queue: mpsc::Sender<ParsedForSqlServerData>,
) -> Result<(), WorkerError> {
    let cancel_token = job_task.cancellation_token;
    let progress = job_task.progress;
    let (tx_stats, mut rx_stats) = mpsc::channel(16);
    let fut = worker.execute(
        0,
        job_task.req.job_request,
        tx_stats,
        db_queue,
        cancel_token,
    );
    tokio::pin!(fut);

    loop {
        tokio::select! {
            res = &mut fut => {
                return match res {
                    Ok(_) => {
                        Ok(())
                    }
                    Err(e) => {
                        Err(e)
                    }
                }
            }
            Some(new_progress) = rx_stats.recv() => {
                let mut current_progress = progress.write().await;
                *current_progress = new_progress.data;
            }
        }
    }
}
