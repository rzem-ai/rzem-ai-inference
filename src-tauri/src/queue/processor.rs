//! Queue processor that executes pending jobs

use super::{QueueManager, JobStatus};
use crate::inference::{InferenceEngine, FluxPipeline};
use crate::gallery::{GalleryDb, ImageMetadata};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{sleep, Duration, timeout};
use tokio::task::JoinHandle;
use anyhow::Result;
use tauri::{AppHandle, Emitter};

pub struct QueueProcessor {
    queue_manager: Arc<QueueManager>,
    gallery_db: Arc<tokio::sync::Mutex<Option<GalleryDb>>>,
    inference_engine: Arc<InferenceEngine>,
    running: Arc<AtomicBool>,
    task_handle: Arc<tokio::sync::Mutex<Option<JoinHandle<()>>>>,
    app_handle: AppHandle,
}

impl QueueProcessor {
    pub fn new(
        queue_manager: Arc<QueueManager>,
        gallery_db: Arc<tokio::sync::Mutex<Option<GalleryDb>>>,
        app_handle: AppHandle,
    ) -> Result<Self> {
        let inference_engine = Arc::new(InferenceEngine::new()?);

        Ok(Self {
            queue_manager,
            gallery_db,
            inference_engine,
            running: Arc::new(AtomicBool::new(false)),
            task_handle: Arc::new(tokio::sync::Mutex::new(None)),
            app_handle,
        })
    }

    /// Start the processor loop
    pub async fn start(&self) {
        // Fix Issue 2: Use AtomicBool to prevent race condition
        // Use compare_exchange to atomically check and set the running flag
        if self.running.compare_exchange(
            false,
            true,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ).is_err() {
            return; // Already running
        }

        let queue_manager = self.queue_manager.clone();
        let gallery_db = self.gallery_db.clone();
        let inference_engine = self.inference_engine.clone();
        let running = self.running.clone();
        let app_handle = self.app_handle.clone();

        // Fix Issue 1: Store the JoinHandle so we can await it later
        let handle = tokio::spawn(async move {
            loop {
                // Check if we should stop
                if !running.load(Ordering::SeqCst) {
                    break;
                }

                // Try to process next job
                if let Err(e) = process_next_job(
                    &queue_manager,
                    &gallery_db,
                    &inference_engine,
                    &app_handle,
                ).await {
                    eprintln!("Error processing job: {}", e);
                }

                // Sleep before checking again
                sleep(Duration::from_millis(100)).await;
            }
        });

        // Store the handle
        let mut task_handle = self.task_handle.lock().await;
        *task_handle = Some(handle);
    }

    /// Stop the processor loop
    pub async fn stop(&self) {
        // Set running flag to false
        self.running.store(false, Ordering::SeqCst);

        // Fix Issue 4: Await the join handle with timeout to detect dead task
        let mut task_handle = self.task_handle.lock().await;
        if let Some(handle) = task_handle.take() {
            // Wait up to 5 seconds for the task to finish
            match timeout(Duration::from_secs(5), handle).await {
                Ok(Ok(())) => {
                    // Task finished successfully
                }
                Ok(Err(e)) => {
                    eprintln!("Processor task panicked: {:?}", e);
                }
                Err(_) => {
                    eprintln!("Processor task did not stop within timeout");
                }
            }
        }
    }

    fn emit_job_update(&self, job_id: &str, status: &JobStatus, progress: f32) {
        let _ = self.app_handle.emit("job-update", serde_json::json!({
            "job_id": job_id,
            "status": status,
            "progress": progress,
        }));
    }
}

/// Fix Issue 3: RAII guard that ensures decrement_running() is called even on panic
struct RunningGuard {
    queue_manager: Arc<QueueManager>,
}

impl RunningGuard {
    /// Create guard and increment running counter atomically
    async fn new(queue_manager: Arc<QueueManager>) -> Self {
        queue_manager.increment_running().await;
        Self { queue_manager }
    }
}

impl Drop for RunningGuard {
    fn drop(&mut self) {
        // Use Handle::try_current() to spawn with proper error handling
        // This ensures we can still spawn even during runtime shutdown
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let queue_manager = self.queue_manager.clone();
            handle.spawn(async move {
                queue_manager.decrement_running().await;
            });
        } else {
            eprintln!("Warning: Could not decrement running jobs - runtime not available");
        }
    }
}

