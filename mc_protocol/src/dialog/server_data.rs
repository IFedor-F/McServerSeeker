use super::DialogError;
use super::server_dst::CantResolveHost;
use crate::states::status::s2c::status_response::ForgeMod;
use crate::types::difficulty::Difficulty;
use crate::types::gamemode::GameMode;
use crate::types::*;
use std::collections::HashSet;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PlayerRecord {
    pub player: Player,
    pub game_mode: Option<GameMode>,
    pub ping: Option<i32>,
}

impl From<Player> for PlayerRecord {
    fn from(value: Player) -> Self {
        Self {
            player: value,
            game_mode: None,
            ping: None,
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ServerData {
    pub description: String,
    pub protocol: i32,
    pub version_name: String,
    pub online_players: i32,
    pub max_players: i32,
    pub players: HashSet<PlayerRecord>,
    pub enforces_secure_chat: bool,
    pub no_chat_reports: bool,
    pub mods: Vec<ForgeMod>,

    pub brand: Option<String>,
    pub links: Vec<String>,
    pub code_of_conduct: Option<String>,
    pub features: Vec<String>,
    pub resource_pack: Option<ResourcePack>,
    pub channels: HashSet<String>,
    pub registered_channels: HashSet<String>,

    pub commands: Vec<String>,
    pub hashed_seed: Option<i64>,
    pub gamemode: Option<GameMode>,
    pub difficulty: Option<Difficulty>,
    pub dimension: Option<String>,
    pub view_distance: Option<u16>,
    pub simulation_distance: Option<u16>,
    pub is_hardcore: Option<bool>,
    pub reduced_debug_info: Option<bool>,
    pub do_limited_crafting: Option<bool>,
    pub is_flat: Option<bool>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ResourcePack {
    pub url: String,
    pub hash: Option<String>,
    pub forced: bool,
}

#[derive(Debug)]
pub enum DisconnectLoginReason {
    OnlineMode { should_authenticate: bool },
    DisconnectByServer { msg: String },
    DialogError(DialogError),
    Timeout,
}

#[derive(Debug)]
pub enum DisconnectConfigurationReason {
    DisconnectByServer { msg: String },
    ShowDialog,
    TooMuchTransfers { count: u32 },
    CantResolveHost(CantResolveHost),
    DialogError(DialogError),
    Timeout,
}
#[derive(Debug)]
pub enum DisconnectPlayReason {
    DisconnectByServer { msg: String },
    DialogError(DialogError),
}

#[derive(Debug)]
pub enum ConnectionResult {
    NoHandshake(DialogError),
    DisconnectAtLogin {
        data: ServerData,
        reason: DisconnectLoginReason,
    },
    DisconnectAtConfiguration {
        data: ServerData,
        reason: DisconnectConfigurationReason,
    },
    DisconnectAtPlay {
        data: ServerData,
        reason: DisconnectPlayReason,
    },
    Successful {
        data: ServerData,
    },
}
impl ConnectionResult {
    pub fn get_data(self) -> Option<ServerData> {
        match self {
            ConnectionResult::NoHandshake(_) => None,
            ConnectionResult::DisconnectAtLogin { data, .. } => Some(data),
            ConnectionResult::DisconnectAtConfiguration { data, .. } => Some(data),
            ConnectionResult::DisconnectAtPlay { data, .. } => Some(data),
            ConnectionResult::Successful { data } => Some(data),
        }
    }
}
