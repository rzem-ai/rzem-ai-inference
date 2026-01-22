//! Image generation methods for FluxPipeline

use anyhow::Result;
use tracing::{debug, info, trace};

use crate::inference::metadata::{ImageMetadata, encode_png_with_metadata};
use crate::inference::samplers::{SamplerType, SchedulerType};
use crate::inference::stats::{GenerationStats, GenerationResult, Timer};
use crate::inference::{GenerationProgress, PipelineStage};
use super::FluxPipeline;

impl FluxPipeline {
    /// Generate image from text prompt
    ///
    /// # Arguments
    /// * `prompt` - Text prompt describing the image
    /// * `steps` - Number of denoising steps (4 for Schnell)
    /// * `width` - Image width (default 1024)
    /// * `height` - Image height (default 1024)
    /// * `guidance` - Guidance scale (default 4.0)
    /// * `seed` - Random seed for reproducible generation
    /// * `sampler` - Sampling algorithm (Euler, EulerA, DPM++ 2M)
    /// * `scheduler` - Noise schedule (Normal, Karras, Exponential)
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
        seed: u64,
        sampler: SamplerType,
        scheduler: SchedulerType,
    ) -> Result<GenerationResult> {
        let total_timer = Timer::start();
        let mut stats = GenerationStats::default();
        stats.steps = steps;

        // Load models if needed
        self.ensure_models_loaded(&mut stats)?;

        // Get T5 for encoding (will be unloaded after)
        let t5 = self.t5.as_mut()
            .ok_or_else(|| anyhow::anyhow!("T5 model not loaded"))?;

        debug!(prompt = %prompt, "Encoding prompt with T5");
        let t5_timer = Timer::start();
        let t5_emb = t5.encode(prompt)?;
        stats.t5_encode_ms = t5_timer.stop();
        stats.t5_embedding_shape = t5_emb.dims().to_vec();
        debug!(shape = ?t5_emb.dims(), time_ms = stats.t5_encode_ms, "T5 embedding");

        // Get CLIP for encoding (will be unloaded after FLUX)
        let clip = self.clip.as_ref()
            .ok_or_else(|| anyhow::anyhow!("CLIP model not loaded"))?;

        debug!("Encoding prompt with CLIP");
        let clip_timer = Timer::start();
        let clip_emb = clip.encode(prompt)?;
        stats.clip_encode_ms = clip_timer.stop();
        stats.clip_embedding_shape = clip_emb.dims().to_vec();
        debug!(shape = ?clip_emb.dims(), time_ms = stats.clip_encode_ms, "CLIP embedding");

        // Note: Model unloading is now handled by apply_cache_config()
        // For direct calls without cache, models stay loaded for potential reuse

        let flux = self.flux.as_ref()
            .ok_or_else(|| anyhow::anyhow!("FLUX model not loaded"))?;

        stats.model_type = if flux.is_quantized() {
            "quantized".to_string()
        } else {
            "full_precision".to_string()
        };
        info!(steps = steps, guidance = guidance, model = %stats.model_type, seed = seed, sampler = ?sampler, scheduler = ?scheduler, "Denoising");
        let denoise_timer = Timer::start();
        let latents = flux.denoise(&t5_emb, &clip_emb, height, width, steps, guidance, seed, sampler, scheduler)?;
        stats.denoise_ms = denoise_timer.stop();
        stats.latent_shape = latents.dims().to_vec();
        debug!(shape = ?latents.dims(), time_ms = stats.denoise_ms, "Latent");

        // Free VRAM by dropping embedding tensors
        // Only unload CLIP - keep T5, FLUX, and VAE loaded
        trace!("Freeing embedding tensors");
        drop(t5_emb);
        drop(clip_emb);

        // Unload only CLIP (~1GB)
        self.clip = None;

        let vae = self.vae.as_ref()
            .ok_or_else(|| anyhow::anyhow!("VAE model not loaded"))?;

        debug!("Decoding to image");
        let vae_timer = Timer::start();
        let image = vae.decode(&latents)?;
        stats.vae_decode_ms = vae_timer.stop();
        stats.image_shape = image.dims().to_vec();
        debug!(shape = ?image.dims(), time_ms = stats.vae_decode_ms, "Image decoded");

        trace!("Converting to PNG with metadata");
        let png_timer = Timer::start();
        let rgb_data = vae.tensor_to_rgb(&image)?;

        // Create metadata for embedding
        let metadata = ImageMetadata {
            prompt: prompt.to_string(),
            negative_prompt: None,
            steps: steps as u32,
            cfg_scale: guidance,
            width: width as u32,
            height: height as u32,
            seed: seed as i64,
            model: self.model_type.to_string(),
            sampler: Some(sampler.to_string()),
            scheduler: Some(scheduler.to_string()),
        };

        // Encode PNG with embedded metadata
        let png_data = encode_png_with_metadata(&rgb_data, width as u32, height as u32, &metadata)?;
        stats.png_encode_ms = png_timer.stop();

        stats.total_ms = total_timer.stop();
        info!(total_ms = stats.total_ms, "Generation complete");

        Ok(GenerationResult {
            image_data: png_data,
            stats,
        })
    }

    /// Simplified generate with defaults
    pub fn generate_simple(&mut self, prompt: &str, steps: usize) -> Result<GenerationResult> {
        // Use random seed when not specified
        let seed = rand::random::<u64>();
        self.generate(prompt, steps, 1024, 1024, 4.0, seed, SamplerType::default(), SchedulerType::default())
    }

    /// Generate image with progress callbacks
    ///
    /// This method provides real-time progress updates during generation,
    /// useful for UI feedback and progress bars.
    ///
    /// # Arguments
    /// * `metadata` - Optional metadata to embed in the PNG. If None, basic metadata
    ///   will be created from the generation parameters.
    /// * `sampler` - Sampling algorithm to use
    /// * `scheduler` - Noise schedule type
    pub fn generate_with_progress<F>(
        &mut self,
        prompt: &str,
        steps: usize,
        width: usize,
        height: usize,
        guidance: f64,
        seed: u64,
        metadata: Option<ImageMetadata>,
        sampler: SamplerType,
        scheduler: SchedulerType,
        on_progress: F,
    ) -> Result<GenerationResult>
    where
        F: Fn(GenerationProgress),
    {
        trace!("generate_with_progress starting");
        let total_timer = Timer::start();
        let mut stats = GenerationStats::default();
        stats.steps = steps;

        // Loading stage
        trace!("Loading models stage");
        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 0.0));
        self.ensure_models_loaded(&mut stats)?;
        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 1.0));

        // T5 encoding
        trace!("T5 encoding stage");
        on_progress(GenerationProgress::new(PipelineStage::EncodingT5, 0.0));
        let t5 = self.t5.as_mut()
            .ok_or_else(|| anyhow::anyhow!("T5 model not loaded"))?;

        let t5_timer = Timer::start();
        trace!(prompt_preview = %&prompt[..prompt.len().min(50)], "Encoding with T5");
        let t5_emb = t5.encode(prompt)?;
        stats.t5_encode_ms = t5_timer.stop();
        stats.t5_embedding_shape = t5_emb.dims().to_vec();
        debug!(shape = ?stats.t5_embedding_shape, time_ms = stats.t5_encode_ms, "T5 encoding complete");
        on_progress(GenerationProgress::new(PipelineStage::EncodingT5, 1.0));

        // CLIP encoding
        trace!("CLIP encoding stage");
        on_progress(GenerationProgress::new(PipelineStage::EncodingClip, 0.0));
        let clip = self.clip.as_ref()
            .ok_or_else(|| anyhow::anyhow!("CLIP model not loaded"))?;

        let clip_timer = Timer::start();
        let clip_emb = clip.encode(prompt)?;
        stats.clip_encode_ms = clip_timer.stop();
        stats.clip_embedding_shape = clip_emb.dims().to_vec();
        debug!(shape = ?stats.clip_embedding_shape, time_ms = stats.clip_encode_ms, "CLIP encoding complete");
        on_progress(GenerationProgress::new(PipelineStage::EncodingClip, 1.0));

        // Note: Model unloading is now handled by apply_cache_config() after generation
        // This allows the cache system to decide what to keep loaded

        // Denoising
        trace!("Denoising stage");
        let flux = self.flux.as_ref()
            .ok_or_else(|| anyhow::anyhow!("FLUX model not loaded"))?;

        stats.model_type = if flux.is_quantized() {
            "quantized".to_string()
        } else {
            "full_precision".to_string()
        };

        // Progress for denoising start
        info!(steps = steps, model = %stats.model_type, sampler = ?sampler, scheduler = ?scheduler, "Starting denoising");
        on_progress(GenerationProgress::denoising_step(0, steps));

        let denoise_timer = Timer::start();
        let latents = flux.denoise(&t5_emb, &clip_emb, height, width, steps, guidance, seed, sampler, scheduler)?;
        stats.denoise_ms = denoise_timer.stop();
        stats.latent_shape = latents.dims().to_vec();
        debug!(shape = ?stats.latent_shape, time_ms = stats.denoise_ms, "Denoising complete");

        // Progress for denoising complete
        on_progress(GenerationProgress::denoising_step(steps, steps));

        // Free VRAM by dropping embedding tensors
        // Only unload CLIP (~1GB) - keep T5, FLUX, and VAE loaded for fast subsequent generations
        trace!("Freeing embedding tensors");
        drop(t5_emb);
        drop(clip_emb);

        // Unload CLIP (encoding complete, frees ~1GB VRAM)
        if self.clip.is_some() {
            debug!("Unloading CLIP encoder");
            self.clip = None;
        }

        // Keep T5, FLUX, and VAE loaded for next generation

        // VAE decoding
        trace!("VAE decoding stage");
        on_progress(GenerationProgress::new(PipelineStage::DecodingVae, 0.0));
        let vae = self.vae.as_ref()
            .ok_or_else(|| anyhow::anyhow!("VAE model not loaded"))?;

        let vae_timer = Timer::start();
        let image = vae.decode(&latents)?;
        stats.vae_decode_ms = vae_timer.stop();
        stats.image_shape = image.dims().to_vec();
        debug!(shape = ?stats.image_shape, time_ms = stats.vae_decode_ms, "VAE decoding complete");
        on_progress(GenerationProgress::new(PipelineStage::DecodingVae, 1.0));

        // PNG encoding with metadata
        trace!("PNG encoding stage");
        on_progress(GenerationProgress::new(PipelineStage::EncodingPng, 0.0));
        let png_timer = Timer::start();
        let rgb_data = vae.tensor_to_rgb(&image)?;

        // Use provided metadata or create default
        let final_metadata = metadata.unwrap_or_else(|| ImageMetadata {
            prompt: prompt.to_string(),
            negative_prompt: None,
            steps: steps as u32,
            cfg_scale: guidance,
            width: width as u32,
            height: height as u32,
            seed: seed as i64,
            model: self.model_type.to_string(),
            sampler: Some(sampler.to_string()),
            scheduler: Some(scheduler.to_string()),
        });

        // Encode PNG with embedded metadata
        let png_data = encode_png_with_metadata(&rgb_data, width as u32, height as u32, &final_metadata)?;
        debug!(size_bytes = png_data.len(), "PNG encoding complete");
        stats.png_encode_ms = png_timer.stop();
        on_progress(GenerationProgress::new(PipelineStage::EncodingPng, 1.0));

        stats.total_ms = total_timer.stop();
        info!(total_ms = stats.total_ms, "Generation complete");

        Ok(GenerationResult {
            image_data: png_data,
            stats,
        })
    }
}
