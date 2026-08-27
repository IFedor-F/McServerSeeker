use crate::api::AppState;
use crate::scan_jobs::worker_manager::WorkerManagerError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{
    Json,
    extract::{Path, State},
};
use data_core::api::manager::{
    JobId, JobProgress, ManagerJobInfo, ManagerJobReq, ManagerScanOneReq, McServerData,
    WorkerStatus,
};

// Manager endpoints
#[utoipa::path(
    post,
    path = "/api/scan_one",
    request_body = ManagerScanOneReq,
    responses(
        (status = 200, description = "Scan job successfully started", body = McServerData),
        (status = 204, description = "No server data"),
        (status = 404, description = "Some worker in request wasn't find in configuration", body = WorkerManagerError),
        (status = 500, description = "Scan error", body = WorkerManagerError)
    ),
    tag = "manager"
)]
pub async fn scan_one(
    State(app): State<AppState>,
    Json(request): Json<ManagerScanOneReq>,
) -> Result<Response, WorkerManagerError> {
    let result = app.manager.run_one_scan(request).await?;

    match result {
        Some(data) => Ok((StatusCode::OK, Json(data)).into_response()),
        None => Ok(StatusCode::NO_CONTENT.into_response()),
    }
}
#[utoipa::path(
    post,
    path = "/api/jobs",
    request_body = ManagerJobReq,
    responses(
        (status = 200, description = "Scan job successfully started", body = JobId),
        (status = 404, description = "Some worker in request wasn't find in configuration", body = WorkerManagerError)
    ),
    tag = "manager"
)]
pub async fn job_new(
    State(app): State<AppState>,
    Json(man_job_req): Json<ManagerJobReq>,
) -> Result<Json<JobId>, WorkerManagerError> {
    app.manager.run_job(man_job_req).await.map(Json)
}

#[utoipa::path(
    delete,
    path = "/api/jobs/{id}",
    params(
        ("id" = JobId, Path, description = "The ID of the job to delete")
    ),
    responses(
        (status = 200, description = "Scan job was cancelled")
    ),
    tag = "manager"
)]
pub async fn cancel(State(app): State<AppState>, Path(id): Path<JobId>) {
    app.manager.cancel_job(id).await;
}

#[utoipa::path(
    get,
    path = "/api/jobs/{id}/progress",
    params(
        ("id" = JobId, Path, description = "The ID of the job to get progress for")
    ),
    responses(
        (status = 200, description = "Job progress", body = JobProgress),
        (status = 404, description = "Job not found")
    ),
    tag = "manager"
)]
pub async fn progress(
    State(app): State<AppState>,
    Path(job_id): Path<JobId>,
) -> Result<Json<JobProgress>, StatusCode> {
    if let Some(progress) = app.manager.get_job_progress(job_id).await {
        Ok(Json(progress))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[utoipa::path(
    get,
    path = "/api/jobs/{id}/info",
    params(
        ("id" = JobId, Path, description = "The ID of the job to retrieve info for")
    ),
    responses(
        (status = 200, description = "Job information", body = ManagerJobInfo),
        (status = 404, description = "Job not found")
    ),
    tag = "manager"
)]
pub async fn info(
    State(app): State<AppState>,
    Path(id): Path<JobId>,
) -> Result<Json<ManagerJobInfo>, StatusCode> {
    let info = app
        .manager
        .get_job_info(id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(info))
}

#[utoipa::path(
    get,
    path = "/api/jobs",
    responses(
        (status = 200, description = "List of all active jobs with info and progress", body = Vec<ManagerJobInfo>)
    ),
    tag = "manager"
)]
pub async fn all_info(State(app): State<AppState>) -> Json<Vec<ManagerJobInfo>> {
    Json(app.manager.get_all_jobs_info().await)
}

#[utoipa::path(
    get,
    path = "/api/workers",
    responses(
        (status = 200, description = "List of all workers with info", body = Vec<WorkerStatus>)
    ),
    tag = "manager"
)]
pub async fn workers_all_info(State(app): State<AppState>) -> Json<Vec<WorkerStatus>> {
    Json(app.manager.get_workers())
}
