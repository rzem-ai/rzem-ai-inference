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

        // Z-Image support removed per NO BACKWARD COMPATIBILITY rule
        return Err(anyhow::anyhow!(
            "Z-Image-Turbo support has been removed. Use FLUX pipelines instead."
        ));
    }

    /// Ensure models are ready for generation (reload if unloaded)
    ///
    /// Call this before generation when using cached pipelines.
    pub fn ensure_ready_for_generation(&mut self, stats: &mut GenerationStats) -> Result<()> {
        self.ensure_models_loaded(stats)
    }

    /// Load Qwen3-4B text encoder (REMOVED - Z-Image support dropped)
    fn load_qwen3_encoder(&mut self, _paths: &ModelPaths, _stats: &mut GenerationStats) -> Result<()> {
        Err(anyhow::anyhow!("Z-Image support removed"))
    }

    /// Load VAE decoder (REMOVED - Z-Image support dropped)
    fn load_vae_decoder(&mut self, _paths: &ModelPaths, _stats: &mut GenerationStats) -> Result<()> {
        Err(anyhow::anyhow!("Z-Image support removed"))
    }

    /// Load Z-Image-Turbo transformer (REMOVED - Z-Image support dropped)
    fn load_zimage_transformer(&mut self, _paths: &ModelPaths, _stats: &mut GenerationStats) -> Result<()> {
        Err(anyhow::anyhow!("Z-Image support removed"))
    }
}
