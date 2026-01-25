//! Model loading logic for FluxPipeline

use anyhow::Result;
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
    pub(crate) fn ensure_models_loaded(&mut self, stats: &mut GenerationStats) -> Result<()> {
        // Quick return if all models are already loaded
        if self.t5.is_some() && self.clip.is_some() && self.vae.is_some() && self.flux.is_some() {
            debug!("All models already loaded, skipping load");
            return Ok(());
        }

        let paths = ModelPaths::new()?;

        // Check if base models are downloaded (shared components)
        if !paths.all_files_exist() {
            return Err(anyhow::anyhow!(
                "FLUX base models not downloaded. Run model downloader first."
            ));
        }

        // Check if the specific model (Dev) is downloaded if needed
        if self.model_type == ModelType::Dev && !paths.is_dev_downloaded() {
            return Err(anyhow::anyhow!(
                "FLUX Dev model not downloaded. Download it from Settings or use 'rzem-cli models download dev'."
            ));
        }

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
            info!(model = %self.model_type, "Models ready");
        }

        Ok(())
    }

    /// Ensure models are ready for generation (reload if unloaded)
    ///
    /// Call this before generation when using cached pipelines.
    pub fn ensure_ready_for_generation(&mut self, stats: &mut GenerationStats) -> Result<()> {
        self.ensure_models_loaded(stats)
    }

    /// Load T5 text encoder
    /// Prefer quantized version (~3.3GB) over full precision (~9GB) to save VRAM
    fn load_t5_encoder(&mut self, paths: &ModelPaths, stats: &mut GenerationStats) -> Result<()> {
        let t5_timer = Timer::start();
        let mem_before = get_gpu_memory_stats();

        if paths.has_quantized_t5() {
            info!("Loading T5 encoder (quantized Q5_K_M ~3.3GB)");
            self.t5 = Some(T5TextEncoder::load_quantized(
                paths.quantized_t5_path(),
                paths.t5_tokenizer_path(),
                self.device.clone(),
            )?);
        } else {
            info!("Loading T5 encoder (full precision BF16 ~9GB)");
            self.t5 = Some(T5TextEncoder::load(
                paths.t5_path(),
                paths.t5_tokenizer_path(),
                self.device.clone(),
            )?);
        }

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
        info!("Loading CLIP encoder");
        let clip_timer = Timer::start();
        let mem_before = get_gpu_memory_stats();

        self.clip = Some(ClipTextEncoder::load(
            paths.clip_path().join("model.safetensors"),
            paths.tokenizer_path(),
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
        info!("Loading VAE decoder");
        let vae_timer = Timer::start();
        let mem_before = get_gpu_memory_stats();

        self.vae = Some(VaeDecoder::load(
            paths.vae_path(),
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

        if paths.has_quantized_for(self.model_type) {
            if has_loras {
                // LoRAs not yet supported with quantized models
                // Fall back to loading without LoRAs and log a warning
                info!(
                    model = %self.model_type,
                    lora_count = self.active_loras.len(),
                    "Loading quantized transformer (LoRAs not supported with GGUF, ignoring)"
                );
            } else {
                info!(model = %self.model_type, "Loading transformer (quantized GGUF)");
            }
            self.flux = Some(FluxTransformer::load_quantized(
                paths.quantized_transformer_path_for(self.model_type),
                self.device.clone(),
            )?);
        } else if has_loras {
            info!(
                model = %self.model_type,
                lora_count = self.active_loras.len(),
                "Loading transformer with LoRAs (full precision)"
            );
            self.flux = Some(FluxTransformer::load_with_loras(
                paths.transformer_path_for(self.model_type),
                self.device.clone(),
                &self.active_loras,
            )?);
        } else {
            info!(model = %self.model_type, "Loading transformer (full precision)");
            self.flux = Some(FluxTransformer::load(
                paths.transformer_path_for(self.model_type),
                self.device.clone(),
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
