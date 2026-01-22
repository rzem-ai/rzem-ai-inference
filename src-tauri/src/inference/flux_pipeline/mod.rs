//! Flux model inference pipeline
//!
//! This module provides the main pipeline for running Flux diffusion models.
//! It handles model loading, text encoding, denoising, and image decoding.

mod cache_integration;
mod generation;
mod loader;

use anyhow::Result;
use candle_core::Device;
use crate::models::{ClipTextEncoder, FluxTransformer, ModelType, T5TextEncoder, VaeDecoder};

/// Flux diffusion model pipeline for image generation
pub struct FluxPipeline {
    pub(crate) device: Device,
    pub(crate) model_type: ModelType,
    pub(crate) t5: Option<T5TextEncoder>,
    pub(crate) clip: Option<ClipTextEncoder>,
    pub(crate) vae: Option<VaeDecoder>,
    pub(crate) flux: Option<FluxTransformer>,
    /// Whether models were loaded this session (for stats)
    pub(crate) models_loaded_this_session: bool,
}

impl FluxPipeline {
    /// Creates a new Flux pipeline instance for Schnell model (default)
    pub fn new(device: Device) -> Result<Self> {
        Self::with_model_type(device, ModelType::Schnell)
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
        })
    }

    /// Get the model type this pipeline is configured for
    pub fn model_type(&self) -> ModelType {
        self.model_type
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
