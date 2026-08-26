use super::{FoundedPlayer, WebHook};
use axum::Json;
use axum::http::StatusCode;
use data_core::manager_api::{PlayerTrackIdent, PlayerTrackInfo, WebhookInfo};
use data_core::sql;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use url::Url;
use uuid::Uuid;

#[derive(thiserror::Error, Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", content = "detail")]
pub enum PlayerTrackingServiceError {
    #[error("service is disabled")]
    Disabled,
    #[error("duplicate webhook name: {0}")]
    WebhookDuplicateName(String),
    #[error("can't find webhook with this name: {0}")]
    WebhookNotFound(String),
    #[error("can't find player with this data")]
    PlayerRecordNotFound {
        name: Option<String>,
        uuid: Option<Uuid>,
    },
    #[error("internal SQL server error")]
    SqlError,
}
type PtsError = PlayerTrackingServiceError;
impl axum::response::IntoResponse for PtsError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            PtsError::Disabled => StatusCode::SERVICE_UNAVAILABLE,
            PtsError::WebhookDuplicateName(_) => StatusCode::CONFLICT,
            PtsError::SqlError => StatusCode::INTERNAL_SERVER_ERROR,
            PtsError::PlayerRecordNotFound { .. } => StatusCode::NOT_FOUND,
            PtsError::WebhookNotFound(_) => StatusCode::NOT_FOUND,
        };

        let payload = json!({
            "message": self.to_string(),
            "error_data": self,
        });

        (status, Json(payload)).into_response()
    }
}

#[derive(Debug)]
pub struct PlayerTrackingService {
    db_pool: PgPool,
    interval: Duration,
}

impl PlayerTrackingService {
    pub fn new(db_pool: PgPool, interval: Duration) -> Self {
        Self { db_pool, interval }
    }
    pub async fn add_webhook(&self, info: WebhookInfo) -> Result<(), PtsError> {
        let result = sqlx::query!(
            r#"
            INSERT INTO analytics.webhooks (name, url) values ($1, $2)
            ON CONFLICT (name) DO NOTHING
            "#,
            info.webhook_name,
            info.url.as_str()
        )
        .execute(&self.db_pool)
        .await;
        match result {
            Ok(v) => {
                if v.rows_affected() == 0 {
                    Err(PtsError::WebhookDuplicateName(info.webhook_name))
                } else {
                    Ok(())
                }
            }
            Err(_) => Err(PtsError::SqlError),
        }
    }

    pub async fn get_webhook_info(&self, name: String) -> Result<WebhookInfo, PtsError> {
        let data: Option<_> = sqlx::query_as!(
            sql::analytics::Webhook,
            "SELECT * FROM analytics.webhooks WHERE name = $1",
            &name
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|_| PtsError::SqlError)?;
        let data = data.ok_or(PtsError::WebhookNotFound(name))?;

        Ok(WebhookInfo {
            url: try_parse_url(&data.url)?,
            webhook_name: data.name,
        })
    }

    pub async fn get_all_webhooks_info(&self) -> Result<Vec<WebhookInfo>, PtsError> {
        let result: Result<Vec<_>, _> =
            sqlx::query_as!(sql::analytics::Webhook, "SELECT * FROM analytics.webhooks")
                .fetch_all(&self.db_pool)
                .await;
        match result {
            Ok(data) => data
                .into_iter()
                .map(|v| {
                    Ok(WebhookInfo {
                        webhook_name: v.name,
                        url: try_parse_url(&v.url)?,
                    })
                })
                .collect(),

            Err(e) => {
                log::error!("database error while trying to get all webhooks info: {e}");
                Err(PtsError::SqlError)
            }
        }
    }

