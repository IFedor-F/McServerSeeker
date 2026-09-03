pub use super::server_data::*;
use crate::proto::scanner;
use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
pub use scanner::ScanMethod;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};

#[allow(unused_imports)]
use serde_json::json;

use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::net::IpAddr;
use url::Url;
use utoipa::ToSchema;
use uuid::Uuid;

fn deserialize_non_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Err(D::Error::custom("string cannot be empty"));
    }
    Ok(s)
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ScheduleData {
    pub name: String,
    pub executor: JobExecutor,
    #[schema(value_type = String, examples("always", "30 10 * * *"))]
    pub schedule: Schedule,
    #[serde(default)]
    #[schema(example = json!(15))]
    pub wait_secs: u64,
    #[schema(inline)]
    pub task: TaskInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct JobId(pub u64);
impl Display for JobId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ManagerJobInfo {
    pub id: JobId,
    pub name: String,
    pub executor: JobExecutor,
    #[schema(inline)]
    pub task: TaskInfo,
    #[schema(inline)]
    pub progress: JobProgress,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ManagerScanOneReq {
    pub executor: JobExecutor,
    #[schema(examples("server.example", "127.0.0.1",))]
    pub target: String,
    #[schema(example = json!(25565))]
    pub port: Option<u16>,
    pub scan_method: ScanMethod,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ManagerJobReq {
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    pub name: String,
    pub executor: JobExecutor,
    #[schema(inline)]
    pub task: TaskInfo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case", tag = "type", deny_unknown_fields)]
pub enum TaskInfo {
    Discover(DiscoverRequest),
    ScanSelected(ScanSelectedRequest),
    RescanDb { rate: u32, method: ScanMethod },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DiscoverRequest {
    #[schema(value_type = Vec<String>, example = json!(["127.0.0.1/32", "192.168.0.0/16"]))]
    pub targets: Vec<IpNetwork>,
    #[serde(default)]
    #[schema(value_type = Vec<String>, example = json!(["10.0.0.0/8", "172.16.0.0/12"]))]
    pub excludes: Vec<IpNetwork>,
    #[serde(default)]
    #[schema(example = json!([25565, 25566, 42059]))]
    pub ports: Vec<u16>,
    #[serde(default)]
    #[schema(example = json!([{"min": 25565, "max": 25570}, {"min": 43123, "max" :45321}]))]
    pub port_ranges: Vec<PortRange>,
    #[schema(example = json!(1000))]
    pub rate: u32,
    #[schema(value_type = String, example = "only_handshake")]
    pub method: ScanMethod,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
#[schema(example = json!({"min": 25565, "max": 25570}))]
pub struct PortRange {
    pub min: u16,
    pub max: u16,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ScanSelectedRequest {
    #[schema(value_type = String, example = "only_handshake")]
    pub method: ScanMethod,
    #[schema(example = json!(1000))]
    pub rate: u32,
    #[schema(
        example = json!([{"ip": "127.0.0.1", "port": 25565, "player_name": "abcdef32"}, {"ip": "192.168.1.2", "port": 25565, "player_name": "mcname123"}])
    )]
    pub targets: Vec<ScanTarget>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ScanTarget {
    #[schema(value_type = String, example = "127.0.0.1")]
    pub ip: IpAddr,
    #[schema(example = "25565")]
    pub port: u16,
    pub player_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case", tag = "type", deny_unknown_fields)]
pub enum JobProgress {
    NoData,
    Discover(DiscoverJobProgress),
    ScanSelected(ScanSelectedJobProgress),
}
impl Default for JobProgress {
    fn default() -> Self {
        Self::NoData
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DiscoverJobProgress {
    pub scanned_progress: f32,
    pub founded: u32,
    pub parsing_now: u32,
    pub successful: u32,
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ScanSelectedJobProgress {
    pub all: u32,
    pub checked: u32,
    pub successful: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case", tag = "type", deny_unknown_fields)]
pub enum JobExecutor {
    Worker { name: String },
    LeastLoadedSpecified { worker_names: HashSet<String> },
    LeastLoadedAll,
    BalanceSpecified { worker_names: HashSet<String> },
    BalanceAll,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkerInfo {
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    pub name: String,
    pub url: Url,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkerStatus {
    pub info: WorkerInfo,
    pub loading: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "String", into = "String", deny_unknown_fields)]
pub enum Schedule {
    Always,
    #[schema(value_type = String)]
    Cron(cron::Schedule),
}
impl TryFrom<String> for Schedule {
    type Error = cron::error::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "always" => Ok(Self::Always),
            other => Ok(Self::Cron(cron::Schedule::try_from(other)?)),
        }
    }
}
impl Into<String> for Schedule {
    fn into(self) -> String {
        match self {
            Schedule::Always => "always".to_string(),
            Schedule::Cron(schedule) => schedule.to_string(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct PlayerTrackInfo {
    pub name: Option<String>,
    pub uuid: Option<Uuid>,
    #[schema(value_type = Option<String>, example = "2026-00-00T00:00:00Z")]
    pub last_send: Option<DateTime<Utc>>,
    #[schema(value_type = String, example = "127.0.0.1")]
    pub last_server_ip: Option<IpAddr>,
    #[schema(example = "25565")]
    pub last_server_port: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WebhookInfo {
    pub url: Url,
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    pub webhook_name: String,
}
