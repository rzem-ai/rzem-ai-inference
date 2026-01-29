//! Model loading logic for FluxPipeline

use anyhow::Result;

#[allow(unused_imports)]
use tracing::{debug, info, warn};

use crate::models::{ClipTextEncoder, FluxTransformer, ModelPaths, ModelType, T5TextEncoder, VaeDecoder};
use crate::inference::stats::{GenerationStats, Timer};
use super::FluxPipeline;

/// Get current GPU memory stats using nvidia-smi
fn get_gpu_memory_stats() -> Option<(u64, u64, f32)> {
    let output = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=memory.used,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let line = stdout.lines().next().unwrap_or("");
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

            if parts.len() >= 2 {
                let mem_used = parts[0].parse::<u64>().ok()?;
                let mem_total = parts[1].parse::<u64>().ok()?;
                let mem_percent = (mem_used as f64 / mem_total as f64 * 100.0) as f32;

                // Return in bytes
                return Some((mem_used * 1024 * 1024, mem_total * 1024 * 1024, mem_percent));
            }
        }
        _ => {}
    }

    None
}

/// Format bytes into human-readable string
fn format_bytes(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    }
}

impl FluxPipeline {
    /// Load models if not already loaded, returning timing stats
    /// Each model is checked individually - only missing models are loaded
    /// Accepts optional custom ModelPaths for bundle/component override
    pub(crate) fn ensure_models_loaded_with_paths(&mut self, stats: &mut GenerationStats, custom_paths: Option<ModelPaths>) -> Result<()> {
        // Quick return if all models are already loaded
        if self.t5.is_some() && self.clip.is_some() && self.vae.is_some() && self.flux.is_some() {
            debug!("All models already loaded, skipping load");
            return Ok(());
        }

        let paths = if let Some(paths) = custom_paths {
            paths
        } else {
            ModelPaths::new()?
        };

        // Log bundle info
        info!(bundle_id = ?paths.bundle_id(), "Loading models from active bundle");

        // Validate model files exist
        if !paths.all_files_exist() {
            return Err(anyhow::anyhow!(
                "Active bundle '{}' has missing components. Please scan for models or activate a different bundle.",
                paths.bundle_id().unwrap_or("unknown")
            ));
        }

        // Model validation is now handled by bundle system

        let total_load_timer = Timer::start();
        let mut any_loaded = false;

        // Load each model component
        if self.t5.is_none() {
            self.load_t5_encoder(&paths, stats)?;
            any_loaded = true;
        } else {
            debug!("T5 encoder already loaded, skipping");
        }

        if self.clip.is_none() {
            self.load_clip_encoder(&paths, stats)?;
            any_loaded = true;
        } else {
            debug!("CLIP encoder already loaded, skipping");
        }

        if self.vae.is_none() {
            self.load_vae_decoder(&paths, stats)?;
            any_loaded = true;
        } else {
            debug!("VAE decoder already loaded, skipping");
        }

        if self.flux.is_none() {
            self.load_flux_transformer(&paths, stats)?;
            any_loaded = true;
        } else {
            debug!("FLUX transformer already loaded, skipping");
        }

        if any_loaded {
            stats.model_load_ms = Some(total_load_timer.stop());
            self.models_loaded_this_session = true;
        }

        Ok(())
    }

