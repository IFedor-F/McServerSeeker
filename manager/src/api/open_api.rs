use super::manager;
use super::scheduling;
use super::tracking;

use crate::player_tracking::tracking_service::PlayerTrackingServiceError;
use crate::scan_jobs::schedule::ScheduleManagerError;
use crate::scan_jobs::worker_manager::WorkerManagerError;
use data_core::api::manager::{
    JobId, JobProgress, ManagerJobInfo, ManagerJobReq, ManagerScanOneReq, McServerData,
    PlayerTrackInfo, ScheduleData, WebhookInfo, WorkerStatus,
};
use utoipa::OpenApi;

// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        // manager
        manager::scan_one,
        manager::all_info,
        manager::job_new,
        manager::cancel,
        manager::progress,
        manager::info,
        manager::workers_all_info,
        // scheduling
        scheduling::get_info,
        scheduling::get_all_info,
        scheduling::add_new,
        scheduling::run,
        scheduling::delete,
        scheduling::stop,
        // tracking
        tracking::all_webhooks_info,
        tracking::get_webhook_info,
        tracking::add_webhook,
        tracking::webhook_delete,
        tracking::webhook_all_players_info,
        tracking::get_player_info,
        tracking::add_player,
        tracking::delete_player_record,
    ),
    components(
        schemas(
            ManagerJobReq,
            ManagerScanOneReq,
            JobId,
            JobProgress,
            WorkerStatus,
            ManagerJobInfo,
            ScheduleData,
            McServerData,
            PlayerTrackInfo,
            WebhookInfo,
            WorkerManagerError,
            ScheduleManagerError,
            PlayerTrackingServiceError
        )
    ),
    tags(
        (name = "manager", description = "Job execution and worker management"),
        (name = "schedule", description = "Task scheduling endpoints"),
        (name = "tracking", description = "Player tracking and webhook management")
    )
)]
pub struct ApiDoc;
