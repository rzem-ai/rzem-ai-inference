//! System information and health endpoints

use std::sync::Arc;
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::server::state::ServerState;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Serialize)]
pub struct ModelInfo {
    id: String,
    name: String,
    is_downloaded: bool,
}

#[derive(Serialize)]
pub struct SystemStatsResponse {
    cpu_usage: f32,
    memory_used: u64,
    memory_total: u64,
    memory_percent: f32,
    gpu_memory_used: Option<u64>,
    gpu_memory_total: Option<u64>,
    gpu_usage_percent: Option<f32>,
    gpu_name: Option<String>,
    is_generating: bool,
}

/// Health check endpoint
///
/// GET /api/v1/health
pub async fn health(
    State(_state): State<Arc<ServerState>>,
) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// List available models
///
/// GET /api/v1/models
pub async fn list_models(
    State(_state): State<Arc<ServerState>>,
) -> Result<Json<Vec<ModelInfo>>, StatusCode> {
    use crate::models::{ModelPaths, ModelType};

    let paths = ModelPaths::new()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let models = vec![
        ModelInfo {
            id: "schnell".to_string(),
            name: "FLUX Schnell".to_string(),
            is_downloaded: paths.all_files_exist(),
        },
        ModelInfo {
            id: "dev".to_string(),
            name: "FLUX Dev".to_string(),
            is_downloaded: paths.is_dev_downloaded(),
        },
    ];

    Ok(Json(models))
}

/// Get system resource statistics
///
/// GET /api/v1/system/stats
pub async fn get_stats(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<SystemStatsResponse>, StatusCode> {
    use sysinfo::System;

    let mut sys = System::new();
    sys.refresh_cpu_all();
    sys.refresh_memory();

    // Wait for CPU readings to stabilize
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    sys.refresh_cpu_all();

    // Calculate average CPU usage
    let cpu_usage: f32 = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>()
        / sys.cpus().len().max(1) as f32;

    let memory_used = sys.used_memory();
    let memory_total = sys.total_memory();
    let memory_percent = if memory_total > 0 {
        (memory_used as f64 / memory_total as f64 * 100.0) as f32
    } else {
        0.0
    };

    // Check if generating
    let tauri_state = state.tauri_state();
    let jobs = tauri_state.queue_manager.get_jobs().await;
    let is_generating = jobs.iter().any(|j| j.status == crate::queue::JobStatus::Running);

    // Try to get GPU stats
    let (gpu_memory_used, gpu_memory_total, gpu_usage_percent, gpu_name) = get_gpu_stats();

    Ok(Json(SystemStatsResponse {
        cpu_usage,
        memory_used,
        memory_total,
        memory_percent,
        gpu_memory_used,
        gpu_memory_total,
        gpu_usage_percent,
        gpu_name,
        is_generating,
    }))
}

/// Get GPU stats using nvidia-smi
fn get_gpu_stats() -> (Option<u64>, Option<u64>, Option<f32>, Option<String>) {
    let output = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=memory.used,memory.total,utilization.gpu,name",
            "--format=csv,noheader,nounits",
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let line = stdout.lines().next().unwrap_or("");
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

            if parts.len() >= 4 {
                let mem_used = parts[0].parse::<u64>().ok().map(|v| v * 1024 * 1024);
                let mem_total = parts[1].parse::<u64>().ok().map(|v| v * 1024 * 1024);
                let gpu_util = parts[2].parse::<f32>().ok();
                let gpu_name = Some(parts[3].to_string());

                return (mem_used, mem_total, gpu_util, gpu_name);
            }
        }
        _ => {}
    }

    (None, None, None, None)
}
