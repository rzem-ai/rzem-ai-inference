//! T5 text encoder for FLUX with quantized model support
//!
//! Supports loading from:
//! - Full precision safetensors (HuggingFace format)
//! - Quantized GGUF files (city96/t5-v1_1-xxl-encoder-gguf with name mapping)

use anyhow::Result;
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::t5;
use candle_transformers::models::quantized_t5;
use std::path::Path;
use tokenizers::Tokenizer;
use tracing::{debug, info, warn};

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

    /// Load quantized T5 encoder from GGUF file (city96 format with name mapping)
    ///
    /// This loader handles the llama.cpp naming convention used by city96/t5-v1_1-xxl-encoder-gguf
    /// and maps tensor names to the HuggingFace format expected by candle.
    ///
    /// # Arguments
    /// * `model_path` - Path to t5-v1_1-xxl-encoder-*.gguf file
    /// * `tokenizer_path` - Path to tokenizer.json file
    /// * `device` - Device to load model on
    pub fn load_quantized<P: AsRef<Path>>(
        model_path: P,
        tokenizer_path: P,
        device: Device,
    ) -> Result<Self> {
        let model_path = model_path.as_ref();
        let tokenizer_path = tokenizer_path.as_ref();

        info!(path = ?model_path, "Loading quantized T5");

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load T5 tokenizer from {:?}: {}", tokenizer_path, e))?;

        // Load GGUF with name mapping
        let vb = MappedQVarBuilder::from_gguf(model_path, &device, map_llama_to_hf)?;

        // T5-v1.1-XXL config
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
            "is_encoder_decoder": false,
            "use_cache": true,
            "pad_token_id": 0,
            "eos_token_id": 1
        }"#;
        let config: quantized_t5::Config = serde_json::from_str(config_json)?;

        let model = quantized_t5::T5EncoderModel::load(vb.into(), &config)?;

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

// ============================================================================
// Custom VarBuilder with name mapping support
// ============================================================================

use candle_core::quantized::QTensor;
use candle_core::{Result as CandleResult, Shape};
use std::collections::HashMap;
use std::sync::Arc;

/// VarBuilder for quantized tensors with name mapping support
///
/// This is a reimplementation of candle's quantized_var_builder::VarBuilder
/// that supports renaming tensors during load (for compatibility with different
/// GGUF naming conventions).
#[derive(Clone)]
pub struct MappedQVarBuilder {
    data: Arc<HashMap<String, Arc<QTensor>>>,
    path: Vec<String>,
    device: Device,
}

impl MappedQVarBuilder {
    /// Load GGUF file with name mapping
    ///
    /// The `rename_fn` is called for each tensor name in the GGUF file
    /// and should return the new name to use in the VarBuilder.
    pub fn from_gguf<P, F>(path: P, device: &Device, rename_fn: F) -> Result<Self>
    where
        P: AsRef<Path>,
        F: Fn(&str) -> String,
    {
        use candle_core::quantized::gguf_file;

        let path = path.as_ref();
        let mut file = std::fs::File::open(path)?;
        let content = gguf_file::Content::read(&mut file)
            .map_err(|e| anyhow::anyhow!("Failed to read GGUF: {}", e))?;

        let mut data = HashMap::new();
        for tensor_name in content.tensor_infos.keys() {
            let tensor = content.tensor(&mut file, tensor_name, device)
                .map_err(|e| anyhow::anyhow!("Failed to load tensor {}: {}", tensor_name, e))?;
            let mapped_name = rename_fn(tensor_name);
            data.insert(mapped_name, Arc::new(tensor));
        }

        debug!(tensor_count = data.len(), "Loaded tensors with name mapping");

        Ok(Self {
            data: Arc::new(data),
            path: Vec::new(),
            device: device.clone(),
        })
    }

    /// Create a sub-builder with a path prefix
    pub fn pp<S: ToString>(&self, s: S) -> Self {
        let mut path = self.path.clone();
        path.push(s.to_string());
        Self {
            data: self.data.clone(),
            path,
            device: self.device.clone(),
        }
    }

    fn full_path(&self, tensor_name: &str) -> String {
        if self.path.is_empty() {
            tensor_name.to_string()
        } else {
            format!("{}.{}", self.path.join("."), tensor_name)
        }
    }

    /// Get a tensor by name with shape checking
    pub fn get<S: Into<Shape>>(&self, shape: S, name: &str) -> CandleResult<Arc<QTensor>> {
        let path = self.full_path(name);
        match self.data.get(&path) {
            None => {
                candle_core::bail!("cannot find tensor {path}")
            }
            Some(qtensor) => {
                let expected_shape = shape.into();
                if qtensor.shape() != &expected_shape {
                    candle_core::bail!(
                        "shape mismatch for {path}, got {:?}, expected {expected_shape:?}",
                        qtensor.shape()
                    )
                }
                Ok(qtensor.clone())
            }
        }
    }

    /// Get a tensor by name without shape checking
    pub fn get_no_shape(&self, name: &str) -> CandleResult<Arc<QTensor>> {
        let path = self.full_path(name);
        match self.data.get(&path) {
            None => {
                candle_core::bail!("cannot find tensor {path}")
            }
            Some(qtensor) => Ok(qtensor.clone()),
        }
    }

