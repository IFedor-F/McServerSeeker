pub mod api;
pub mod config;
pub mod database;
pub mod player_tracking;
pub mod scan_jobs;
pub mod types;

use crate::config::Config;
use crate::database::DbQueueWorker;
use crate::player_tracking::PlayerTrackingService;
use crate::scan_jobs::schedule::ScheduleService;
use crate::scan_jobs::{Worker, WorkerManagerService};
use data_core::manager_api::ScheduleData;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs};
use tokio::task::JoinSet;
use tonic::transport::{Certificate, ClientTlsConfig, Endpoint, Identity};

#[tokio::main]
async fn main() {
    // config and env
    env_logger::init();
    let config_path =
        env::var("CONFIG_PATH").expect("env 'CONFIG_PATH' is expected to run program");

    let config: Config = toml::from_str(
        &fs::read_to_string(config_path)
            .expect("can't read cert key path, which was set by 'CONFIG_PATH' env"),
    )
    .expect("invalid config");

    let db_url = env::var("DATABASE_URL")
        .ok()
        .or(config.general.database_url.clone())
        .expect(
            "env 'DATABASE_URL' or 'database_url' in manager config is expected to run program",
        );

    // join set for some tasks
    let mut join_set = JoinSet::new();

    // database
    let db_pool = PgPool::connect(&db_url)
        .await
        .expect("can't connect to database");

    // worker manager
    let manager_service = configure_manager(&db_pool, &config);

    // scheduler
    let scheduler = configure_scheduler(&db_pool, manager_service.clone(), config.jobs).await;

    // tracking
    let player_tracking_service = if let Some(t) = config.player_tracking {
        let s = Arc::new(PlayerTrackingService::new(
            db_pool.clone(),
            Duration::from_secs(t.interval_secs),
        ));
        let cloned_s = s.clone();
        join_set.spawn(async move {
            cloned_s.run_tracking().await;
        });
        Some(s)
    } else {
        None
    };

    // api
    if let Some(addr) = config.general.bind_address {
        let app = api::setup_router(
            manager_service.clone(),
            scheduler.clone(),
            player_tracking_service,
        );
        log::info!("API Server running on {addr}");
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("can't bind port");

        join_set.spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
    }

    tokio::select! {
        res = join_set.join_next() => {
            match res {
                Some(Ok(_)) => unreachable!(),
                None => unreachable!(),
                Some(Err(e)) => {
                    panic!("a background task panicked: {}", e);
                }
            }
        }

        _ = tokio::signal::ctrl_c() => {
            log::info!("received shutdown signal");
            return;
        }
    }
}

async fn configure_scheduler(
    db_pool: &PgPool,
    manager_service: Arc<WorkerManagerService>,
    jobs: Vec<ScheduleData>,
) -> Arc<ScheduleService> {
    let scheduler = ScheduleService::new(db_pool.clone(), manager_service);
    for job in jobs {
        let job_name = job.name.clone();
        scheduler
            .upsert_schedule(job)
            .await
            .expect("can't add schedule");
        scheduler
            .run_schedule(job_name)
            .await
            .expect("can't run schedule");
    }
    Arc::new(scheduler)
}
fn configure_manager(db_pool: &PgPool, config: &Config) -> Arc<WorkerManagerService> {
    let tls_config = if config.general.use_tls_for_workers {
        Some(configure_tls(&config.general))
    } else {
        None
    };
    let db_queue_worker = DbQueueWorker::new(db_pool.clone());
    let mut manager_service = WorkerManagerService::new(db_queue_worker);
    for worker_info in config.workers.iter() {
        let mut endpoint =
            Endpoint::from_shared(worker_info.url.to_string()).expect("invalid worker endpoint");
        if let Some(tls_config) = tls_config.clone() {
            endpoint = endpoint
                .tls_config(tls_config)
                .expect("invalid tls configuration")
                .timeout(Duration::from_secs(5))
                .connect_timeout(Duration::from_secs(5))
        }
        let worker = Worker::new(worker_info.clone(), endpoint);
        manager_service.add_worker(worker)
    }
    Arc::new(manager_service)
}

fn configure_tls(config: &config::General) -> ClientTlsConfig {
    let cert_key_path = env::var("MANAGER_CERT_KEY_PATH")
        .ok()
        .or(config.manager_cert_key_path.clone())
        .expect("env 'MANAGER_CERT_KEY_PATH' or 'manager_cert_key_path' in manager config is expected to run program");
    let cert_pem_path = env::var("MANAGER_CERT_PEM_PATH")
        .ok()
        .or(config.manager_cert_pem_path.clone())
        .expect("env 'MANAGER_CERT_PEM_PATH' or 'manager_cert_pem_path' in manager config is expected to run program");
    let ca_cert_path = env::var("CA_CERT_PEM_PATH")
        .ok()
        .or(config.ca_cert_pem_path.clone())
        .expect("env 'CA_CERT_PEM_PATH' or 'ca_cert_pem_path' in manager config is expected to run program");

    let cert_key = fs::read_to_string(cert_key_path).expect("can't read manager cert key");
    let cert_pem = fs::read_to_string(cert_pem_path).expect("can't read manager cert pem");
    let cert_ca = fs::read_to_string(ca_cert_path).expect("can't read CA cert pem");
    let identity = Identity::from_pem(cert_pem, cert_key);
    let ca = Certificate::from_pem(cert_ca);
    ClientTlsConfig::new().ca_certificate(ca).identity(identity)
}
