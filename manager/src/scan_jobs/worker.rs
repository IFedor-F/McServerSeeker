use crate::database::ParsedForSqlServerData;
use data_core::api::manager::{
    DiscoverJobProgress, DiscoverRequest, JobProgress, McServerData, ScanMethod,
    ScanSelectedJobProgress, ScanSelectedRequest, WorkerInfo,
};
use data_core::proto::scanner as pb;
use data_core::proto::scanner::worker_service_client::WorkerServiceClient;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tonic::transport::Endpoint;

#[derive(thiserror::Error, Debug)]
pub enum WorkerError {
    #[error(transparent)]
    ChannelError(#[from] tonic::transport::Error),

    #[error(transparent)]
    InvalidStatusGet(#[from] tonic::Status),

    #[error(transparent)]
    InvalidData(#[from] data_core::api::manager::ParseServerDataError),

    #[error(transparent)]
    CantSendJobProgress(#[from] mpsc::error::SendError<WorkerJobProgressUpdate>),

    #[error(transparent)]
    CantSendDatabaseReq(#[from] mpsc::error::SendError<ParsedForSqlServerData>),
}

pub enum WorkerJobReq {
    Discover(DiscoverRequest),
    ScanSelected(ScanSelectedRequest),
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
    pub async fn execute_one_server(
        &self,
        target: String,
        port: Option<u16>,
        method: ScanMethod,
    ) -> Result<Option<McServerData>, WorkerError> {
        let _load_guard = self.load.add_load(1f64);
        let pb_req = tonic::Request::new(pb::ScanOneRequest {
            target,
            port: port.map(|v| v as u32),
            scan_method: method.into(),
        });
        let mut client = WorkerServiceClient::new(self.endpoint.connect().await?);
        let result = client.scan_one(pb_req).await?.into_inner();
        match result.server_data {
            None => Ok(None),
            Some(data) => Ok(Some(data.try_into()?)),
        }
    }

    pub async fn execute_job(
        &self,
        id: usize,
        req: WorkerJobReq,
        tx_stats: mpsc::Sender<WorkerJobProgressUpdate>,
        tx_db_queue: mpsc::Sender<ParsedForSqlServerData>,
        cancellation_token: CancellationToken,
    ) -> Result<(), WorkerError> {
        match req {
            WorkerJobReq::Discover(req) => {
                self.execute_discover(id, req, tx_stats, tx_db_queue, cancellation_token)
                    .await
            }
            WorkerJobReq::ScanSelected(req) => {
                self.execute_scan_selected(id, req, tx_stats, tx_db_queue, cancellation_token)
                    .await
            }
        }
    }

    pub async fn execute_discover(
        &self,
        id: usize,
        req: DiscoverRequest,
        tx_stats: mpsc::Sender<WorkerJobProgressUpdate>,
        tx_db_queue: mpsc::Sender<ParsedForSqlServerData>,
        cancellation_token: CancellationToken,
    ) -> Result<(), WorkerError> {
        let _load_guard = self.load.add_load(req.rate as f64);
        let mut client = WorkerServiceClient::new(self.endpoint.connect().await?);

        let req: pb::DiscoverRequest = req.into();
        let mut stream = client.discover(req).await?.into_inner();
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
                        let result = client.cancel(tonic::Request::new(pb::JobCancel {job_id: id})).await;
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

    pub async fn execute_scan_selected(
        &self,
        id: usize,
        req: ScanSelectedRequest,
        tx_stats: mpsc::Sender<WorkerJobProgressUpdate>,
        tx_db_queue: mpsc::Sender<ParsedForSqlServerData>,
        cancellation_token: CancellationToken,
    ) -> Result<(), WorkerError> {
        let _load_guard = self.load.add_load(req.rate as f64);
        let mut client = WorkerServiceClient::new(self.endpoint.connect().await?);

        let mut work_id: Option<u32> = None;
        let all_count = req.targets.len() as u32;
        let mut successful_count = 0;

        let req: pb::ScanSelectedRequest = req.into();
        let mut stream = client.scan_selected(req).await?.into_inner();
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
                            let progress = JobProgress::ScanSelected(ScanSelectedJobProgress {
                                all: all_count,
                                checked: data.checked,
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
                        let result = client.cancel(tonic::Request::new(pb::JobCancel {job_id: id})).await;
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
}