    pub async fn delete_webhook(&self, name: &str) -> Result<(), PtsError> {
        let result = sqlx::query!("DELETE FROM analytics.webhooks WHERE name = $1", name)
            .execute(&self.db_pool)
            .await;
        if let Err(e) = result {
            log::error!("database error while trying to delete webhook: {e}");
            Err(PtsError::SqlError)
        } else {
            Ok(())
        }
    }
    pub async fn add_player_track(
        &self,
        webhook_name: String,
        track: PlayerTrackIdent,
    ) -> Result<(), PtsError> {
        let result = sqlx::query!(
            r#"
            WITH w_id AS (
                SELECT id FROM analytics.webhooks WHERE name = $3
            ),
            insert AS (
                INSERT INTO analytics.player_tracks (name, uuid, webhook_id)
                SELECT $1, $2, id FROM w_id
                ON CONFLICT (name, uuid, webhook_id) DO NOTHING
            )
            SELECT id FROM w_id
            "#,
            track.name,
            track.uuid,
            &webhook_name
        )
        .fetch_optional(&self.db_pool)
        .await;

        match result {
            Ok(data) => data
                .ok_or(PtsError::WebhookNotFound(webhook_name))
                .map(|_| ()),
            Err(e) => {
                log::error!("database error while trying to add player track: {e}");
                Err(PtsError::SqlError)
            }
        }
    }
    pub async fn get_track_info(
        &self,
        webhook_name: String,
        track: PlayerTrackIdent,
    ) -> Result<PlayerTrackInfo, PtsError> {
        let PlayerTrackIdent { name, uuid } = track;

        let result = sqlx::query!(
            r#"
            SELECT t.name, t.uuid, t.last_send, s.ip, s.port
            FROM analytics.webhooks w
            LEFT JOIN analytics.player_tracks t
               ON t.webhook_id = w.id
               AND t.name IS NOT DISTINCT FROM $1
               AND t.uuid IS NOT DISTINCT FROM $2
            LEFT JOIN data.servers s
                ON t.last_server_id = s.id
            WHERE w.name = $3
            "#,
            &name as &Option<String>,
            &uuid as &Option<Uuid>,
            &webhook_name
        )
        .fetch_optional(&self.db_pool)
        .await;

        // sql query will return no rows if no such webhook_name in table
        match result {
            Ok(Some(row)) => {
                if row.name.is_none() && row.uuid.is_none() {
                    Err(PtsError::PlayerRecordNotFound { name, uuid })
                } else {
                    Ok(PlayerTrackInfo {
                        name: row.name,
                        uuid: row.uuid,
                        last_send: row.last_send,
                        last_server_ip: row.ip.map(|v| v.ip()),
                        last_server_port: row.port.map(|v| v as u16),
                    })
                }
            }
            Ok(None) => Err(PtsError::WebhookNotFound(webhook_name)),
            Err(e) => {
                log::error!("database error while trying to get track info: {e}");
                Err(PtsError::SqlError)
            }
        }
    }