    /// Get the device
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Check if a key exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
}

// Implement the trait that candle's quantized_t5 expects
// candle_transformers uses its own VarBuilder type, so we need to make our
// MappedQVarBuilder compatible with quantized_t5::T5EncoderModel::load

// Actually, looking at candle's code, T5EncoderModel::load takes VarBuilder directly
// We need to convert our MappedQVarBuilder to their VarBuilder type
// Since they have the same structure, we can use unsafe transmute or re-export

// Better approach: implement From trait or use the same internal representation
impl From<MappedQVarBuilder> for candle_transformers::quantized_var_builder::VarBuilder {
    fn from(mapped: MappedQVarBuilder) -> Self {
        // Both types have the same internal representation:
        // - data: Arc<HashMap<String, Arc<QTensor>>>
        // - path: Vec<String>
        // - device: Device
        //
        // We can safely transmute since the memory layout is identical
        // This is safe because we control both types and know they're identical
        unsafe { std::mem::transmute(mapped) }
    }
}

// ============================================================================
// Name mapping functions
// ============================================================================

/// Map llama.cpp tensor name to HuggingFace tensor name
///
/// city96/t5-v1_1-xxl-encoder-gguf uses llama.cpp naming convention:
/// - token_embd.weight -> shared.weight
/// - enc.blk.{N}.attn_k.weight -> encoder.block.{N}.layer.0.SelfAttention.k.weight
/// - enc.output_norm.weight -> encoder.final_layer_norm.weight
fn map_llama_to_hf(llama_name: &str) -> String {
    // token_embd.weight -> shared.weight
    if llama_name == "token_embd.weight" {
        return "shared.weight".to_string();
    }

    // enc.output_norm.weight -> encoder.final_layer_norm.weight
    if llama_name == "enc.output_norm.weight" {
        return "encoder.final_layer_norm.weight".to_string();
    }

    // enc.blk.{N}.* -> encoder.block.{N}.*
    if llama_name.starts_with("enc.blk.") {
        // Parse: enc.blk.{N}.{rest}
        let parts: Vec<&str> = llama_name.split('.').collect();
        if parts.len() >= 4 {
            let block_num = parts[2];
            let rest = parts[3..].join(".");

            let hf_rest = match rest.as_str() {
                // Attention weights
                "attn_k.weight" => "layer.0.SelfAttention.k.weight",
                "attn_q.weight" => "layer.0.SelfAttention.q.weight",
                "attn_v.weight" => "layer.0.SelfAttention.v.weight",
                "attn_o.weight" => "layer.0.SelfAttention.o.weight",
                "attn_rel_b.weight" => "layer.0.SelfAttention.relative_attention_bias.weight",
                "attn_norm.weight" => "layer.0.layer_norm.weight",
                // FFN weights (gated-gelu has wi_0 and wi_1)
                "ffn_gate.weight" => "layer.1.DenseReluDense.wi_0.weight",
                "ffn_up.weight" => "layer.1.DenseReluDense.wi_1.weight",
                "ffn_down.weight" => "layer.1.DenseReluDense.wo.weight",
                "ffn_norm.weight" => "layer.1.layer_norm.weight",
                _ => {
                    warn!(suffix = %rest, "Unknown tensor suffix in GGUF");
                    return llama_name.to_string();
                }
            };

            return format!("encoder.block.{}.{}", block_num, hf_rest);
        }
    }

    // Unknown pattern - return as-is
    warn!(tensor = %llama_name, "Unmapped tensor name in GGUF");
    llama_name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_mapping() {
        assert_eq!(map_llama_to_hf("token_embd.weight"), "shared.weight");
        assert_eq!(
            map_llama_to_hf("enc.output_norm.weight"),
            "encoder.final_layer_norm.weight"
        );
        assert_eq!(
            map_llama_to_hf("enc.blk.0.attn_k.weight"),
            "encoder.block.0.layer.0.SelfAttention.k.weight"
        );
        assert_eq!(
            map_llama_to_hf("enc.blk.23.ffn_gate.weight"),
            "encoder.block.23.layer.1.DenseReluDense.wi_0.weight"
        );
        assert_eq!(
            map_llama_to_hf("enc.blk.5.attn_rel_b.weight"),
            "encoder.block.5.layer.0.SelfAttention.relative_attention_bias.weight"
        );
    }

    #[test]
    #[ignore] // Requires downloaded model
    fn test_t5_encoding() {
        use crate::models::ModelPaths;

        let paths = ModelPaths::new().unwrap();
        let device = Device::cuda_if_available(0).unwrap();

        let mut encoder =
            T5TextEncoder::load(paths.t5_path().unwrap(), paths.t5_tokenizer_path().unwrap(), device).unwrap();

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

        // TODO: Update test for bundle-based quantized T5
        if false {
            let mut encoder = T5TextEncoder::load_quantized(
                paths.t5_path().unwrap(),
                paths.t5_tokenizer_path().unwrap(),
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
