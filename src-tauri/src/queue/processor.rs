//! Queue processor that executes pending jobs

use super::{QueueManager, JobStatus};
use crate::inference::{InferenceEngine, GenerationStats, ModelCache, ModelCacheConfig};
use crate::db::{InferenceDb, ImageMetadata};
use crate::models::{LoraManager, ModelType, ModelPaths};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{sleep, Duration, timeout};
use tokio::task::JoinHandle;
use anyhow::Result;
use tauri::{AppHandle, Emitter};
use tracing::{debug, error, info, warn};

/// Infer model type from transformer component
async fn infer_model_type_from_component(component_id: &str) -> Result<ModelType> {
    let db_path = ModelPaths::get_db_path()?;
    let db = InferenceDb::new(&db_path)?;
    let component = db.get_component(component_id)?;

    let arch = component.architecture.unwrap_or_default().to_lowercase();

    if arch.contains("schnell") {
        Ok(ModelType::schnell())
    } else if arch.contains("dev") {
        Ok(ModelType::dev())
    } else if arch.contains("z-image") || arch.contains("zimage") {
        Ok(ModelType::zimage_turbo())
    } else {
        Ok(ModelType::schnell()) // Default
    }
}

pub struct QueueProcessor {
    queue_manager: Arc<QueueManager>,
    inference_db: Arc<tokio::sync::Mutex<Option<InferenceDb>>>,
    inference_engine: Arc<InferenceEngine>,
    model_cache: Arc<ModelCache>,
    lora_manager: Arc<tokio::sync::Mutex<LoraManager>>,
    running: Arc<AtomicBool>,
    task_handle: Arc<tokio::sync::Mutex<Option<JoinHandle<()>>>>,
    app_handle: AppHandle,
    ws_state: Option<Arc<crate::server::ws_state::WsState>>,
}

impl QueueProcessor {
    pub fn new(
        queue_manager: Arc<QueueManager>,
        inference_db: Arc<tokio::sync::Mutex<Option<InferenceDb>>>,
        app_handle: AppHandle,
        ws_state: Option<Arc<crate::server::ws_state::WsState>>,
    ) -> Result<Self> {
        let inference_engine = Arc::new(InferenceEngine::new()?);

        // Create model cache with default config
        let device = inference_engine.get_device().clone();
        let model_cache = ModelCache::new(device, ModelCacheConfig::default());

        // Create LoRA manager
        let lora_manager = Arc::new(tokio::sync::Mutex::new(LoraManager::new(inference_db.clone())?));

        Ok(Self {
            queue_manager,
            inference_db,
            inference_engine,
            model_cache,
            lora_manager,
            running: Arc::new(AtomicBool::new(false)),
            task_handle: Arc::new(tokio::sync::Mutex::new(None)),
            app_handle,
            ws_state,
        })
    }

    /// Get a reference to the LoRA manager
    pub fn lora_manager(&self) -> &Arc<tokio::sync::Mutex<LoraManager>> {
        &self.lora_manager
    }

    /// Get a reference to the model cache for external configuration
    pub fn model_cache(&self) -> &Arc<ModelCache> {
        &self.model_cache
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
        let inference_db = self.inference_db.clone();
        let model_cache = self.model_cache.clone();
        let lora_manager = self.lora_manager.clone();
        let running = self.running.clone();
        let app_handle = self.app_handle.clone();
        let ws_state = self.ws_state.clone();

        // Spawn idle timeout checker
        let model_cache_for_timeout = self.model_cache.clone();
        let running_for_timeout = self.running.clone();
        tokio::spawn(async move {
            loop {
                // Check every 60 seconds
                sleep(Duration::from_secs(60)).await;

                // Stop if processor stopped
                if !running_for_timeout.load(Ordering::SeqCst) {
                    break;
                }

                // Check and handle idle timeout
                model_cache_for_timeout.check_idle_timeout().await;
            }
        });

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
                    &inference_db,
                    &model_cache,
                    &lora_manager,
                    &app_handle,
                    &ws_state,
                ).await {
                    error!(error = %e, "Error processing job");
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
                    error!(error = ?e, "Processor task panicked");
                }
                Err(_) => {
                    warn!("Processor task did not stop within timeout");
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
            warn!("Could not decrement running jobs - runtime not available");
        }
    }
}