    /// Load models using bundle context
    pub(crate) fn ensure_models_loaded(&mut self, stats: &mut GenerationStats) -> Result<()> {
        // Get context (clone to avoid borrow issues)
        let context = self.bundle_context.clone()
            .ok_or_else(|| anyhow::anyhow!("No bundle context set - component IDs required"))?;

        // Create ModelPaths from bundle or individual components
        let paths = if let Some(bundle_id) = context.bundle_id {
            // Use bundle
            let db_path = ModelPaths::get_db_path()?;
            let db = crate::gallery::GalleryDb::new(&db_path)?;
            let bundle_info = db.get_bundle(&bundle_id)?;
            info!(bundle_id = %bundle_id, "Loading models from bundle");
            ModelPaths::from_bundle_info(&bundle_info)?
        } else {
            // Use individual component IDs
            let model_id = context.model_component_id.as_deref()
                .ok_or_else(|| anyhow::anyhow!("model_component_id required"))?;
            let t5_id = context.t5_component_id.as_deref()
                .ok_or_else(|| anyhow::anyhow!("t5_component_id required"))?;
            let clip_id = context.clip_component_id.as_deref()
                .ok_or_else(|| anyhow::anyhow!("clip_component_id required"))?;
            let vae_id = context.vae_component_id.as_deref()
                .ok_or_else(|| anyhow::anyhow!("vae_component_id required"))?;

            info!(
                model = model_id,
                t5 = t5_id,
                clip = clip_id,
                vae = vae_id,
                "Loading models from individual components"
            );

            ModelPaths::from_component_ids(model_id, Some(t5_id), Some(clip_id), Some(vae_id))?
        };

        // Load models with resolved paths
        self.ensure_models_loaded_with_paths(stats, Some(paths))
    }

    /// Ensure models are ready for generation (reload if unloaded)
    ///
    /// Call this before generation when using cached pipelines.
    pub fn ensure_ready_for_generation(&mut self, stats: &mut GenerationStats) -> Result<()> {
        self.ensure_models_loaded(stats)
    }

    /// Load T5 text encoder from bundle
    fn load_t5_encoder(&mut self, paths: &ModelPaths, stats: &mut GenerationStats) -> Result<()> {
        let t5_timer = Timer::start();
        let mem_before = get_gpu_memory_stats();

        let model_path = paths.t5_path()?;
        let tokenizer_path = paths.t5_tokenizer_path()?;
        info!(
            model = %model_path.display(),
            tokenizer = %tokenizer_path.display(),
            "Loading T5 encoder from bundle"
        );
        self.t5 = Some(T5TextEncoder::load(
            model_path,
            tokenizer_path,
            self.device.clone(),
        )?);

        let elapsed = t5_timer.stop();
        stats.t5_load_ms = Some(elapsed);

        // Log completion with memory stats
        if let Some((used, total, percent)) = get_gpu_memory_stats() {
            let delta = mem_before.map(|(before, _, _)| used as i64 - before as i64);
            if let Some(delta) = delta {
                info!(
                    elapsed_ms = elapsed,
                    gpu_used = format_bytes(used),
                    gpu_total = format_bytes(total),
                    gpu_percent = format!("{:.1}%", percent),
                    delta = format_bytes(delta.abs() as u64),
                    "T5 encoder loaded"
                );
            } else {
                info!(
                    elapsed_ms = elapsed,
                    gpu_used = format_bytes(used),
                    gpu_total = format_bytes(total),
                    gpu_percent = format!("{:.1}%", percent),
                    "T5 encoder loaded"
                );
            }
        } else {
            info!(elapsed_ms = elapsed, "T5 encoder loaded (GPU stats unavailable)");
        }

        Ok(())
    }

    /// Load CLIP text encoder
    fn load_clip_encoder(&mut self, paths: &ModelPaths, stats: &mut GenerationStats) -> Result<()> {
        let model_path = paths.clip_path()?.join("model.safetensors");
        let tokenizer_path = paths.tokenizer_path()?;
        info!(
            model = %model_path.display(),
            tokenizer = %tokenizer_path.display(),
            "Loading CLIP encoder from bundle"
        );
        let clip_timer = Timer::start();
        let mem_before = get_gpu_memory_stats();

        self.clip = Some(ClipTextEncoder::load(
            model_path,
            tokenizer_path,
            self.device.clone(),
        )?);

        let elapsed = clip_timer.stop();
        stats.clip_load_ms = Some(elapsed);

        // Log completion with memory stats
        if let Some((used, total, percent)) = get_gpu_memory_stats() {
            let delta = mem_before.map(|(before, _, _)| used as i64 - before as i64);
            if let Some(delta) = delta {
                info!(
                    elapsed_ms = elapsed,
                    gpu_used = format_bytes(used),
                    gpu_total = format_bytes(total),
                    gpu_percent = format!("{:.1}%", percent),
                    delta = format_bytes(delta.abs() as u64),
                    "CLIP encoder loaded"
                );
            } else {
                info!(
                    elapsed_ms = elapsed,
                    gpu_used = format_bytes(used),
                    gpu_total = format_bytes(total),
                    gpu_percent = format!("{:.1}%", percent),
                    "CLIP encoder loaded"
                );
            }
        } else {
            info!(elapsed_ms = elapsed, "CLIP encoder loaded (GPU stats unavailable)");
        }

        Ok(())
    }

