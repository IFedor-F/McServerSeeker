use chrono::{DateTime, Utc};
use data_core::proto::scanner as pb;
use data_core::sql::data as sql_data;
use data_core::sql::data::{
    NewPlayerSession, NewPlayersBatch, NewResourcePack, NewServer, NewServerChannels,
    NewServerCommands, NewServerFeatures, NewServerLinks, NewServerMods, NewWorldData,
};
use ipnetwork::IpNetwork;
use std::net::IpAddr;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum SqlParsePbDataError {
    #[error("{0} is expected field")]
    NoField(&'static str),

    #[error("Unknown {enum_name} variant: {value}")]
    UnknownEnumVariant { enum_name: &'static str, value: i32 },

    #[error("Can't parse type {0}")]
    CantParse(&'static str),
}

pub struct ParsedForSqlServerData {
    new_server: NewServer,
    new_world_data: NewWorldData,
    new_players_batch: NewPlayersBatch,
    new_server_mods: NewServerMods,
    new_server_channels: NewServerChannels,
    new_server_links: NewServerLinks,
    new_server_commands: NewServerCommands,
    new_server_features: NewServerFeatures,
    new_resource_pack: Option<NewResourcePack>,
}
impl ParsedForSqlServerData {
    pub fn try_parse(pb_server_data: pb::McServerData) -> Result<Self, SqlParsePbDataError> {
        let pb::McServerData {
            ip,
            port,
            domain,
            connection_result,
            description,
            protocol,
            version_name,
            online_players,
            max_players,
            players,
            enforces_secure_chat,
            no_chat_reports,
            mods,
            channels,
            brand,
            links,
            code_of_conduct,
            features,
            resource_pack,
            hashed_seed,
            gamemode,
            difficulty,
            dimension,
            view_distance,
            simulation_distance,
            is_hardcore,
            reduced_debug_info,
            do_limited_crafting,
            is_flat,
            commands,
            is_whitelist,
            is_online_mode,
            offline_auth,
            player_name,
        } = pb_server_data;

        let ip = IpAddr::try_from(ip.ok_or(SqlParsePbDataError::NoField("ip"))?)
            .map_err(|_| SqlParsePbDataError::CantParse("ip"))?;
        let conn_result: Option<sql_data::ConnectionResult> =
            pb::ConnectionResult::try_from(connection_result)
                .map_err(|e| SqlParsePbDataError::UnknownEnumVariant {
                    enum_name: "connection_result",
                    value: e.0,
                })?
                .into();
        let last_connection_result =
            conn_result.ok_or(SqlParsePbDataError::NoField("connection_result"))?;

        let difficulty: Option<sql_data::Difficulty> =
            difficulty.and_then(|v| pb::Difficulty::try_from(v).ok().and_then(|v| v.into()));

        let players: Vec<NewPlayerSession> = players
            .into_iter()
            .map(|p| {
                Ok(NewPlayerSession {
                    uuid: Uuid::try_from(p.uuid)
                        .map_err(|_| SqlParsePbDataError::CantParse("uuid"))?,
                    name: p.name,
                    is_online_mode: p.is_online,
                    game_mode: parse_gamemode(p.gamemode),
                    ping: p.ping,
                })
            })
            .collect::<Result<_, _>>()?;

        let time: DateTime<Utc> = Utc::now();

        let new_server = NewServer {
            ip: IpNetwork::from(ip),
            port: port as i32,
            domain,
            last_connection_result,
            last_used_nick: player_name,
            description,
            protocol,
            version_name,
            brand,
            time,
            online_players,
            max_players,
            enforces_secure_chat,
            no_chat_reports,
            is_whitelist,
            is_online_mode,
            offline_auth,
            code_of_conduct,
        };

        let new_world_data = NewWorldData {
            hashed_seed,
            gamemode: parse_gamemode(gamemode),
            difficulty,
            dimension,
            view_distance: view_distance.and_then(|v| Some(v as i16)),
            simulation_distance: simulation_distance.and_then(|v| Some(v as i16)),
            is_hardcore,
            reduced_debug_info,
            do_limited_crafting,
            is_flat,
        };
        Ok(Self {
            new_server,
            new_world_data,
            new_players_batch: NewPlayersBatch { time, players },
            new_server_mods: NewServerMods {
                mods_names: mods.into_iter().map(|m| m.mod_id).collect(),
            },
            new_server_channels: NewServerChannels { channels },
            new_server_links: NewServerLinks { links },
            new_server_commands: NewServerCommands { commands },
            new_server_features: NewServerFeatures { features },
            new_resource_pack: resource_pack.and_then(|v| {
                Some(NewResourcePack {
                    url: v.url,
                    hash: v.hash,
                    forced: v.forced,
                })
            }),
        })
    }

    pub async fn write_to_tx(self, tx: &mut sqlx::PgConnection) -> Result<(), sqlx::Error> {
        let server_id = self.new_server.upsert_with_id_returning(&mut *tx).await?;
        self.new_world_data.upsert(server_id, &mut *tx).await?;
        self.new_players_batch.upsert(server_id, &mut *tx).await?;
        self.new_server_mods.sync(server_id, &mut *tx).await?;
        self.new_server_channels.sync(server_id, &mut *tx).await?;
        self.new_server_links.sync(server_id, &mut *tx).await?;
        self.new_server_commands.sync(server_id, &mut *tx).await?;
        self.new_server_features.sync(server_id, &mut *tx).await?;
        if let Some(rp) = self.new_resource_pack {
            rp.upsert(server_id, &mut *tx).await?;
        }
        Ok(())
    }
}

fn parse_gamemode(gamemode: Option<i32>) -> Option<sql_data::GameMode> {
    gamemode.and_then(|v| pb::GameMode::try_from(v).ok().and_then(|v| v.into()))
}
