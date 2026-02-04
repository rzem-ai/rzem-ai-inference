//! Model loading logic for FluxPipeline

use anyhow::Result;
use std::sync::Arc;

#[allow(unused_imports)]
use tracing::{debug, info, warn};

use crate::models::{ClipTextEncoder, FluxTransformer, ModelPaths, T5TextEncoder, VaeDecoder};
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

/// Required VRAM threshold (in GB) above which staged loading is used
const STAGED_LOADING_VRAM_THRESHOLD_GB: u64 = 34;

impl FluxPipeline {
    /// Check if staged loading is required based on available VRAM
    ///
    /// Returns true if total required VRAM exceeds available.
    /// FLUX-dev needs ~34GB for all models simultaneously.
    /// With staged loading, peak is ~24GB (just transformer + VAE).
    pub fn needs_staged_loading(&self) -> bool {
        // Only FLUX-dev needs staged loading (schnell transformer is smaller)
        if self.model_type.id() != "dev" {
            return false;
        }

        // Check GPU memory
        if let Some((_, total_bytes, _)) = get_gpu_memory_stats() {
            let total_gb = total_bytes / (1024 * 1024 * 1024);
            // Need ~34GB for all models, use staged if less available
            return total_gb < STAGED_LOADING_VRAM_THRESHOLD_GB;
        }

        // If we can't query GPU memory, assume staged loading needed for dev
        // This is safer - prevents OOM on unknown hardware
        true
    }

    /// Load only text encoders (T5 + CLIP) for prompt encoding
    ///
    /// Used in staged loading to encode prompts before loading the transformer.
    pub(crate) fn ensure_text_encoders_loaded(&mut self, stats: &mut GenerationStats) -> Result<()> {
        // Get paths from bundle context
        let context = self.bundle_context.clone()
            .ok_or_else(|| anyhow::anyhow!("No bundle context set - component IDs required"))?;

        let paths = if let Some(bundle_id) = context.bundle_id {
            let db_path = ModelPaths::get_db_path()?;
            let db = crate::db::InferenceDb::new(&db_path)?;
            let bundle_info = db.get_bundle(&bundle_id)?;
            ModelPaths::from_bundle_info(&bundle_info)?
        } else {
            let model_id = context.model_component_id.as_deref()
                .ok_or_else(|| anyhow::anyhow!("model_component_id required"))?;
            let t5_id = context.t5_component_id.as_deref()
                .ok_or_else(|| anyhow::anyhow!("t5_component_id required"))?;
            let clip_id = context.clip_component_id.as_deref()
                .ok_or_else(|| anyhow::anyhow!("clip_component_id required"))?;
            let vae_id = context.vae_component_id.as_deref()
                .ok_or_else(|| anyhow::anyhow!("vae_component_id required"))?;

            let db_path = ModelPaths::get_db_path()?;
            let db = crate::db::InferenceDb::new(&db_path)?;
            ModelPaths::from_component_ids(&db, model_id, t5_id, clip_id, vae_id)?
        };

        let total_load_timer = Timer::start();
        let mut any_loaded = false;

        // Load T5 if not present
        if self.t5.is_none() {
            self.load_t5_encoder(&paths, stats)?;
            any_loaded = true;
        } else {
            debug!("T5 encoder already loaded, skipping");
        }

        // Load CLIP if not present
        if self.clip.is_none() {
            self.load_clip_encoder(&paths, stats)?;
            any_loaded = true;
        } else {
            debug!("CLIP encoder already loaded, skipping");
        }

        if any_loaded {
            stats.model_load_ms = Some(total_load_timer.stop());
            self.models_loaded_this_session = true;
            info!("Text encoders loaded for staged generation");
        }

        Ok(())
    }