async fn process_next_job(
    queue_manager: &Arc<QueueManager>,
    inference_db: &Arc<tokio::sync::Mutex<Option<InferenceDb>>>,
    model_cache: &Arc<ModelCache>,
    lora_manager: &Arc<tokio::sync::Mutex<LoraManager>>,
    app_handle: &AppHandle,
    ws_state: &Option<Arc<crate::server::ws_state::WsState>>,
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
    let image_id = job.image_id.clone();

    // Update database: PENDING → PROCESSING
    if let Some(ref img_id) = image_id {
        let db_guard = inference_db.lock().await;
        if let Some(db) = db_guard.as_ref() {
            let _ = db.update_image_status(img_id, "processing");
        }
    }

    // Mark as running and emit event
    queue_manager.update_job_status(&job_id, JobStatus::Running).await
        .map_err(|e| anyhow::anyhow!(e))?;

    emit_job_event(app_handle, ws_state, &job_id, "running", 0.0, None, None).await;

    // Create guard that increments counter and will decrement it even on panic
    // This eliminates the race window between increment and guard creation
    let _guard = RunningGuard::new(queue_manager.clone()).await;

    // Execute generation with progress callbacks
    let result = execute_generation(
        &job_id,
        &params,
        queue_manager,
        model_cache,
        lora_manager,
        app_handle,
    ).await;

    // Handle result and emit completion/failure event
    match result {
        Ok((image_path, stats)) => {
            // Update database: PROCESSING → COMPLETED
            if let Some(ref img_id) = image_id {
                let db_guard = inference_db.lock().await;
                if let Some(db) = db_guard.as_ref() {
                    // Generate thumbnail
                    let thumbnail_path = match generate_thumbnail(&image_path, 400) {
                        Ok(path) => Some(path),
                        Err(e) => {
                            warn!(error = ?e, "Failed to generate thumbnail");
                            None
                        }
                    };

                    // Get file metadata
                    let file_size = std::fs::metadata(&image_path)
                        .map(|m| m.len() as i64)
                        .unwrap_or(0);

                    // Update image on completion
                    if let Err(e) = db.update_image_on_completion(
                        img_id,
                        &image_path,
                        thumbnail_path,
                        params.width as i32,
                        params.height as i32,
                        file_size,
                        stats.total_ms as i64,
                    ) {
                        warn!(error = %e, "Failed to update image on completion");
                    }

                    // Store generation stats
                    if let Err(e) = db.insert_generation_stats(img_id, &stats) {
                        warn!(error = %e, "Failed to insert generation stats");
                    }

                    // Auto-tag if enabled (runs asynchronously, non-blocking)
                    let inference_db = inference_db.clone();
                    let app_handle = app_handle.clone();
                    let image_path_clone = image_path.clone();
                    let img_id_clone = img_id.clone();
                    tokio::spawn(async move {
                        auto_tag_if_enabled(&img_id_clone, &image_path_clone, &inference_db, &app_handle).await;
                    });
                }
            }

            // Mark as completed
            queue_manager.complete_job(&job_id, image_path.clone()).await
                .map_err(|e| anyhow::anyhow!(e))?;

            emit_job_event(app_handle, ws_state, &job_id, "completed", 1.0, Some(&image_path), None).await;
        }
        Err(e) => {
            let error_msg = e.to_string();

            // Update database: PROCESSING → FAILED
            if let Some(ref img_id) = image_id {
                let db_guard = inference_db.lock().await;
                if let Some(db) = db_guard.as_ref() {
                    let _ = db.update_image_on_failure(img_id, &error_msg);
                }
            }

            // Mark as failed
            queue_manager.fail_job(&job_id, error_msg.clone()).await
                .map_err(|e| anyhow::anyhow!(e))?;

            emit_job_event(app_handle, ws_state, &job_id, "failed", 0.0, None, Some(&error_msg)).await;
        }
    }

    // Guard will automatically decrement counter when dropped here
    Ok(())
}

