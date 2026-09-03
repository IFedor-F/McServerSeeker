use crate::api::AppState;
use crate::player_tracking::tracking_service::PlayerTrackingServiceError;
use axum::extract::Query;
use axum::{Json, extract::State};
use data_core::api::manager::{PlayerTrackInfo, WebhookInfo};
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::IntoParams;
use uuid::Uuid;

#[derive(Debug, Deserialize, IntoParams)]
pub struct WebhookUrlQuery {
    pub webhook_url: Url,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, IntoParams)]
#[serde(deny_unknown_fields, try_from = "TrackIdentAndUrlRaw")]
pub struct TrackIdentAndUrlQuery {
    pub webhook_url: Url,
    pub name: Option<String>,
    pub uuid: Option<Uuid>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TrackIdentAndUrlRaw {
    webhook_url: Url,
    name: Option<String>,
    uuid: Option<Uuid>,
}
impl TryFrom<TrackIdentAndUrlRaw> for TrackIdentAndUrlQuery {
    type Error = &'static str;

    fn try_from(raw: TrackIdentAndUrlRaw) -> Result<Self, Self::Error> {
        if raw.name.is_none() && raw.uuid.is_none() {
            return Err("At least one of 'name' or 'uuid' must be provided");
        }
        if let Some(name) = &raw.name
            && name.is_empty()
        {
            return Err("'name' can't be empty");
        }

        Ok(Self {
            webhook_url: raw.webhook_url,
            name: raw.name,
            uuid: raw.uuid,
        })
    }
}

// Tracking endpoints
#[utoipa::path(
    get,
    path = "/api/tracking",
    params(
        WebhookUrlQuery
    ),
    responses(
        (status = 200, description = "Webhook info", body = Vec<WebhookInfo>),
        (status = 404, description = "Webhook with this name not found", body = PlayerTrackingServiceError),
        (status = 500, description = "Player tracking service internal server error", body = PlayerTrackingServiceError),
        (status = 503, description = "Player tracking service is disabled in config", body = PlayerTrackingServiceError)
    ),
    tag = "tracking"
)]
pub async fn get_webhook_info(
    State(app): State<AppState>,
    Query(w): Query<WebhookUrlQuery>,
) -> Result<Json<WebhookInfo>, PlayerTrackingServiceError> {
    let s = app
        .player_tracking_service
        .ok_or(PlayerTrackingServiceError::Disabled)?;
    s.get_webhook_info(&w.webhook_url).await.map(Json)
}

#[utoipa::path(
    get,
    path = "/api/tracking/all",
    responses(
        (status = 200, description = "List of webhooks with info", body = Vec<WebhookInfo>),
        (status = 500, description = "Player tracking service internal server error", body = PlayerTrackingServiceError),
        (status = 503, description = "Player tracking service is disabled in config", body = PlayerTrackingServiceError)
    ),
    tag = "tracking"
)]
pub async fn all_webhooks_info(
    State(app): State<AppState>,
) -> Result<Json<Vec<WebhookInfo>>, PlayerTrackingServiceError> {
    let s = app
        .player_tracking_service
        .ok_or(PlayerTrackingServiceError::Disabled)?;
    s.get_all_webhooks_info().await.map(Json)
}

#[utoipa::path(
    post,
    path = "/api/tracking",
    request_body = WebhookInfo,
    responses(
        (status = 200, description = "Webhook successfully added"),
        (status = 409, description = "Webhook with this name exists already", body = PlayerTrackingServiceError),
        (status = 500, description = "Player tracking service internal server error", body = PlayerTrackingServiceError),
        (status = 503, description = "Player tracking service is disabled in config", body = PlayerTrackingServiceError)
    ),
    tag = "tracking"
)]
pub async fn add_webhook(
    State(app): State<AppState>,
    Json(info): Json<WebhookInfo>,
) -> Result<(), PlayerTrackingServiceError> {
    let s = app
        .player_tracking_service
        .ok_or(PlayerTrackingServiceError::Disabled)?;
    s.add_webhook(info).await
}

