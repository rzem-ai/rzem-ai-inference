//! Flux model inference pipeline

use anyhow::Result;
use candle_core::Device;
use crate::models::{ClipTextEncoder, VaeDecoder, FluxTransformer, ModelPaths, ModelType, T5TextEncoder};
use super::stats::{GenerationStats, GenerationResult, Timer};

/// Flux diffusion model pipeline for image generation
pub struct FluxPipeline {
    device: Device,
    model_type: ModelType,
    t5: Option<T5TextEncoder>,
    clip: Option<ClipTextEncoder>,
    vae: Option<VaeDecoder>,
    flux: Option<FluxTransformer>,
    /// Whether models were loaded this session (for stats)
    models_loaded_this_session: bool,
}

impl FluxPipeline {
    /// Creates a new Flux pipeline instance for Schnell model (default)
    pub fn new(device: Device) -> Result<Self> {
        Self::with_model_type(device, ModelType::Schnell)
    }

    /// Creates a new Flux pipeline instance for a specific model type
    pub fn with_model_type(device: Device, model_type: ModelType) -> Result<Self> {
        Ok(Self {
            device,
            model_type,
            t5: None,
            clip: None,
            vae: None,
            flux: None,
            models_loaded_this_session: false,
        })
    }

    /// Get the model type this pipeline is configured for
    pub fn model_type(&self) -> ModelType {
        self.model_type
    }

