//! Z-Image-Turbo inference pipeline
//!
//! Single-stream DiT architecture for text-to-image generation.
//! Uses Qwen3-4B for text encoding and FLUX VAE for image decoding.

mod cache_integration;
mod generation;
mod loader;

use anyhow::Result;
use candle_core::Device;
use crate::models::{Qwen3TextEncoder, VaeDecoder, ZImageTransformer};

/// Z-Image-Turbo diffusion model pipeline for image generation
///
/// Architecture:
/// - Text Encoder: Qwen3-4B (returns sequence hidden states)
/// - Transformer: Single-stream DiT (6B parameters, 8 fixed steps)
/// - VAE: FLUX AutoencoderKL (shared with FLUX models)
///
/// Note: No SigLIP encoder (only needed for Z-Image-Edit variant)
pub struct ZIndexPipeline {
    pub(crate) device: Device,
    pub(crate) qwen3: Option<Qwen3TextEncoder>,
    pub(crate) vae: Option<VaeDecoder>,
    pub(crate) zimage: Option<ZImageTransformer>,
    /// Whether models were loaded this session (for stats tracking)
    pub(crate) models_loaded_this_session: bool,
}

impl ZIndexPipeline {
    /// Creates a new Z-Image-Turbo pipeline instance
    pub fn new(device: Device) -> Result<Self> {
        Ok(Self {
            device,
            qwen3: None,
            vae: None,
            zimage: None,
            models_loaded_this_session: false,
        })
    }

    /// Check which models are currently loaded
    ///
    /// Returns (qwen3_loaded, vae_loaded, zimage_loaded)
    pub fn models_loaded(&self) -> (bool, bool, bool) {
        (
            self.qwen3.is_some(),
            self.vae.is_some(),
            self.zimage.is_some(),
        )
    }

    /// Check if all required models are loaded
    pub fn all_models_loaded(&self) -> bool {
        let (qwen3, vae, zimage) = self.models_loaded();
        qwen3 && vae && zimage
    }

    /// Unload all models to free VRAM
    pub fn unload_models(&mut self) {
        self.qwen3 = None;
        self.vae = None;
        self.zimage = None;
        self.models_loaded_this_session = false;
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
        let _pipeline = ZIndexPipeline::new(device).unwrap();
    }
}
