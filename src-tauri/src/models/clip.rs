//! CLIP text encoder for FLUX

use anyhow::Result;
use candle_core::{Device, Module, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::clip;
use tokenizers::{Tokenizer, models::bpe::BPE};
use std::path::Path;

/// CLIP text encoder wrapper for FLUX
pub struct ClipTextEncoder {
    model: clip::text_model::ClipTextTransformer,
    tokenizer: Tokenizer,
    device: Device,
}

impl ClipTextEncoder {
    /// Load CLIP model from safetensors file
    pub fn load<P: AsRef<Path>>(
        model_path: P,
        tokenizer_path: P,
        device: Device,
    ) -> Result<Self> {
        // Load tokenizer from directory containing vocab.json and merges.txt
        // CLIP uses BPE tokenizer, not a unified tokenizer.json
        let tokenizer_dir = tokenizer_path.as_ref();
        let tokenizer = if tokenizer_dir.join("tokenizer.json").exists() {
            // Unified tokenizer format
            Tokenizer::from_file(tokenizer_dir.join("tokenizer.json"))
                .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?
        } else if tokenizer_dir.join("vocab.json").exists() && tokenizer_dir.join("merges.txt").exists() {
            // Separate BPE files format (CLIP default)
            // Build BPE tokenizer from local vocab and merges files
            let vocab_path = tokenizer_dir.join("vocab.json");
            let merges_path = tokenizer_dir.join("merges.txt");

            let bpe = BPE::from_file(&vocab_path.to_string_lossy(), &merges_path.to_string_lossy())
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build BPE tokenizer: {}", e))?;

            let mut tokenizer = Tokenizer::new(bpe);

            // CLIP uses specific pre/post processing:
            // - Lowercase and strip accents
            // - Add [CLS] and [SEP] tokens
            // - Pad to max length of 77 tokens
            tokenizer.with_padding(Some(tokenizers::PaddingParams {
                strategy: tokenizers::PaddingStrategy::Fixed(77),
                pad_id: 49407,  // <|endoftext|>
                pad_token: "<|endoftext|>".to_string(),
                ..Default::default()
            }));

            tokenizer.with_truncation(Some(tokenizers::TruncationParams {
                max_length: 77,
                ..Default::default()
            })).map_err(|e| anyhow::anyhow!("Failed to set truncation: {}", e))?;

            tokenizer
        } else {
            anyhow::bail!("Could not find tokenizer files in {}", tokenizer_dir.display());
        };

        // Load model weights
        // SAFETY: from_mmaped_safetensors uses memory-mapped IO which is safe
        // because safetensors format is designed to be safely memory-mapped
        // without requiring trust of the file contents.
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(
                &[model_path.as_ref()],
                candle_core::DType::F32,
                &device,
            )?
        };

        // Create CLIP text model
        // FLUX Schnell uses CLIP with 768-dim embeddings (ViT-L configuration)
        let config = clip::text_model::ClipTextConfig {
            vocab_size: 49408,
            embed_dim: 768,  // ViT-L size, required by FLUX
            activation: clip::text_model::Activation::QuickGelu,
            intermediate_size: 3072,
            max_position_embeddings: 77,
            pad_with: None,
            num_hidden_layers: 12,
            num_attention_heads: 12,
            projection_dim: 768,
        };
        let model = clip::text_model::ClipTextTransformer::new(vb, &config)?;

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    /// Encode text prompt to embeddings
    ///
    /// Returns tensor of shape [1, 77, 768] (batch, seq_len, embed_dim)
    pub fn encode(&self, prompt: &str) -> Result<Tensor> {
        // Tokenize prompt
        // Note: encode(prompt, true) enables padding/truncation to 77 tokens (CLIP max length)
        let tokens = self.tokenizer
            .encode(prompt, true)
            .map_err(|e| anyhow::anyhow!("Tokenization error: {}", e))?;

        // Convert to tensor
        let token_ids: Vec<u32> = tokens.get_ids().to_vec();
        let token_ids = Tensor::new(token_ids.as_slice(), &self.device)?
            .unsqueeze(0)?; // Add batch dimension

        // Encode with CLIP
        let embeddings = self.model.forward(&token_ids)?;

        Ok(embeddings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires downloaded model
    fn test_clip_encoding() {
        use crate::models::ModelPaths;

        let paths = ModelPaths::new().unwrap();
        let device = Device::cuda_if_available(0).unwrap();

        let encoder = ClipTextEncoder::load(
            paths.clip_path().join("model.safetensors"),
            paths.tokenizer_path(),
            device,
        ).unwrap();

        let embeddings = encoder.encode("a cat").unwrap();
        let shape = embeddings.dims();

        assert_eq!(shape[0], 1); // batch
        assert_eq!(shape[1], 77); // sequence length
        assert_eq!(shape[2], 768); // embedding dim
    }
}
