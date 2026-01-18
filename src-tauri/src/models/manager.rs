//! Persistent model manager for efficient generation

use anyhow::Result;
use candle_core::Device;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{ClipTextEncoder, FluxTransformer, ModelPaths, ModelType, T5TextEncoder, VaeDecoder};

/// Precision level for model loading
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precision {
    Full,
    Quantized,
}

/// Manages model lifecycle and VRAM
pub struct ModelManager {
    device: Device,
    paths: ModelPaths,

    // Shared components (kept loaded)
    t5: Option<T5TextEncoder>,
    clip: Option<ClipTextEncoder>,
    vae: Option<VaeDecoder>,

    // Model-specific transformer (swapped between Schnell/Dev)
    flux: Option<FluxTransformer>,
    current_model: Option<ModelType>,
    current_precision: Option<Precision>,
}

impl ModelManager {
    /// Create a new model manager
    pub fn new(device: Device) -> Result<Self> {
        let paths = ModelPaths::new()?;

        Ok(Self {
            device,
            paths,
            t5: None,
            clip: None,
            vae: None,
            flux: None,
            current_model: None,
            current_precision: None,
        })
    }

    /// Get the device
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get model paths
    pub fn paths(&self) -> &ModelPaths {
        &self.paths
    }

    /// Check if shared components are loaded
    pub fn shared_loaded(&self) -> bool {
        self.t5.is_some() && self.clip.is_some() && self.vae.is_some()
    }

    /// Get current model type
    pub fn current_model(&self) -> Option<ModelType> {
        self.current_model
    }

    /// Check available VRAM in MB (returns total if can't query)
    pub fn available_vram_mb(&self) -> usize {
        #[cfg(feature = "cuda")]
        {
            if self.device.is_cuda() {
                // Try to get free memory from CUDA
                // For now, estimate based on what's loaded
                let total = 32_000; // Assume 32GB for RTX 5090
                let used = self.estimate_loaded_vram();
                return total.saturating_sub(used);
            }
        }
        // CPU or Metal - return large number
        64_000
    }

    /// Estimate VRAM used by loaded models
    fn estimate_loaded_vram(&self) -> usize {
        let mut used = 0;
        if self.t5.is_some() {
            used += 9_000; // T5 ~9GB
        }
        if self.clip.is_some() {
            used += 250; // CLIP ~250MB
        }
        if self.vae.is_some() {
            used += 160; // VAE ~160MB
        }
        if let (Some(model), Some(precision)) = (self.current_model, self.current_precision) {
            used += match precision {
                Precision::Full => model.vram_full_precision(),
                Precision::Quantized => model.vram_quantized(),
            };
        }
        used
    }

    /// Select precision based on available VRAM
    pub fn select_precision(&self, model: ModelType) -> Precision {
        let available = self.available_vram_mb();
        let full_requirement = model.vram_full_precision();

        // Need 2GB headroom
        if available > full_requirement + 2_000 {
            Precision::Full
        } else {
            Precision::Quantized
        }
    }

    /// Load shared components (T5, CLIP, VAE)
    pub fn load_shared(&mut self) -> Result<()> {
        if self.shared_loaded() {
            return Ok(());
        }

        if !self.paths.all_files_exist() {
            anyhow::bail!("Required model files not downloaded");
        }

        println!("Loading shared components...");

        if self.t5.is_none() {
            println!("  Loading T5 encoder...");
            self.t5 = Some(T5TextEncoder::load(
                self.paths.t5_path(),
                self.paths.t5_tokenizer_path(),
                self.device.clone(),
            )?);
        }

        if self.clip.is_none() {
            println!("  Loading CLIP encoder...");
            self.clip = Some(ClipTextEncoder::load(
                self.paths.clip_path().join("model.safetensors"),
                self.paths.tokenizer_path(),
                self.device.clone(),
            )?);
        }

        if self.vae.is_none() {
            println!("  Loading VAE decoder...");
            self.vae = Some(VaeDecoder::load(
                self.paths.vae_path(),
                self.device.clone(),
            )?);
        }

        Ok(())
    }

    /// Load or switch to a specific model
    pub fn load_model(&mut self, model: ModelType) -> Result<()> {
        // Load shared components first
        self.load_shared()?;

        // Check if already loaded
        if self.current_model == Some(model) && self.flux.is_some() {
            return Ok(());
        }

        // Unload current transformer if different model
        if self.current_model.is_some() && self.current_model != Some(model) {
            println!("  Unloading {} transformer...", self.current_model.unwrap());
            self.flux = None;
            self.current_model = None;
            self.current_precision = None;
        }

        // Determine precision
        let precision = self.select_precision(model);
        let use_quantized = precision == Precision::Quantized;

        println!(
            "  Loading {} transformer ({})...",
            model,
            if use_quantized { "quantized" } else { "full precision" }
        );

        // Load transformer
        let flux = if use_quantized && self.paths.has_quantized_for(model) {
            FluxTransformer::load_quantized(
                self.paths.quantized_transformer_path_for(model),
                self.device.clone(),
            )?
        } else {
            FluxTransformer::load(
                self.paths.transformer_path_for(model),
                self.device.clone(),
            )?
        };

        self.flux = Some(flux);
        self.current_model = Some(model);
        self.current_precision = Some(precision);

        println!("  {} loaded successfully!", model);
        Ok(())
    }

    /// Unload transformer to free VRAM (keeps shared components)
    pub fn unload_transformer(&mut self) {
        self.flux = None;
        self.current_model = None;
        self.current_precision = None;
    }

    /// Unload T5 to free memory (call after encoding)
    pub fn unload_t5(&mut self) {
        self.t5 = None;
    }

    /// Unload everything
    pub fn unload_all(&mut self) {
        self.t5 = None;
        self.clip = None;
        self.vae = None;
        self.flux = None;
        self.current_model = None;
        self.current_precision = None;
    }

    /// Get T5 encoder reference
    pub fn t5(&self) -> Option<&T5TextEncoder> {
        self.t5.as_ref()
    }

    /// Get mutable T5 encoder reference
    pub fn t5_mut(&mut self) -> Option<&mut T5TextEncoder> {
        self.t5.as_mut()
    }

    /// Get CLIP encoder reference
    pub fn clip(&self) -> Option<&ClipTextEncoder> {
        self.clip.as_ref()
    }

    /// Get VAE decoder reference
    pub fn vae(&self) -> Option<&VaeDecoder> {
        self.vae.as_ref()
    }

    /// Get FLUX transformer reference
    pub fn flux(&self) -> Option<&FluxTransformer> {
        self.flux.as_ref()
    }
}

/// Thread-safe shared model manager
pub type SharedModelManager = Arc<RwLock<ModelManager>>;

/// Create a shared model manager
pub fn create_shared_manager(device: Device) -> Result<SharedModelManager> {
    Ok(Arc::new(RwLock::new(ModelManager::new(device)?)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let device = Device::Cpu;
        let manager = ModelManager::new(device).unwrap();
        assert!(!manager.shared_loaded());
        assert!(manager.current_model().is_none());
    }

    #[test]
    fn test_precision_selection() {
        let device = Device::Cpu;
        let manager = ModelManager::new(device).unwrap();
        // On CPU, should always have "enough" VRAM
        let precision = manager.select_precision(ModelType::Schnell);
        assert_eq!(precision, Precision::Full);
    }
}
