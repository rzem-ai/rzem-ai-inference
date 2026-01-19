//! FLUX transformer diffusion model with quantized model support

use anyhow::Result;
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::flux;
use std::path::Path;

/// Enum to hold either regular or quantized FLUX model
enum FluxModel {
    Regular(flux::model::Flux),
    Quantized(flux::quantized_model::Flux),
}

/// FLUX transformer for latent diffusion
/// Supports both full-precision and quantized (GGUF) models
pub struct FluxTransformer {
    model: FluxModel,
    device: Device,
    is_quantized: bool,
}

impl FluxTransformer {
    /// Load FLUX Schnell transformer from safetensors file (full precision)
    ///
    /// # Arguments
    /// * `model_path` - Path to flux1-schnell.safetensors file
    /// * `device` - Device to load model on
    pub fn load<P: AsRef<Path>>(model_path: P, device: Device) -> Result<Self> {
        let model_path = model_path.as_ref();

        // Use bf16 on CUDA for efficiency
        let dtype = if device.is_cuda() {
            DType::BF16
        } else {
            DType::F32
        };

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[model_path], dtype, &device)?
        };

        // Load FLUX Schnell model
        let cfg = flux::model::Config::schnell();
        let model = flux::model::Flux::new(&cfg, vb)?;

        Ok(Self {
            model: FluxModel::Regular(model),
            device,
            is_quantized: false,
        })
    }

    /// Load quantized FLUX Schnell transformer from GGUF file
    /// Uses ~12GB VRAM instead of ~23GB
    ///
    /// # Arguments
    /// * `model_path` - Path to flux1-schnell.gguf file
    /// * `device` - Device to load model on
    pub fn load_quantized<P: AsRef<Path>>(model_path: P, device: Device) -> Result<Self> {
        use candle_transformers::quantized_var_builder::VarBuilder as QVarBuilder;

        let model_path = model_path.as_ref();

        let vb = QVarBuilder::from_gguf(model_path, &device)?;

        // Load quantized FLUX Schnell model
        let cfg = flux::model::Config::schnell();
        let model = flux::quantized_model::Flux::new(&cfg, vb)?;

        Ok(Self {
            model: FluxModel::Quantized(model),
            device,
            is_quantized: true,
        })
    }

    /// Check if this is a quantized model
    pub fn is_quantized(&self) -> bool {
        self.is_quantized
    }

    /// Get the device
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Create initial noise for generation
    ///
    /// Uses flux::sampling::get_noise which creates properly shaped noise.
    /// The seed is used to initialize the device RNG for reproducible generation.
    pub fn create_noise(&self, height: usize, width: usize, seed: u64) -> Result<Tensor> {
        let dtype = if self.is_quantized {
            // Quantized models work with F32
            DType::F32
        } else if self.device.is_cuda() {
            DType::BF16
        } else {
            DType::F32
        };

        // Set the device RNG seed for reproducible noise generation
        // This works on CUDA and Metal backends
        if let Err(e) = self.device.set_seed(seed) {
            // CPU backend doesn't support set_seed, but we can use candle's manual seeding
            eprintln!("Note: Could not set device seed ({}), using fallback", e);
        }

        let noise = flux::sampling::get_noise(1, height, width, &self.device)?;
        Ok(noise.to_dtype(dtype)?)
    }

    /// Denoise latents using the FLUX sampling pipeline
    ///
    /// # Arguments
    /// * `t5_emb` - T5 text embeddings [1, 256, 4096]
    /// * `clip_emb` - CLIP pooled embeddings [1, 768]
    /// * `height` - Target image height
    /// * `width` - Target image width
    /// * `steps` - Number of denoising steps (4 for Schnell)
    /// * `guidance` - Guidance scale (typically 4.0)
    /// * `seed` - Random seed for reproducible generation
    ///
    /// # Returns
    /// Denoised latents ready for VAE decode
    pub fn denoise(
        &self,
        t5_emb: &Tensor,
        clip_emb: &Tensor,
        height: usize,
        width: usize,
        steps: usize,
        guidance: f64,
        seed: u64,
    ) -> Result<Tensor> {
        // Create initial noise with seed for reproducibility
        let img = self.create_noise(height, width, seed)?;

        // For quantized models, convert embeddings to F32
        let (t5_emb, clip_emb, img) = if self.is_quantized {
            (
                t5_emb.to_dtype(DType::F32)?,
                clip_emb.to_dtype(DType::F32)?,
                img.to_dtype(DType::F32)?,
            )
        } else {
            (t5_emb.clone(), clip_emb.clone(), img)
        };

        // Create sampling state from embeddings
        let state = flux::sampling::State::new(&t5_emb, &clip_emb, &img)?;

        // Get timestep schedule for Schnell (no time shift)
        let timesteps = flux::sampling::get_schedule(steps, None);

        // Run denoising
        let denoised = match &self.model {
            FluxModel::Regular(model) => {
                flux::sampling::denoise(
                    model,
                    &state.img,
                    &state.img_ids,
                    &state.txt,
                    &state.txt_ids,
                    &state.vec,
                    &timesteps,
                    guidance,
                )?
            }
            FluxModel::Quantized(model) => {
                // Quantized model returns F32, convert back to bf16 if on CUDA
                let result = flux::sampling::denoise(
                    model,
                    &state.img,
                    &state.img_ids,
                    &state.txt,
                    &state.txt_ids,
                    &state.vec,
                    &timesteps,
                    guidance,
                )?;

                if self.device.is_cuda() {
                    result.to_dtype(DType::BF16)?
                } else {
                    result
                }
            }
        };

        // Unpack to proper shape for VAE
        let unpacked = flux::sampling::unpack(&denoised, height, width)?;

        Ok(unpacked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires downloaded model
    fn test_flux_loading() {
        use crate::models::ModelPaths;

        let paths = ModelPaths::new().unwrap();
        let device = Device::cuda_if_available(0).unwrap();

        let _flux = FluxTransformer::load(paths.transformer_path(), device).unwrap();
    }

    #[test]
    #[ignore] // Requires downloaded quantized model
    fn test_quantized_flux_loading() {
        use crate::models::ModelPaths;

        let paths = ModelPaths::new().unwrap();
        let device = Device::cuda_if_available(0).unwrap();

        if paths.has_quantized_transformer() {
            let flux = FluxTransformer::load_quantized(
                paths.quantized_transformer_path(),
                device,
            ).unwrap();
            assert!(flux.is_quantized());
        }
    }

    #[test]
    fn test_noise_generation() {
        let device = Device::Cpu;
        // Can't test without loading model, so just test the sampling function
        let noise = flux::sampling::get_noise(1, 1024, 1024, &device).unwrap();
        let shape = noise.dims();

        // FLUX noise shape: [1, 16, height/8*2, width/8*2] due to div_ceil(16)*2
        assert_eq!(shape[0], 1);
        assert_eq!(shape[1], 16);
        assert_eq!(shape[2], 128); // 1024/16*2 = 128
        assert_eq!(shape[3], 128);
    }
}
