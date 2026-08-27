mod analyze;
mod mc_proto_server_data;
mod scan;

use crate::analyze::ExtraServerData;
use crate::mc_proto_server_data::gen_protobuf_server_data;
use crate::scan::MasscanBuilder;
use data_core::proto::scanner as pb;
use data_core::proto::scanner::{McServerData, ScanOneRequest, ScanOneResult};
use hickory_resolver::TokioResolver;
use mc_protocol::dialog::ConnectionMethod;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;
use tonic::transport::{Certificate, Identity, ServerTlsConfig};
use tonic::{Request, Response, Status};

#[derive(Debug)]
struct WorkerJobRecord {
    job_type: pb::WorkerJobType,
    cancel_token: CancellationToken,
}
#[must_use]
#[derive(Debug)]
enum WorkerJobCancelResult {
    Canceled,
    NotFound,
}
#[derive(Default)]
struct WorkerState {
    next_job_id: u32,
    active_jobs: HashMap<u32, WorkerJobRecord>,
}
impl WorkerState {
    fn insert_job(&mut self, job_record: WorkerJobRecord) -> u32 {
        self.next_job_id += 1;
        self.active_jobs.insert(self.next_job_id, job_record);
        self.next_job_id
    }
    fn cancel_job(&mut self, id: u32) -> WorkerJobCancelResult {
        match self.active_jobs.get(&id) {
            None => WorkerJobCancelResult::NotFound,
            Some(job_record) => {
                job_record.cancel_token.cancel();
                self.active_jobs.remove(&id);
                WorkerJobCancelResult::Canceled
            }
        }
    }
}

