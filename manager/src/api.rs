mod manager;
mod open_api;
mod scheduling;
mod tracking;

use crate::player_tracking::PlayerTrackingService;
use crate::scan_jobs::WorkerManagerService;
use crate::scan_jobs::schedule::ScheduleService;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Router, middleware};
use open_api::ApiDoc;
use serde_json::json;
use std::sync::Arc;
use utoipa::OpenApi;

#[derive(Clone)]
struct AppState {
    manager: Arc<WorkerManagerService>,
    schedule_service: Arc<ScheduleService>,
    player_tracking_service: Option<Arc<PlayerTrackingService>>,
    auth_settings: AuthSettings,
}
#[derive(Clone)]
pub struct AuthSettings {
    token: Option<String>,
}
impl AuthSettings {
    pub fn new(token: Option<String>) -> AuthSettings {
        Self { token }
    }
}

async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    match state.auth_settings.token {
        None => Ok(next.run(req).await),
        Some(token) => {
            let expected_token = format!("Bearer {token}");
            if let Some(auth_header) = req.headers().get(axum::http::header::AUTHORIZATION) {
                if let Ok(auth_str) = auth_header.to_str() {
                    if auth_str == expected_token {
                        return Ok(next.run(req).await);
                    }
                }
            }
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

pub fn setup_router(
    manager: Arc<WorkerManagerService>,
    schedule_service: Arc<ScheduleService>,
    player_tracking_service: Option<Arc<PlayerTrackingService>>,
    auth_settings: AuthSettings,
) -> Router {
    let app_state = AppState {
        manager,
        schedule_service,
        player_tracking_service,
        auth_settings,
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
        .route("/api/schedules", post(scheduling::add_new))
        .route("/api/schedules/{name}/run", post(scheduling::run))
        .route("/api/schedules/{name}/delete", delete(scheduling::delete))
        .route("/api/schedules/{name}/stop", post(scheduling::stop))
        // tracking
        .route("/api/tracking", get(tracking::all_webhooks_info))
        .route(
            "/api/tracking/{webhook_name}",
            get(tracking::get_webhook_info),
        )
        .route("/api/tracking", post(tracking::add_webhook))
        .route(
            "/api/tracking/{webhook_name}",
            delete(tracking::webhook_delete),
        )
        .route(
            "/api/tracking/{webhook_name}/players/all",
            get(tracking::webhook_all_players_info),
        )
        .route(
            "/api/tracking/{webhook_name}/players",
            get(tracking::get_player_info),
        )
        .route(
            "/api/tracking/{webhook_name}/players",
            post(tracking::add_player),
        )
        .route(
            "/api/tracking/{webhook_name}/players",
            delete(tracking::delete_player_record),
        )
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ))
        .with_state(app_state)
        .merge(scalar_route)
        .merge(asset_route)
}