    pub async fn get_all_tracks_from_webhook(
        &self,
        webhook_name: String,
    ) -> Result<Vec<PlayerTrackInfo>, PtsError> {
        let result: Result<Vec<_>, _> = sqlx::query!(
            r#"
            SELECT t.name, t.uuid, t.last_send, s.ip, s.port
            FROM analytics.webhooks w
                LEFT JOIN analytics.player_tracks t ON t.webhook_id = w.id
                LEFT JOIN data.servers s on t.last_server_id = s.id
            WHERE w.name = $1
            "#,
            &webhook_name
        )
        .fetch_all(&self.db_pool)
        .await;
        let tracks = match result {
            Ok(tracks) => tracks,
            Err(e) => {
                log::error!("database error while trying to get all webhook tracks info: {e}");
                return Err(PtsError::SqlError);
            }
        };

        // sql query will return no rows if no such webhook_name in table
        if tracks.is_empty() {
            return Err(PtsError::WebhookNotFound(webhook_name));
        }
        // if webhook_name exists, but no records in player_records table, will be one row with nulls
        if tracks.len() == 1 {
            if tracks[0].name.is_none() && tracks[0].uuid.is_none() {
                return Ok(vec![]);
            }
        }
        Ok(tracks
            .into_iter()
            .map(|row| PlayerTrackInfo {
                name: row.name,
                uuid: row.uuid,
                last_send: row.last_send,
                last_server_ip: row.ip.map(|v| v.ip()),
                last_server_port: row.port.map(|v| v as u16),
            })
            .collect())
    }
    pub async fn remove_track(
        &self,
        webhook_name: String,
        track: PlayerTrackIdent,
    ) -> Result<(), PtsError> {
        let result = sqlx::query!(
            r#"
            WITH target_webhook AS (
                SELECT id FROM analytics.webhooks WHERE name = $3
            ),
            deleted_track AS (
                DELETE FROM analytics.player_tracks
                WHERE name IS NOT DISTINCT FROM $1
                  AND uuid IS NOT DISTINCT FROM $2
                  AND webhook_id = (SELECT id FROM target_webhook)
                RETURNING id
            )
            SELECT EXISTS(SELECT 1 FROM target_webhook) AS "webhook_exists!"
            "#,
            track.name,
            track.uuid,
            webhook_name
        )
        .fetch_one(&self.db_pool)
        .await;

        match result {
            Ok(result) => {
                if result.webhook_exists {
                    Ok(())
                } else {
                    Err(PtsError::WebhookNotFound(webhook_name))
                }
            }
            Err(e) => {
                log::error!("database error while trying to remove track: {e}");
                Err(PtsError::SqlError)
            }
        }
    }
    pub async fn run_tracking(&self) {
        loop {
            tokio::time::sleep(self.interval).await;
            let records = sqlx::query!(
                r#"
                WITH ranked_records AS (
                    SELECT pt.id AS track_id,
                    pt.webhook_id AS webhook_id,
                    pr.server_id as active_server_id,
                    p.name AS player_name,
                    p.uuid AS player_uuid,
                    s.ip AS server_ip,
                    s.port AS server_port,
                    pr.last_seen AS last_seen,

                    ROW_NUMBER() OVER (
                        PARTITION BY pt.id
                        ORDER BY pr.last_seen DESC NULLS LAST
                    ) as rn
                    FROM analytics.player_tracks pt
                    JOIN data.players p ON p.uuid = pt.uuid OR (pt.uuid IS NULL AND p.name = pt.name)
                    JOIN data.player_records pr ON p.id = pr.player_id
                    JOIN data.servers s ON pr.server_id = s.id
                    WHERE pr.last_seen = s.last_seen
                )
                UPDATE analytics.player_tracks pt
                SET last_server_id = rr.active_server_id, last_send = NOW()
                FROM ranked_records rr
                JOIN analytics.webhooks w ON rr.webhook_id = w.id
                WHERE pt.id = rr.track_id AND rr.rn = 1 AND rr.active_server_id IS DISTINCT FROM pt.last_server_id
                RETURNING w.url, rr.player_uuid, rr.player_name, rr.server_ip, rr.server_port, rr.last_seen
                "#
            )
                .fetch_all(&self.db_pool)
                .await;
            let records = match records {
                Ok(v) => v,
                Err(e) => {
                    log::error!("error occurred while checking database: {e}");
                    continue;
                }
            };

            let webhook_map: HashMap<String, HashSet<FoundedPlayer>> = records
                .into_iter()
                .map(|r| {
                    (
                        r.url,
                        FoundedPlayer {
                            uuid: r.player_uuid,
                            name: r.player_name,
                            ip: r.server_ip.ip(),
                            port: r.server_port,
                            last_seen: r.last_seen,
                        },
                    )
                })
                .into_grouping_map()
                .collect();

            for (webhook_url, players) in webhook_map {
                let url = match Url::parse(&webhook_url) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("can't parse webhook url from database: {e}");
                        continue;
                    }
                };
                tokio::spawn(async move {
                    WebHook::new(url).send_players(players).await;
                });
            }
        }
    }
}

fn try_parse_url(url: &str) -> Result<Url, PtsError> {
    match url.parse() {
        Ok(v) => Ok(v),
        Err(e) => {
            log::error!(
                "invalid data in database, broken url: {}\nparse error: {e}",
                url
            );
            Err(PtsError::SqlError)
        }
    }
}
