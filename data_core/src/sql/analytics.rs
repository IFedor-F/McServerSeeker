// schema "analytics"

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow, Debug, Clone)]
pub struct PlayerTrack {
    pub id: i32,
    pub name: Option<String>,
    pub uuid: Option<Uuid>,
    pub webhook_id: i32,
    pub last_send: Option<DateTime<Utc>>,
    pub last_server_id: Option<i32>,
}
#[derive(FromRow, Debug, Clone)]
pub struct Webhook {
    pub id: i32,
    pub name: String,
    pub url: String,
}
