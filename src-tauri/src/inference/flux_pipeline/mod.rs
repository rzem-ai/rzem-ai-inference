//! Flux model inference pipeline
//!
//! This module provides the main pipeline for running Flux diffusion models.
//! It handles model loading, text encoding, denoising, and image decoding.

mod cache_integration;
mod generation;
mod loader;

use anyhow::Result;
use candle_core::Device;
use std::sync::Arc;
use crate::models::{ClipTextEncoder, FluxTransformer, LoraAdapter, LoraConfig, ModelType, T5TextEncoder, VaeDecoder};

/// Flux diffusion model pipeline for image generation
pub struct FluxPipeline {
    pub(crate) device: Device,
    pub(crate) model_type: ModelType,
    pub(crate) t5: Option<T5TextEncoder>,
    pub(crate) clip: Option<ClipTextEncoder>,
    pub(crate) vae: Option<Arc<VaeDecoder>>,
    pub(crate) flux: Option<FluxTransformer>,
    /// Whether models were loaded this session (for stats)
    pub(crate) models_loaded_this_session: bool,
    /// Currently active LoRA adapters
    pub(crate) active_loras: Vec<(Arc<LoraAdapter>, f32)>,
    /// Hash of current LoRA config for change detection
    pub(crate) lora_config_hash: u64,
    /// Bundle/component context for custom path resolution
    pub(crate) bundle_context: Option<BundleContext>,
}

/// Context for bundle or component-based path resolution
#[derive(Debug, Clone)]
pub struct BundleContext {
    pub bundle_id: Option<String>,
    pub model_component_id: Option<String>,
    pub t5_component_id: Option<String>,
    pub clip_component_id: Option<String>,
    pub vae_component_id: Option<String>,
}

impl FluxPipeline {
    /// Creates a new Flux pipeline instance for Schnell model (default)
    pub fn new(device: Device) -> Result<Self> {
        Self::with_model_type(device, ModelType::schnell())
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
            active_loras: Vec::new(),
            lora_config_hash: 0,
            bundle_context: None,
        })
    }

    /// Set bundle/component context for custom path resolution
    pub fn set_bundle_context(&mut self, context: BundleContext) {
        self.bundle_context = Some(context);
    }

    /// Clear bundle/component context (use default path resolution)
    pub fn clear_bundle_context(&mut self) {
        self.bundle_context = None;
    }

    /// Set LoRA adapters for this pipeline
    ///
    /// If the LoRAs differ from the current set, the FLUX transformer
    /// will be reloaded on the next generation.
    pub fn set_loras(&mut self, loras: Vec<(Arc<LoraAdapter>, f32)>) {
        let new_hash = Self::compute_lora_hash(&loras);

        if new_hash != self.lora_config_hash {
            // LoRAs changed, need to reload transformer
            self.flux = None;
            self.active_loras = loras;
            self.lora_config_hash = new_hash;
            tracing::info!(lora_count = self.active_loras.len(), "LoRAs updated, transformer will reload");
        }
    }

    /// Check if LoRAs have changed
    pub fn loras_changed(&self, loras: &[(Arc<LoraAdapter>, f32)]) -> bool {
        let new_hash = Self::compute_lora_hash(loras);
        new_hash != self.lora_config_hash
    }

    /// Get currently active LoRAs
    pub fn active_loras(&self) -> &[(Arc<LoraAdapter>, f32)] {
        &self.active_loras
    }

    /// Compute a hash for LoRA configuration
    fn compute_lora_hash(loras: &[(Arc<LoraAdapter>, f32)]) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        for (lora, strength) in loras {
            lora.id.hash(&mut hasher);
            strength.to_bits().hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Get the model type this pipeline is configured for
    pub fn model_type(&self) -> ModelType {
        self.model_type.clone()
    }

    /// Check which models are currently loaded
    pub fn models_loaded(&self) -> (bool, bool, bool, bool) {
        (
            self.t5.is_some(),
            self.clip.is_some(),
            self.vae.is_some(),
            self.flux.is_some(),
        )
    }

    /// Unload T5 and CLIP encoders to free VRAM
    ///
    /// Call after encoding is complete, before loading transformer.
    /// This enables staged loading for memory-constrained GPUs.
    pub fn unload_text_encoders(&mut self) {
        if self.t5.is_some() {
            tracing::info!("Unloading T5 encoder to free VRAM");
            self.t5 = None;
        }
        if self.clip.is_some() {
            tracing::info!("Unloading CLIP encoder to free VRAM");
            self.clip = None;
        }
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
