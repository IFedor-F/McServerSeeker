use crate::proto::scanner;
use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use itertools::Itertools;
use sqlx::{FromRow, PgConnection, Type};
use uuid::Uuid;

// schema 'data'
#[derive(Debug, Clone, Type)]
#[sqlx(type_name = "data.difficulty", rename_all = "lowercase")]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Type)]
#[sqlx(type_name = "data.gamemode", rename_all = "lowercase")]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Debug, Clone, Type)]
#[sqlx(type_name = "data.connection_result", rename_all = "snake_case")]
pub enum ConnectionResult {
    LoginDisconnect,
    ConfigurationDisconnect,
    PlayDisconnect,
    Successful,
}

#[derive(Debug, Clone, FromRow)]
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
}

#[derive(Debug, Clone, FromRow)]
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

#[derive(Debug, Clone, FromRow)]
pub struct Mod {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct ServerMod {
    pub server_id: i32,
    pub mod_id: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct ResourcePack {
    pub server_id: i32,
    pub url: String,
    pub hash: Option<String>,
    pub forced: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct Link {
    pub id: i32,
    pub server_id: i32,
    pub url: String,
}

#[derive(Debug, Clone, FromRow)]
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

#[derive(FromRow, Debug, Clone)]
pub struct Command {
    pub id: i32,
    pub name: String,
}

#[derive(FromRow, Debug, Clone)]
pub struct ServerCommand {
    pub server_id: i32,
    pub command_id: i32,
}

#[derive(FromRow, Debug, Clone)]
pub struct Feature {
    pub id: i32,
    pub identifier: String,
}

#[derive(FromRow, Debug, Clone)]
pub struct ServerFeature {
    pub server_id: i32,
    pub feature_id: i32,
}

#[derive(FromRow, Debug, Clone)]
pub struct Channel {
    pub id: i32,
    pub identifier: String,
}
#[derive(FromRow, Debug, Clone)]
pub struct ServerChannel {
    pub server_id: i32,
    pub channel_id: i32,
}

// structs for inserts
pub struct NewServer {
    pub ip: IpNetwork,
    pub port: i32,
    pub domain: Option<String>,
    pub last_connection_result: ConnectionResult,
    pub last_used_nick: Option<String>,
    pub description: String,
    pub protocol: i32,
    pub version_name: String,
    pub brand: Option<String>,
    pub time: DateTime<Utc>,
    pub online_players: i32,
    pub max_players: i32,
    pub enforces_secure_chat: bool,
    pub no_chat_reports: bool,
    pub is_whitelist: Option<bool>,
    pub is_online_mode: Option<bool>,
    pub offline_auth: Option<bool>,
    pub code_of_conduct: Option<String>,
}
pub struct NewWorldData {
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

pub struct NewPlayerSession {
    pub uuid: Uuid,
    pub name: String,
    pub is_online_mode: bool,
    pub game_mode: Option<GameMode>,
    pub ping: Option<i32>,
}
pub struct NewPlayersBatch {
    pub time: DateTime<Utc>,
    pub players: Vec<NewPlayerSession>,
}

pub struct NewServerCommands {
    pub commands: Vec<String>,
}
pub struct NewServerFeatures {
    pub features: Vec<String>,
}

pub struct NewServerChannels {
    pub channels: Vec<String>,
}

pub struct NewServerMods {
    pub mods_names: Vec<String>,
}

pub struct NewServerLinks {
    pub links: Vec<String>,
}

pub struct NewResourcePack {
    pub url: String,
    pub hash: Option<String>,
    pub forced: bool,
}
impl NewServer {
    pub async fn upsert_with_id_returning(self, tx: &mut PgConnection) -> Result<i32, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
            INSERT INTO data.servers (
                ip, port, domain, last_connection_result, last_used_nick, description, protocol, version_name, brand,
                first_seen, last_seen, last_checked, online_players, max_players,
                enforces_secure_chat, no_chat_reports, is_whitelist, is_online_mode,
                offline_auth, code_of_conduct
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10, $10, $11, $12, $13, $14, $15, $16, $17, $18
            )
            ON CONFLICT (ip, port) DO UPDATE SET
                domain = COALESCE(EXCLUDED.domain, data.servers.domain),
                last_connection_result = EXCLUDED.last_connection_result,
                last_used_nick = EXCLUDED.last_used_nick,
                description = EXCLUDED.description,
                protocol = EXCLUDED.protocol,
                version_name = excluded.version_name,
                brand = COALESCE(excluded.brand, data.servers.brand),
                last_seen = EXCLUDED.last_seen,
                last_checked = EXCLUDED.last_checked,
                online_players = excluded.online_players,
                max_players = excluded.max_players,
                enforces_secure_chat = excluded.enforces_secure_chat,
                no_chat_reports = excluded.no_chat_reports,
                is_whitelist = COALESCE(excluded.is_whitelist, data.servers.is_whitelist),
                is_online_mode = COALESCE(excluded.is_online_mode, data.servers.is_online_mode),
                offline_auth = COALESCE(excluded.offline_auth, data.servers.offline_auth),
                code_of_conduct = COALESCE(excluded.code_of_conduct, data.servers.code_of_conduct)
            RETURNING id
            "#,
            self.ip,
            self.port,
            self.domain,
            self.last_connection_result as ConnectionResult,
            self.last_used_nick,
            self.description,
            self.protocol,
            self.version_name,
            self.brand,
            self.time,
            self.online_players,
            self.max_players,
            self.enforces_secure_chat,
            self.no_chat_reports,
            self.is_whitelist,
            self.is_online_mode,
            self.offline_auth,
            self.code_of_conduct
        )
            .fetch_one(tx)
            .await
    }
}
impl NewWorldData {
    pub async fn upsert(self, server_id: i32, tx: &mut PgConnection) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO data.world_data (
                server_id, hashed_seed, gamemode, difficulty, dimension, view_distance,
                simulation_distance, is_hardcore, reduced_debug_info, do_limited_crafting, is_flat
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (server_id) DO UPDATE SET
                hashed_seed = COALESCE(excluded.hashed_seed, world_data.hashed_seed),
                gamemode = COALESCE(excluded.gamemode, world_data.gamemode),
                difficulty = COALESCE(excluded.difficulty, world_data.difficulty),
                dimension = COALESCE(excluded.dimension, world_data.dimension),
                view_distance = COALESCE(excluded.view_distance, world_data.view_distance),
                simulation_distance = COALESCE(excluded.simulation_distance, world_data.simulation_distance),
                is_hardcore = COALESCE(excluded.is_hardcore, world_data.is_hardcore),
                reduced_debug_info = COALESCE(excluded.reduced_debug_info, world_data.reduced_debug_info),
                do_limited_crafting = COALESCE(excluded.do_limited_crafting, world_data.do_limited_crafting),
                is_flat = COALESCE(excluded.is_flat, world_data.is_flat)
            "#,
            server_id,
            self.hashed_seed,
            self.gamemode as Option<GameMode>,
            self.difficulty as Option<Difficulty>,
            self.dimension,
            self.view_distance,
            self.simulation_distance,
            self.is_hardcore,
            self.reduced_debug_info,
            self.do_limited_crafting,
            self.is_flat
        )
            .execute(&mut *tx)
            .await?;

