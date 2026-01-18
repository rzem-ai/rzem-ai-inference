//! Generation statistics tracking

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Statistics for a single generation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationStats {
    /// Time to load all models (if not already loaded)
    pub model_load_ms: Option<u64>,
    /// Time to load T5 encoder
    pub t5_load_ms: Option<u64>,
    /// Time to load CLIP encoder
    pub clip_load_ms: Option<u64>,
    /// Time to load VAE decoder
    pub vae_load_ms: Option<u64>,
    /// Time to load FLUX transformer
    pub flux_load_ms: Option<u64>,

    /// Time for T5 text encoding
    pub t5_encode_ms: u64,
    /// Time for CLIP text encoding
    pub clip_encode_ms: u64,
    /// Time for denoising (all steps)
    pub denoise_ms: u64,
    /// Time for VAE decoding
    pub vae_decode_ms: u64,
    /// Time for PNG encoding
    pub png_encode_ms: u64,

    /// Total generation time
    pub total_ms: u64,

    /// T5 embedding dimensions
    pub t5_embedding_shape: Vec<usize>,
    /// CLIP embedding dimensions
    pub clip_embedding_shape: Vec<usize>,
    /// Latent dimensions
    pub latent_shape: Vec<usize>,
    /// Output image dimensions
    pub image_shape: Vec<usize>,

    /// Number of denoising steps
    pub steps: usize,
    /// Model type (full precision or quantized)
    pub model_type: String,
}

impl Default for GenerationStats {
    fn default() -> Self {
        Self {
            model_load_ms: None,
            t5_load_ms: None,
            clip_load_ms: None,
            vae_load_ms: None,
            flux_load_ms: None,
            t5_encode_ms: 0,
            clip_encode_ms: 0,
            denoise_ms: 0,
            vae_decode_ms: 0,
            png_encode_ms: 0,
            total_ms: 0,
            t5_embedding_shape: vec![],
            clip_embedding_shape: vec![],
            latent_shape: vec![],
            image_shape: vec![],
            steps: 0,
            model_type: String::new(),
        }
    }
}

/// Helper for timing operations
pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn start() -> Self {
        Self { start: Instant::now() }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    pub fn stop(self) -> u64 {
        self.elapsed_ms()
    }
}

/// Result of generation including image data and stats
#[derive(Debug)]
pub struct GenerationResult {
    pub image_data: Vec<u8>,
    pub stats: GenerationStats,
}
