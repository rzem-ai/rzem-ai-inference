//! Model downloading for vision models
//!
//! Handles downloading Moondream weights from HuggingFace Hub
//! with progress tracking for UI feedback.

use anyhow::{Context, Result};
use hf_hub::api::tokio::Api;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tracing::{debug, info};

use super::models::moondream_repo_id;

/// Approximate size of Moondream 2 model in bytes (~1.8GB)
pub const MOONDREAM_SIZE_BYTES: u64 = 1_800_000_000;

/// Files required for Moondream model
const MOONDREAM_FILES: &[&str] = &[
    "model.safetensors",
    "tokenizer.json",
    "config.json",
];

/// Vision model download status
#[derive(Debug, Clone, serde::Serialize)]
pub struct VisionModelStatus {
    /// Whether the model is fully downloaded
    pub is_downloaded: bool,
    /// Download progress (0.0 - 1.0), None if not downloading
    pub download_progress: Option<f32>,
    /// Model size in bytes
    pub model_size: u64,
    /// Human-readable model size
    pub model_size_display: String,
    /// Any error message
    pub error: Option<String>,
}

impl Default for VisionModelStatus {
    fn default() -> Self {
        Self {
            is_downloaded: false,
            download_progress: None,
            model_size: MOONDREAM_SIZE_BYTES,
            model_size_display: format_bytes(MOONDREAM_SIZE_BYTES),
            error: None,
        }
    }
}

/// Format bytes as human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Check if Moondream model is fully downloaded
pub fn is_moondream_downloaded() -> bool {
    let api = match hf_hub::api::sync::Api::new() {
        Ok(api) => api,
        Err(_) => return false,
    };

    let repo = api.model(moondream_repo_id().to_string());

    // Check if all required files exist
    for file in MOONDREAM_FILES {
        if repo.get(file).is_err() {
            return false;
        }
    }

    true
}

/// Get the current vision model status
pub fn get_model_status() -> VisionModelStatus {
    VisionModelStatus {
        is_downloaded: is_moondream_downloaded(),
        download_progress: None,
        model_size: MOONDREAM_SIZE_BYTES,
        model_size_display: format_bytes(MOONDREAM_SIZE_BYTES),
        error: None,
    }
}

/// Get path to Moondream model in HuggingFace cache
pub fn get_moondream_cache_path() -> Result<PathBuf> {
    let api = hf_hub::api::sync::Api::new()?;
    let repo = api.model(moondream_repo_id().to_string());

    // Get the path by fetching model.safetensors (it will be in cache)
    let model_path = repo.get("model.safetensors")
        .context("Moondream model not found in cache")?;

    // Return parent directory
    Ok(model_path.parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| model_path))
}

/// Download Moondream model from HuggingFace Hub
///
/// Emits progress events via Tauri for UI feedback.
pub async fn download_moondream(app_handle: Option<&AppHandle>) -> Result<PathBuf> {
    info!("Starting Moondream 2 download");

    let api = Api::new()?;
    let repo = api.model(moondream_repo_id().to_string());

    let total_files = MOONDREAM_FILES.len();
    let mut downloaded_path = None;

    for (idx, file) in MOONDREAM_FILES.iter().enumerate() {
        let progress = idx as f32 / total_files as f32;

        // Emit progress event
        if let Some(handle) = app_handle {
            let _ = handle.emit("vision-model-download", serde_json::json!({
                "status": "downloading",
                "file": file,
                "progress": progress,
                "current": idx + 1,
                "total": total_files,
            }));
        }

        info!(file = %file, progress = %format!("{:.0}%", progress * 100.0), "Downloading");

        let path = repo.get(file).await
            .with_context(|| format!("Failed to download {}", file))?;

        if *file == "model.safetensors" {
            downloaded_path = Some(path);
        }
    }

    // Emit completion event
    if let Some(handle) = app_handle {
        let _ = handle.emit("vision-model-download", serde_json::json!({
            "status": "completed",
            "progress": 1.0,
        }));
    }

    info!("Moondream download complete");

    downloaded_path
        .map(|p| p.parent().map(|p| p.to_path_buf()).unwrap_or(p))
        .context("Model path not found after download")
}

/// Ensure Moondream weights are available, downloading if necessary
pub async fn ensure_moondream_weights(app_handle: Option<&AppHandle>) -> Result<PathBuf> {
    if is_moondream_downloaded() {
        debug!("Moondream already downloaded");
        return get_moondream_cache_path();
    }

    download_moondream(app_handle).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1_500_000), "1.4 MB");
        assert_eq!(format_bytes(1_800_000_000), "1.7 GB");
    }

    #[test]
    fn test_default_status() {
        let status = VisionModelStatus::default();
        assert!(!status.is_downloaded);
        assert!(status.download_progress.is_none());
        assert_eq!(status.model_size, MOONDREAM_SIZE_BYTES);
    }
}