    /// Load VAE decoder
    fn load_vae_decoder(&mut self, paths: &ModelPaths, stats: &mut GenerationStats) -> Result<()> {
        let model_path = paths.vae_path()?;
        info!(
            model = %model_path.display(),
            "Loading VAE decoder from bundle"
        );
        let vae_timer = Timer::start();
        let mem_before = get_gpu_memory_stats();

        self.vae = Some(VaeDecoder::load(
            &model_path,
            self.device.clone(),
        )?);

        let elapsed = vae_timer.stop();
        stats.vae_load_ms = Some(elapsed);

        // Log completion with memory stats
        if let Some((used, total, percent)) = get_gpu_memory_stats() {
            let delta = mem_before.map(|(before, _, _)| used as i64 - before as i64);
            if let Some(delta) = delta {
                info!(
                    elapsed_ms = elapsed,
                    gpu_used = format_bytes(used),
                    gpu_total = format_bytes(total),
                    gpu_percent = format!("{:.1}%", percent),
                    delta = format_bytes(delta.abs() as u64),
                    "VAE decoder loaded"
                );
            } else {
                info!(
                    elapsed_ms = elapsed,
                    gpu_used = format_bytes(used),
                    gpu_total = format_bytes(total),
                    gpu_percent = format!("{:.1}%", percent),
                    "VAE decoder loaded"
                );
            }
        } else {
            info!(elapsed_ms = elapsed, "VAE decoder loaded (GPU stats unavailable)");
        }

        Ok(())
    }

    /// Load FLUX transformer
    fn load_flux_transformer(&mut self, paths: &ModelPaths, stats: &mut GenerationStats) -> Result<()> {
        let flux_timer = Timer::start();
        let mem_before = get_gpu_memory_stats();

        // Check if LoRAs are active
        let has_loras = !self.active_loras.is_empty();

        // Load transformer from bundle (quantization now handled via component metadata)
        let model_path = paths.transformer_path()?;

        if has_loras {
            info!(
                model = %self.model_type,
                path = %model_path.display(),
                lora_count = self.active_loras.len(),
                "Loading transformer with LoRAs from bundle"
            );
            self.flux = Some(FluxTransformer::load_with_loras(
                model_path,
                self.device.clone(),
                self.model_type.clone(),
                &self.active_loras,
            )?);
        } else {
            info!(
                model = %self.model_type,
                path = %model_path.display(),
                "Loading transformer from bundle"
            );
            self.flux = Some(FluxTransformer::load(
                model_path,
                self.device.clone(),
                self.model_type.clone(),
            )?);
        }

        let elapsed = flux_timer.stop();
        stats.flux_load_ms = Some(elapsed);

        // Log completion with memory stats
        if let Some((used, total, percent)) = get_gpu_memory_stats() {
            let delta = mem_before.map(|(before, _, _)| used as i64 - before as i64);
            if let Some(delta) = delta {
                info!(
                    model = %self.model_type,
                    elapsed_ms = elapsed,
                    gpu_used = format_bytes(used),
                    gpu_total = format_bytes(total),
                    gpu_percent = format!("{:.1}%", percent),
                    delta = format_bytes(delta.abs() as u64),
                    "FLUX transformer loaded"
                );
            } else {
                info!(
                    model = %self.model_type,
                    elapsed_ms = elapsed,
                    gpu_used = format_bytes(used),
                    gpu_total = format_bytes(total),
                    gpu_percent = format!("{:.1}%", percent),
                    "FLUX transformer loaded"
                );
            }
        } else {
            info!(model = %self.model_type, elapsed_ms = elapsed, "FLUX transformer loaded (GPU stats unavailable)");
        }

        Ok(())
    }
}
