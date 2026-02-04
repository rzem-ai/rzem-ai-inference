//! Image generation methods for FluxPipeline

use anyhow::Result;
use tracing::{debug, info, trace, warn};
use base64::{engine::general_purpose, Engine as _};
use candle_core::Tensor;

use crate::inference::metadata::{ImageMetadata, encode_png_with_metadata};
use crate::inference::samplers::{SamplerType, SchedulerType};
use crate::inference::stats::{GenerationStats, GenerationResult, Timer};
use crate::inference::{GenerationProgress, PipelineStage};
use candle_transformers::models::flux;
use super::FluxPipeline;

/// Encode RGB data as JPEG and return base64 string
///
/// # Arguments
/// * `rgb_data` - Raw RGB pixel data (width * height * 3 bytes)
/// * `width` - Image width in pixels
/// * `height` - Image height in pixels
/// * `quality` - JPEG quality (1-100, recommended 70-80)
fn encode_jpeg_base64(rgb_data: &[u8], width: u32, height: u32, quality: u8) -> Result<String> {
    use image::{ImageBuffer, RgbImage};
    use std::io::Cursor;

    // Create image buffer
    let img: RgbImage = ImageBuffer::from_raw(width, height, rgb_data.to_vec())
        .ok_or_else(|| anyhow::anyhow!("Failed to create image buffer"))?;

    // Encode as JPEG to memory
    let mut jpeg_data = Cursor::new(Vec::new());
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_data, quality);
    encoder.encode(
        img.as_raw(),
        width,
        height,
        image::ExtendedColorType::Rgb8,
    )?;

    // Base64 encode
    let base64 = general_purpose::STANDARD.encode(jpeg_data.into_inner());
    Ok(base64)
}

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
        info!(steps = steps, guidance = guidance, model = %stats.model_type, seed = seed, sampler = ?sampler, scheduler = ?scheduler, "Drawing");
        let denoise_timer = Timer::start();
        let latents = flux.denoise(&t5_emb, &clip_emb, height, width, steps, guidance, seed, sampler, scheduler, None::<fn(usize, usize, &Tensor)>)?;
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
            loras: Vec::new(),  // No LoRAs in simple generate
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

        // Loading models and encoding stage (0.0 - 0.5)
        trace!("Loading models stage");
        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 0.0));
        self.ensure_models_loaded(&mut stats)?;

        // Progress after models loaded (before encoding)
        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 0.5));

        // Get T5 for encoding
        let t5 = self.t5.as_mut()
            .ok_or_else(|| anyhow::anyhow!("T5 model not loaded"))?;

        let t5_timer = Timer::start();
        trace!(prompt_preview = %&prompt[..prompt.len().min(50)], "Encoding with T5");
        let t5_emb = t5.encode(prompt)?;
        stats.t5_encode_ms = t5_timer.stop();
        stats.t5_embedding_shape = t5_emb.dims().to_vec();
        debug!(shape = ?stats.t5_embedding_shape, time_ms = stats.t5_encode_ms, "T5 encoding complete");

        // Progress after T5 encoding
        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 0.75));

        // Get CLIP for encoding
        let clip = self.clip.as_ref()
            .ok_or_else(|| anyhow::anyhow!("CLIP model not loaded"))?;

        let clip_timer = Timer::start();
        let clip_emb = clip.encode(prompt)?;
        stats.clip_encode_ms = clip_timer.stop();
        stats.clip_embedding_shape = clip_emb.dims().to_vec();
        debug!(shape = ?stats.clip_embedding_shape, time_ms = stats.clip_encode_ms, "CLIP encoding complete");

        // Models ready, encoding complete (progress = 0.5)
        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 1.0));

        // Note: Model unloading is now handled by apply_cache_config() after generation
        // This allows the cache system to decide what to keep loaded

        // Drawing stage (0.5 - 0.95)
        trace!("Drawing stage");
        let flux = self.flux.as_ref()
            .ok_or_else(|| anyhow::anyhow!("FLUX model not loaded"))?;

        stats.model_type = if flux.is_quantized() {
            "quantized".to_string()
        } else {
            "full_precision".to_string()
        };

        // Log GPU memory before drawing starts
        if let Some((used, total, percent)) = get_gpu_memory_stats() {
            let available = total - used;
            info!(
                steps = steps,
                model = %stats.model_type,
                sampler = ?sampler,
                scheduler = ?scheduler,
                gpu_used = format_bytes(used),
                gpu_available = format_bytes(available),
                gpu_percent = format!("{:.1}%", percent),
                "Starting drawing"
            );

            // Warn if memory is critically low (>90% used)
            if percent > 90.0 {
                warn!(
                    gpu_percent = format!("{:.1}%", percent),
                    available = format_bytes(available),
                    "GPU memory critically low before drawing - OOM risk"
                );
            }
        } else {
            info!(steps = steps, model = %stats.model_type, sampler = ?sampler, scheduler = ?scheduler, "Starting drawing");
        }

        let denoise_timer = Timer::start();

        // Calculate preview interval (generate 5 previews total: 20%, 40%, 60%, 80%, 100%)
        let preview_interval = (steps / 5).max(1);

        // Clone VAE Arc and dimensions for callback closure
        let vae_for_callback = self.vae.clone();
        let width_for_callback = width;
        let height_for_callback = height;

        // Wrap on_progress in Arc so it can be shared between closure and code after denoise
        let on_progress = std::sync::Arc::new(on_progress);
        let on_progress_clone = on_progress.clone();

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
            Some(move |current_step: usize, total_steps: usize, latent: &Tensor| {
                // Always report progress
                let mut progress = GenerationProgress::denoising_step(current_step, total_steps);

                // Generate preview at intervals or final step
                if current_step % preview_interval == 0 || current_step == total_steps {
                    debug!(
                        step = current_step,
                        total = total_steps,
                        interval = preview_interval,
                        "Generating preview"
                    );
                    match vae_for_callback.as_ref() {
                        Some(vae) => {
                            // Unpack latent from packed format [1, 2048, H] to VAE format [1, 16, H/8, W/8]
                            match flux::sampling::unpack(latent, height_for_callback, width_for_callback) {
                                Ok(unpacked_latent) => {
                                    // Convert to BF16 for VAE (VAE expects BF16 on CUDA, F32 on CPU)
                                    let latent_for_vae = if unpacked_latent.device().is_cuda() {
                                        match unpacked_latent.to_dtype(candle_core::DType::BF16) {
                                            Ok(converted) => converted,
                                            Err(e) => {
                                                warn!(step = current_step, error = ?e, "Failed to convert latent dtype");
                                                return on_progress_clone(progress);
                                            }
                                        }
                                    } else {
                                        unpacked_latent
                                    };

                                    // Decode latent to RGB image
                                    match vae.decode(&latent_for_vae) {
                                        Ok(image) => {
                                            match vae.tensor_to_rgb(&image) {
                                                Ok(rgb_data) => {
                                                    // Encode as JPEG with lower quality (60) for speed
                                                    match encode_jpeg_base64(
                                                        &rgb_data,
                                                        width_for_callback as u32,
                                                        height_for_callback as u32,
                                                        60  // Lower quality than final (75) for faster encoding
                                                    ) {
                                                        Ok(preview_base64) => {
                                                            debug!(
                                                                step = current_step,
                                                                preview_size = preview_base64.len(),
                                                                "Preview generated successfully"
                                                            );
                                                            progress = progress.with_preview(preview_base64);
                                                        }
                                                        Err(e) => {
                                                            warn!(step = current_step, error = ?e, "Failed to encode preview JPEG");
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    warn!(step = current_step, error = ?e, "Failed to convert preview to RGB");
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            warn!(step = current_step, error = ?e, "Failed to decode preview latent");
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!(step = current_step, error = ?e, "Failed to unpack preview latent");
                                }
                            }
                        }
                        None => {
                            warn!(step = current_step, "VAE not available for preview generation");
                        }
                    }
                }

                on_progress_clone(progress);
            }),
        )?;
        stats.denoise_ms = denoise_timer.stop();
        stats.latent_shape = latents.dims().to_vec();
        debug!(shape = ?stats.latent_shape, time_ms = stats.denoise_ms, "Denoising complete");

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

        // Generate preview JPEG (quality 75 for good balance)
        if let Ok(preview_base64) = encode_jpeg_base64(&rgb_data, width as u32, height as u32, 75) {
            let preview_progress = GenerationProgress::new(PipelineStage::DecodingVae, 1.0)
                .with_preview(preview_base64);
            on_progress(preview_progress);
        } else {
            warn!("Failed to generate preview JPEG, continuing without preview");
        }

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
            loras: Vec::new(),  // No LoRAs if metadata not provided
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

    /// Generate image using staged loading (for memory-constrained GPUs)
    ///
    /// This method implements a two-phase loading strategy:
    /// 1. Loads T5 + CLIP, encodes prompt, then unloads them (~9.5GB freed)
    /// 2. Loads VAE + FLUX (~24GB), generates image
    ///
    /// Peak memory: ~24GB instead of ~34GB, enabling FLUX.1-dev on 32GB GPUs.
    pub fn generate_with_staged_loading<F>(
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
        info!("Using staged loading for memory-constrained GPU");
        let total_timer = Timer::start();
        let mut stats = GenerationStats::default();
        stats.steps = steps;

        // ═══════════════════════════════════════════════════════════════════
        // PHASE 1: Text Encoding
        // Load T5 + CLIP (~9.5GB), encode prompt, then unload
        // ═══════════════════════════════════════════════════════════════════

        trace!("Phase 1: Loading text encoders");
        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 0.0));

        self.ensure_text_encoders_loaded(&mut stats)?;
        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 0.2));

        // Encode with T5
        let t5 = self.t5.as_mut()
            .ok_or_else(|| anyhow::anyhow!("T5 model not loaded after ensure_text_encoders_loaded"))?;

        let t5_timer = Timer::start();
        trace!(prompt_preview = %&prompt[..prompt.len().min(50)], "Encoding with T5");
        let t5_emb = t5.encode(prompt)?;
        stats.t5_encode_ms = t5_timer.stop();
        stats.t5_embedding_shape = t5_emb.dims().to_vec();
        debug!(shape = ?stats.t5_embedding_shape, time_ms = stats.t5_encode_ms, "T5 encoding complete");

        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 0.3));

        // Encode with CLIP
        let clip = self.clip.as_ref()
            .ok_or_else(|| anyhow::anyhow!("CLIP model not loaded after ensure_text_encoders_loaded"))?;

        let clip_timer = Timer::start();
        let clip_emb = clip.encode(prompt)?;
        stats.clip_encode_ms = clip_timer.stop();
        stats.clip_embedding_shape = clip_emb.dims().to_vec();
        debug!(shape = ?stats.clip_embedding_shape, time_ms = stats.clip_encode_ms, "CLIP encoding complete");

        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 0.4));

        // Unload text encoders to free ~9.5GB VRAM
        info!("Unloading text encoders to free VRAM before loading transformer");
        self.unload_text_encoders();

        // Log memory after unload
        if let Some((used, total, percent)) = get_gpu_memory_stats() {
            info!(
                gpu_used = format_bytes(used),
                gpu_total = format_bytes(total),
                gpu_percent = format!("{:.1}%", percent),
                "VRAM after unloading text encoders"
            );
        }

        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 0.5));

        // ═══════════════════════════════════════════════════════════════════
        // PHASE 2: Diffusion
        // Load VAE + FLUX (~24GB), denoise and decode
        // ═══════════════════════════════════════════════════════════════════

        trace!("Phase 2: Loading diffusion models");
        self.ensure_diffusion_models_loaded(&mut stats)?;

        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 1.0));

        // Get FLUX transformer
        let flux = self.flux.as_ref()
            .ok_or_else(|| anyhow::anyhow!("FLUX model not loaded after ensure_diffusion_models_loaded"))?;

        stats.model_type = if flux.is_quantized() {
            "quantized".to_string()
        } else {
            "full_precision".to_string()
        };

        // Log GPU memory before drawing
        if let Some((used, total, percent)) = get_gpu_memory_stats() {
            let available = total - used;
            info!(
                steps = steps,
                model = %stats.model_type,
                sampler = ?sampler,
                scheduler = ?scheduler,
                gpu_used = format_bytes(used),
                gpu_available = format_bytes(available),
                gpu_percent = format!("{:.1}%", percent),
                "Starting drawing (staged loading)"
            );

            if percent > 90.0 {
                warn!(
                    gpu_percent = format!("{:.1}%", percent),
                    available = format_bytes(available),
                    "GPU memory critically low before drawing - OOM risk"
                );
            }
        }

        // Denoise
        let denoise_timer = Timer::start();

        // Calculate preview interval (generate 5 previews total: 20%, 40%, 60%, 80%, 100%)
        let preview_interval = (steps / 5).max(1);

        // Clone VAE Arc and dimensions for callback closure
        let vae_for_callback = self.vae.clone();
        let width_for_callback = width;
        let height_for_callback = height;

        // Wrap on_progress in Arc so it can be shared between closure and code after denoise
        let on_progress = std::sync::Arc::new(on_progress);
        let on_progress_clone = on_progress.clone();

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
            Some(move |current_step: usize, total_steps: usize, latent: &Tensor| {
                // Always report progress
                let mut progress = GenerationProgress::denoising_step(current_step, total_steps);

                // Generate preview at intervals or final step
                if current_step % preview_interval == 0 || current_step == total_steps {
                    debug!(
                        step = current_step,
                        total = total_steps,
                        interval = preview_interval,
                        "Generating preview"
                    );
                    match vae_for_callback.as_ref() {
                        Some(vae) => {
                            // Unpack latent from packed format [1, 2048, H] to VAE format [1, 16, H/8, W/8]
                            match flux::sampling::unpack(latent, height_for_callback, width_for_callback) {
                                Ok(unpacked_latent) => {
                                    // Convert to BF16 for VAE (VAE expects BF16 on CUDA, F32 on CPU)
                                    let latent_for_vae = if unpacked_latent.device().is_cuda() {
                                        match unpacked_latent.to_dtype(candle_core::DType::BF16) {
                                            Ok(converted) => converted,
                                            Err(e) => {
                                                warn!(step = current_step, error = ?e, "Failed to convert latent dtype");
                                                return on_progress_clone(progress);
                                            }
                                        }
                                    } else {
                                        unpacked_latent
                                    };

                                    // Decode latent to RGB image
                                    match vae.decode(&latent_for_vae) {
                                        Ok(image) => {
                                            match vae.tensor_to_rgb(&image) {
                                                Ok(rgb_data) => {
                                                    // Encode as JPEG with lower quality (60) for speed
                                                    match encode_jpeg_base64(
                                                        &rgb_data,
                                                        width_for_callback as u32,
                                                        height_for_callback as u32,
                                                        60  // Lower quality than final (75) for faster encoding
                                                    ) {
                                                        Ok(preview_base64) => {
                                                            debug!(
                                                                step = current_step,
                                                                preview_size = preview_base64.len(),
                                                                "Preview generated successfully"
                                                            );
                                                            progress = progress.with_preview(preview_base64);
                                                        }
                                                        Err(e) => {
                                                            warn!(step = current_step, error = ?e, "Failed to encode preview JPEG");
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    warn!(step = current_step, error = ?e, "Failed to convert preview to RGB");
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            warn!(step = current_step, error = ?e, "Failed to decode preview latent");
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!(step = current_step, error = ?e, "Failed to unpack preview latent");
                                }
                            }
                        }
                        None => {
                            warn!(step = current_step, "VAE not available for preview generation");
                        }
                    }
                }

                on_progress_clone(progress);
            }),
        )?;
        stats.denoise_ms = denoise_timer.stop();
        stats.latent_shape = latents.dims().to_vec();
        debug!(shape = ?stats.latent_shape, time_ms = stats.denoise_ms, "Denoising complete");

        // Free embedding tensors
        trace!("Freeing embedding tensors");
        drop(t5_emb);
        drop(clip_emb);

        // VAE decoding
        trace!("VAE decoding stage");
        on_progress(GenerationProgress::new(PipelineStage::DecodingVae, 0.0));

        let vae = self.vae.as_ref()
            .ok_or_else(|| anyhow::anyhow!("VAE model not loaded after ensure_diffusion_models_loaded"))?;

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

        // Generate preview JPEG
        if let Ok(preview_base64) = encode_jpeg_base64(&rgb_data, width as u32, height as u32, 75) {
            let preview_progress = GenerationProgress::new(PipelineStage::DecodingVae, 1.0)
                .with_preview(preview_base64);
            on_progress(preview_progress);
        } else {
            warn!("Failed to generate preview JPEG, continuing without preview");
        }

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
            loras: Vec::new(),  // No LoRAs if metadata not provided
        });

        let png_data = encode_png_with_metadata(&rgb_data, width as u32, height as u32, &final_metadata)?;
        debug!(size_bytes = png_data.len(), "PNG encoding complete");
        stats.png_encode_ms = png_timer.stop();

        on_progress(GenerationProgress::new(PipelineStage::EncodingPng, 1.0));

        stats.total_ms = total_timer.stop();
        info!(total_ms = stats.total_ms, "Staged generation complete");

        Ok(GenerationResult {
            image_data: png_data,
            stats,
        })
    }
}
