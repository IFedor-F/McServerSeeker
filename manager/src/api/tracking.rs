use crate::api::AppState;
use crate::player_tracking::tracking_service::PlayerTrackingServiceError;
use axum::extract::Query;
use axum::{
    Json,
    extract::{Path, State},
};
use data_core::api::manager::{PlayerTrackIdent, PlayerTrackInfo, WebhookInfo};

// Tracking endpoints
#[utoipa::path(
    get,
    path = "/api/tracking/{name}",
    params(
        ("name" = String, Path, description = "Name of the webhook")
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
    Path(name): Path<String>,
) -> Result<Json<WebhookInfo>, PlayerTrackingServiceError> {
    let s = app
        .player_tracking_service
        .ok_or(PlayerTrackingServiceError::Disabled)?;
    s.get_webhook_info(name).await.map(Json)
}

#[utoipa::path(
    get,
    path = "/api/tracking",
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
    path = "/api/tracking/{name}",
    params(
        ("name" = String, Path, description = "Name of the webhook")
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
    Path(name): Path<String>,
) -> Result<(), PlayerTrackingServiceError> {
    app.player_tracking_service
        .ok_or(PlayerTrackingServiceError::Disabled)?
        .delete_webhook(&name)
        .await
}

#[utoipa::path(
    get,
    path = "/api/tracking/{name}/players/all",
    params(
        ("name" = String, Path, description = "Name of the webhook")
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
    Path(name): Path<String>,
) -> Result<Json<Vec<PlayerTrackInfo>>, PlayerTrackingServiceError> {
    let s = app
        .player_tracking_service
        .ok_or(PlayerTrackingServiceError::Disabled)?;
    s.get_all_tracks_from_webhook(name).await.map(Json)
}

#[utoipa::path(
    get,
    path = "/api/tracking/{name}/players",
    params(
        ("name" = String, Path, description = "Name of the webhook"),
        PlayerTrackIdent
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
    Path(webhook_name): Path<String>,
    Query(data): Query<PlayerTrackIdent>,
) -> Result<Json<PlayerTrackInfo>, PlayerTrackingServiceError> {
    let s = app
        .player_tracking_service
        .ok_or(PlayerTrackingServiceError::Disabled)?;
    s.get_track_info(webhook_name, data).await.map(Json)
}

#[utoipa::path(
    post,
    path = "/api/tracking/{name}/players",
    params(
        ("name" = String, Path, description = "Name of the webhook"),
        PlayerTrackIdent
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
    Path(webhook_name): Path<String>,
    Query(data): Query<PlayerTrackIdent>,
) -> Result<(), PlayerTrackingServiceError> {
    let s = app
        .player_tracking_service
        .ok_or(PlayerTrackingServiceError::Disabled)?;
    s.add_player_track(webhook_name, data).await
}

#[utoipa::path(
    delete,
    path = "/api/tracking/{name}/players",
    params(
        ("webhook_name" = String, Path, description = "Name of the webhook to delete from"),
        PlayerTrackIdent
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
    Path(webhook_name): Path<String>,
    Query(data): Query<PlayerTrackIdent>,
) -> Result<(), PlayerTrackingServiceError> {
    let s = app
        .player_tracking_service
        .ok_or(PlayerTrackingServiceError::Disabled)?;
    s.remove_track(webhook_name, data).await
}
