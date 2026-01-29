//! Cache integration methods for FluxPipeline

use anyhow::Result;
use candle_core::Tensor;
use tracing::{debug, info};

use crate::models::{ClipTextEncoder, FluxTransformer, ModelPaths, T5TextEncoder, VaeDecoder};
use crate::inference::cache::ModelCacheConfig;
use crate::inference::embedding_cache::{CachedEmbedding, EmbeddingCache};
use crate::inference::metadata::{ImageMetadata, encode_png_with_metadata};
use crate::inference::samplers::{SamplerType, SchedulerType};
use crate::inference::stats::{GenerationStats, GenerationResult, Timer};
use crate::inference::{GenerationProgress, PipelineStage};
use super::FluxPipeline;

impl FluxPipeline {
    /// Apply cache configuration to selectively unload models
    ///
    /// Call this after generation to free memory based on config.
    pub fn apply_cache_config(&mut self, config: &ModelCacheConfig) {
        if !config.keep_t5_loaded && self.t5.is_some() {
            debug!("Unloading T5 per cache config");
            self.t5 = None;
        }
        if !config.keep_clip_loaded && self.clip.is_some() {
            debug!("Unloading CLIP per cache config");
            self.clip = None;
        }
        if !config.keep_flux_loaded && self.flux.is_some() {
            debug!("Unloading FLUX per cache config");
            self.flux = None;
        }
        if !config.keep_vae_loaded && self.vae.is_some() {
            debug!("Unloading VAE per cache config");
            self.vae = None;
        }
    }

    /// Encode prompt with caching support
    ///
    /// Returns cached embeddings if available, otherwise encodes and caches.
    pub fn encode_prompt_cached(
        &mut self,
        prompt: &str,
        cache: &mut EmbeddingCache,
        stats: &mut GenerationStats,
    ) -> Result<(Tensor, Tensor)> {
        // Check cache first
        if let Some(cached) = cache.get(prompt) {
            debug!("Using cached embeddings for prompt");
            return Ok((cached.t5_embedding, cached.clip_embedding));
        }

        // Need to encode - ensure T5 and CLIP are loaded
        if self.t5.is_none() || self.clip.is_none() {
            // Reload encoders if they were unloaded
            let paths = ModelPaths::new()?;

            if self.t5.is_none() {
                let t5_timer = Timer::start();
                let model_path = paths.t5_path()?;
                let tokenizer_path = paths.t5_tokenizer_path()?;
                info!(
                    model = %model_path.display(),
                    tokenizer = %tokenizer_path.display(),
                    "Reloading T5 encoder from bundle"
                );
                self.t5 = Some(T5TextEncoder::load(
                    model_path,
                    tokenizer_path,
                    self.device.clone(),
                )?);
                stats.t5_load_ms = Some(t5_timer.stop());
            }

            if self.clip.is_none() {
                let model_path = paths.clip_path()?.join("model.safetensors");
                let tokenizer_path = paths.tokenizer_path()?;
                info!(
                    model = %model_path.display(),
                    tokenizer = %tokenizer_path.display(),
                    "Reloading CLIP encoder"
                );
                let clip_timer = Timer::start();
                self.clip = Some(ClipTextEncoder::load(
                    model_path,
                    tokenizer_path,
                    self.device.clone(),
                )?);
                stats.clip_load_ms = Some(clip_timer.stop());
            }
        }

        // Encode with T5
        let t5 = self.t5.as_mut().ok_or_else(|| anyhow::anyhow!("T5 not loaded"))?;
        debug!(prompt = %prompt, "Encoding prompt with T5");
        let t5_timer = Timer::start();
        let t5_emb = t5.encode(prompt)?;
        stats.t5_encode_ms = t5_timer.stop();
        stats.t5_embedding_shape = t5_emb.dims().to_vec();

        // Encode with CLIP
        let clip = self.clip.as_ref().ok_or_else(|| anyhow::anyhow!("CLIP not loaded"))?;
        debug!("Encoding prompt with CLIP");
        let clip_timer = Timer::start();
        let clip_emb = clip.encode(prompt)?;
        stats.clip_encode_ms = clip_timer.stop();
        stats.clip_embedding_shape = clip_emb.dims().to_vec();

        // Cache the embeddings
        cache.insert(
            prompt,
            CachedEmbedding {
                t5_embedding: t5_emb.clone(),
                clip_embedding: clip_emb.clone(),
            },
        );
        debug!("Cached embeddings for prompt");

        Ok((t5_emb, clip_emb))
    }

