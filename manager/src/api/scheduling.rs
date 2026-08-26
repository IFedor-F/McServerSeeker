use crate::api::AppState;
use crate::scan_jobs::schedule::ScheduleManagerError;
use axum::{
    Json,
    extract::{Path, State},
};
use data_core::api::manager::ScheduleData;

// Scheduling endpoints
#[utoipa::path(
    get,
    path = "/api/schedules/{name}",
    params(
        ("name" = String, Path, description = "Name of the schedule")
    ),
    responses(
        (status = 200, description = "Schedule info", body = ScheduleData),
        (status = 404, description = "Schedule name not found", body = ScheduleManagerError)
    ),
    tag = "schedule"
)]

pub async fn get_info(
    State(app): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<ScheduleData>, ScheduleManagerError> {
    app.schedule_service.get_schedule(name).await.map(Json)
}

// Scheduling endpoints
#[utoipa::path(
    get,
    path = "/api/schedules",
    responses(
        (status = 200, description = "Schedule info", body = Vec<ScheduleData>),
    ),
    tag = "schedule"
)]

pub async fn get_all_info(State(app): State<AppState>) -> Json<Vec<ScheduleData>> {
    Json(app.schedule_service.get_all_schedules().await)
}

#[utoipa::path(
    put,
    path = "/api/schedules",
    request_body = ScheduleData,
    responses(
        (status = 200, description = "Schedule successfully added or updated"),
        (status = 404, description = "Worker in job request doesn't exist", body = ScheduleManagerError)
    ),
    tag = "schedule"
)]
pub async fn upsert(
    State(app): State<AppState>,
    Json(schedule_job_req): Json<ScheduleData>,
) -> Result<(), ScheduleManagerError> {
    app.schedule_service.upsert_schedule(schedule_job_req).await
}

#[utoipa::path(
    post,
    path = "/api/schedules/{name}/run",
    params(
        ("name" = String, Path, description = "Name of the schedule to run")
    ),
    responses(
        (status = 200, description = "Schedule run initiated"),
        (status = 404, description = "Schedule name not found", body = ScheduleManagerError)
    ),
    tag = "schedule"
)]
pub async fn run(
    State(app): State<AppState>,
    Path(name): Path<String>,
) -> Result<(), ScheduleManagerError> {
    app.schedule_service.run_schedule(name).await
}

#[utoipa::path(
    delete,
    path = "/api/schedules/{name}/delete",
    params(
        ("name" = String, Path, description = "Name of the schedule to remove")
    ),
    responses(
        (status = 200, description = "Schedule successfully removed"),
        (status = 404, description = "Schedule name not found", body = ScheduleManagerError)
    ),
    tag = "schedule"
)]
pub async fn delete(
    State(app): State<AppState>,
    Path(name): Path<String>,
) -> Result<(), ScheduleManagerError> {
    app.schedule_service.remove_schedule(&name).await
}

#[utoipa::path(
    post,
    path = "/api/schedules/{name}/stop",
    params(
        ("name" = String, Path, description = "Name of the schedule to stop")
    ),
    responses(
        (status = 200, description = "Schedule successfully stopped"),
        (status = 404, description = "Schedule name not found", body = ScheduleManagerError)
    ),
    tag = "schedule"
)]
pub async fn stop(
    State(app): State<AppState>,
    Path(job_name): Path<String>,
) -> Result<(), ScheduleManagerError> {
    app.schedule_service.stop_schedule(&job_name).await
}
