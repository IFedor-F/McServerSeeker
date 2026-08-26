use data_core::manager_api::{ScheduleData, WorkerInfo};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: General,

    pub workers: Vec<WorkerInfo>,

    pub jobs: Vec<ScheduleData>,

    pub player_tracking: Option<PlayerTracking>,
}

#[derive(Debug, Deserialize)]
pub struct General {
    #[serde(default)]
    pub bind_address: Option<String>,
    pub use_tls_for_workers: bool,
    pub database_url: Option<String>,
    pub manager_cert_key_path: Option<String>,
    pub manager_cert_pem_path: Option<String>,
    pub ca_cert_pem_path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlayerTracking {
    pub interval_secs: u64,
}
