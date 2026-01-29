//! Model loading logic for ZIndexPipeline

use anyhow::Result;
use tracing::{debug, info};

use crate::models::{ModelPaths, Qwen3TextEncoder, VaeDecoder, ZImageTransformer};
use crate::inference::stats::{GenerationStats, Timer};
use super::ZIndexPipeline;

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

impl ZIndexPipeline {
    /// Load models if not already loaded, returning timing stats
    /// Each model is checked individually - only missing models are loaded
    pub(crate) fn ensure_models_loaded(&mut self, stats: &mut GenerationStats) -> Result<()> {
        // Quick return if all models are already loaded
        if self.all_models_loaded() {
            debug!("All Z-Image-Turbo models already loaded, skipping load");
            return Ok(());
        }

        let paths = ModelPaths::new()?;

        // Check if Z-Image-Turbo is downloaded
        if !paths.is_zimage_downloaded() {
            return Err(anyhow::anyhow!(
                "Z-Image-Turbo model not downloaded. Download manually using: huggingface-cli download Tongyi-MAI/Z-Image-Turbo"
            ));
        }

        let total_load_timer = Timer::start();
        let mut any_loaded = false;

        // Load each model component
        if self.qwen3.is_none() {
            self.load_qwen3_encoder(&paths, stats)?;
            any_loaded = true;
        } else {
            debug!("Qwen3 encoder already loaded, skipping");
        }

        if self.vae.is_none() {
            self.load_vae_decoder(&paths, stats)?;
            any_loaded = true;
        } else {
            debug!("VAE decoder already loaded, skipping");
        }

        if self.zimage.is_none() {
            self.load_zimage_transformer(&paths, stats)?;
            any_loaded = true;
        } else {
            debug!("Z-Image transformer already loaded, skipping");
        }

        if any_loaded {
            stats.model_load_ms = Some(total_load_timer.stop());
            self.models_loaded_this_session = true;
            info!("Z-Image-Turbo models ready");
        }

        Ok(())
    }

    /// Ensure models are ready for generation (reload if unloaded)
    ///
    /// Call this before generation when using cached pipelines.
    pub fn ensure_ready_for_generation(&mut self, stats: &mut GenerationStats) -> Result<()> {
        self.ensure_models_loaded(stats)
    }

    /// Load Qwen3-4B text encoder
    fn load_qwen3_encoder(&mut self, paths: &ModelPaths, stats: &mut GenerationStats) -> Result<()> {
        info!("Loading Qwen3-4B text encoder (~7.5GB)");
        let qwen3_timer = Timer::start();
        let mem_before = get_gpu_memory_stats();

        // TODO: Remove Z-Index support
        self.qwen3 = Some(Qwen3TextEncoder::load(
            paths.qwen3_path()?,
            paths.qwen3_tokenizer_path()?,
            self.device.clone(),
        )?);

        let elapsed = qwen3_timer.stop();
        stats.t5_load_ms = Some(elapsed); // Reuse t5_load_ms field for text encoder timing

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
                    "Qwen3 encoder loaded"
                );
            } else {
                info!(
                    elapsed_ms = elapsed,
                    gpu_used = format_bytes(used),
                    gpu_total = format_bytes(total),
                    gpu_percent = format!("{:.1}%", percent),
                    "Qwen3 encoder loaded"
                );
            }
        } else {
            info!(elapsed_ms = elapsed, "Qwen3 encoder loaded (GPU stats unavailable)");
        }

        Ok(())
    }

    /// Load VAE decoder (shared with FLUX)
    fn load_vae_decoder(&mut self, paths: &ModelPaths, stats: &mut GenerationStats) -> Result<()> {
        info!("Loading VAE decoder (shared with FLUX)");
        let vae_timer = Timer::start();
        let mem_before = get_gpu_memory_stats();

        // Z-Image uses FLUX's VAE - load from Z-Image directory
        // TODO: Remove Z-Image support
        let vae_file = paths.zimage_vae_path()?.join("diffusion_pytorch_model.safetensors");

        self.vae = Some(VaeDecoder::load(
            vae_file,
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

    /// Load Z-Image-Turbo transformer (stub - Phase 4)
    fn load_zimage_transformer(&mut self, paths: &ModelPaths, stats: &mut GenerationStats) -> Result<()> {
        info!("Loading Z-Image-Turbo transformer (stub - Phase 4)");
        let zimage_timer = Timer::start();
        let mem_before = get_gpu_memory_stats();

        // TODO: Remove Z-Image support
        self.zimage = Some(ZImageTransformer::load(
            paths.zimage_transformer_path()?,
            self.device.clone(),
        )?);

        let elapsed = zimage_timer.stop();
        stats.flux_load_ms = Some(elapsed); // Reuse flux_load_ms for transformer timing

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
                    "Z-Image transformer loaded (stub)"
                );
            } else {
                info!(
                    elapsed_ms = elapsed,
                    gpu_used = format_bytes(used),
                    gpu_total = format_bytes(total),
                    gpu_percent = format!("{:.1}%", percent),
                    "Z-Image transformer loaded (stub)"
                );
            }
        } else {
            info!(elapsed_ms = elapsed, "Z-Image transformer loaded (GPU stats unavailable)");
        }

        Ok(())
    }
}
