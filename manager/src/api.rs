mod manager;
mod open_api;
mod scheduling;
mod tracking;

use crate::player_tracking::PlayerTrackingService;
use crate::scan_jobs::WorkerManagerService;
use crate::scan_jobs::schedule::ScheduleService;
use axum::Router;
use axum::routing::{delete, get, post, put};
use open_api::ApiDoc;
use serde_json::json;
use std::sync::Arc;
use utoipa::OpenApi;

#[derive(Clone)]
struct AppState {
    manager: Arc<WorkerManagerService>,
    schedule_service: Arc<ScheduleService>,
    player_tracking_service: Option<Arc<PlayerTrackingService>>,
}

pub fn setup_router(
    manager: Arc<WorkerManagerService>,
    schedule_service: Arc<ScheduleService>,
    player_tracking_service: Option<Arc<PlayerTrackingService>>,
) -> Router {
    let app_state = AppState {
        manager,
        schedule_service,
        player_tracking_service,
    };
    let openapi = ApiDoc::openapi();
    let configuration = json!({
      "layout": "classic",
      "telemetry": false,
      "externalUrls": { },
      "slug": "serverseeker-manager-api",
      "title": "ServerSeeker Manager API",
      "agent": { "disabled": true },
      "content": openapi
    });
    let (scalar_route, asset_route) = scalar_api_reference::axum::routes("/scalar", &configuration);
    Router::new()
        // manager
        .route("/api/jobs", get(manager::all_info))
        .route("/api/jobs/{id}/info", get(manager::info))
        .route("/api/jobs/{id}/progress", get(manager::progress))
        .route("/api/jobs", post(manager::job_new))
        .route("/api/jobs/{id}", delete(manager::cancel))
        .route("/api/scan_one", post(manager::scan_one))
        .route("/api/workers", get(manager::workers_all_info))
        // scheduling
        .route("/api/schedules", get(scheduling::get_all_info))
        .route("/api/schedules/{name}", get(scheduling::get_info))
        .route("/api/schedules", put(scheduling::upsert))
        .route("/api/schedules/{name}/run", post(scheduling::run))
        .route("/api/schedules/{name}/delete", delete(scheduling::delete))
        .route("/api/schedules/{name}/stop", post(scheduling::stop))
        // tracking
        .route("/api/tracking", get(tracking::all_webhooks_info))
        .route("/api/tracking/{name}", get(tracking::get_webhook_info))
        .route("/api/tracking", post(tracking::add_webhook))
        .route("/api/tracking/{name}", delete(tracking::webhook_delete))
        .route(
            "/api/tracking/{name}/players/all",
            get(tracking::webhook_all_players_info),
        )
        .route(
            "/api/tracking/{name}/players",
            get(tracking::get_player_info),
        )
        .route("/api/tracking/{name}/players", post(tracking::add_player))
        .route(
            "/api/tracking/{name}/players",
            delete(tracking::delete_player_record),
        )
        .with_state(app_state)
        .merge(scalar_route)
        .merge(asset_route)
}
