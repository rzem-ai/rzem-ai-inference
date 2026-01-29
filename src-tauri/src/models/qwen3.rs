//! Qwen3 text encoder for Z-Image-Turbo
//!
//! Qwen3-4B serves as the primary text encoder for Z-Image-Turbo, providing
//! semantic understanding of prompts. Unlike CLIP which produces pooled embeddings,
//! Qwen3 produces full sequence hidden states used by the single-stream transformer.
//!
//! Note: Uses Qwen2 module from candle_transformers since Qwen3 is architecturally
//! compatible with Qwen2 (same transformer structure, RoPE, GQA, etc.). The main
//! differences are in training data and scale, not architecture.

use anyhow::{Context, Result};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::qwen2;
use std::path::Path;
use tokenizers::Tokenizer;
use tracing::{debug, info, warn};

/// Qwen3 text encoder for Z-Image-Turbo
///
/// Uses Qwen3-4B model with:
/// - Hidden size: 2560
/// - 36 layers
/// - 32 attention heads (8 key-value heads for GQA)
/// - Vocab size: 151936
pub struct Qwen3TextEncoder {
    model: qwen2::Model,
    tokenizer: Tokenizer,
    device: Device,
    max_length: usize,
}

impl Qwen3TextEncoder {
    /// Load Qwen3 encoder from sharded safetensors files
    ///
    /// # Arguments
    /// * `model_dir` - Directory containing model-00001-of-00003.safetensors, etc.
    /// * `tokenizer_path` - Path to tokenizer directory containing tokenizer.json
    /// * `device` - Device to load model on
    ///
    /// # Returns
    /// Qwen3TextEncoder ready to encode prompts
    pub fn load<P: AsRef<Path>>(
        model_dir: P,
        tokenizer_path: P,
        device: Device,
    ) -> Result<Self> {
        let model_dir = model_dir.as_ref();
        let tokenizer_path = tokenizer_path.as_ref();

        info!("Loading Qwen3 encoder from {:?}", model_dir);

        // Load tokenizer
        let tokenizer_file = if tokenizer_path.is_dir() {
            tokenizer_path.join("tokenizer.json")
        } else {
            tokenizer_path.to_path_buf()
        };

        let tokenizer = Tokenizer::from_file(&tokenizer_file)
            .map_err(|e| anyhow::anyhow!("Failed to load Qwen3 tokenizer from {:?}: {}", tokenizer_file, e))?;

        debug!("Loaded Qwen3 tokenizer with vocab size {}", tokenizer.get_vocab_size(true));

        // Load config
        let config_file = model_dir.join("config.json");
        let config: qwen2::Config = serde_json::from_str(
            &std::fs::read_to_string(&config_file)
                .context("Failed to read Qwen3 config.json")?
        ).context("Failed to parse Qwen3 config.json")?;

        warn!("Loading Qwen3 model using Qwen2 architecture (compatible)");
        debug!("Qwen3 config: hidden_size={}, num_layers={}, vocab_size={}",
               config.hidden_size, config.num_hidden_layers, config.vocab_size);

        // Find all sharded model files (model-00001-of-00003.safetensors, etc.)
        let mut model_files: Vec<_> = std::fs::read_dir(model_dir)
            .context("Failed to read model directory")?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("model-") && n.ends_with(".safetensors"))
                    .unwrap_or(false)
            })
            .map(|e| e.path())
            .collect();

        if model_files.is_empty() {
            anyhow::bail!("No Qwen3 model files found in {:?}", model_dir);
        }

        // Sort to ensure consistent loading order
        model_files.sort();
        info!("Found {} Qwen3 model shards", model_files.len());

        let model_files_refs: Vec<&Path> = model_files.iter().map(|p| p.as_path()).collect();

        // Load model weights - use bfloat16 for efficiency on GPU
        let dtype = if device.is_cuda() {
            DType::BF16
        } else {
            DType::F32
        };

        debug!("Loading Qwen3 weights with dtype {:?}", dtype);

        // SAFETY: from_mmaped_safetensors uses memory-mapped IO which is safe
        // because safetensors format is designed to be safely memory-mapped
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&model_files_refs, dtype, &device)?
        };

        // Create Qwen3 model using Qwen2 architecture (compatible)
        let model = qwen2::Model::new(&config, vb)?;

        info!("Qwen3 encoder loaded successfully");

        Ok(Self {
            model,
            tokenizer,
            device,
            max_length: 512, // Z-Image uses flexible sequence length
        })
    }

    /// Encode text prompt to hidden states
    ///
    /// # Arguments
    /// * `prompt` - Text prompt to encode
    ///
    /// # Returns
    /// Tensor of shape [1, seq_len, 2560] containing hidden states
    /// Unlike CLIP which returns pooled output, Qwen3 returns full sequence
    /// for use in Z-Image's single-stream transformer.
    pub fn encode(&mut self, prompt: &str) -> Result<Tensor> {
        // Tokenize prompt
        let encoding = self.tokenizer
            .encode(prompt, true)
            .map_err(|e| anyhow::anyhow!("Qwen3 tokenization error: {}", e))?;

        let token_ids: Vec<u32> = encoding.get_ids().to_vec();
        let seq_len = token_ids.len();

        debug!("Qwen3 tokenized to {} tokens", seq_len);

        if seq_len > self.max_length {
            anyhow::bail!(
                "Qwen3 prompt too long: {} tokens (max {})",
                seq_len,
                self.max_length
            );
        }

        // Convert to tensor [1, seq_len]
        let input_ids = Tensor::new(token_ids.as_slice(), &self.device)?
            .unsqueeze(0)?;

        // Forward pass through Qwen3 encoder
        // Returns hidden states of shape [1, seq_len, 2560]
        // No KV cache needed for encoding (None)
        let hidden_states = self.model.forward(&input_ids, 0, None)?;

        let shape = hidden_states.dims();
        debug!("Qwen3 output shape: {:?}", shape);

        // Validate output shape
        if shape.len() != 3 {
            anyhow::bail!("Unexpected Qwen3 output shape: {:?}", shape);
        }
        if shape[0] != 1 {
            anyhow::bail!("Expected batch size 1, got {}", shape[0]);
        }
        if shape[2] != 2560 {
            anyhow::bail!("Expected hidden size 2560, got {}", shape[2]);
        }

        Ok(hidden_states)
    }

    /// Get the hidden dimension size (2560 for Qwen3-4B)
    pub fn hidden_size(&self) -> usize {
        2560
    }

    /// Get the maximum sequence length
    pub fn max_length(&self) -> usize {
        self.max_length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires downloaded Z-Image-Turbo model
    fn test_qwen3_loading() {
        let paths = crate::models::ModelPaths::new().unwrap();
        let device = Device::cuda_if_available(0).unwrap();

        // TODO: Remove Z-Image support
        let encoder = Qwen3TextEncoder::load(
            paths.qwen3_path().unwrap(),
            paths.qwen3_tokenizer_path().unwrap(),
            device,
        );

        assert!(encoder.is_ok(), "Failed to load Qwen3: {:?}", encoder.err());
    }

    #[test]
    #[ignore] // Requires downloaded Z-Image-Turbo model
    fn test_qwen3_encoding() {
        let paths = crate::models::ModelPaths::new().unwrap();
        let device = Device::cuda_if_available(0).unwrap();

        // TODO: Remove Z-Image support
        let mut encoder = Qwen3TextEncoder::load(
            paths.qwen3_path().unwrap(),
            paths.qwen3_tokenizer_path().unwrap(),
            device,
        ).unwrap();

        let hidden_states = encoder.encode("A serene mountain landscape at sunset").unwrap();
        let shape = hidden_states.dims();

        assert_eq!(shape[0], 1, "Expected batch size 1");
        assert_eq!(shape[2], 2560, "Expected hidden size 2560");
        assert!(shape[1] > 0 && shape[1] <= 512, "Sequence length out of expected range");

        println!("Qwen3 output shape: {:?}", shape);
    }

    #[test]
    #[ignore] // Requires downloaded Z-Image-Turbo model
    fn test_qwen3_chinese() {
        let paths = crate::models::ModelPaths::new().unwrap();
        let device = Device::cuda_if_available(0).unwrap();

        // TODO: Remove Z-Image support
        let mut encoder = Qwen3TextEncoder::load(
            paths.qwen3_path().unwrap(),
            paths.qwen3_tokenizer_path().unwrap(),
            device,
        ).unwrap();

        // Test Chinese prompt (Z-Image-Turbo supports bilingual)
        let hidden_states = encoder.encode("一个美丽的中国传统建筑").unwrap();
        let shape = hidden_states.dims();

        assert_eq!(shape[0], 1);
        assert_eq!(shape[2], 2560);

        println!("Qwen3 Chinese output shape: {:?}", shape);
    }
}
