mod balancing;
pub mod schedule;
pub mod worker;
pub mod worker_manager;

pub use worker::Worker;

use data_core::manager_api::{JobProgress, ManagerJobReq};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
pub use worker_manager::WorkerManagerService;

#[derive(Debug, Clone)]
struct ManagerTask {
    req: ManagerJobReq,
    progress: Arc<RwLock<JobProgress>>,
    cancellation_token: CancellationToken,
}
impl ManagerTask {
    fn new(req: ManagerJobReq) -> Self {
        Self {
            req,
            progress: Arc::new(RwLock::new(JobProgress::NoData)),
            cancellation_token: CancellationToken::new(),
        }
    }
}