async fn execute_generation(
    job_id: &str,
    params: &super::GenerationParams,
    queue_manager: &Arc<QueueManager>,
    model_cache: &Arc<ModelCache>,
    lora_manager: &Arc<tokio::sync::Mutex<LoraManager>>,
    app_handle: &AppHandle,
) -> Result<(String, GenerationStats)> {
    use crate::inference::{GenerationProgress, ImageMetadata};
    use crate::models::LoraAdapter;
    use candle_core::DType;

    // Determine model type from params or infer from components
    let model_type: ModelType = if let Some(ref model_type_str) = params.model_type {
        model_type_str.parse().unwrap_or(ModelType::schnell())
    } else {
        // Infer from transformer component
        infer_model_type_from_component(&params.model_component_id).await.unwrap_or(ModelType::schnell())
    };

    // Log bundle/component selection
    if let Some(ref bundle_id) = params.bundle_id {
        info!(
            bundle_id = %bundle_id,
            model_type = ?model_type,
            "Using bundle for generation"
        );
    } else {
        info!(
            model_component = %params.model_component_id,
            t5 = %params.t5_component_id,
            clip = %params.clip_component_id,
            vae = %params.vae_component_id,
            model_type = ?model_type,
            "Using individual components for generation"
        );
    }

    // Load LoRA adapters if specified
    let mut loaded_loras: Vec<(Arc<LoraAdapter>, f32)> = Vec::new();
    if !params.loras.is_empty() {
        let device = model_cache.device();
        let dtype = if device.is_cuda() { DType::BF16 } else { DType::F32 };

        let mut lora_mgr = lora_manager.lock().await;
        for lora_config in &params.loras {
            match lora_mgr.get_or_load_lora(&lora_config.id, device, dtype).await {
                Ok(adapter) => {
                    info!(
                        lora_id = %lora_config.id,
                        strength = lora_config.strength,
                        "Loaded LoRA adapter"
                    );
                    loaded_loras.push((adapter, lora_config.strength));
                }
                Err(e) => {
                    warn!(
                        lora_id = %lora_config.id,
                        error = %e,
                        "Failed to load LoRA adapter, skipping"
                    );
                }
            }
        }
    }

    // Get or create cached pipeline for this model type
    let created_new = model_cache.get_or_create_pipeline(model_type.clone()).await?;
    if created_new {
        info!(model = ?model_type, "Created new pipeline");
    } else {
        debug!(model = ?model_type, "Reusing cached pipeline");
    }

    // Set component context and LoRAs on the pipeline
    model_cache.with_pipeline(|pipeline| {
        use crate::inference::flux_pipeline::BundleContext;

        // Always set bundle context with component IDs
        pipeline.set_bundle_context(BundleContext {
            bundle_id: params.bundle_id.clone(),
            model_component_id: Some(params.model_component_id.clone()),
            t5_component_id: Some(params.t5_component_id.clone()),
            clip_component_id: Some(params.clip_component_id.clone()),
            vae_component_id: Some(params.vae_component_id.clone()),
        });

        // Set LoRAs (will trigger reload if changed, including removal)
        pipeline.set_loras(loaded_loras.clone());

        Ok(())
    }).await?;

    // Clone values for the progress callback closure
    let job_id_clone = job_id.to_string();
    let app_handle_clone = app_handle.clone();
    let queue_manager_clone = queue_manager.clone();

    // Progress callback that emits Tauri events
    let on_progress = move |progress: GenerationProgress| {
        // Emit detailed progress event for frontend
        let _ = app_handle_clone.emit("job-progress", serde_json::json!({
            "job_id": job_id_clone,
            "stage": progress.stage,
            "stage_progress": progress.stage_progress,
            "overall_progress": progress.overall_progress,
            "message": progress.message,
            "eta_seconds": progress.eta_seconds,
            "current_step": progress.current_step,
            "total_steps": progress.total_steps,
            "preview_data": progress.preview_data,
        }));

        // Also update job progress in queue manager (fire-and-forget)
        let qm = queue_manager_clone.clone();
        let jid = job_id_clone.clone();
        let overall = progress.overall_progress;
        tokio::spawn(async move {
            let _ = qm.update_job_progress(&jid, overall).await;
        });
    };

    // Generate using FLUX model with progress callbacks
    // Use cfg_scale as guidance (FLUX typically uses 4.0 for Schnell)
    let guidance = if params.cfg_scale > 0.0 { params.cfg_scale } else { 4.0 };
    // Convert seed to u64 (params.seed is i64, use absolute value for consistency)
    let seed = if params.seed >= 0 { params.seed as u64 } else { rand::random::<u64>() };

    // Get sampler and scheduler from params (with defaults)
    let sampler = params.sampler.unwrap_or_default();
    let scheduler = params.scheduler.unwrap_or_default();

    // Create metadata for embedding in the PNG
    let model_name = if let Some(ref bundle_id) = params.bundle_id {
        format!("bundle:{}", bundle_id)
    } else {
        format!("component:{}", params.model_component_id)
    };

    // Convert LoRA configs to metadata format
    let loras = params.loras.iter()
        .map(|lora| crate::inference::metadata::LoraInfo {
            id: lora.id.clone(),
            strength: lora.strength,
        })
        .collect();

    let metadata = ImageMetadata {
        prompt: params.prompt.clone(),
        negative_prompt: params.negative_prompt.clone(),
        steps: params.steps,
        cfg_scale: guidance,
        width: params.width,
        height: params.height,
        seed: params.seed,
        model: model_name,
        sampler: Some(sampler.to_string()),
        scheduler: Some(scheduler.to_string()),
        loras,
    };

    // Use the cached pipeline for generation
    // Choose between standard and staged loading based on VRAM availability
    debug!("Calling with_pipeline");
    let result = model_cache.with_pipeline(|pipeline| {
        debug!("Inside with_pipeline, checking if staged loading is needed");

        // Check if staged loading is required (FLUX-dev on <34GB GPU)
        let use_staged = pipeline.needs_staged_loading();
        if use_staged {
            info!("Using staged loading for memory-constrained GPU (FLUX-dev on <34GB VRAM)");
        }

        let gen_result = if use_staged {
            // Staged loading: load encoders, encode, unload, load transformer
            pipeline.generate_with_staged_loading(
                &params.prompt,
                params.steps as usize,
                params.width as usize,
                params.height as usize,
                guidance,
                seed,
                Some(metadata),
                sampler,
                scheduler,
                on_progress,
            )
        } else {
            // Standard loading: all models loaded together
            debug!("Using standard loading (sufficient VRAM or non-dev model)");
            pipeline.generate_with_progress(
                &params.prompt,
                params.steps as usize,
                params.width as usize,
                params.height as usize,
                guidance,
                seed,
                Some(metadata),
                sampler,
                scheduler,
                on_progress,
            )
        };

        if let Err(ref e) = gen_result {
            error!(error = ?e, "Generation error");
        }
        gen_result
    }).await;

    if let Err(ref e) = result {
        error!(error = ?e, "with_pipeline error");
    }
    let result = result?;
    debug!(size_bytes = result.image_data.len(), "Generation successful");

    // Apply post-generation cleanup based on cache config
    model_cache.apply_post_generation_cleanup().await;

    // Save to file
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let output_dir = home.join(".rzem-ai-inference").join("outputs");
    std::fs::create_dir_all(&output_dir)?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let filename = format!("flux_{}_{}.png", timestamp, params.seed);
    let output_path = output_dir.join(&filename);

    std::fs::write(&output_path, &result.image_data)?;

    // Final progress update
    queue_manager.update_job_progress(job_id, 1.0).await
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok((output_path.to_string_lossy().to_string(), result.stats))
}

