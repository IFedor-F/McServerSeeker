use crate::proto;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum ParseServerDataError {
    #[error("{0} is expected field")]
    NoField(&'static str),

    #[error("Unknown {enum_name} variant: {value}")]
    UnknownEnumVariant { enum_name: &'static str, value: i32 },

    #[error("Can't parse type {0}")]
    CantParse(&'static str),
}

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
    pub port: u16,
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
}

impl TryFrom<proto::scanner::McServerData> for McServerData {
    type Error = ParseServerDataError;

    fn try_from(value: proto::scanner::McServerData) -> Result<Self, Self::Error> {
        let players = value
            .players
            .into_iter()
            .map(|p| {
                Ok(PlayerRecord {
                    name: p.name,
                    uuid: Uuid::try_from(p.uuid)
                        .map_err(|_| ParseServerDataError::CantParse("uuid"))?,
                    is_online: p.is_online,
                    gamemode: parse_gamemode(p.gamemode)?,
                    ping: p.ping,
                })
            })
            .collect::<Result<Vec<_>, ParseServerDataError>>()?;

        let mods = value
            .mods
            .into_iter()
            .map(|m| McMod {
                mod_id: m.mod_id,
                version: m.version,
            })
            .collect();

        let resource_pack = value.resource_pack.map(|rp| ResourcePack {
            url: rp.url,
            hash: rp.hash,
            forced: rp.forced,
        });

        let world_data = WorldData {
            hashed_seed: value.hashed_seed,
            gamemode: parse_gamemode(value.gamemode)?,
            difficulty: parse_difficulty(value.difficulty)?,
            dimension: value.dimension,
            view_distance: value.view_distance,
            simulation_distance: value.simulation_distance,
            is_hardcore: value.is_hardcore,
            reduced_debug_info: value.reduced_debug_info,
            do_limited_crafting: value.do_limited_crafting,
            is_flat: value.is_flat,
        };

        Ok(Self {
            ip: value
                .ip
                .ok_or(Self::Error::NoField("ip"))?
                .try_into()
                .map_err(|_| Self::Error::CantParse("ip"))?,
            port: value.port as u16,
            connection_result: parse_conn_result(value.connection_result)?,
            description: value.description,
            protocol: value.protocol,
            version_name: value.version_name,
            online_players: value.online_players,
            max_players: value.max_players,
            players,
            enforces_secure_chat: value.enforces_secure_chat,
            no_chat_reports: value.no_chat_reports,
            mods,
            channels: value.channels,
            brand: value.brand,
            links: value.links,
            code_of_conduct: value.code_of_conduct,
            features: value.features,
            resource_pack,
            commands: value.commands,
            world_data,
            // The remaining connection-related fields
            is_whitelist: value.is_whitelist,
            is_online_mode: value.is_online_mode,
            offline_auth: value.offline_auth,
        })
    }
}

fn parse_gamemode(gm: Option<i32>) -> Result<Option<GameMode>, ParseServerDataError> {
    if let Some(gm) = gm {
        let value = proto::scanner::GameMode::try_from(gm).map_err(|e| {
            ParseServerDataError::UnknownEnumVariant {
                enum_name: "GameMode",
                value: e.0,
            }
        })?;
        Ok(value.into())
    } else {
        Ok(None)
    }
}
fn parse_difficulty(dif: Option<i32>) -> Result<Option<Difficulty>, ParseServerDataError> {
    if let Some(dif) = dif {
        let value = proto::scanner::Difficulty::try_from(dif).map_err(|e| {
            ParseServerDataError::UnknownEnumVariant {
                enum_name: "Difficulty",
                value: e.0,
            }
        })?;
        Ok(value.into())
    } else {
        Ok(None)
    }
}
fn parse_conn_result(conn_result: i32) -> Result<ConnectionResult, ParseServerDataError> {
    let value = proto::scanner::ConnectionResult::try_from(conn_result).map_err(|e| {
        ParseServerDataError::UnknownEnumVariant {
            enum_name: "ConnectionResult",
            value: e.0,
        }
    })?;
    let result: Option<ConnectionResult> = value.into();
    result.ok_or(ParseServerDataError::NoField("ConnectionResult"))
}
