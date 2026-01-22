//! Model loading logic for FluxPipeline

use anyhow::Result;
use tracing::{debug, info};

use crate::models::{ClipTextEncoder, FluxTransformer, ModelPaths, ModelType, T5TextEncoder, VaeDecoder};
use crate::inference::stats::{GenerationStats, Timer};
use super::FluxPipeline;

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
        stats.t5_load_ms = Some(t5_timer.stop());
        Ok(())
    }

    /// Load CLIP text encoder
    fn load_clip_encoder(&mut self, paths: &ModelPaths, stats: &mut GenerationStats) -> Result<()> {
        info!("Loading CLIP encoder");
        let clip_timer = Timer::start();
        self.clip = Some(ClipTextEncoder::load(
            paths.clip_path().join("model.safetensors"),
            paths.tokenizer_path(),
            self.device.clone(),
        )?);
        stats.clip_load_ms = Some(clip_timer.stop());
        Ok(())
    }

    /// Load VAE decoder
    fn load_vae_decoder(&mut self, paths: &ModelPaths, stats: &mut GenerationStats) -> Result<()> {
        info!("Loading VAE decoder");
        let vae_timer = Timer::start();
        self.vae = Some(VaeDecoder::load(
            paths.vae_path(),
            self.device.clone(),
        )?);
        stats.vae_load_ms = Some(vae_timer.stop());
        Ok(())
    }

    /// Load FLUX transformer
    fn load_flux_transformer(&mut self, paths: &ModelPaths, stats: &mut GenerationStats) -> Result<()> {
        let flux_timer = Timer::start();
        if paths.has_quantized_for(self.model_type) {
            info!(model = %self.model_type, "Loading transformer (quantized GGUF)");
            self.flux = Some(FluxTransformer::load_quantized(
                paths.quantized_transformer_path_for(self.model_type),
                self.device.clone(),
            )?);
        } else {
            info!(model = %self.model_type, "Loading transformer (full precision)");
            self.flux = Some(FluxTransformer::load(
                paths.transformer_path_for(self.model_type),
                self.device.clone(),
            )?);
        }
        stats.flux_load_ms = Some(flux_timer.stop());
        Ok(())
    }
}