/// Generate a cover/thumbnail image for gallery display
fn generate_thumbnail(image_path: &str, thumbnail_size: u32) -> Result<String> {
    use image::imageops::FilterType;
    use std::path::Path;

    let path = Path::new(image_path);
    let img = image::open(path)?;

    // Create thumbnail maintaining aspect ratio, fitting within thumbnail_size x thumbnail_size
    let thumbnail = img.resize(thumbnail_size, thumbnail_size, FilterType::Lanczos3);

    // Create thumbnail path: same directory, with _cover suffix
    let stem = path.file_stem()
        .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?
        .to_string_lossy();
    let thumbnail_filename = format!("{}_cover.jpg", stem);
    let thumbnail_path = path.parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?
        .join(&thumbnail_filename);

    // Save as JPEG with good quality (smaller file size than PNG)
    thumbnail.save(&thumbnail_path)?;

    Ok(thumbnail_path.to_string_lossy().to_string())
}

// save_to_gallery function removed - images are now created upfront
// and updated on completion via update_image_on_completion()

/// Auto-tag a newly generated image if enabled in settings
async fn auto_tag_if_enabled(
    image_id: &str,
    image_path: &str,
    inference_db: &Arc<tokio::sync::Mutex<Option<InferenceDb>>>,
    app_handle: &AppHandle,
) {
    use crate::vision::{AutoTagSettings, TaggingBackend, ClaudeTagger, TagWithConfidence};

    // Phase 1: Get settings (with lock)
    let settings: AutoTagSettings = {
        let db_guard = inference_db.lock().await;
        let Some(db) = db_guard.as_ref() else {
            return; // Database not initialized
        };
        db.get_auto_tag_settings().unwrap_or_default()
    };

    // Check if auto-tagging on generation is enabled
    if !settings.enabled || !settings.auto_tag_on_generation {
        return;
    }

    // Check if we have an available backend
    let effective_backend = settings.effective_backend();
    let Some(backend) = effective_backend else {
        debug!("Auto-tagging skipped: no backend available");
        return;
    };

    info!(
        image_id = %image_id,
        backend = %backend,
        "Auto-tagging generated image"
    );

    // Phase 2: Run tagging (without lock for Claude's async API call)
    let tags_result: Result<Vec<TagWithConfidence>, String> = match backend {
        TaggingBackend::Local => {
            // Local tagger is currently a stub - skip
            debug!("Local tagger not yet implemented, skipping auto-tag");
            return;
        }
        TaggingBackend::Claude => {
            let Some(api_key) = settings.claude_api_key.clone() else {
                warn!("Claude API key not configured");
                return;
            };

            let tagger = ClaudeTagger::new(api_key);
            let path = std::path::Path::new(image_path);

            match tagger.extract_tags(path).await {
                Ok(mut tags) => {
                    // Filter by confidence threshold
                    tags.retain(|t| t.confidence >= settings.min_confidence);
                    Ok(tags)
                }
                Err(e) => Err(e.to_string()),
            }
        }
    };

    // Phase 3: Save tags to database (with lock)
    match tags_result {
        Ok(tags) => {
            let tag_count = tags.len();
            let db_guard = inference_db.lock().await;
            if let Some(db) = db_guard.as_ref() {
                if let Err(e) = db.bulk_add_tags_with_category(image_id, &tags) {
                    warn!(error = %e, "Failed to save auto-tags to database");
                } else {
                    info!(image_id = %image_id, tag_count = tag_count, "Auto-tagging complete");

                    // Emit event so frontend can update
                    let _ = app_handle.emit("auto-tag-complete", serde_json::json!({
                        "image_id": image_id,
                        "tags": tags,
                        "backend": backend.to_string(),
                    }));
                }
            }
        }
        Err(e) => {
            warn!(image_id = %image_id, error = %e, "Auto-tagging failed");
            let _ = app_handle.emit("auto-tag-failed", serde_json::json!({
                "image_id": image_id,
                "error": e,
            }));
        }
    }
}

