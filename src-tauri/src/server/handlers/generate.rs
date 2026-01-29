//! Image generation API endpoints

use std::sync::Arc;
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use tauri::Emitter;

use crate::server::state::ServerState;
use crate::queue::GenerationParams;

#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    pub job_id: String,
    pub status: String,
    pub queue_position: usize,
}

/// Submit a new image generation job
///
/// POST /api/v1/generate
pub async fn generate(
    State(state): State<Arc<ServerState>>,
    Json(params): Json<GenerationParams>,
) -> Result<Json<GenerateResponse>, StatusCode> {
    info!(
        prompt = %params.prompt,
        bundle_id = ?params.bundle_id,
        model_component = %params.model_component_id,
        "Received generation request"
    );

    let tauri_state = state.tauri_state();

    // Add job to queue (reuse existing Tauri command logic)
    let job_id = tauri_state.queue_manager.add_job(params).await;

    // Emit event for new job (same as Tauri command)
    if let Err(e) = tauri_state.app_handle.emit("job-update", serde_json::json!({
        "job_id": &job_id,
        "status": "pending",
        "progress": 0.0,
    })) {
        error!(error = %e, "Failed to emit job-update event");
    }

    // Get queue position
    let jobs = tauri_state.queue_manager.get_jobs().await;
    let queue_position = jobs.iter()
        .filter(|j| matches!(j.status, crate::queue::JobStatus::Pending))
        .position(|j| j.id == job_id)
        .unwrap_or(0);

    Ok(Json(GenerateResponse {
        job_id,
        status: "pending".to_string(),
        queue_position,
    }))
}
