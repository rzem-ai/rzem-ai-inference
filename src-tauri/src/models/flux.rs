//! FLUX transformer diffusion model with quantized model support

use anyhow::Result;
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::flux;
use candle_transformers::models::flux::WithForward;
use safetensors::SafeTensors;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::inference::samplers::{SamplerType, SchedulerType, get_timesteps};
use crate::models::lora::{LoraAdapter, LoraConfig};
use crate::models::ModelType;

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
    /// * `model_path` - Path to flux safetensors file
    /// * `device` - Device to load model on
    /// * `model_type` - Which FLUX model variant (Schnell or Dev)
    pub fn load<P: AsRef<Path>>(model_path: P, device: Device, model_type: ModelType) -> Result<Self> {
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

        // Load appropriate FLUX config
        let cfg = match model_type.id() {
            "schnell" => flux::model::Config::schnell(),
            "dev" => flux::model::Config::dev(),
            _ => anyhow::bail!("Unsupported model type for FLUX: {}", model_type),
        };
        let model = flux::model::Flux::new(&cfg, vb)?;

        Ok(Self {
            model: FluxModel::Regular(model),
            device,
            is_quantized: false,
        })
    }

    /// Load quantized FLUX transformer from GGUF file
    /// Uses ~12GB VRAM instead of ~23GB
    ///
    /// # Arguments
    /// * `model_path` - Path to flux GGUF file
    /// * `device` - Device to load model on
    /// * `model_type` - Which FLUX model variant (Schnell or Dev)
    pub fn load_quantized<P: AsRef<Path>>(model_path: P, device: Device, model_type: ModelType) -> Result<Self> {
        use candle_transformers::quantized_var_builder::VarBuilder as QVarBuilder;

        let model_path = model_path.as_ref();

        let vb = QVarBuilder::from_gguf(model_path, &device)?;

        // Load appropriate FLUX config
        let cfg = match model_type.id() {
            "schnell" => flux::model::Config::schnell(),
            "dev" => flux::model::Config::dev(),
            _ => anyhow::bail!("Unsupported model type for FLUX: {}", model_type),
        };
        let model = flux::quantized_model::Flux::new(&cfg, vb)?;

        Ok(Self {
            model: FluxModel::Quantized(model),
            device,
            is_quantized: true,
        })
    }

    /// Load FLUX transformer with LoRA adapters merged at load time
    ///
    /// This method loads the base model weights, applies LoRA modifications,
    /// and builds the model from the merged weights.
    ///
    /// # Arguments
    /// * `model_path` - Path to flux safetensors file
    /// * `device` - Device to load model on
    /// * `model_type` - Which FLUX model variant (Schnell or Dev)
    /// * `loras` - List of LoRA adapters with their strengths
    pub fn load_with_loras<P: AsRef<Path>>(
        model_path: P,
        device: Device,
        model_type: ModelType,
        loras: &[(Arc<LoraAdapter>, f32)],
    ) -> Result<Self> {
        let model_path = model_path.as_ref();

        // Use bf16 on CUDA for efficiency
        let dtype = if device.is_cuda() {
            DType::BF16
        } else {
            DType::F32
        };

        if loras.is_empty() {
            // No LoRAs, use standard loading
            return Self::load(model_path, device, model_type);
        }

        info!(
            lora_count = loras.len(),
            "Loading FLUX with LoRA adapters"
        );

        // Load base weights from safetensors
        let file_data = std::fs::read(model_path)?;
        let base_tensors = SafeTensors::deserialize(&file_data)?;

        // Create a hashmap to hold modified tensors
        let mut merged_tensors: HashMap<String, Tensor> = HashMap::new();

        // Load all base tensors
        for (name, _) in base_tensors.tensors() {
            let tensor = load_safetensor_to_candle(&base_tensors, &name, &device, dtype)?;
            merged_tensors.insert(name.clone(), tensor);
        }

        // Apply each LoRA
        for (lora, strength) in loras {
            debug!(
                lora_id = %lora.id,
                lora_name = %lora.name,
                strength = strength,
                weights = lora.weight_count(),
                "Applying LoRA"
            );

            for (layer_name, lora_weight) in &lora.weights {
                // Map LoRA layer name to FLUX model tensor name
                let flux_tensor_name = map_lora_to_flux_tensor(layer_name);

                if let Some(base_tensor) = merged_tensors.get_mut(&flux_tensor_name) {
                    // Compute LoRA delta: (alpha/rank) * strength * (B @ A)
                    match lora_weight.compute_delta(*strength) {
                        Ok(delta) => {
                            // Ensure delta has same dtype as base
                            let delta = if delta.dtype() != base_tensor.dtype() {
                                delta.to_dtype(base_tensor.dtype())?
                            } else {
                                delta
                            };

                            // Add delta to base weight
                            match base_tensor.add(&delta) {
                                Ok(merged) => {
                                    *base_tensor = merged;
                                    debug!(
                                        layer = %layer_name,
                                        flux_name = %flux_tensor_name,
                                        "Applied LoRA weight"
                                    );
                                }
                                Err(e) => {
                                    warn!(
                                        layer = %layer_name,
                                        flux_name = %flux_tensor_name,
                                        error = %e,
                                        "Shape mismatch applying LoRA"
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            warn!(
                                layer = %layer_name,
                                error = %e,
                                "Failed to compute LoRA delta"
                            );
                        }
                    }
                } else {
                    debug!(
                        layer = %layer_name,
                        flux_name = %flux_tensor_name,
                        "LoRA layer not found in base model"
                    );
                }
            }
        }

        // Build VarBuilder from merged tensors
        let vb = VarBuilder::from_tensors(merged_tensors, dtype, &device);

        // Load appropriate FLUX config
        let cfg = match model_type.id() {
            "schnell" => flux::model::Config::schnell(),
            "dev" => flux::model::Config::dev(),
            _ => anyhow::bail!("Unsupported model type for FLUX: {}", model_type),
        };
        let model = flux::model::Flux::new(&cfg, vb)?;

        info!("FLUX loaded with LoRA adapters merged");

        Ok(Self {
            model: FluxModel::Regular(model),
            device,
            is_quantized: false,
        })
    }

    /// Load quantized FLUX transformer from GGUF file with LoRA adapters
    /// Combines the memory efficiency of quantization (~12GB) with LoRA flexibility
    ///
    /// # Arguments
    /// * `model_path` - Path to flux GGUF file
    /// * `device` - Device to load model on
    /// * `model_type` - Which FLUX model variant (Schnell or Dev)
    /// * `loras` - List of LoRA adapters with their strengths
    pub fn load_quantized_with_loras<P: AsRef<Path>>(
        model_path: P,
        device: Device,
        model_type: ModelType,
        loras: &[(Arc<LoraAdapter>, f32)],
    ) -> Result<Self> {
        use candle_transformers::quantized_var_builder::VarBuilder as QVarBuilder;
        use std::collections::HashMap;

        let model_path = model_path.as_ref();

        if loras.is_empty() {
            // No LoRAs, use standard quantized loading
            return Self::load_quantized(model_path, device, model_type);
        }

        info!(
            lora_count = loras.len(),
            "Loading quantized FLUX with LoRA adapters"
        );

        // Load quantized model
        let vb = QVarBuilder::from_gguf(model_path, &device)?;
        let cfg = match model_type.id() {
            "schnell" => flux::model::Config::schnell(),
            "dev" => flux::model::Config::dev(),
            _ => anyhow::bail!("Unsupported model type for FLUX: {}", model_type),
        };
        let mut model = flux::quantized_model::Flux::new(&cfg, vb)?;

        // Prepare LoRA weights for injection
        let mut lora_map: HashMap<String, (Tensor, Tensor, f32)> = HashMap::new();

        for (lora, strength) in loras {
            debug!(
                lora_id = %lora.id,
                lora_name = %lora.name,
                strength = strength,
                weights = lora.weight_count(),
                "Preparing LoRA for injection"
            );

            for (layer_name, lora_weight) in &lora.weights {
                // Map LoRA layer name to FLUX model tensor name
                let flux_tensor_name = map_lora_to_flux_tensor(layer_name);

                // Compute LoRA scale: (alpha / rank) * strength
                let scale = (lora_weight.alpha as f32 / lora_weight.rank as f32) * strength;

                // Convert LoRA tensors to f32 for computation
                // NOTE: LoRA files store tensors as [rank, in] and [out, rank]
                // But quantized_nn expects [in, rank] and [rank, out] for (x @ A) @ B computation
                // So we need to transpose both tensors
                let lora_a = lora_weight.lora_down.to_dtype(DType::F32)?.to_device(&device)?.t()?;
                let lora_b = lora_weight.lora_up.to_dtype(DType::F32)?.to_device(&device)?.t()?;

                debug!(
                    layer = %layer_name,
                    flux_name = %flux_tensor_name,
                    rank = lora_weight.rank,
                    alpha = lora_weight.alpha,
                    scale = scale,
                    lora_a_shape = ?lora_a.shape(),
                    lora_b_shape = ?lora_b.shape(),
                    "Mapped LoRA layer"
                );

                // Store in map (will be injected into model)
                lora_map.insert(flux_tensor_name.clone(), (lora_a, lora_b, scale));
            }
        }

        // Inject LoRAs into the quantized model
        let injected_count = model.inject_loras(&lora_map)?;
        info!(
            total_loras = lora_map.len(),
            injected = injected_count,
            "LoRA adapters injected into quantized model"
        );

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
    /// * `on_step` - Optional callback for progress updates per step
    ///
    /// # Returns
    /// Denoised latents ready for VAE decode
    pub fn denoise<F>(
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
        on_step: Option<F>,
    ) -> Result<Tensor>
    where
        F: Fn(usize, usize),
    {
        // Create initial noise with seed for reproducibility
        let img = self.create_noise(height, width, seed)?;

        // Convert embeddings to match model dtype
        // Quantized: F32, Non-quantized CUDA: BF16, Non-quantized CPU: F32
        let target_dtype = if self.is_quantized {
            DType::F32
        } else if self.device.is_cuda() {
            DType::BF16
        } else {
            DType::F32
        };

        let (t5_emb, clip_emb, img) = (
            t5_emb.to_dtype(target_dtype)?,
            clip_emb.to_dtype(target_dtype)?,
            img.to_dtype(target_dtype)?,
        );

        // Create sampling state from embeddings
        let state = flux::sampling::State::new(&t5_emb, &clip_emb, &img)?;

        // Get timestep schedule based on scheduler type
        let timesteps = get_timesteps(steps, scheduler);
        debug!(sampler = ?sampler, scheduler = ?scheduler, steps = steps, "Denoising with custom sampler");

        // Run denoising with selected sampler
        let denoised = match &self.model {
            FluxModel::Regular(model) => {
                match sampler {
                    SamplerType::Euler => denoise_euler_impl(model, &state, &timesteps, guidance, &self.device, on_step.as_ref())?,
                    SamplerType::EulerA => denoise_euler_ancestral_impl(model, &state, &timesteps, guidance, seed, &self.device, on_step.as_ref())?,
                    SamplerType::DpmPP2M => denoise_dpm_pp_2m_impl(model, &state, &timesteps, guidance, &self.device, on_step.as_ref())?,
                }
            }
            FluxModel::Quantized(model) => {
                let result = match sampler {
                    SamplerType::Euler => denoise_euler_impl_q(model, &state, &timesteps, guidance, &self.device, on_step.as_ref())?,
                    SamplerType::EulerA => denoise_euler_ancestral_impl_q(model, &state, &timesteps, guidance, seed, &self.device, on_step.as_ref())?,
                    SamplerType::DpmPP2M => denoise_dpm_pp_2m_impl_q(model, &state, &timesteps, guidance, &self.device, on_step.as_ref())?,
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
fn denoise_euler_impl<F>(
    model: &flux::model::Flux,
    state: &flux::sampling::State,
    timesteps: &[f64],
    guidance: f64,
    device: &Device,
    on_step: Option<&F>,
) -> Result<Tensor>
where
    F: Fn(usize, usize),
{
    let mut img = state.img.clone();
    let b_sz = img.dim(0)?;

    // Create guidance tensor with batch size (matches candle's denoise implementation)
    let guidance_tensor = Tensor::full(guidance as f32, b_sz, device)?;

    let total_steps = timesteps.len() - 1;

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

        // Report progress for this step
        if let Some(callback) = on_step {
            callback(i + 1, total_steps);
        }
    }

    Ok(img)
}

/// Euler method for quantized model
fn denoise_euler_impl_q<F>(
    model: &flux::quantized_model::Flux,
    state: &flux::sampling::State,
    timesteps: &[f64],
    guidance: f64,
    device: &Device,
    on_step: Option<&F>,
) -> Result<Tensor>
where
    F: Fn(usize, usize),
{
    let mut img = state.img.clone();
    let b_sz = img.dim(0)?;

    // Create guidance tensor with batch size (matches candle's denoise implementation)
    let guidance_tensor = Tensor::full(guidance as f32, b_sz, device)?;

    let total_steps = timesteps.len() - 1;

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

        // Report progress for this step
        if let Some(callback) = on_step {
            callback(i + 1, total_steps);
        }
    }

    Ok(img)
}

/// Euler Ancestral - adds noise at each step for more variation
fn denoise_euler_ancestral_impl<F>(
    model: &flux::model::Flux,
    state: &flux::sampling::State,
    timesteps: &[f64],
    guidance: f64,
    seed: u64,
    device: &Device,
    on_step: Option<&F>,
) -> Result<Tensor>
where
    F: Fn(usize, usize),
{
    let mut img = state.img.clone();
    let b_sz = img.dim(0)?;

    let guidance_tensor = Tensor::full(guidance as f32, b_sz, device)?;

    let total_steps = timesteps.len() - 1;

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

        // Report progress for this step
        if let Some(callback) = on_step {
            callback(i + 1, total_steps);
        }
    }

    Ok(img)
}

/// Euler Ancestral for quantized model
fn denoise_euler_ancestral_impl_q<F>(
    model: &flux::quantized_model::Flux,
    state: &flux::sampling::State,
    timesteps: &[f64],
    guidance: f64,
    seed: u64,
    device: &Device,
    on_step: Option<&F>,
) -> Result<Tensor>
where
    F: Fn(usize, usize),
{
    let mut img = state.img.clone();
    let b_sz = img.dim(0)?;

    let guidance_tensor = Tensor::full(guidance as f32, b_sz, device)?;

    let total_steps = timesteps.len() - 1;

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

        // Report progress for this step
        if let Some(callback) = on_step {
            callback(i + 1, total_steps);
        }
    }

    Ok(img)
}

/// DPM++ 2M - second-order multistep method
fn denoise_dpm_pp_2m_impl<F>(
    model: &flux::model::Flux,
    state: &flux::sampling::State,
    timesteps: &[f64],
    guidance: f64,
    device: &Device,
    on_step: Option<&F>,
) -> Result<Tensor>
where
    F: Fn(usize, usize),
{
    let mut img = state.img.clone();
    let b_sz = img.dim(0)?;
    let mut prev_velocity: Option<Tensor> = None;

    // Create guidance tensor with batch size
    let guidance_tensor = Tensor::full(guidance as f32, b_sz, device)?;

    let total_steps = timesteps.len() - 1;

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

        // Report progress for this step
        if let Some(callback) = on_step {
            callback(i + 1, total_steps);
        }
    }

    Ok(img)
}

/// DPM++ 2M for quantized model
fn denoise_dpm_pp_2m_impl_q<F>(
    model: &flux::quantized_model::Flux,
    state: &flux::sampling::State,
    timesteps: &[f64],
    guidance: f64,
    device: &Device,
    on_step: Option<&F>,
) -> Result<Tensor>
where
    F: Fn(usize, usize),
{
    let mut img = state.img.clone();
    let b_sz = img.dim(0)?;
    let mut prev_velocity: Option<Tensor> = None;

    // Create guidance tensor with batch size
    let guidance_tensor = Tensor::full(guidance as f32, b_sz, device)?;

    let total_steps = timesteps.len() - 1;

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

        // Report progress for this step
        if let Some(callback) = on_step {
            callback(i + 1, total_steps);
        }
    }

    Ok(img)
}

// ============================================================================
// LoRA Helper Functions
// ============================================================================

/// Load a tensor from safetensors to a candle Tensor
fn load_safetensor_to_candle(
    tensors: &SafeTensors,
    key: &str,
    device: &Device,
    dtype: DType,
) -> Result<Tensor> {
    let view = tensors.tensor(key)?;

    // Convert safetensors dtype to candle dtype
    let st_dtype = view.dtype();
    let candle_dtype = match st_dtype {
        safetensors::Dtype::F32 => DType::F32,
        safetensors::Dtype::F16 => DType::F16,
        safetensors::Dtype::BF16 => DType::BF16,
        _ => anyhow::bail!("Unsupported tensor dtype: {:?}", st_dtype),
    };

    // Create tensor from raw data
    let shape: Vec<usize> = view.shape().to_vec();
    let data = view.data();

    // Load as original dtype first
    let tensor = match candle_dtype {
        DType::F32 => {
            let floats: Vec<f32> = data
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();
            Tensor::from_vec(floats, shape.as_slice(), device)?
        }
        DType::F16 => {
            let halfs: Vec<half::f16> = data
                .chunks_exact(2)
                .map(|b| half::f16::from_le_bytes([b[0], b[1]]))
                .collect();
            Tensor::from_vec(halfs, shape.as_slice(), device)?
        }
        DType::BF16 => {
            let bhalfs: Vec<half::bf16> = data
                .chunks_exact(2)
                .map(|b| half::bf16::from_le_bytes([b[0], b[1]]))
                .collect();
            Tensor::from_vec(bhalfs, shape.as_slice(), device)?
        }
        _ => unreachable!(),
    };

    // Convert to target dtype if different
    if candle_dtype != dtype {
        Ok(tensor.to_dtype(dtype)?)
    } else {
        Ok(tensor)
    }
}

/// Map LoRA layer name to FLUX model tensor name
///
/// FLUX LoRA files use naming conventions like:
/// - `lora_unet_double_blocks_0_img_attn_qkv` -> `double_blocks.0.img_attn.qkv.weight`
/// - `transformer_blocks.0.attn.to_q` -> `single_blocks.0.linear1.weight`
fn map_lora_to_flux_tensor(lora_layer: &str) -> String {
    // FLUX model uses these layer patterns:
    // - double_blocks.N.* (joint attention blocks)
    // - single_blocks.N.* (single stream blocks)
    //
    // LoRA files typically target attention layers:
    // - img_attn.qkv, img_attn.proj
    // - txt_attn.qkv, txt_attn.proj
    // - img_mlp.*, txt_mlp.*

    let mut name = lora_layer.to_string();

    // Remove common LoRA prefixes
    for prefix in ["lora_unet_", "lora_te_", "transformer."] {
        if name.starts_with(prefix) {
            name = name[prefix.len()..].to_string();
        }
    }

    // Convert underscores to dots for structure
    // double_blocks_0_img_attn_qkv -> double_blocks.0.img_attn.qkv
    let parts: Vec<&str> = name.split('.').collect();
    if parts.len() == 1 {
        // No dots yet, need to parse underscores
        let underscore_parts: Vec<&str> = name.split('_').collect();
        let mut result = Vec::new();
        let mut i = 0;

        while i < underscore_parts.len() {
            let part = underscore_parts[i];

            // Check if next part is a number (block index)
            if i + 1 < underscore_parts.len() {
                if let Ok(_) = underscore_parts[i + 1].parse::<u32>() {
                    // Combine: "double" + "blocks" or just "blocks" + "0"
                    if part == "double" || part == "single" {
                        result.push(format!(
                            "{}_{}.{}",
                            part,
                            underscore_parts[i + 1],
                            underscore_parts[i + 2]
                        ));
                        i += 3;
                        continue;
                    } else if part == "blocks" {
                        // Just "blocks_0"
                        result.push(format!("{}.{}", part, underscore_parts[i + 1]));
                        i += 2;
                        continue;
                    }
                }
            }

            result.push(part.to_string());
            i += 1;
        }

        name = result.join(".");
    }

    // Add .weight suffix if not present
    if !name.ends_with(".weight") && !name.ends_with(".bias") {
        name.push_str(".weight");
    }

    name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires downloaded model
    fn test_flux_loading() {
        use crate::models::ModelPaths;

        let paths = ModelPaths::new_for_test().unwrap();
        let device = Device::cuda_if_available(0).unwrap();

        let _flux = FluxTransformer::load(paths.transformer_path().unwrap(), device, ModelType::schnell()).unwrap();
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
