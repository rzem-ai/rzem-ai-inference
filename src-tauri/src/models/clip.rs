//! CLIP text encoder for FLUX

use anyhow::Result;
use candle_core::{Device, IndexOp, Module, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::clip;
use tokenizers::{Tokenizer, models::bpe::BPE, AddedToken};
use tokenizers::processors::template::TemplateProcessing;
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

            // Add special tokens
            tokenizer.add_special_tokens(&[
                AddedToken::from("<|startoftext|>", true).single_word(false),
                AddedToken::from("<|endoftext|>", true).single_word(false),
            ]);

            // CLIP post-processing: add SOT at start, EOT at end
            // SOT = 49406, EOT = 49407
            let template = TemplateProcessing::builder()
                .try_single("<|startoftext|> $A <|endoftext|>")
                .map_err(|e| anyhow::anyhow!("Failed to set template: {}", e))?
                .special_tokens(vec![
                    ("<|startoftext|>", 49406u32),
                    ("<|endoftext|>", 49407u32),
                ])
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build template: {}", e))?;
            tokenizer.with_post_processor(template);

            // Padding and truncation to 77 tokens
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

        // FLUX CLIP model has "text_model." prefix on all tensors
        // Push prefix to match the actual tensor names in the safetensors file
        let vb = vb.pp("text_model");

        // Create CLIP text model
        // FLUX Schnell uses CLIP with 768-dim embeddings (ViT-L configuration)
        let config = clip::text_model::ClipTextConfig {
            vocab_size: 49408,
            embed_dim: 768,  // ViT-L size, required by FLUX
            activation: clip::text_model::Activation::QuickGelu,
            intermediate_size: 3072,
            max_position_embeddings: 77,
            pad_with: Some("<|endoftext|>".to_string()),  // EOT token - tells CLIP where to extract pooled output
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

    /// Encode text prompt to pooled embeddings
    ///
    /// Returns tensor of shape [1, 768] (batch, embed_dim)
    /// Note: Returns POOLED output for FLUX's `vec` parameter
    pub fn encode(&self, prompt: &str) -> Result<Tensor> {
        // Tokenize prompt - tokenizer adds SOT/EOT and pads to 77 tokens
        let tokens = self.tokenizer
            .encode(prompt, true)
            .map_err(|e| anyhow::anyhow!("Tokenization error: {}", e))?;

        let token_ids: Vec<u32> = tokens.get_ids().to_vec();

        // Find EOS position (first occurrence of 49407 after the text tokens)
        let eos_position = token_ids.iter()
            .position(|&id| id == 49407)
            .unwrap_or(token_ids.len() - 1);

        if token_ids.len() != 77 {
            anyhow::bail!("CLIP tokenization produced {} tokens, expected 77", token_ids.len());
        }

        let input_ids = Tensor::new(token_ids.as_slice(), &self.device)?
            .unsqueeze(0)?; // Add batch dimension [1, 77]

        // Use forward_with_mask to get full hidden states, then manually extract pooled output
        // This bypasses the buggy argmax in the default forward() method
        let hidden_states = self.model.forward_with_mask(&input_ids, usize::MAX)?;

        // Extract hidden state at EOS position for pooled output [1, 768]
        let pooled = hidden_states.i((0, eos_position))?.unsqueeze(0)?;

        Ok(pooled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires downloaded model
    fn test_clip_encoding() {
        use crate::models::ModelPaths;

        let paths = ModelPaths::new_for_test().unwrap();
        let device = Device::cuda_if_available(0).unwrap();

        let encoder = ClipTextEncoder::load(
            paths.clip_path().unwrap().join("model.safetensors"),
            paths.tokenizer_path().unwrap(),
            device,
        ).unwrap();

        let embeddings = encoder.encode("a cat").unwrap();
        let shape = embeddings.dims();

        assert_eq!(shape[0], 1); // batch
        assert_eq!(shape[1], 768); // pooled embedding dim
    }
}