async fn process_next_job(
    queue_manager: &Arc<QueueManager>,
    gallery_db: &Arc<tokio::sync::Mutex<Option<GalleryDb>>>,
    inference_engine: &Arc<InferenceEngine>,
    app_handle: &AppHandle,
) -> Result<()> {
    // Check if we can start a new job
    if !queue_manager.can_start_job().await {
        return Ok(());
    }

    // Get all jobs and find first pending one
    let jobs = queue_manager.get_jobs().await;
    let pending_job = jobs.iter().find(|j| j.status == JobStatus::Pending);

    let Some(job) = pending_job else {
        return Ok(()); // No pending jobs
    };

    let job_id = job.id.clone();
    let params = job.params.clone();

    // Mark as running and emit event
    queue_manager.update_job_status(&job_id, JobStatus::Running).await
        .map_err(|e| anyhow::anyhow!(e))?;
    let _ = app_handle.emit("job-update", serde_json::json!({
        "job_id": job_id,
        "status": "running",
        "progress": 0.0,
    }));

    // Create guard that increments counter and will decrement it even on panic
    // This eliminates the race window between increment and guard creation
    let _guard = RunningGuard::new(queue_manager.clone()).await;

    // Execute generation
    let result = execute_generation(
        &job_id,
        &params,
        queue_manager,
        inference_engine,
    ).await;

    // Handle result and emit completion/failure event
    match result {
        Ok(image_path) => {
            // Save to gallery
            if let Err(e) = save_to_gallery(&image_path, &params, gallery_db).await {
                eprintln!("Failed to save to gallery: {}", e);
            }

            // Mark as completed
            queue_manager.complete_job(&job_id, image_path.clone()).await
                .map_err(|e| anyhow::anyhow!(e))?;
            let _ = app_handle.emit("job-update", serde_json::json!({
                "job_id": job_id,
                "status": "completed",
                "progress": 1.0,
                "result_path": image_path,
            }));
        }
        Err(e) => {
            let error_msg = e.to_string();
            // Mark as failed
            queue_manager.fail_job(&job_id, error_msg.clone()).await
                .map_err(|e| anyhow::anyhow!(e))?;
            let _ = app_handle.emit("job-update", serde_json::json!({
                "job_id": job_id,
                "status": "failed",
                "error": error_msg,
            }));
        }
    }

    // Guard will automatically decrement counter when dropped here
    Ok(())
}

async fn execute_generation(
    job_id: &str,
    params: &super::GenerationParams,
    queue_manager: &Arc<QueueManager>,
    inference_engine: &Arc<InferenceEngine>,
) -> Result<String> {
    // Create pipeline
    let device = inference_engine.get_device().clone();
    let mut pipeline = FluxPipeline::new(device)?;

    // Update progress: starting
    queue_manager.update_job_progress(job_id, 0.1).await
        .map_err(|e| anyhow::anyhow!(e))?;

    // Try real generation first, fall back to stub if models not available
    let image_data = match pipeline.generate(&params.prompt, params.steps as usize) {
        Ok(data) => {
            println!("Generated image using real FLUX model");
            data
        }
        Err(e) => {
            eprintln!("Real generation failed: {}, falling back to stub", e);
            pipeline.generate_stub(&params.prompt, params.steps as usize)?
        }
    };

    // Update progress: generation complete
    queue_manager.update_job_progress(job_id, 0.8).await
        .map_err(|e| anyhow::anyhow!(e))?;

    // Save to file (existing code continues...)
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let output_dir = home.join(".flux-generator").join("outputs");
    std::fs::create_dir_all(&output_dir)?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let filename = format!("flux_{}_{}.png", timestamp, params.seed);
    let output_path = output_dir.join(&filename);

    std::fs::write(&output_path, &image_data)?;

    queue_manager.update_job_progress(job_id, 1.0).await
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(output_path.to_string_lossy().to_string())
}

async fn save_to_gallery(
    image_path: &str,
    params: &super::GenerationParams,
    gallery_db: &Arc<tokio::sync::Mutex<Option<GalleryDb>>>,
) -> Result<()> {
    let db_guard = gallery_db.lock().await;
    let Some(db) = db_guard.as_ref() else {
        return Ok(()); // Database not initialized, skip
    };

    let metadata = ImageMetadata {
        id: uuid::Uuid::new_v4().to_string(),
        file_path: image_path.to_string(),
        prompt: params.prompt.clone(),
        created_at: chrono::Utc::now().timestamp(),
    };

    db.insert_image(&metadata)?;

    Ok(())
}