    /// Load models if not already loaded, returning timing stats
    fn ensure_models_loaded(&mut self, stats: &mut GenerationStats) -> Result<()> {
        if self.t5.is_some() && self.clip.is_some() && self.vae.is_some() && self.flux.is_some() {
            return Ok(()); // Already loaded
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
        println!("Loading {} models...", self.model_type);

        // Load T5 text encoder (full precision) - shared between Schnell and Dev
        println!("  Loading T5 encoder...");
        let t5_timer = Timer::start();
        self.t5 = Some(T5TextEncoder::load(
            paths.t5_path(),
            paths.t5_tokenizer_path(),
            self.device.clone(),
        )?);
        stats.t5_load_ms = Some(t5_timer.stop());

        // Load CLIP text encoder (pooled embedding for vec) - shared between Schnell and Dev
        println!("  Loading CLIP encoder...");
        let clip_timer = Timer::start();
        self.clip = Some(ClipTextEncoder::load(
            paths.clip_path().join("model.safetensors"),
            paths.tokenizer_path(),
            self.device.clone(),
        )?);
        stats.clip_load_ms = Some(clip_timer.stop());

        // Load VAE decoder - shared between Schnell and Dev
        println!("  Loading VAE decoder...");
        let vae_timer = Timer::start();
        self.vae = Some(VaeDecoder::load(
            paths.vae_path(),
            self.device.clone(),
        )?);
        stats.vae_load_ms = Some(vae_timer.stop());

        // Load FLUX transformer - model-specific, prefer quantized if available
        let flux_timer = Timer::start();
        if paths.has_quantized_for(self.model_type) {
            println!("  Loading {} transformer (quantized GGUF - memory optimized)...", self.model_type);
            self.flux = Some(FluxTransformer::load_quantized(
                paths.quantized_transformer_path_for(self.model_type),
                self.device.clone(),
            )?);
        } else {
            println!("  Loading {} transformer (full precision)...", self.model_type);
            self.flux = Some(FluxTransformer::load(
                paths.transformer_path_for(self.model_type),
                self.device.clone(),
            )?);
        }
        stats.flux_load_ms = Some(flux_timer.stop());

        stats.model_load_ms = Some(total_load_timer.stop());
        self.models_loaded_this_session = true;

        println!("{} models loaded successfully!", self.model_type);
        Ok(())
    }

    /// Generate image from text prompt
    ///
    /// # Arguments
    /// * `prompt` - Text prompt describing the image
    /// * `steps` - Number of denoising steps (4 for Schnell)
    /// * `width` - Image width (default 1024)
    /// * `height` - Image height (default 1024)
    /// * `guidance` - Guidance scale (default 4.0)
    ///
    /// # Returns
    /// GenerationResult containing image data and timing statistics
    pub fn generate(
        &mut self,
        prompt: &str,
        steps: usize,
        width: usize,
        height: usize,
        guidance: f64,
    ) -> Result<GenerationResult> {
        let total_timer = Timer::start();
        let mut stats = GenerationStats::default();
        stats.steps = steps;

        // Load models if needed
        self.ensure_models_loaded(&mut stats)?;

        // Get T5 for encoding (will be unloaded after)
        let t5 = self.t5.as_mut()
            .ok_or_else(|| anyhow::anyhow!("T5 model not loaded"))?;

        println!("Encoding prompt with T5: {}", prompt);
        let t5_timer = Timer::start();
        let t5_emb = t5.encode(prompt)?;
        stats.t5_encode_ms = t5_timer.stop();
        stats.t5_embedding_shape = t5_emb.dims().to_vec();
        println!("  T5 embedding shape: {:?} ({}ms)", t5_emb.dims(), stats.t5_encode_ms);

        // Get CLIP for encoding (will be unloaded after FLUX)
        let clip = self.clip.as_ref()
            .ok_or_else(|| anyhow::anyhow!("CLIP model not loaded"))?;

        println!("Encoding prompt with CLIP...");
        let clip_timer = Timer::start();
        let clip_emb = clip.encode(prompt)?;
        stats.clip_encode_ms = clip_timer.stop();
        stats.clip_embedding_shape = clip_emb.dims().to_vec();
        println!("  CLIP embedding shape: {:?} ({}ms)", clip_emb.dims(), stats.clip_encode_ms);

        // Unload T5 to free ~9GB VRAM (no longer needed after encoding)
        println!("  Unloading T5 to free memory...");
        self.t5 = None;

        let flux = self.flux.as_ref()
            .ok_or_else(|| anyhow::anyhow!("FLUX model not loaded"))?;

        stats.model_type = if flux.is_quantized() {
            "quantized".to_string()
        } else {
            "full_precision".to_string()
        };
        println!("Denoising for {} steps (guidance={}, model={})...", steps, guidance, stats.model_type);
        let denoise_timer = Timer::start();
        let latents = flux.denoise(&t5_emb, &clip_emb, height, width, steps, guidance)?;
        stats.denoise_ms = denoise_timer.stop();
        stats.latent_shape = latents.dims().to_vec();
        println!("  Latent shape: {:?} ({}ms)", latents.dims(), stats.denoise_ms);

        // Unload FLUX to free ~12GB VRAM (no longer needed after denoising)
        println!("  Unloading FLUX to free memory...");
        drop(t5_emb);
        drop(clip_emb);
        self.flux = None;
        self.clip = None;

        let vae = self.vae.as_ref()
            .ok_or_else(|| anyhow::anyhow!("VAE model not loaded"))?;

        println!("Decoding to image...");
        let vae_timer = Timer::start();
        let image = vae.decode(&latents)?;
        stats.vae_decode_ms = vae_timer.stop();
        stats.image_shape = image.dims().to_vec();
        println!("  Image shape: {:?} ({}ms)", image.dims(), stats.vae_decode_ms);

        println!("Converting to PNG...");
        let png_timer = Timer::start();
        let rgb_data = vae.tensor_to_rgb(&image)?;

        // Convert RGB data to PNG
        let img = image::RgbImage::from_raw(width as u32, height as u32, rgb_data)
            .ok_or_else(|| anyhow::anyhow!("Failed to create image from raw data"))?;

        let mut png_data = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut png_data),
            image::ImageFormat::Png
        )?;
        stats.png_encode_ms = png_timer.stop();

        stats.total_ms = total_timer.stop();
        println!("Generation complete! Total time: {}ms", stats.total_ms);

