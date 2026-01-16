//! CLIP text encoder for FLUX

use anyhow::Result;
use candle_core::{Device, Module, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::clip;
use tokenizers::Tokenizer;
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
        // Load tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_path.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

        // Load model weights
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(
                &[model_path.as_ref()],
                candle_core::DType::F32,
                &device,
            )?
        };

        // Create CLIP text model
        let config = clip::text_model::ClipTextConfig::vit_base_patch32();
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
            paths.clip_path().join("tokenizer.json"),
            device,
        ).unwrap();

        let embeddings = encoder.encode("a cat").unwrap();
        let shape = embeddings.dims();

        assert_eq!(shape[0], 1); // batch
        assert_eq!(shape[1], 77); // sequence length
        assert_eq!(shape[2], 768); // embedding dim
    }
}