/// Helper function to emit job events to both Tauri and WebSocket
async fn emit_job_event(
    app_handle: &AppHandle,
    ws_state: &Option<Arc<crate::server::ws_state::WsState>>,
    job_id: &str,
    status: &str,
    progress: f32,
    result_path: Option<&str>,
    error: Option<&str>,
) {
    // Emit to Tauri (for desktop UI)
    let mut event_data = serde_json::json!({
        "job_id": job_id,
        "status": status,
        "progress": progress,
    });

    // Include result_path if provided
    if let Some(path) = result_path {
        event_data["result_path"] = serde_json::json!(path);
    }

    // Include error if provided
    if let Some(err) = error {
        event_data["error"] = serde_json::json!(err);
    }

    let _ = app_handle.emit("job-update", event_data);

    // Emit to WebSocket (if in server mode)
    if let Some(ws_state) = ws_state {
        let message = if status == "completed" && result_path.is_some() {
            crate::server::websocket::ServerMessage::JobComplete {
                job_id: job_id.to_string(),
                result_url: format!("/api/v1/files/{}",
                    std::path::Path::new(result_path.unwrap())
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                ),
            }
        } else if status == "failed" && error.is_some() {
            crate::server::websocket::ServerMessage::JobFailed {
                job_id: job_id.to_string(),
                error: error.unwrap().to_string(),
            }
        } else {
            crate::server::websocket::ServerMessage::JobProgress {
                job_id: job_id.to_string(),
                status: status.to_string(),
                progress,
                stage: None,
                current_step: None,
                total_steps: None,
            }
        };

        crate::server::websocket::broadcast_job_update(ws_state, job_id, message).await;
    }
}