        Ok(GenerationResult {
            image_data: png_data,
            stats,
        })
    }

    /// Simplified generate with defaults
    pub fn generate_simple(&mut self, prompt: &str, steps: usize) -> Result<GenerationResult> {
        self.generate(prompt, steps, 1024, 1024, 4.0)
    }

    /// Generate image with progress callbacks
    ///
    /// This method provides real-time progress updates during generation,
    /// useful for UI feedback and progress bars.
    pub fn generate_with_progress<F>(
        &mut self,
        prompt: &str,
        steps: usize,
        width: usize,
        height: usize,
        guidance: f64,
        on_progress: F,
    ) -> Result<GenerationResult>
    where
        F: Fn(super::GenerationProgress),
    {
        use super::{GenerationProgress, PipelineStage};

        let total_timer = Timer::start();
        let mut stats = GenerationStats::default();
        stats.steps = steps;

        // Loading stage
        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 0.0));
        self.ensure_models_loaded(&mut stats)?;
        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 1.0));

        // T5 encoding
        on_progress(GenerationProgress::new(PipelineStage::EncodingT5, 0.0));
        let t5 = self.t5.as_mut()
            .ok_or_else(|| anyhow::anyhow!("T5 model not loaded"))?;

        let t5_timer = Timer::start();
        let t5_emb = t5.encode(prompt)?;
        stats.t5_encode_ms = t5_timer.stop();
        stats.t5_embedding_shape = t5_emb.dims().to_vec();
        on_progress(GenerationProgress::new(PipelineStage::EncodingT5, 1.0));

        // CLIP encoding
        on_progress(GenerationProgress::new(PipelineStage::EncodingClip, 0.0));
        let clip = self.clip.as_ref()
            .ok_or_else(|| anyhow::anyhow!("CLIP model not loaded"))?;

        let clip_timer = Timer::start();
        let clip_emb = clip.encode(prompt)?;
        stats.clip_encode_ms = clip_timer.stop();
        stats.clip_embedding_shape = clip_emb.dims().to_vec();
        on_progress(GenerationProgress::new(PipelineStage::EncodingClip, 1.0));

        // Unload T5 to free memory
        self.t5 = None;

        // Denoising
        let flux = self.flux.as_ref()
            .ok_or_else(|| anyhow::anyhow!("FLUX model not loaded"))?;

        stats.model_type = if flux.is_quantized() {
            "quantized".to_string()
        } else {
            "full_precision".to_string()
        };

        // Progress for denoising start
        on_progress(GenerationProgress::denoising_step(0, steps));

        let denoise_timer = Timer::start();
        let latents = flux.denoise(&t5_emb, &clip_emb, height, width, steps, guidance)?;
        stats.denoise_ms = denoise_timer.stop();
        stats.latent_shape = latents.dims().to_vec();

        // Progress for denoising complete
        on_progress(GenerationProgress::denoising_step(steps, steps));

        // Unload FLUX and CLIP to free memory
        drop(t5_emb);
        drop(clip_emb);
        self.flux = None;
        self.clip = None;

        // VAE decoding
        on_progress(GenerationProgress::new(PipelineStage::DecodingVae, 0.0));
        let vae = self.vae.as_ref()
            .ok_or_else(|| anyhow::anyhow!("VAE model not loaded"))?;

        let vae_timer = Timer::start();
        let image = vae.decode(&latents)?;
        stats.vae_decode_ms = vae_timer.stop();
        stats.image_shape = image.dims().to_vec();
        on_progress(GenerationProgress::new(PipelineStage::DecodingVae, 1.0));

        // PNG encoding
        on_progress(GenerationProgress::new(PipelineStage::EncodingPng, 0.0));
        let png_timer = Timer::start();
        let rgb_data = vae.tensor_to_rgb(&image)?;

        let img = image::RgbImage::from_raw(width as u32, height as u32, rgb_data)
            .ok_or_else(|| anyhow::anyhow!("Failed to create image from raw data"))?;

        let mut png_data = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut png_data),
            image::ImageFormat::Png
        )?;
        stats.png_encode_ms = png_timer.stop();
        on_progress(GenerationProgress::new(PipelineStage::EncodingPng, 1.0));

        stats.total_ms = total_timer.stop();

        Ok(GenerationResult {
            image_data: png_data,
            stats,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::InferenceEngine;

    #[test]
    fn test_pipeline_creation() {
        let engine = InferenceEngine::new().unwrap();
        let device = engine.get_device().clone();
        let _pipeline = FluxPipeline::new(device).unwrap();
    }

    #[test]
    #[ignore] // Requires downloaded models
    fn test_real_generation() {
        let engine = InferenceEngine::new().unwrap();
        let device = engine.get_device().clone();
        let mut pipeline = FluxPipeline::new(device).unwrap();

        let result = pipeline.generate_simple("a cat", 4).unwrap();
        assert!(!result.image_data.is_empty());
        // PNG magic number
        assert_eq!(&result.image_data[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
        // Check stats are populated
        assert!(result.stats.total_ms > 0);
        assert!(result.stats.denoise_ms > 0);
    }
}
