//! Queue processor that executes pending jobs

use super::{QueueManager, JobStatus};
use crate::inference::{InferenceEngine, FluxPipeline};
use crate::gallery::{GalleryDb, ImageMetadata};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use anyhow::Result;

pub struct QueueProcessor {
    queue_manager: Arc<QueueManager>,
    gallery_db: Arc<tokio::sync::Mutex<Option<GalleryDb>>>,
    inference_engine: Arc<InferenceEngine>,
    running: Arc<tokio::sync::Mutex<bool>>,
}

impl QueueProcessor {
    pub fn new(
        queue_manager: Arc<QueueManager>,
        gallery_db: Arc<tokio::sync::Mutex<Option<GalleryDb>>>,
    ) -> Result<Self> {
        let inference_engine = Arc::new(InferenceEngine::new()?);

        Ok(Self {
            queue_manager,
            gallery_db,
            inference_engine,
            running: Arc::new(tokio::sync::Mutex::new(false)),
        })
    }

    /// Start the processor loop
    pub async fn start(&self) {
        let mut running = self.running.lock().await;
        if *running {
            return; // Already running
        }
        *running = true;
        drop(running); // Release lock

        let queue_manager = self.queue_manager.clone();
        let gallery_db = self.gallery_db.clone();
        let inference_engine = self.inference_engine.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            loop {
                // Check if we should stop
                if !*running.lock().await {
                    break;
                }

                // Try to process next job
                if let Err(e) = process_next_job(
                    &queue_manager,
                    &gallery_db,
                    &inference_engine,
                ).await {
                    eprintln!("Error processing job: {}", e);
                }

                // Sleep before checking again
                sleep(Duration::from_millis(100)).await;
            }
        });
    }

    /// Stop the processor loop
    pub async fn stop(&self) {
        let mut running = self.running.lock().await;
        *running = false;
    }
}

async fn process_next_job(
    queue_manager: &Arc<QueueManager>,
    gallery_db: &Arc<tokio::sync::Mutex<Option<GalleryDb>>>,
    inference_engine: &Arc<InferenceEngine>,
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

    // Mark as running and increment counter
    queue_manager.update_job_status(&job_id, JobStatus::Running).await
        .map_err(|e| anyhow::anyhow!(e))?;
    queue_manager.increment_running().await;

    // Execute generation
    let result = execute_generation(
        &job_id,
        &params,
        queue_manager,
        inference_engine,
    ).await;

    // Decrement counter
    queue_manager.decrement_running().await;

    // Handle result
    match result {
        Ok(image_path) => {
            // Save to gallery
            if let Err(e) = save_to_gallery(&image_path, &params, gallery_db).await {
                eprintln!("Failed to save to gallery: {}", e);
            }

            // Mark as completed
            queue_manager.complete_job(&job_id, image_path).await
                .map_err(|e| anyhow::anyhow!(e))?;
        }
        Err(e) => {
            // Mark as failed
            queue_manager.fail_job(&job_id, e.to_string()).await
                .map_err(|e| anyhow::anyhow!(e))?;
        }
    }

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
    let pipeline = FluxPipeline::new(device)?;

    // Update progress: starting
    queue_manager.update_job_progress(job_id, 0.1).await
        .map_err(|e| anyhow::anyhow!(e))?;

    // Generate image (currently stub)
    let image_data = pipeline.generate_stub(&params.prompt, params.steps as usize)?;

    // Update progress: generation complete
    queue_manager.update_job_progress(job_id, 0.8).await
        .map_err(|e| anyhow::anyhow!(e))?;

    // Determine output path
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let output_dir = home.join(".flux-generator").join("outputs");

    // Create output directory
    std::fs::create_dir_all(&output_dir)?;

    // Generate filename with timestamp and seed
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let filename = format!("flux_{}_{}.png", timestamp, params.seed);
    let output_path = output_dir.join(&filename);

    // Save PNG image data
    std::fs::write(&output_path, &image_data)?;

    // Update progress: complete
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
