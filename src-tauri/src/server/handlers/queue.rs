//! Queue management API endpoints

use std::sync::Arc;
use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json,
};
use tracing::{info, error};
use tauri::Emitter;

use crate::server::state::ServerState;
use crate::queue::GenerationJob;

/// Get all jobs in the queue
///
/// GET /api/v1/queue
pub async fn list_jobs(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<Vec<GenerationJob>>, StatusCode> {
    let tauri_state = state.tauri_state();
    let jobs = tauri_state.queue_manager.get_jobs().await;

    Ok(Json(jobs))
}

/// Get a specific job by ID
///
/// GET /api/v1/queue/:job_id
pub async fn get_job(
    State(state): State<Arc<ServerState>>,
    Path(job_id): Path<String>,
) -> Result<Json<GenerationJob>, StatusCode> {
    let tauri_state = state.tauri_state();

    match tauri_state.queue_manager.get_job(&job_id).await {
        Some(job) => Ok(Json(job)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Cancel a pending job
///
/// DELETE /api/v1/queue/:job_id
pub async fn cancel_job(
    State(state): State<Arc<ServerState>>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!(job_id = %job_id, "Cancelling job");

    let tauri_state = state.tauri_state();
    let cancelled = tauri_state.queue_manager.cancel_job(&job_id).await;

    if cancelled {
        // Emit event for cancelled job
        if let Err(e) = tauri_state.app_handle.emit("job-update", serde_json::json!({
            "job_id": &job_id,
            "status": "cancelled",
        })) {
            error!(error = %e, "Failed to emit job-update event");
        }

        Ok(Json(serde_json::json!({
            "success": true,
            "message": "Job cancelled"
        })))
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}
