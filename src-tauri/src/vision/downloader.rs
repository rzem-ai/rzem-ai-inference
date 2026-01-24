//! Model downloading for vision models
//!
//! Handles downloading Moondream weights from HuggingFace Hub
//! with progress tracking for UI feedback.

use anyhow::{Context, Result};
use hf_hub::api::tokio::Api;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tracing::{debug, info, warn};

use super::models::moondream_repo_id;

/// Clean up stale lock files from HuggingFace cache
///
/// Lock files are left behind when downloads are interrupted.
/// This function removes them so downloads can proceed.
fn cleanup_stale_locks() -> Result<usize> {
    let cache_dir = dirs::cache_dir()
        .context("Could not find cache directory")?
        .join("huggingface/hub/models--santiagomed--candle-moondream");

    if !cache_dir.exists() {
        return Ok(0);
    }

    let mut removed = 0;

    // Clean locks in blobs directory
    let blobs_dir = cache_dir.join("blobs");
    if blobs_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&blobs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "lock") {
                    // Check if lock is stale (older than 5 minutes)
                    if let Ok(metadata) = path.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            let age = std::time::SystemTime::now()
                                .duration_since(modified)
                                .unwrap_or_default();

                            if age.as_secs() > 300 {
                                info!(path = %path.display(), age_secs = age.as_secs(), "Removing stale lock file");
                                if std::fs::remove_file(&path).is_ok() {
                                    removed += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Also check refs directory
    let refs_dir = cache_dir.join("refs");
    if refs_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&refs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "lock") {
                    if let Ok(metadata) = path.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            let age = std::time::SystemTime::now()
                                .duration_since(modified)
                                .unwrap_or_default();

                            if age.as_secs() > 300 {
                                info!(path = %path.display(), age_secs = age.as_secs(), "Removing stale lock file");
                                if std::fs::remove_file(&path).is_ok() {
                                    removed += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(removed)
}

/// Force remove all lock files (use when user explicitly requests retry)
pub fn force_cleanup_locks() -> Result<usize> {
    let cache_dir = dirs::cache_dir()
        .context("Could not find cache directory")?
        .join("huggingface/hub/models--santiagomed--candle-moondream");

    if !cache_dir.exists() {
        return Ok(0);
    }

    let mut removed = 0;

    // Remove ALL lock files regardless of age
    for subdir in ["blobs", "refs", "snapshots"] {
        let dir = cache_dir.join(subdir);
        if dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "lock") {
                        info!(path = %path.display(), "Force removing lock file");
                        if std::fs::remove_file(&path).is_ok() {
                            removed += 1;
                        }
                    }
                }
            }
        }
    }

    Ok(removed)
}

/// Approximate size of Moondream quantized model in bytes (~1.51GB)
pub const MOONDREAM_SIZE_BYTES: u64 = 1_510_000_000;

/// Files required for Moondream quantized model (GGUF format)
const MOONDREAM_FILES: &[&str] = &[
    "model-q4_0.gguf",
    "tokenizer.json",
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
///
/// This checks the filesystem directly rather than using hf_hub API
/// to avoid network requests just for status checks.
pub fn is_moondream_downloaded() -> bool {
    // First try direct filesystem check (fast, no network)
    if let Some(cache_dir) = dirs::cache_dir() {
        let model_dir = cache_dir.join("huggingface/hub/models--santiagomed--candle-moondream/snapshots");

        if model_dir.exists() {
            // Find the latest snapshot directory
            if let Ok(entries) = std::fs::read_dir(&model_dir) {
                for entry in entries.flatten() {
                    let snapshot_dir = entry.path();
                    if snapshot_dir.is_dir() {
                        // Check if all required files exist in this snapshot
                        let all_exist = MOONDREAM_FILES.iter().all(|file| {
                            snapshot_dir.join(file).exists()
                        });

                        if all_exist {
                            debug!(
                                snapshot = %snapshot_dir.display(),
                                "Moondream model found via filesystem check"
                            );
                            return true;
                        }
                    }
                }
            }
        }
    }

    // Fallback to hf_hub API check (may involve network)
    let api = match hf_hub::api::sync::Api::new() {
        Ok(api) => api,
        Err(e) => {
            debug!(error = %e, "Failed to create HF API for model check");
            return false;
        }
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

    // Get the path by fetching model-q4_0.gguf (it will be in cache)
    let model_path = repo.get("model-q4_0.gguf")
        .context("Moondream model not found in cache")?;

    // Return parent directory
    Ok(model_path.parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| model_path))
}

/// Download a single file with retry logic
async fn download_file_with_retry(
    repo: &hf_hub::api::tokio::ApiRepo,
    file: &str,
    max_retries: u32,
) -> Result<PathBuf> {
    let mut last_error = None;

    for attempt in 1..=max_retries {
        match repo.get(file).await {
            Ok(path) => {
                info!(file = %file, path = %path.display(), attempt = attempt, "Downloaded successfully");
                return Ok(path);
            }
            Err(e) => {
                warn!(
                    file = %file,
                    attempt = attempt,
                    max_retries = max_retries,
                    error = %e,
                    "Download attempt failed"
                );
                last_error = Some(e);

                // Wait before retrying (exponential backoff)
                if attempt < max_retries {
                    let delay = std::time::Duration::from_millis(500 * (1 << attempt));
                    info!(delay_ms = delay.as_millis(), "Waiting before retry");
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    Err(last_error.unwrap()).with_context(|| format!("Failed to download {} after {} attempts", file, max_retries))
}

/// Download Moondream model from HuggingFace Hub
///
/// Emits progress events via Tauri for UI feedback.
pub async fn download_moondream(app_handle: Option<&AppHandle>) -> Result<PathBuf> {
    info!("Starting Moondream 2 download from {}", moondream_repo_id());

    // Clean up any stale lock files from previous interrupted downloads
    match cleanup_stale_locks() {
        Ok(0) => debug!("No stale lock files found"),
        Ok(n) => info!(count = n, "Cleaned up stale lock files"),
        Err(e) => warn!(error = %e, "Failed to clean up stale locks, continuing anyway"),
    }

    let api = Api::new()
        .context("Failed to initialize HuggingFace Hub API")?;
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

        let path = match download_file_with_retry(&repo, file, 3).await {
            Ok(path) => path,
            Err(e) => {
                // Log detailed error information
                tracing::error!(
                    file = %file,
                    error = %e,
                    error_debug = ?e,
                    "Failed to download file from HuggingFace Hub"
                );

                // Emit error event
                if let Some(handle) = app_handle {
                    let _ = handle.emit("vision-model-download", serde_json::json!({
                        "status": "error",
                        "file": file,
                        "error": format!("{}", e),
                    }));
                }

                return Err(e).with_context(|| {
                    format!(
                        "Failed to download '{}' from {}. \
                        This could be due to network issues, rate limiting, or the file may have been moved. \
                        Try again in a few moments.",
                        file,
                        moondream_repo_id()
                    )
                });
            }
        };

        if *file == "model-q4_0.gguf" {
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
        assert_eq!(format_bytes(1_510_000_000), "1.4 GB");
    }

    #[test]
    fn test_default_status() {
        let status = VisionModelStatus::default();
        assert!(!status.is_downloaded);
        assert!(status.download_progress.is_none());
        assert_eq!(status.model_size, MOONDREAM_SIZE_BYTES);
    }
}
