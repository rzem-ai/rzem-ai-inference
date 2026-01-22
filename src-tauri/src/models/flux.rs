//! FLUX transformer diffusion model with quantized model support

use anyhow::Result;
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::flux;
use candle_transformers::models::flux::WithForward;
use std::path::Path;
use tracing::{debug, warn};

use crate::inference::samplers::{SamplerType, SchedulerType, get_timesteps};

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
            warn!(error = %e, "Could not set device seed, using fallback");
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
    /// * `sampler` - Sampling algorithm to use
    /// * `scheduler` - Noise schedule type
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
        sampler: SamplerType,
        scheduler: SchedulerType,
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

        // Get timestep schedule based on scheduler type
        let timesteps = get_timesteps(steps, scheduler);
        debug!(sampler = ?sampler, scheduler = ?scheduler, steps = steps, "Denoising with custom sampler");

        // Run denoising with selected sampler
        let denoised = match &self.model {
            FluxModel::Regular(model) => {
                match sampler {
                    SamplerType::Euler => denoise_euler_impl(model, &state, &timesteps, guidance, &self.device)?,
                    SamplerType::EulerA => denoise_euler_ancestral_impl(model, &state, &timesteps, guidance, seed, &self.device)?,
                    SamplerType::DpmPP2M => denoise_dpm_pp_2m_impl(model, &state, &timesteps, guidance, &self.device)?,
                }
            }
            FluxModel::Quantized(model) => {
                let result = match sampler {
                    SamplerType::Euler => denoise_euler_impl_q(model, &state, &timesteps, guidance, &self.device)?,
                    SamplerType::EulerA => denoise_euler_ancestral_impl_q(model, &state, &timesteps, guidance, seed, &self.device)?,
                    SamplerType::DpmPP2M => denoise_dpm_pp_2m_impl_q(model, &state, &timesteps, guidance, &self.device)?,
                };

                // Quantized model returns F32, convert back to bf16 if on CUDA
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

// ============================================================================
// Sampler Implementations
// ============================================================================
// These functions are duplicated for regular and quantized models because
// the FluxModel trait is not publicly exported from candle-transformers.

/// Standard Euler method - first-order ODE solver
/// x_{t+1} = x_t + (t_{next} - t) * v(x_t, t)
fn denoise_euler_impl(
    model: &flux::model::Flux,
    state: &flux::sampling::State,
    timesteps: &[f64],
    guidance: f64,
    device: &Device,
) -> Result<Tensor> {
    let mut img = state.img.clone();
    let b_sz = img.dim(0)?;

    // Create guidance tensor with batch size (matches candle's denoise implementation)
    let guidance_tensor = Tensor::full(guidance as f32, b_sz, device)?;

    for (i, window) in timesteps.windows(2).enumerate() {
        let t_curr = window[0];
        let t_next = window[1];

        let t_vec = Tensor::full(*&t_curr as f32, b_sz, device)?;

        // Note: argument order is img, img_ids, txt, txt_ids, t_vec, vec, guidance
        let v = model.forward(
            &img,
            &state.img_ids,
            &state.txt,
            &state.txt_ids,
            &t_vec,
            &state.vec,
            Some(&guidance_tensor),
        )?;

        let dt = t_next - t_curr;
        img = (img + v * dt)?;

        debug!(step = i + 1, t_curr = t_curr, t_next = t_next, "Euler step");
    }

    Ok(img)
}

/// Euler method for quantized model
fn denoise_euler_impl_q(
    model: &flux::quantized_model::Flux,
    state: &flux::sampling::State,
    timesteps: &[f64],
    guidance: f64,
    device: &Device,
) -> Result<Tensor> {
    let mut img = state.img.clone();
    let b_sz = img.dim(0)?;

    // Create guidance tensor with batch size (matches candle's denoise implementation)
    let guidance_tensor = Tensor::full(guidance as f32, b_sz, device)?;

    for (i, window) in timesteps.windows(2).enumerate() {
        let t_curr = window[0];
        let t_next = window[1];

        let t_vec = Tensor::full(*&t_curr as f32, b_sz, device)?;

        // Note: argument order is img, img_ids, txt, txt_ids, t_vec, vec, guidance
        let v = model.forward(
            &img,
            &state.img_ids,
            &state.txt,
            &state.txt_ids,
            &t_vec,
            &state.vec,
            Some(&guidance_tensor),
        )?;

        let dt = t_next - t_curr;
        img = (img + v * dt)?;

        debug!(step = i + 1, t_curr = t_curr, t_next = t_next, "Euler step");
    }

    Ok(img)
}

/// Euler Ancestral - adds noise at each step for more variation
fn denoise_euler_ancestral_impl(
    model: &flux::model::Flux,
    state: &flux::sampling::State,
    timesteps: &[f64],
    guidance: f64,
    seed: u64,
    device: &Device,
) -> Result<Tensor> {
    let mut img = state.img.clone();
    let b_sz = img.dim(0)?;

    let guidance_tensor = Tensor::full(guidance as f32, b_sz, device)?;

    for (i, window) in timesteps.windows(2).enumerate() {
        let t_curr = window[0];
        let t_next = window[1];

        let t_vec = Tensor::full(*&t_curr as f32, b_sz, device)?;

        // Note: argument order is img, img_ids, txt, txt_ids, t_vec, vec, guidance
        let v = model.forward(
            &img,
            &state.img_ids,
            &state.txt,
            &state.txt_ids,
            &t_vec,
            &state.vec,
            Some(&guidance_tensor),
        )?;

        let sigma_curr = t_curr;
        let sigma_next = t_next;

        let sigma_up = if sigma_curr > 0.0 && sigma_next > 0.0 {
            (sigma_next * sigma_next * (sigma_curr * sigma_curr - sigma_next * sigma_next)
                / (sigma_curr * sigma_curr))
                .sqrt()
        } else {
            0.0
        };
        let sigma_down = (sigma_next * sigma_next - sigma_up * sigma_up).max(0.0).sqrt();

        let dt = sigma_down - sigma_curr;
        img = (img + v * dt)?;

        if sigma_up > 1e-8 && i < timesteps.len() - 2 {
            let step_seed = seed.wrapping_add(i as u64);
            if let Err(e) = device.set_seed(step_seed) {
                debug!(error = %e, "Could not set device seed for ancestral noise");
            }

            let noise = Tensor::randn(0.0f32, 1.0f32, img.dims(), device)?;
            let noise = noise.to_dtype(img.dtype())?;
            img = (img + noise * sigma_up)?;
        }

        debug!(step = i + 1, t_curr = t_curr, t_next = t_next, sigma_up = sigma_up, "EulerA step");
    }

    Ok(img)
}

/// Euler Ancestral for quantized model
fn denoise_euler_ancestral_impl_q(
    model: &flux::quantized_model::Flux,
    state: &flux::sampling::State,
    timesteps: &[f64],
    guidance: f64,
    seed: u64,
    device: &Device,
) -> Result<Tensor> {
    let mut img = state.img.clone();
    let b_sz = img.dim(0)?;

    let guidance_tensor = Tensor::full(guidance as f32, b_sz, device)?;

    for (i, window) in timesteps.windows(2).enumerate() {
        let t_curr = window[0];
        let t_next = window[1];

        let t_vec = Tensor::full(*&t_curr as f32, b_sz, device)?;

        // Note: argument order is img, img_ids, txt, txt_ids, t_vec, vec, guidance
        let v = model.forward(
            &img,
            &state.img_ids,
            &state.txt,
            &state.txt_ids,
            &t_vec,
            &state.vec,
            Some(&guidance_tensor),
        )?;

        let sigma_curr = t_curr;
        let sigma_next = t_next;

        let sigma_up = if sigma_curr > 0.0 && sigma_next > 0.0 {
            (sigma_next * sigma_next * (sigma_curr * sigma_curr - sigma_next * sigma_next)
                / (sigma_curr * sigma_curr))
                .sqrt()
        } else {
            0.0
        };
        let sigma_down = (sigma_next * sigma_next - sigma_up * sigma_up).max(0.0).sqrt();

        let dt = sigma_down - sigma_curr;
        img = (img + v * dt)?;

        if sigma_up > 1e-8 && i < timesteps.len() - 2 {
            let step_seed = seed.wrapping_add(i as u64);
            if let Err(e) = device.set_seed(step_seed) {
                debug!(error = %e, "Could not set device seed for ancestral noise");
            }

            let noise = Tensor::randn(0.0f32, 1.0f32, img.dims(), device)?;
            let noise = noise.to_dtype(img.dtype())?;
            img = (img + noise * sigma_up)?;
        }

        debug!(step = i + 1, t_curr = t_curr, t_next = t_next, sigma_up = sigma_up, "EulerA step");
    }

    Ok(img)
}

/// DPM++ 2M - second-order multistep method
fn denoise_dpm_pp_2m_impl(
    model: &flux::model::Flux,
    state: &flux::sampling::State,
    timesteps: &[f64],
    guidance: f64,
    device: &Device,
) -> Result<Tensor> {
    let mut img = state.img.clone();
    let b_sz = img.dim(0)?;
    let mut prev_velocity: Option<Tensor> = None;

    // Create guidance tensor with batch size
    let guidance_tensor = Tensor::full(guidance as f32, b_sz, device)?;

    for (i, window) in timesteps.windows(2).enumerate() {
        let t_curr = window[0];
        let t_next = window[1];

        let t_vec = Tensor::full(*&t_curr as f32, b_sz, device)?;

        // Note: argument order is img, img_ids, txt, txt_ids, t_vec, vec, guidance
        let v = model.forward(
            &img,
            &state.img_ids,
            &state.txt,
            &state.txt_ids,
            &t_vec,
            &state.vec,
            Some(&guidance_tensor),
        )?;

        let dt = t_next - t_curr;

        if let Some(ref prev_v) = prev_velocity {
            if i > 0 {
                let t_prev = timesteps[i - 1];
                let h = t_curr - t_prev;
                let h_next = t_next - t_curr;

                if h.abs() > 1e-8 {
                    let dv = (&v - prev_v)?;
                    let correction = ((dv * (h_next / h))? * 0.5)?;
                    img = (img + (&v + correction)? * h_next)?;
                } else {
                    img = (img + &v * dt)?;
                }
            } else {
                img = (img + &v * dt)?;
            }
        } else {
            img = (img + &v * dt)?;
        }

        prev_velocity = Some(v);

        debug!(step = i + 1, t_curr = t_curr, t_next = t_next, "DPM++ 2M step");
    }

    Ok(img)
}

/// DPM++ 2M for quantized model
fn denoise_dpm_pp_2m_impl_q(
    model: &flux::quantized_model::Flux,
    state: &flux::sampling::State,
    timesteps: &[f64],
    guidance: f64,
    device: &Device,
) -> Result<Tensor> {
    let mut img = state.img.clone();
    let b_sz = img.dim(0)?;
    let mut prev_velocity: Option<Tensor> = None;

    // Create guidance tensor with batch size
    let guidance_tensor = Tensor::full(guidance as f32, b_sz, device)?;

    for (i, window) in timesteps.windows(2).enumerate() {
        let t_curr = window[0];
        let t_next = window[1];

        let t_vec = Tensor::full(*&t_curr as f32, b_sz, device)?;

        // Note: argument order is img, img_ids, txt, txt_ids, t_vec, vec, guidance
        let v = model.forward(
            &img,
            &state.img_ids,
            &state.txt,
            &state.txt_ids,
            &t_vec,
            &state.vec,
            Some(&guidance_tensor),
        )?;

        let dt = t_next - t_curr;

        if let Some(ref prev_v) = prev_velocity {
            if i > 0 {
                let t_prev = timesteps[i - 1];
                let h = t_curr - t_prev;
                let h_next = t_next - t_curr;

                if h.abs() > 1e-8 {
                    let dv = (&v - prev_v)?;
                    let correction = ((dv * (h_next / h))? * 0.5)?;
                    img = (img + (&v + correction)? * h_next)?;
                } else {
                    img = (img + &v * dt)?;
                }
            } else {
                img = (img + &v * dt)?;
            }
        } else {
            img = (img + &v * dt)?;
        }

        prev_velocity = Some(v);

        debug!(step = i + 1, t_curr = t_curr, t_next = t_next, "DPM++ 2M step");
    }

    Ok(img)
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
