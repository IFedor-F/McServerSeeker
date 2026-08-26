use chrono::{DateTime, Utc};
use std::net::IpAddr;
use uuid::Uuid;

pub mod tracking_service;
pub mod webhook;
pub use tracking_service::PlayerTrackingService;
pub use webhook::WebHook;
#[derive(Debug, Clone, Hash, PartialEq, Eq, sqlx::FromRow)]
pub struct FoundedPlayer {
    pub uuid: Uuid,
    pub name: String,
    pub ip: IpAddr,
    pub port: i32,
    pub last_seen: DateTime<Utc>,
}