#[utoipa::path(
    delete,
    path = "/api/tracking",
    params(
        WebhookUrlQuery
    ),
    responses(
        (status = 200, description = "Webhook successfully removed"),
        (status = 500, description = "Player tracking service internal server error", body = PlayerTrackingServiceError),
        (status = 503, description = "Player tracking service is disabled in config", body = PlayerTrackingServiceError)
    ),
    tag = "tracking"
)]
pub async fn webhook_delete(
    State(app): State<AppState>,
    Query(w): Query<WebhookUrlQuery>,
) -> Result<(), PlayerTrackingServiceError> {
    app.player_tracking_service
        .ok_or(PlayerTrackingServiceError::Disabled)?
        .delete_webhook(&w.webhook_url)
        .await
}

#[utoipa::path(
    get,
    path = "/api/tracking/players/all",
    params(
        WebhookUrlQuery
    ),
    responses(
        (status = 200, description = "List of player tracks info", body = Vec<PlayerTrackInfo>),
        (status = 404, description = "Webhook or player not found", body = PlayerTrackingServiceError),
        (status = 500, description = "Player tracking service internal server error", body = PlayerTrackingServiceError),
        (status = 503, description = "Player tracking service is disabled in config", body = PlayerTrackingServiceError)
    ),
    tag = "tracking"
)]
pub async fn webhook_all_players_info(
    State(app): State<AppState>,
    Query(w): Query<WebhookUrlQuery>,
) -> Result<Json<Vec<PlayerTrackInfo>>, PlayerTrackingServiceError> {
    let s = app
        .player_tracking_service
        .ok_or(PlayerTrackingServiceError::Disabled)?;
    s.get_all_tracks_from_webhook(&w.webhook_url)
        .await
        .map(Json)
}

#[utoipa::path(
    get,
    path = "/api/tracking/players",
    params(
        TrackIdentAndUrlQuery
    ),
    responses(
        (status = 200, description = "Player track info", body = PlayerTrackInfo),
        (status = 404, description = "Webhook with this name not found", body = PlayerTrackingServiceError),
        (status = 500, description = "Player tracking service internal server error", body = PlayerTrackingServiceError),
        (status = 503, description = "Player tracking service is disabled in config", body = PlayerTrackingServiceError)
    ),
    tag = "tracking"
)]
pub async fn get_player_info(
    State(app): State<AppState>,
    Query(data): Query<TrackIdentAndUrlQuery>,
) -> Result<Json<PlayerTrackInfo>, PlayerTrackingServiceError> {
    let s = app
        .player_tracking_service
        .ok_or(PlayerTrackingServiceError::Disabled)?;
    s.get_track_info(&data.webhook_url, data.name, data.uuid)
        .await
        .map(Json)
}

#[utoipa::path(
    post,
    path = "/api/tracking/players",
    params(
        TrackIdentAndUrlQuery
    ),
    responses(
        (status = 200, description = "Player track successfully added"),
        (status = 404, description = "Webhook with this name not found", body = PlayerTrackingServiceError),
        (status = 500, description = "Player tracking service internal server error", body = PlayerTrackingServiceError),
        (status = 503, description = "Player tracking service is disabled in config", body = PlayerTrackingServiceError)
    ),
    tag = "tracking"
)]
pub async fn add_player(
    State(app): State<AppState>,
    Query(data): Query<TrackIdentAndUrlQuery>,
) -> Result<(), PlayerTrackingServiceError> {
    let s = app
        .player_tracking_service
        .ok_or(PlayerTrackingServiceError::Disabled)?;
    s.add_player_track(&data.webhook_url, data.name, data.uuid)
        .await
}

#[utoipa::path(
    delete,
    path = "/api/tracking/players",
    params(
        TrackIdentAndUrlQuery
    ),
    responses(
        (status = 200, description = "Player track successfully deleted"),
        (status = 404, description = "Webhook with this name not found", body = PlayerTrackingServiceError),
        (status = 503, description = "Player tracking service is disabled in config", body = PlayerTrackingServiceError),
        (status = 500, description = "Player tracking service internal server error", body = PlayerTrackingServiceError)
    ),
    tag = "tracking"
)]
pub async fn delete_player_record(
    State(app): State<AppState>,
    Query(data): Query<TrackIdentAndUrlQuery>,
) -> Result<(), PlayerTrackingServiceError> {
    let s = app
        .player_tracking_service
        .ok_or(PlayerTrackingServiceError::Disabled)?;
    s.remove_track(&data.webhook_url, data.name, data.uuid)
        .await
}
