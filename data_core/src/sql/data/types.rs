use crate::proto::scanner;
use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use sqlx::{FromRow, Type};
use std::str::FromStr;
use uuid::Uuid;

// schema 'data'
#[derive(Debug, Clone, PartialEq, Eq, Hash, Type)]
#[sqlx(type_name = "data.difficulty", rename_all = "lowercase")]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}
impl Difficulty {
    pub fn as_str(&self) -> &'static str {
        match self {
            Difficulty::Peaceful => "peaceful",
            Difficulty::Easy => "easy",
            Difficulty::Normal => "normal",
            Difficulty::Hard => "hard",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Type)]
#[sqlx(type_name = "data.gamemode", rename_all = "lowercase")]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl FromStr for GameMode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "survival" => Ok(GameMode::Survival),
            "creative" => Ok(GameMode::Creative),
            "adventure" => Ok(GameMode::Adventure),
            "spectator" => Ok(GameMode::Spectator),
            _ => Err(()),
        }
    }
}

impl GameMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameMode::Survival => "survival",
            GameMode::Creative => "creative",
            GameMode::Adventure => "adventure",
            GameMode::Spectator => "spectator",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Type)]
#[sqlx(type_name = "data.connection_result", rename_all = "snake_case")]
pub enum ConnectionResult {
    LoginDisconnect,
    ConfigurationDisconnect,
    PlayDisconnect,
    Successful,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromRow)]
pub struct Server {
    pub id: i32,
    pub ip: IpNetwork,
    pub port: i32,
    pub domain: Option<String>,
    pub last_connection_result: ConnectionResult,
    pub last_used_nick: Option<String>,
    pub description: String,
    pub protocol: i32,
    pub version_name: String,
    pub brand: Option<String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub last_checked: DateTime<Utc>,
    pub online_players: i32,
    pub max_players: i32,
    pub enforces_secure_chat: bool,
    pub no_chat_reports: bool,
    pub is_whitelist: Option<bool>,
    pub is_online_mode: Option<bool>,
    pub offline_auth: Option<bool>,
    pub code_of_conduct: Option<String>,
    pub favicon_id: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromRow)]
pub struct Player {
    pub id: i32,
    pub uuid: Uuid,
    pub name: String,
    pub is_online_mode: bool,
    pub last_seen: DateTime<Utc>,
    pub first_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct PlayerRecord {
    pub server_id: i32,
    pub player_id: i32,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub last_gamemode: Option<GameMode>,
    pub last_ping: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromRow)]
pub struct Mod {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromRow)]
pub struct ServerMod {
    pub server_id: i32,
    pub mod_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromRow)]
pub struct ResourcePack {
    pub server_id: i32,
    pub url: String,
    pub hash: Option<String>,
    pub forced: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromRow)]
pub struct Link {
    pub id: i32,
    pub server_id: i32,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromRow)]
pub struct WorldData {
    pub server_id: i32,
    pub hashed_seed: Option<i64>,
    pub gamemode: Option<GameMode>,
    pub difficulty: Option<Difficulty>,
    pub dimension: Option<String>,
    pub view_distance: Option<i16>,
    pub simulation_distance: Option<i16>,
    pub is_hardcore: Option<bool>,
    pub reduced_debug_info: Option<bool>,
    pub do_limited_crafting: Option<bool>,
    pub is_flat: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromRow)]
pub struct Command {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromRow)]
pub struct ServerCommand {
    pub server_id: i32,
    pub command_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromRow)]
pub struct Feature {
    pub id: i32,
    pub identifier: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromRow)]
pub struct ServerFeature {
    pub server_id: i32,
    pub feature_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromRow)]
pub struct Channel {
    pub id: i32,
    pub identifier: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromRow)]
pub struct ServerChannel {
    pub server_id: i32,
    pub channel_id: i32,
}

impl From<scanner::ConnectionResult> for Option<ConnectionResult> {
    fn from(value: scanner::ConnectionResult) -> Self {
        use ConnectionResult::*;
        match value {
            scanner::ConnectionResult::LoginDisconnect => Some(LoginDisconnect),
            scanner::ConnectionResult::ConfigurationDisconnect => Some(ConfigurationDisconnect),
            scanner::ConnectionResult::PlayDisconnect => Some(PlayDisconnect),
            scanner::ConnectionResult::Successful => Some(Successful),
            scanner::ConnectionResult::UnknownConnectionResult => None,
        }
    }
}

impl From<scanner::Difficulty> for Option<Difficulty> {
    fn from(value: scanner::Difficulty) -> Self {
        use Difficulty::*;
        match value {
            scanner::Difficulty::Peaceful => Some(Peaceful),
            scanner::Difficulty::Easy => Some(Easy),
            scanner::Difficulty::Normal => Some(Normal),
            scanner::Difficulty::Hard => Some(Hard),
            scanner::Difficulty::UnknownDifficulty => None,
        }
    }
}

impl From<scanner::GameMode> for Option<GameMode> {
    fn from(value: scanner::GameMode) -> Self {
        use GameMode::*;
        match value {
            scanner::GameMode::Survival => Some(Survival),
            scanner::GameMode::Creative => Some(Creative),
            scanner::GameMode::Adventure => Some(Adventure),
            scanner::GameMode::Spectator => Some(Spectator),
            scanner::GameMode::UnknownGameMode => None,
        }
    }
}
