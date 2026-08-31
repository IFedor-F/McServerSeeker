use data_core::api::manager::{ScheduleData, WorkerInfo};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub general: General,
    #[serde(default)]
    pub workers: Vec<WorkerInfo>,
    #[serde(default)]
    pub jobs: Vec<ConfigScheduleData>,
    #[serde(default)]
    pub player_tracking: PlayerTracking,
    #[serde(default)]
    #[serde(rename = "api")]
    pub api_settings: ApiSettings,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct General {
    pub database_url: Option<String>,
    #[serde(default)]
    pub use_tls_for_workers: bool,
    pub manager_cert_key_path: Option<String>,
    pub manager_cert_pem_path: Option<String>,
    pub ca_cert_pem_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigScheduleData {
    #[serde(flatten)]
    pub data: ScheduleData,
    #[serde(default = "default_true")]
    pub run_on_load: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerTracking {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_player_track_interval")]
    pub interval_secs: u64,
}
impl Default for PlayerTracking {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_secs: default_player_track_interval(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiSettings {
    #[serde(default)]
    pub enabled: bool,
    pub bind_address: Option<String>,
    pub use_api_token: Option<bool>,
    pub api_token: Option<String>,
}

pub fn default_true() -> bool {
    true
}
pub fn default_player_track_interval() -> u64 {
    30
}
