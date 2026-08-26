use crate::database::ParsedForSqlServerData;
use data_core::manager_api::{
    DiscoverJobProgress, DiscoverRequest, JobProgress, RescanJobProgress, RescanRequest,
    WorkerInfo, WorkerJobReq,
};
use data_core::proto::scanner::JobCancel;
use data_core::proto::scanner::worker_service_client::WorkerServiceClient;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tonic::transport::{Channel, Endpoint};

#[derive(thiserror::Error, Debug)]
pub enum WorkerError {
    #[error(transparent)]
    ChannelError(#[from] tonic::transport::Error),

    #[error(transparent)]
    InvalidStatusGet(#[from] tonic::Status),

    #[error(transparent)]
    CantSendJobProgress(#[from] mpsc::error::SendError<WorkerJobProgressUpdate>),

    #[error(transparent)]
    CantSendDatabaseReq(#[from] mpsc::error::SendError<ParsedForSqlServerData>),
}

pub struct WorkerJobProgressUpdate {
    pub id: usize,
    pub data: JobProgress,
}
impl WorkerJobProgressUpdate {
    fn new(id: usize, data: JobProgress) -> Self {
        Self { id, data }
    }
}

#[derive(Debug)]
pub struct Worker {
    pub worker_info: WorkerInfo,
    pub endpoint: Endpoint,
    pub load: WorkerLoad,
}

#[derive(Debug)]
pub struct WorkerLoad {
    current_load: Arc<std::sync::RwLock<f64>>,
}

impl WorkerLoad {
    pub fn new() -> Self {
        Self {
            current_load: Arc::new(std::sync::RwLock::new(0f64)),
        }
    }

    pub fn get_load(&self) -> f64 {
        self.current_load.read().unwrap().clone()
    }

    #[must_use]
    fn add_load(&self, load: f64) -> LoadGuard<'_> {
        *self.current_load.write().unwrap() += load;
        LoadGuard {
            worker_load: self,
            keeping_load: load,
        }
    }

    fn remove_load(&self, rate: f64) {
        let mut current_load = self.current_load.write().unwrap();
        *current_load = (*current_load - rate).max(0f64)
    }
}

struct LoadGuard<'a> {
    worker_load: &'a WorkerLoad,
    keeping_load: f64,
}

impl<'a> Drop for LoadGuard<'a> {
    fn drop(&mut self) {
        self.worker_load.remove_load(self.keeping_load);
    }
}

impl Worker {
    pub fn new(worker_info: WorkerInfo, endpoint: Endpoint) -> Self {
        Self {
            worker_info,
            endpoint,
            load: WorkerLoad::new(),
        }
    }
    pub async fn execute(
        &self,
        id: usize,
        req: WorkerJobReq,
        tx_stats: mpsc::Sender<WorkerJobProgressUpdate>,
        tx_db_queue: mpsc::Sender<ParsedForSqlServerData>,
        cancellation_token: CancellationToken,
    ) -> Result<(), WorkerError> {
        let _load_guard = self.load.add_load(calc_load(&req));
        let client = WorkerServiceClient::new(self.endpoint.connect().await?);
        match req {
            WorkerJobReq::Discover(req) => {
                handle_discover(id, req, client, tx_stats, tx_db_queue, cancellation_token).await
            }
            WorkerJobReq::Rescan(req) => {
                handle_rescan(id, req, client, tx_stats, tx_db_queue, cancellation_token).await
            }
        }
    }
}
fn calc_load(req: &WorkerJobReq) -> f64 {
    match req {
        WorkerJobReq::Discover(v) => v.rate as f64,
        WorkerJobReq::Rescan(v) => v.rate as f64,
    }
}

async fn handle_discover(
    id: usize,
    req: DiscoverRequest,
    mut client: WorkerServiceClient<Channel>,
    tx_stats: mpsc::Sender<WorkerJobProgressUpdate>,
    tx_db_queue: mpsc::Sender<ParsedForSqlServerData>,
    cancellation_token: CancellationToken,
) -> Result<(), WorkerError> {
    let discover_req = tonic::Request::new(req.into());
    let mut stream = client.discover(discover_req).await?.into_inner();
    let mut work_id: Option<u32> = None;

    loop {
        tokio::select! {
            msg = stream.message() =>  {
                match msg {
                    Ok(Some(data)) => {
                        if let Some(stats) = data.stats {
                            work_id = Some(data.job_id);
                            let progress = JobProgress::Discover(DiscoverJobProgress {
                                scanned_progress: stats.scanned_progress,
                                founded: stats.founded,
                                parsing_now: stats.parsing_now,
                                successful: stats.successful,
                            });
                            tx_stats.send(WorkerJobProgressUpdate::new(id, progress)).await?
                        }
                        if let Some(data) = data.server_data {
                            let parse_result = ParsedForSqlServerData::try_parse(data);
                            match parse_result {
                                Ok(parsed) => {
                                    tx_db_queue.send(parsed).await?
                                }
                                Err(e) => {
                                    log::error!("error occurred while parsing data from worker: {e}")
                                }
                            }
                        }
                    }
                    Ok(None) => {
                        break
                    }
                    Err(e) => {
                        log::error!("{e}");
                    }
                }
            }
            _ = cancellation_token.cancelled() => {
                if let Some(id)  = work_id {
                    let result = client.cancel(tonic::Request::new(JobCancel {job_id: id})).await;
                    if let Err(e) = result {
                        log::error!("failed to send cancel job request to worker: {e}")
                    }
                }
                break;
            }
        }
    }
    Ok(())
}

async fn handle_rescan(
    id: usize,
    req: RescanRequest,
    mut client: WorkerServiceClient<Channel>,
    tx_stats: mpsc::Sender<WorkerJobProgressUpdate>,
    tx_db_queue: mpsc::Sender<ParsedForSqlServerData>,
    cancellation_token: CancellationToken,
) -> Result<(), WorkerError> {
    let mut work_id: Option<u32> = None;
    let all_count = req.targets.len();
    let mut successful_count = 0;
    let rescan_req = tonic::Request::new(req.into());
    let mut stream = client.rescan(rescan_req).await?.into_inner();
    loop {
        tokio::select! {
            msg = stream.message() =>  {
                match msg {
                    Ok(Some(data)) => {
                        work_id = Some(data.job_id);
                        if let Some(data) = data.server_data {
                            successful_count += 1;
                            let parse_result = ParsedForSqlServerData::try_parse(data);
                            match parse_result {
                                Ok(parsed) => {
                                    tx_db_queue.send(parsed).await?
                                }
                                Err(e) => {
                                    log::error!("error occurred while parsing data from worker: {e}")
                                }
                            }
                        }
                        let progress = JobProgress::Rescan(RescanJobProgress {
                            all: all_count,
                            checked: data.checked as usize,
                            successful: successful_count,
                        });
                        tx_stats.send(WorkerJobProgressUpdate::new(id, progress)).await?
                    }
                    Ok(None) => {
                        break
                    }
                    Err(e) => {
                        log::error!("{e}");
                    }
                }
            }
            _ = cancellation_token.cancelled() => {
                if let Some(id) = work_id {
                    let result = client.cancel(tonic::Request::new(JobCancel {job_id: id})).await;
                    if let Err(e) = result {
                        log::error!("failed to send cancel job request to worker: {e}")
                    }
                }
                break;
            }
        }
    }
    Ok(())
}