        Ok(())
    }
}
impl NewPlayersBatch {
    pub async fn upsert(self, server_id: i32, tx: &mut PgConnection) -> Result<(), sqlx::Error> {
        if self.players.is_empty() {
            return Ok(());
        }

        let (uuids, names, is_online_modes, gamemodes, pings): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = self
            .players
            .into_iter()
            .map(|p| (p.uuid, p.name, p.is_online_mode, p.game_mode, p.ping))
            .multiunzip();

        sqlx::query!(
            r#"
            WITH input AS (
                SELECT uuid, name, is_online_mode, last_gamemode, last_ping
                FROM UNNEST($3::uuid[], $4::text[], $5::bool[], $6::data.gamemode[], $7::integer[])
                AS t(uuid, name, is_online_mode, last_gamemode, last_ping)
            ),
            upserted_players AS (
                INSERT INTO data.players (uuid, name, is_online_mode, first_seen, last_seen)
                SELECT uuid, name, is_online_mode, $2, $2 FROM input i
                ON CONFLICT (uuid) DO UPDATE SET
                    name = EXCLUDED.name,
                    is_online_mode = EXCLUDED.is_online_mode,
                    last_seen = EXCLUDED.last_seen
                RETURNING id as player_id, uuid
            )
            INSERT INTO data.player_records (server_id, player_id, first_seen, last_seen, last_gamemode, last_ping)
            SELECT $1, up.player_id, $2, $2, i.last_gamemode, i.last_ping
            FROM upserted_players up
            JOIN input i ON up.uuid = i.uuid
            ON CONFLICT (server_id, player_id) DO UPDATE SET
                last_seen = EXCLUDED.last_seen,
                last_gamemode = EXCLUDED.last_gamemode,
                last_ping = EXCLUDED.last_ping
            "#,
            server_id,
            self.time,
            &uuids,
            &names,
            &is_online_modes,
            &gamemodes as &[Option<GameMode>],
            &pings as &[Option<i32>]
        )
            .execute(&mut *tx)
            .await?;

        Ok(())
    }
}
impl NewServerCommands {
    pub async fn sync(self, server_id: i32, tx: &mut PgConnection) -> Result<(), sqlx::Error> {
        if self.commands.is_empty() {
            return Ok(());
        }

        sqlx::query!(
            r#"
            WITH inserted AS (
                INSERT INTO data.commands (name)
                SELECT UNNEST($2::text[])
                ON CONFLICT (name) DO NOTHING
                RETURNING id
            ),
            existing AS (
                SELECT id
                FROM data.commands
                WHERE name = ANY($2::text[])
            ),
            all_ids AS (
                SELECT id FROM inserted
                UNION ALL
                SELECT id FROM existing
            ),
            deleted_mappings AS (
                DELETE FROM data.server_commands
                WHERE server_id = $1 AND command_id NOT IN (SELECT id FROM all_ids)
            )
            INSERT INTO data.server_commands (server_id, command_id)
            SELECT $1, id FROM all_ids
            ON CONFLICT (server_id, command_id) DO NOTHING;
            "#,
            server_id,
            &self.commands
        )
        .execute(&mut *tx)
        .await?;
        Ok(())
    }
}

impl NewServerFeatures {
    pub async fn sync(self, server_id: i32, tx: &mut PgConnection) -> Result<(), sqlx::Error> {
        if self.features.is_empty() {
            return Ok(());
        }

        sqlx::query!(
            r#"
            WITH inserted AS (
                INSERT INTO data.features (identifier)
                SELECT UNNEST($2::text[])
                ON CONFLICT (identifier) DO NOTHING
                RETURNING id
            ),
            existing AS (
                SELECT id
                FROM data.features
                WHERE identifier = ANY($2::text[])
            ),
            all_ids AS (
                SELECT id FROM inserted
                UNION ALL
                SELECT id FROM existing
            ),
            deleted_mappings AS (
                DELETE FROM data.server_features
                WHERE server_id = $1 AND feature_id NOT IN (SELECT id FROM all_ids)
            )
            -- FIX applied here: previously it was inserting into server_commands
            INSERT INTO data.server_features (server_id, feature_id)
            SELECT $1, id FROM all_ids
            ON CONFLICT (server_id, feature_id) DO NOTHING;
            "#,
            server_id,
            &self.features
        )
        .execute(&mut *tx)
        .await?;
        Ok(())
    }
}

impl NewServerChannels {
    pub async fn sync(self, server_id: i32, tx: &mut PgConnection) -> Result<(), sqlx::Error> {
        if self.channels.is_empty() {
            return Ok(());
        }
        sqlx::query!(
            r#"
                WITH inserted AS (
                    INSERT INTO data.channels (identifier)
                    SELECT UNNEST($2::text[])
                    ON CONFLICT (identifier) DO NOTHING
                    RETURNING id
                ),
                existing AS (
                    SELECT id
                    FROM data.channels
                    WHERE identifier = ANY($2::text[])
                ),
                all_ids AS (
                    SELECT id FROM inserted
                    UNION ALL
                    SELECT id FROM existing
                ),
                deleted_mappings AS (
                    DELETE FROM data.server_channels
                    WHERE server_id = $1 AND channel_id NOT IN (SELECT id FROM all_ids)
                )
                INSERT INTO data.server_channels (server_id, channel_id)
                SELECT $1, id FROM all_ids
                ON CONFLICT (server_id, channel_id) DO NOTHING;
                "#,
            server_id,
            &self.channels
        )
        .execute(tx)
        .await?;
        Ok(())
    }
}

impl NewServerMods {
    pub async fn sync(self, server_id: i32, tx: &mut PgConnection) -> Result<(), sqlx::Error> {
        if self.mods_names.is_empty() {
            return Ok(());
        }
        sqlx::query!(
            r#"
                WITH inserted AS (
                    INSERT INTO data.mods (name)
                    SELECT UNNEST($2::text[])
                    ON CONFLICT (name) DO NOTHING
                    RETURNING id
                ),
                existing AS (
                    SELECT id
                    FROM data.mods
                    WHERE name = ANY($2::text[])
                ),
                all_ids AS (
                    SELECT id FROM inserted
                    UNION ALL
                    SELECT id FROM existing
                ),
                deleted_mappings AS (
                    DELETE FROM data.server_mods
                    WHERE server_id = $1 AND mod_id NOT IN (SELECT id FROM all_ids)
                )
                INSERT INTO data.server_mods (server_id, mod_id)
                SELECT $1, id FROM all_ids
                ON CONFLICT (server_id, mod_id) DO NOTHING;
                "#,
            server_id,
            &self.mods_names,
        )
        .execute(tx)
        .await?;
        Ok(())
    }
}

impl NewServerLinks {
    pub async fn sync(self, server_id: i32, tx: &mut PgConnection) -> Result<(), sqlx::Error> {
        if self.links.is_empty() {
            return Ok(());
        }

        sqlx::query!(
            r#"
            WITH input AS (
                SELECT unnest($2::text[]) AS url
            ),
            deleted AS (
                DELETE FROM data.links
                WHERE server_id = $1 AND url NOT IN (SELECT url FROM input)
            )
            INSERT INTO data.links (server_id, url)
            SELECT $1, url FROM input
            ON CONFLICT (server_id, url) DO NOTHING
            "#,
            server_id,
            &self.links
        )
        .execute(&mut *tx)
        .await?;
        Ok(())
    }
}

impl NewResourcePack {
    pub async fn upsert(self, server_id: i32, tx: &mut PgConnection) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO data.resource_packs (server_id, url, hash, forced) VALUES ($1, $2, $3, $4)
            ON CONFLICT (server_id) DO UPDATE SET
                url = EXCLUDED.url,
                hash = EXCLUDED.hash,
                forced = EXCLUDED.forced
            "#,
            server_id,
            self.url,
            self.hash,
            self.forced
        )
        .execute(&mut *tx)
        .await?;
        Ok(())
    }
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