struct WorkerService {
    state: Arc<Mutex<WorkerState>>,
    dns_resolver: TokioResolver,
}
impl WorkerService {
    pub fn new(dns_resolver: TokioResolver) -> Self {
        Self {
            state: Arc::new(Mutex::new(WorkerState::default())),
            dns_resolver,
        }
    }
    async fn parse_one_server(
        &self,
        target: String,
        port: Option<u16>,
        conn_method: ConnectionMethod,
    ) -> Option<McServerData> {
        use mc_protocol::dialog::*;
        use mc_protocol::types::Player;
        let player = Player::random_like_offline();
        let serv_dst_log = format!(
            "{target}{}",
            port.map(|v| format!(":{v}")).unwrap_or("".to_string())
        );

        let dst = ServerDst::from_like_mc(target, port, &self.dns_resolver).await;
        let dst = match dst {
            Ok(dst) => dst,
            Err(e) => {
                log::debug!("[{serv_dst_log}] can't get server destination: {e}");
                return None;
            }
        };

        let dialog = ServerDialog::new(dst.clone(), player.clone());
        let conn_settings = ConnectionSettings {
            conn_method,
            ..Default::default()
        };

        let result = dialog.connect(conn_settings).await;
        log::trace!("[{serv_dst_log}] result: {:?}", result);
        match result {
            ConnectionResult::NoHandshake(e) => {
                log::debug!("[{serv_dst_log}] can't handshake: {e}");
                return None;
            }
            _ => {}
        }
        log::debug!("[{serv_dst_log}] parsed");
        let extra_data = ExtraServerData::parse(player, conn_method, &result);
        let data = result.get_data().unwrap(); // has data because we check this is not 'NoHandshake'
        Some(gen_protobuf_server_data(dst, data, extra_data))
    }
}
#[tonic::async_trait]
impl pb::worker_service_server::WorkerService for WorkerService {
    type DiscoverStream = ReceiverStream<Result<pb::DiscoverUpdate, Status>>;
    async fn discover(
        &self,
        request: Request<pb::DiscoverRequest>,
    ) -> Result<Response<Self::DiscoverStream>, Status> {
        log::debug!("discover request: {:?}", request);

        let pb::DiscoverRequest {
            targets,
            excludes,
            ports,
            port_ranges,
            rate,
            method,
        } = request.into_inner();

        let mut masscan_builder = MasscanBuilder::new();

        // ip ranges
        if targets.len() == 0 {
            return Err(Status::invalid_argument("IpPrefix is missing in message"));
        }
        for target in targets {
            masscan_builder = masscan_builder.target(target.try_into()?)
        }
        for exclude in excludes {
            masscan_builder = masscan_builder.exclude(exclude.try_into()?)
        }
        // ports
        for port in ports {
            masscan_builder = masscan_builder.port(port as u16)
        }
        for port_range in port_ranges {
            masscan_builder =
                masscan_builder.port_range(port_range.min as u16, port_range.max as u16)
        }
        // method
        let method = pb::ScanMethod::try_from(method)
            .map_err(|e| Status::invalid_argument(format!("unknown scan method: {}", e.0)))?;
        let method = ConnectionMethod::from(method);

        // rate
        if rate <= 0 {
            return Err(Status::invalid_argument("rate must be more than zero"));
        }
        masscan_builder = masscan_builder.rate(rate);

        let (tx, rx) = mpsc::channel(128);
        let cancel_token = CancellationToken::new();
        let job_id = self.state.lock().await.insert_job(WorkerJobRecord {
            job_type: pb::WorkerJobType::Discover,
            cancel_token: cancel_token.clone(),
        });

        let state_clone = self.state.clone();

        log::info!("[job {job_id}] start discover");
        tokio::spawn(async move {
            let mut scan_rx = scan::scan_inet(job_id, masscan_builder, rate as usize, method);
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        return
                    }
                    msg = scan_rx.recv() => {
                        match msg {
                            Some(upd) => {
                                match tx.send(Ok(upd)).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        log::info!("[job {job_id}] failed to send stream to client: {e}");
                                        break;
                                    }
                                }
                            }
                            None => {
                                break
                            }
                        }
                    }
                }
            }
            let mut state = state_clone.lock().await;
            _ = state.cancel_job(job_id);
            log::info!("[job {job_id}] end discover")
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type ScanSelectedStream = ReceiverStream<Result<pb::ScanSelectedUpdate, Status>>;

    async fn scan_selected(
        &self,
        request: Request<pb::ScanSelectedRequest>,
    ) -> Result<Response<Self::ScanSelectedStream>, Status> {
        log::debug!("rescan request: {:?}", request);
        let pb::ScanSelectedRequest {
            method,
            rate,
            targets,
        } = request.into_inner();
        let method = pb::ScanMethod::try_from(method)
            .map_err(|e| Status::invalid_argument(format!("unknown scan method: {}", e.0)))?;
        let method = ConnectionMethod::from(method);
        if rate <= 0 {
            return Err(Status::invalid_argument("rate must be more than zero"));
        }

        let (tx, rx) = mpsc::channel(128);
        let cancel_token = CancellationToken::new();
        let job_id = self.state.lock().await.insert_job(WorkerJobRecord {
            job_type: pb::WorkerJobType::Discover,
            cancel_token: cancel_token.clone(),
        });
        let state_clone = self.state.clone();

        log::info!("[job {job_id}] started rescan");
        let mut scan_rx = scan::scan_selected(job_id, targets, rate as usize, method).await?;
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        return
                    }
                    msg = scan_rx.recv() => {
                        match msg {
                            Some(upd) => {
                                match tx.send(Ok(upd)).await{
                                    Ok(_) => {}
                                    Err(e) => {
                                        log::info!("[job {job_id}] failed to send stream to client: {e}");
                                        break
                                    }
                                };
                            }
                            None => {
                                break
                            }
                        }
                    }
                }
            }
            let mut state = state_clone.lock().await;
            _ = state.cancel_job(job_id);
            log::info!("[job {job_id}] end rescan")
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn scan_one(
        &self,
        request: Request<ScanOneRequest>,
    ) -> Result<Response<ScanOneResult>, Status> {
        let ScanOneRequest {
            target,
            port,
            scan_method,
        } = request.into_inner();
        log::info!(
            "scan one request: {target}{}",
            port.map(|v| format!(":{v}")).unwrap_or("".to_string())
        );

        let scan_method = pb::ScanMethod::try_from(scan_method)
            .map_err(|e| Status::invalid_argument(format!("invalid enum variant {}", e.0)))?
            .into();
        let server_data = self
            .parse_one_server(target, port.map(|v| v as u16), scan_method)
            .await;
        Ok(Response::new(ScanOneResult { server_data }))
    }

    async fn get_status(&self, _: Request<()>) -> Result<Response<pb::WorkerStatus>, Status> {
        let works = self
            .state
            .lock()
            .await
            .active_jobs
            .iter()
            .map(|(&id, record)| pb::WorkerJob {
                id,
                job_type: record.job_type as i32,
            })
            .collect();
        Ok(Response::new(pb::WorkerStatus { works }))
    }

    async fn cancel(&self, request: Request<pb::JobCancel>) -> Result<Response<()>, Status> {
        let job_id = request.into_inner().job_id;
        log::info!("[job {job_id}] cancel job by client");
        match self.state.lock().await.cancel_job(job_id) {
            WorkerJobCancelResult::Canceled => Ok(Response::new(())),
            WorkerJobCancelResult::NotFound => Err(Status::not_found(format!(
                "job with id {} isn't exist or already canceled",
                job_id
            ))),
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let addr = env::var("BIND_ADDR").expect("env 'BIND_ADDR' is expected to run program");
    let addr: SocketAddr = addr
        .parse()
        .expect("{} is incorrect addr, should be an ip with port, for example: '127.0.0.1:50051'");

    // TLS
    let use_tls = env::var("USE_TLS").ok().unwrap_or_else(|| {
        log::warn!("env 'USE_TLS' should be set, default is 'false'");
        "false".to_string()
    });
    let use_tls: bool = use_tls
        .parse()
        .expect("invalid 'USE_TLS' env, should be 'true' or 'false'");

    let tls_config = if use_tls {
        let cert_pem_path = env::var("WORKER_CERT_PEM_PATH")
            .expect("env 'WORKER_CERT_PEM_PATH' is expected to run program");
        let cert_key_path = env::var("WORKER_CERT_KEY_PATH")
            .expect("env 'WORKER_CERT_KEY_PATH' is expected to run program");
        let ca_cert_path = env::var("CA_CERT_PEM_PATH")
            .expect("env 'CA_CERT_PEM_PATH' is expected to run program");

        let cert_pem = fs::read_to_string(cert_pem_path)
            .expect("can't read cert pem path, which was set by 'WORKER_CERT_PEM_PATH' env");
        let cert_key = fs::read_to_string(cert_key_path)
            .expect("can't read cert key path, which was set by 'WORKER_CERT_KEY_PATH' env");
        let cert_ca = fs::read_to_string(ca_cert_path)
            .expect("can't read cert ca path, which was set by CA_CERT_PEM_PATH'");

        let identity = Identity::from_pem(cert_pem, cert_key);
        let ca = Certificate::from_pem(cert_ca);
        Some(ServerTlsConfig::new().identity(identity).client_ca_root(ca))
    } else {
        None
    };

    // start service
    let dns_resolver = TokioResolver::builder_tokio()
        .expect("failed to get dns resolving settings")
        .build()
        .expect("failed to build dns resolver");

    let worker = WorkerService::new(dns_resolver);

    log::info!("listen {}", addr);
    let mut builder = tonic::transport::Server::builder();
    if let Some(tls_config) = tls_config {
        builder = builder.tls_config(tls_config).expect("invalid tls config")
    }
    builder
        .add_service(pb::worker_service_server::WorkerServiceServer::new(worker))
        .serve(addr)
        .await
        .expect("failed to start service");
}
