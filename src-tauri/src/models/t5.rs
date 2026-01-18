//! T5 text encoder for FLUX with quantized model support

use anyhow::Result;
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::t5;
use candle_transformers::models::quantized_t5;
use std::path::Path;
use tokenizers::Tokenizer;

/// Enum to hold either regular or quantized T5 model
enum T5Model {
    Regular(t5::T5EncoderModel),
    Quantized(quantized_t5::T5EncoderModel),
}

/// T5 text encoder for FLUX (provides main text conditioning)
/// Supports both full-precision and quantized (GGUF) models
pub struct T5TextEncoder {
    model: T5Model,
    tokenizer: Tokenizer,
    device: Device,
    max_length: usize,
    is_quantized: bool,
}

impl T5TextEncoder {
    /// Load T5 encoder from safetensors files (full precision)
    ///
    /// # Arguments
    /// * `model_dir` - Directory containing model-00001-of-00002.safetensors, etc.
    /// * `tokenizer_path` - Path to tokenizer.json file (e.g., from lmz/mt5-tokenizers)
    /// * `device` - Device to load model on
    pub fn load<P: AsRef<Path>>(
        model_dir: P,
        tokenizer_path: P,
        device: Device,
    ) -> Result<Self> {
        let model_dir = model_dir.as_ref();
        let tokenizer_path = tokenizer_path.as_ref();

        // Load tokenizer directly from file
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load T5 tokenizer from {:?}: {}", tokenizer_path, e))?;

        // Load config
        let config_file = model_dir.join("config.json");
        let config: t5::Config = serde_json::from_str(&std::fs::read_to_string(&config_file)?)?;

        // Find model files (split into 2 parts)
        let model_files: Vec<_> = std::fs::read_dir(model_dir)?
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
            anyhow::bail!("No T5 model files found in {:?}", model_dir);
        }

        let model_files_refs: Vec<&Path> = model_files.iter().map(|p| p.as_path()).collect();

        // Load model weights - use bf16 for efficiency
        let dtype = if device.is_cuda() {
            DType::BF16
        } else {
            DType::F32
        };

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&model_files_refs, dtype, &device)?
        };

        let model = t5::T5EncoderModel::load(vb, &config)?;

        Ok(Self {
            model: T5Model::Regular(model),
            tokenizer,
            device,
            max_length: 256, // FLUX uses 256 tokens for T5
            is_quantized: false,
        })
    }

    /// Load quantized T5 encoder from GGUF file
    /// Uses ~3.3GB VRAM instead of ~9GB (Q5_K_M quantization)
    ///
    /// # Arguments
    /// * `model_path` - Path to t5-v1_1-xxl-encoder-Q5_K_M.gguf file
    /// * `tokenizer_path` - Path to tokenizer.json file
    /// * `device` - Device to load model on
    pub fn load_quantized<P: AsRef<Path>>(
        model_path: P,
        tokenizer_path: P,
        device: Device,
    ) -> Result<Self> {
        use candle_transformers::quantized_var_builder::VarBuilder as QVarBuilder;

        let model_path = model_path.as_ref();
        let tokenizer_path = tokenizer_path.as_ref();

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load T5 tokenizer from {:?}: {}", tokenizer_path, e))?;

        // Load quantized model
        let vb = QVarBuilder::from_gguf(model_path, &device)?;

        // T5-v1.1-XXL config - use JSON deserialization since Config fields are private
        let config_json = r#"{
            "vocab_size": 32128,
            "d_model": 4096,
            "d_kv": 64,
            "d_ff": 10240,
            "num_layers": 24,
            "num_heads": 64,
            "relative_attention_num_buckets": 32,
            "relative_attention_max_distance": 128,
            "dropout_rate": 0.1,
            "layer_norm_epsilon": 1e-6,
            "initializer_factor": 1.0,
            "feed_forward_proj": "gated-gelu",
            "tie_word_embeddings": false,
            "is_decoder": false,
            "is_encoder_decoder": true,
            "use_cache": true,
            "pad_token_id": 0,
            "eos_token_id": 1
        }"#;
        let config: quantized_t5::Config = serde_json::from_str(config_json)?;

        let model = quantized_t5::T5EncoderModel::load(vb, &config)?;

        Ok(Self {
            model: T5Model::Quantized(model),
            tokenizer,
            device,
            max_length: 256,
            is_quantized: true,
        })
    }

    /// Check if this is a quantized model
    pub fn is_quantized(&self) -> bool {
        self.is_quantized
    }

    /// Encode text prompt to embeddings
    ///
    /// Returns tensor of shape [1, seq_len, 4096] (batch, tokens, d_model)
    pub fn encode(&mut self, prompt: &str) -> Result<Tensor> {
        // Tokenize prompt
        let tokens = self
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| anyhow::anyhow!("T5 tokenization error: {}", e))?;

        let mut token_ids: Vec<u32> = tokens.get_ids().to_vec();

        // Pad or truncate to max_length
        token_ids.resize(self.max_length, 0); // 0 is pad token for T5

        let input_ids = Tensor::new(token_ids.as_slice(), &self.device)?.unsqueeze(0)?;

        // Forward through encoder
        let embeddings = match &mut self.model {
            T5Model::Regular(model) => model.forward(&input_ids)?,
            T5Model::Quantized(model) => {
                // Quantized model works in F32
                let result = model.forward(&input_ids)?;
                // Convert to bf16 if on CUDA for consistency with rest of pipeline
                if self.device.is_cuda() {
                    result.to_dtype(DType::BF16)?
                } else {
                    result
                }
            }
        };

        Ok(embeddings)
    }

    /// Clear KV cache (useful for memory management)
    pub fn clear_cache(&mut self) {
        match &mut self.model {
            T5Model::Regular(model) => model.clear_kv_cache(),
            T5Model::Quantized(model) => model.clear_kv_cache(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires downloaded model
    fn test_t5_encoding() {
        use crate::models::ModelPaths;

        let paths = ModelPaths::new().unwrap();
        let device = Device::cuda_if_available(0).unwrap();

        let mut encoder = T5TextEncoder::load(
            paths.t5_path(),
            paths.t5_tokenizer_path(),
            device,
        )
        .unwrap();

        let embeddings = encoder.encode("a beautiful sunset").unwrap();
        let shape = embeddings.dims();

        assert_eq!(shape[0], 1); // batch
        assert_eq!(shape[1], 256); // sequence length
        assert_eq!(shape[2], 4096); // d_model for T5-xxl
    }

    #[test]
    #[ignore] // Requires downloaded quantized model
    fn test_quantized_t5_encoding() {
        use crate::models::ModelPaths;

        let paths = ModelPaths::new().unwrap();
        let device = Device::cuda_if_available(0).unwrap();

        if paths.has_quantized_t5() {
            let mut encoder = T5TextEncoder::load_quantized(
                paths.quantized_t5_path(),
                paths.t5_tokenizer_path(),
                device,
            )
            .unwrap();

            assert!(encoder.is_quantized());

            let embeddings = encoder.encode("a beautiful sunset").unwrap();
            let shape = embeddings.dims();

            assert_eq!(shape[0], 1);
            assert_eq!(shape[1], 256);
            assert_eq!(shape[2], 4096);
        }
    }
}
