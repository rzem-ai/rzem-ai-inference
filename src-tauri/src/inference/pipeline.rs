//! Flux model inference pipeline

use anyhow::Result;
use candle_core::Device;
use crate::models::{ClipTextEncoder, VaeDecoder, FluxTransformer, ModelPaths};
use image::{ImageBuffer, Rgb};

/// Flux diffusion model pipeline for image generation
pub struct FluxPipeline {
    device: Device,
    clip: Option<ClipTextEncoder>,
    vae: Option<VaeDecoder>,
    flux: Option<FluxTransformer>,
}

impl FluxPipeline {
    /// Creates a new Flux pipeline instance
    pub fn new(device: Device) -> Result<Self> {
        Ok(Self {
            device,
            clip: None,
            vae: None,
            flux: None,
        })
    }

    /// Load models if not already loaded
    fn ensure_models_loaded(&mut self) -> Result<()> {
        if self.clip.is_some() && self.vae.is_some() && self.flux.is_some() {
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

        // Load CLIP text encoder
        self.clip = Some(ClipTextEncoder::load(
            paths.clip_path().join("model.safetensors"),
            paths.tokenizer_path().join("tokenizer.json"),
            self.device.clone(),
        )?);

        // Load VAE decoder
        self.vae = Some(VaeDecoder::load(
            paths.vae_path().join("diffusion_pytorch_model.safetensors"),
            self.device.clone(),
        )?);

        // Load FLUX transformer
        self.flux = Some(FluxTransformer::load(
            paths.transformer_path().join("diffusion_pytorch_model.safetensors"),
            self.device.clone(),
        )?);

        println!("Models loaded successfully!");
        Ok(())
    }

    /// Generate image from text prompt (real implementation)
    pub fn generate(&mut self, prompt: &str, steps: usize) -> Result<Vec<u8>> {
        // Load models if needed
        self.ensure_models_loaded()?;

        let clip = self.clip.as_ref()
            .ok_or_else(|| anyhow::anyhow!("CLIP model not loaded"))?;
        let vae = self.vae.as_ref()
            .ok_or_else(|| anyhow::anyhow!("VAE model not loaded"))?;
        let flux = self.flux.as_ref()
            .ok_or_else(|| anyhow::anyhow!("FLUX model not loaded"))?;

        println!("Encoding prompt: {}", prompt);
        // 1. Encode text prompt
        let embeddings = clip.encode(prompt)?;

        println!("Creating initial noise...");
        // 2. Create random noise
        let noise = flux.create_noise(1024, 1024)?;

        println!("Denoising for {} steps...", steps);
        // 3. Denoise latents
        let latents = flux.denoise(&noise, &embeddings, steps)?;

        println!("Decoding to image...");
        // 4. Decode latents to RGB
        let image = vae.decode(&latents)?;

        println!("Converting to PNG...");
        // 5. Convert to PNG
        let rgb_data = vae.tensor_to_rgb(&image)?;

        // Convert RGB data to PNG
        let (height, width) = (1024, 1024);
        let img = image::RgbImage::from_raw(width as u32, height as u32, rgb_data)
            .ok_or_else(|| anyhow::anyhow!("Failed to create image from raw data"))?;

        let mut png_data = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut png_data),
            image::ImageFormat::Png
        )?;

        Ok(png_data)
    }

    /// Keep stub method for backward compatibility
    pub fn generate_stub(&self, prompt: &str, _steps: usize) -> Result<Vec<u8>> {
        // Simplified stub that still works
        const WIDTH: u32 = 1024;
        const HEIGHT: u32 = 1024;

        let mut img = ImageBuffer::new(WIDTH, HEIGHT);
        let intensity = (prompt.len() % 256) as u8;

        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let r = ((x + y) % 256) as u8;
            let g = intensity;
            let b = ((x * y) % 256) as u8;
            *pixel = Rgb([r, g, b]);
        }

        let mut png_data = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)?;

        Ok(png_data)
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
    fn test_generate_stub() {
        let engine = InferenceEngine::new().unwrap();
        let device = engine.get_device().clone();
        let pipeline = FluxPipeline::new(device).unwrap();

        let result = pipeline.generate_stub("test prompt", 4).unwrap();
        assert!(!result.is_empty());
        assert_eq!(&result[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    #[ignore] // Requires downloaded models
    fn test_real_generation() {
        let engine = InferenceEngine::new().unwrap();
        let device = engine.get_device().clone();
        let mut pipeline = FluxPipeline::new(device).unwrap();

        let result = pipeline.generate("a cat", 4).unwrap();
        assert!(!result.is_empty());
        assert_eq!(&result[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }
}
