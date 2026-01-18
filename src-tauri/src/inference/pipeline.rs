//! Flux model inference pipeline

use anyhow::Result;
use candle_core::Device;
use crate::models::{ClipTextEncoder, VaeDecoder, FluxTransformer, ModelPaths, T5TextEncoder};

/// Flux diffusion model pipeline for image generation
pub struct FluxPipeline {
    device: Device,
    t5: Option<T5TextEncoder>,
    clip: Option<ClipTextEncoder>,
    vae: Option<VaeDecoder>,
    flux: Option<FluxTransformer>,
}

impl FluxPipeline {
    /// Creates a new Flux pipeline instance
    pub fn new(device: Device) -> Result<Self> {
        Ok(Self {
            device,
            t5: None,
            clip: None,
            vae: None,
            flux: None,
        })
    }

    /// Load models if not already loaded
    fn ensure_models_loaded(&mut self) -> Result<()> {
        if self.t5.is_some() && self.clip.is_some() && self.vae.is_some() && self.flux.is_some() {
            return Ok(()); // Already loaded
        }

        let paths = ModelPaths::new()?;

        // Check if models are downloaded
        if !paths.all_files_exist() {
            return Err(anyhow::anyhow!(
                "FLUX models not downloaded. Run model downloader first."
            ));
        }

        println!("Loading FLUX Schnell models...");

        // Load T5 text encoder (full precision)
        // Note: Quantized T5 GGUF from city96 uses llama.cpp tensor naming
        // which is incompatible with Candle's expected format
        println!("  Loading T5 encoder...");
        self.t5 = Some(T5TextEncoder::load(
            paths.t5_path(),
            paths.t5_tokenizer_path(),
            self.device.clone(),
        )?);

        // Load CLIP text encoder (pooled embedding for vec)
        println!("  Loading CLIP encoder...");
        self.clip = Some(ClipTextEncoder::load(
            paths.clip_path().join("model.safetensors"),
            paths.tokenizer_path(),
            self.device.clone(),
        )?);

        // Load VAE decoder
        println!("  Loading VAE decoder...");
        self.vae = Some(VaeDecoder::load(
            paths.vae_path(),
            self.device.clone(),
        )?);

        // Load FLUX transformer - prefer quantized model if available (saves ~11GB VRAM)
        if paths.has_quantized_transformer() {
            println!("  Loading FLUX transformer (quantized GGUF - memory optimized)...");
            self.flux = Some(FluxTransformer::load_quantized(
                paths.quantized_transformer_path(),
                self.device.clone(),
            )?);
        } else {
            println!("  Loading FLUX transformer (full precision)...");
            self.flux = Some(FluxTransformer::load(
                paths.transformer_path(),
                self.device.clone(),
            )?);
        }

        println!("Models loaded successfully!");
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
    pub fn generate(
        &mut self,
        prompt: &str,
        steps: usize,
        width: usize,
        height: usize,
        guidance: f64,
    ) -> Result<Vec<u8>> {
        // Load models if needed
        self.ensure_models_loaded()?;

        // Get T5 for encoding (will be unloaded after)
        let t5 = self.t5.as_mut()
            .ok_or_else(|| anyhow::anyhow!("T5 model not loaded"))?;

        println!("Encoding prompt with T5: {}", prompt);
        let t5_emb = t5.encode(prompt)?;
        println!("  T5 embedding shape: {:?}", t5_emb.dims());

        // Get CLIP for encoding (will be unloaded after FLUX)
        let clip = self.clip.as_ref()
            .ok_or_else(|| anyhow::anyhow!("CLIP model not loaded"))?;

        println!("Encoding prompt with CLIP...");
        let clip_emb = clip.encode(prompt)?;
        println!("  CLIP embedding shape: {:?}", clip_emb.dims());

        // Unload T5 to free ~9GB VRAM (no longer needed after encoding)
        println!("  Unloading T5 to free memory...");
        self.t5 = None;

        let flux = self.flux.as_ref()
            .ok_or_else(|| anyhow::anyhow!("FLUX model not loaded"))?;

        let model_type = if flux.is_quantized() { "quantized" } else { "full precision" };
        println!("Denoising for {} steps (guidance={}, model={})...", steps, guidance, model_type);
        let latents = flux.denoise(&t5_emb, &clip_emb, height, width, steps, guidance)?;
        println!("  Latent shape: {:?}", latents.dims());

        // Unload FLUX to free ~12GB VRAM (no longer needed after denoising)
        println!("  Unloading FLUX to free memory...");
        drop(t5_emb);
        drop(clip_emb);
        self.flux = None;
        self.clip = None;

        let vae = self.vae.as_ref()
            .ok_or_else(|| anyhow::anyhow!("VAE model not loaded"))?;

        println!("Decoding to image...");
        let image = vae.decode(&latents)?;
        println!("  Image shape: {:?}", image.dims());

        println!("Converting to PNG...");
        let rgb_data = vae.tensor_to_rgb(&image)?;

        // Convert RGB data to PNG
        let img = image::RgbImage::from_raw(width as u32, height as u32, rgb_data)
            .ok_or_else(|| anyhow::anyhow!("Failed to create image from raw data"))?;

        let mut png_data = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut png_data),
            image::ImageFormat::Png
        )?;

        println!("Generation complete!");
        Ok(png_data)
    }

    /// Simplified generate with defaults
    pub fn generate_simple(&mut self, prompt: &str, steps: usize) -> Result<Vec<u8>> {
        self.generate(prompt, steps, 1024, 1024, 4.0)
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
        assert!(!result.is_empty());
        // PNG magic number
        assert_eq!(&result[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }
}