    /// Load only diffusion models (VAE + FLUX) for image generation
    ///
    /// Used in staged loading after text encoding is complete and
    /// text encoders have been unloaded to free VRAM.
    pub(crate) fn ensure_diffusion_models_loaded(&mut self, stats: &mut GenerationStats) -> Result<()> {
        // Get paths from bundle context
        let context = self.bundle_context.clone()
            .ok_or_else(|| anyhow::anyhow!("No bundle context set - component IDs required"))?;

        let paths = if let Some(bundle_id) = context.bundle_id {
            let db_path = ModelPaths::get_db_path()?;
            let db = crate::db::InferenceDb::new(&db_path)?;
            let bundle_info = db.get_bundle(&bundle_id)?;
            ModelPaths::from_bundle_info(&bundle_info)?
        } else {
            let model_id = context.model_component_id.as_deref()
                .ok_or_else(|| anyhow::anyhow!("model_component_id required"))?;
            let t5_id = context.t5_component_id.as_deref()
                .ok_or_else(|| anyhow::anyhow!("t5_component_id required"))?;
            let clip_id = context.clip_component_id.as_deref()
                .ok_or_else(|| anyhow::anyhow!("clip_component_id required"))?;
            let vae_id = context.vae_component_id.as_deref()
                .ok_or_else(|| anyhow::anyhow!("vae_component_id required"))?;

            let db_path = ModelPaths::get_db_path()?;
            let db = crate::db::InferenceDb::new(&db_path)?;
            ModelPaths::from_component_ids(&db, model_id, t5_id, clip_id, vae_id)?
        };

        let total_load_timer = Timer::start();
        let mut any_loaded = false;

        // Load VAE if not present
        if self.vae.is_none() {
            self.load_vae_decoder(&paths, stats)?;
            any_loaded = true;
        } else {
            debug!("VAE decoder already loaded, skipping");
        }

        // Load FLUX transformer if not present
        if self.flux.is_none() {
            self.load_flux_transformer(&paths, stats)?;
            any_loaded = true;
        } else {
            debug!("FLUX transformer already loaded, skipping");
        }

        if any_loaded {
            // Add to existing model_load_ms if present (from text encoder loading)
            let elapsed = total_load_timer.stop();
            stats.model_load_ms = Some(stats.model_load_ms.unwrap_or(0) + elapsed);
            self.models_loaded_this_session = true;
            info!("Diffusion models loaded for staged generation");
        }

        Ok(())
    }

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
            let db_path = ModelPaths::get_db_path()?;
            let db = crate::db::InferenceDb::new(&db_path)?;
            ModelPaths::new(&db)?
        };

        // Validate model files exist
        if !paths.all_files_exist() {
            return Err(anyhow::anyhow!(
                "Active bundle has missing components. Please scan for models or activate a different bundle."
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
            let db = crate::db::InferenceDb::new(&db_path)?;
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

            let db_path = ModelPaths::get_db_path()?;
            let db = crate::db::InferenceDb::new(&db_path)?;
            ModelPaths::from_component_ids(&db, model_id, t5_id, clip_id, vae_id)?
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
            "Loading T5 encoder"
        );

        // Check if this is a quantized GGUF file or full-precision safetensors
        let is_gguf = model_path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "gguf")
            .unwrap_or(false);

        self.t5 = Some(if is_gguf {
            T5TextEncoder::load_quantized(
                model_path,
                tokenizer_path,
                self.device.clone(),
            )?
        } else {
            T5TextEncoder::load(
                model_path,
                tokenizer_path,
                self.device.clone(),
            )?
        });

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
        let clip_path = paths.clip_path()?;
        // Handle both file path (from scanner) and directory path (legacy)
        let model_path = if clip_path.is_file() {
            clip_path
        } else {
            clip_path.join("model.safetensors")
        };
        let tokenizer_path = paths.tokenizer_path()?;
        info!(
            model = %model_path.display(),
            tokenizer = %tokenizer_path.display(),
            "Loading CLIP encoder"
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
            "Loading VAE decoder"
        );
        let vae_timer = Timer::start();
        let mem_before = get_gpu_memory_stats();

        self.vae = Some(Arc::new(VaeDecoder::load(
            &model_path,
            self.device.clone(),
        )?));

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

        // Load transformer from bundle
        let model_path = paths.transformer_path()?;

        // Check if this is a quantized GGUF file
        let is_gguf = model_path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "gguf")
            .unwrap_or(false);

        if is_gguf {
            if has_loras {
                info!(
                    model = %self.model_type,
                    path = %model_path.display(),
                    lora_count = self.active_loras.len(),
                    "Loading quantized transformer (GGUF) with LoRAs"
                );
                self.flux = Some(FluxTransformer::load_quantized_with_loras(
                    model_path,
                    self.device.clone(),
                    self.model_type.clone(),
                    &self.active_loras,
                )?);
            } else {
                info!(
                    model = %self.model_type,
                    path = %model_path.display(),
                    "Loading quantized transformer (GGUF)"
                );
                self.flux = Some(FluxTransformer::load_quantized(
                    model_path,
                    self.device.clone(),
                    self.model_type.clone(),
                )?);
            }
        } else if has_loras {
            info!(
                model = %self.model_type,
                path = %model_path.display(),
                lora_count = self.active_loras.len(),
                "Loading transformer with LoRAs"
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
                "Loading transformer"
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
