use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionResult {
    LoginDisconnect,
    ConfigurationDisconnect,
    PlayDisconnect,
    Successful,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerRecord {
    pub name: String,
    pub uuid: Uuid,
    pub is_online: bool,
    pub gamemode: Option<GameMode>,
    pub ping: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct McMod {
    pub mod_id: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResourcePack {
    pub url: String,
    pub hash: Option<String>,
    pub forced: bool,
}

// Extracted Play Data
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WorldData {
    pub hashed_seed: Option<i64>,
    pub gamemode: Option<GameMode>,
    pub difficulty: Option<Difficulty>,
    pub dimension: Option<String>,
    pub view_distance: Option<u32>,
    pub simulation_distance: Option<u32>,
    pub is_hardcore: Option<bool>,
    pub reduced_debug_info: Option<bool>,
    pub do_limited_crafting: Option<bool>,
    pub is_flat: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct McServerData {
    #[schema(value_type = String, example = "127.0.0.1")]
    pub ip: IpAddr,
    pub port: u32,
    pub domain: Option<String>,
    pub connection_result: ConnectionResult,
    pub description: String,
    pub protocol: i32,
    pub version_name: String,
    pub online_players: i32,
    pub max_players: i32,
    pub players: Vec<PlayerRecord>,
    pub enforces_secure_chat: bool,
    pub no_chat_reports: bool,
    pub mods: Vec<McMod>,
    pub channels: Vec<String>,
    pub brand: Option<String>,
    pub links: Vec<String>,
    pub code_of_conduct: Option<String>,
    pub features: Vec<String>,
    pub resource_pack: Option<ResourcePack>,
    pub commands: Vec<String>,

    #[schema(inline)]
    pub world_data: WorldData,

    // Other
    pub is_whitelist: Option<bool>,
    pub is_online_mode: Option<bool>,
    pub offline_auth: Option<bool>,
    pub player_name: Option<String>,
}