    /// Generate image using pre-computed or cached embeddings
    ///
    /// This is a lower-level method that allows reusing embeddings from the cache.
    /// Use this with `encode_prompt_cached` for maximum efficiency.
    pub fn generate_from_embeddings(
        &mut self,
        t5_emb: &Tensor,
        clip_emb: &Tensor,
        prompt: &str, // For metadata only
        steps: usize,
        width: usize,
        height: usize,
        guidance: f64,
        seed: u64,
        sampler: SamplerType,
        scheduler: SchedulerType,
        stats: &mut GenerationStats,
    ) -> Result<GenerationResult> {
        let total_timer = Timer::start();
        stats.steps = steps;

        // Ensure FLUX and VAE are loaded
        if self.flux.is_none() || self.vae.is_none() {
            let paths = ModelPaths::new()?;

            if self.flux.is_none() {
                let flux_timer = Timer::start();
                let model_path = paths.transformer_path()?;
                info!(
                    model = %self.model_type,
                    path = %model_path.display(),
                    "Reloading transformer from bundle"
                );
                self.flux = Some(FluxTransformer::load(
                    model_path,
                    self.device.clone(),
                    self.model_type.clone(),
                )?);
                stats.flux_load_ms = Some(flux_timer.stop());
            }

            if self.vae.is_none() {
                let model_path = paths.vae_path()?;
                info!(
                    model = %model_path.display(),
                    "Reloading VAE decoder"
                );
                let vae_timer = Timer::start();
                self.vae = Some(VaeDecoder::load(model_path, self.device.clone())?);
                stats.vae_load_ms = Some(vae_timer.stop());
            }
        }

        let flux = self.flux.as_ref().ok_or_else(|| anyhow::anyhow!("FLUX not loaded"))?;

        stats.model_type = if flux.is_quantized() {
            "quantized".to_string()
        } else {
            "full_precision".to_string()
        };

        info!(steps = steps, guidance = guidance, seed = seed, sampler = ?sampler, scheduler = ?scheduler, "Drawing");
        let denoise_timer = Timer::start();
        let latents = flux.denoise(t5_emb, clip_emb, height, width, steps, guidance, seed, sampler, scheduler, None::<fn(usize, usize)>)?;
        stats.denoise_ms = denoise_timer.stop();
        stats.latent_shape = latents.dims().to_vec();

        let vae = self.vae.as_ref().ok_or_else(|| anyhow::anyhow!("VAE not loaded"))?;

        debug!("Decoding to image");
        let vae_timer = Timer::start();
        let image = vae.decode(&latents)?;
        stats.vae_decode_ms = vae_timer.stop();
        stats.image_shape = image.dims().to_vec();

        debug!("Converting to PNG with metadata");
        let png_timer = Timer::start();
        let rgb_data = vae.tensor_to_rgb(&image)?;

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

        let png_data = encode_png_with_metadata(&rgb_data, width as u32, height as u32, &metadata)?;
        stats.png_encode_ms = png_timer.stop();

        stats.total_ms = total_timer.stop();
        info!(total_ms = stats.total_ms, "Generation complete");

        Ok(GenerationResult {
            image_data: png_data,
            stats: stats.clone(),
        })
    }

    /// Generate with progress callbacks using embedding cache
    ///
    /// This is the preferred method when using ModelCache, as it integrates
    /// with the embedding cache for maximum efficiency.
    pub fn generate_with_embedding_cache<F>(
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
        embedding_cache: &mut EmbeddingCache,
        on_progress: F,
    ) -> Result<GenerationResult>
    where
        F: Fn(GenerationProgress),
    {
        let total_timer = Timer::start();
        let mut stats = GenerationStats::default();
        stats.steps = steps;

        // Loading models and encoding stage (0.0 - 0.5)
        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 0.0));
        self.ensure_models_loaded(&mut stats)?;
        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 0.5));

        // Encoding with cache
        let (t5_emb, clip_emb) = self.encode_prompt_cached(prompt, embedding_cache, &mut stats)?;
        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 1.0));

        // Denoising
        let flux = self.flux.as_ref().ok_or_else(|| anyhow::anyhow!("FLUX model not loaded"))?;

        stats.model_type = if flux.is_quantized() {
            "quantized".to_string()
        } else {
            "full_precision".to_string()
        };

        let denoise_timer = Timer::start();
        let latents = flux.denoise(
            &t5_emb,
            &clip_emb,
            height,
            width,
            steps,
            guidance,
            seed,
            sampler,
            scheduler,
            Some(|current_step: usize, total_steps: usize| {
                on_progress(GenerationProgress::denoising_step(current_step, total_steps));
            }),
        )?;
        stats.denoise_ms = denoise_timer.stop();
        stats.latent_shape = latents.dims().to_vec();

        // VAE decoding
        on_progress(GenerationProgress::new(PipelineStage::DecodingVae, 0.0));
        let vae = self.vae.as_ref().ok_or_else(|| anyhow::anyhow!("VAE model not loaded"))?;

        let vae_timer = Timer::start();
        let image = vae.decode(&latents)?;
        stats.vae_decode_ms = vae_timer.stop();
        stats.image_shape = image.dims().to_vec();
        on_progress(GenerationProgress::new(PipelineStage::DecodingVae, 1.0));

        // PNG encoding
        on_progress(GenerationProgress::new(PipelineStage::EncodingPng, 0.0));
        let png_timer = Timer::start();
        let rgb_data = vae.tensor_to_rgb(&image)?;

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

        let png_data = encode_png_with_metadata(&rgb_data, width as u32, height as u32, &final_metadata)?;
        stats.png_encode_ms = png_timer.stop();
        on_progress(GenerationProgress::new(PipelineStage::EncodingPng, 1.0));

        stats.total_ms = total_timer.stop();

        Ok(GenerationResult {
            image_data: png_data,
            stats,
        })
    }
}
