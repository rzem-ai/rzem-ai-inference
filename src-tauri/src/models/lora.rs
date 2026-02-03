//! LoRA (Low-Rank Adaptation) weight loading and management
//!
//! LoRA allows fine-tuning large models by training small adapter weights
//! that are merged with the base model: W' = W + (alpha/rank) * strength * (B @ A)

use anyhow::{Context, Result};
use candle_core::{DType, Device, Tensor};
use safetensors::SafeTensors;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tracing::{debug, warn};

/// Individual LoRA weight pair for a single layer
#[derive(Debug)]
pub struct LoraWeight {
    /// Layer name this weight applies to (e.g., "double_blocks.0.img_attn.qkv")
    pub layer_name: String,
    /// Down projection matrix (A) - reduces dimensionality
    pub lora_down: Tensor,
    /// Up projection matrix (B) - restores dimensionality
    pub lora_up: Tensor,
    /// Alpha scaling factor (defaults to rank if not specified)
    pub alpha: f32,
    /// Rank (dimension of the low-rank decomposition)
    pub rank: usize,
}

impl LoraWeight {
    /// Compute the LoRA delta: (alpha/rank) * (B @ A)
    /// This is the adjustment to add to the original weight
    pub fn compute_delta(&self, strength: f32) -> Result<Tensor> {
        let scale = strength * (self.alpha / self.rank as f32);

        // Compute B @ A (up @ down)
        // lora_down: [rank, in_features] or [in_features, rank] depending on format
        // lora_up: [out_features, rank] or [rank, out_features]
        let delta = self.lora_up.matmul(&self.lora_down)?;

        // Apply scaling
        let scaled = (delta * scale as f64)?;

        Ok(scaled)
    }
}

/// LoRA adapter containing all weights for a fine-tuned model
#[derive(Debug)]
pub struct LoraAdapter {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// All weight pairs keyed by normalized layer name
    pub weights: HashMap<String, LoraWeight>,
    /// Current strength/multiplier (0.0 - 2.0)
    pub strength: f32,
    /// Trigger words that activate this LoRA's style
    pub trigger_words: Option<String>,
    /// Original file path
    pub source_path: String,
}

/// Metadata for a LoRA stored on disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub trigger_words: Option<String>,
    pub base_model: Option<String>,
    pub size_bytes: u64,
    pub created_at: i64,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(default = "default_lora_strength")]
    pub default_strength: f32,
    #[serde(default = "default_lora_strength_min")]
    pub strength_min: f32,
    #[serde(default = "default_lora_strength_max")]
    pub strength_max: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub civitai_model_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub civitai_version_id: Option<i64>,
}

fn default_lora_strength() -> f32 {
    1.0
}

fn default_lora_strength_min() -> f32 {
    0.5
}

fn default_lora_strength_max() -> f32 {
    1.5
}

/// Configuration for applying a LoRA during generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraConfig {
    pub id: String,
    pub strength: f32,
}

impl LoraAdapter {
    /// Load a LoRA adapter from a safetensors file
    ///
    /// # Arguments
    /// * `path` - Path to the .safetensors file
    /// * `device` - Device to load tensors onto
    /// * `dtype` - Data type for tensors (usually F32 or BF16)
    pub fn load<P: AsRef<Path>>(
        path: P,
        id: String,
        name: String,
        device: &Device,
        dtype: DType,
    ) -> Result<Self> {
        let path = path.as_ref();
        let file_data = std::fs::read(path)
            .with_context(|| format!("Failed to read LoRA file: {}", path.display()))?;

        let tensors = SafeTensors::deserialize(&file_data)
            .with_context(|| format!("Failed to parse safetensors: {}", path.display()))?;

        let mut weights = HashMap::new();
        let mut alpha_values: HashMap<String, f32> = HashMap::new();
        let mut down_tensors: HashMap<String, Tensor> = HashMap::new();
        let mut up_tensors: HashMap<String, Tensor> = HashMap::new();

        // First pass: collect all tensors and alphas
        for (key, _) in tensors.tensors() {
            if key.ends_with(".alpha") {
                // Extract alpha value
                let tensor = load_tensor_from_safetensors(&tensors, &key, device, dtype)?;
                // Convert to F32 and move to CPU to safely extract scalar value
                let alpha = tensor
                    .to_dtype(DType::F32)?
                    .to_device(&Device::Cpu)?
                    .to_scalar::<f32>()?;
                let base_name = key.strip_suffix(".alpha").unwrap();
                let normalized = normalize_lora_key(base_name);
                debug!(
                    key = %key,
                    alpha = alpha,
                    normalized = %normalized,
                    "Extracted alpha value"
                );
                alpha_values.insert(normalized, alpha);
            } else if key.contains(".lora_down.") || key.contains(".lora_A.") {
                let tensor = load_tensor_from_safetensors(&tensors, &key, device, dtype)?;
                let base_name = extract_lora_base_name(&key);
                let normalized = normalize_lora_key(&base_name);
                down_tensors.insert(normalized, tensor);
            } else if key.contains(".lora_up.") || key.contains(".lora_B.") {
                let tensor = load_tensor_from_safetensors(&tensors, &key, device, dtype)?;
                let base_name = extract_lora_base_name(&key);
                let normalized = normalize_lora_key(&base_name);
                up_tensors.insert(normalized, tensor);
            }
        }

        // Second pass: pair up the weights
        for (layer_name, lora_down) in down_tensors {
            if let Some(lora_up) = up_tensors.remove(&layer_name) {
                // Determine rank from the down tensor shape
                let rank = lora_down.dims()[0];

                // Get alpha, defaulting to rank if not specified
                let alpha = alpha_values.get(&layer_name).copied().unwrap_or(rank as f32);

                debug!(
                    layer = %layer_name,
                    rank = rank,
                    alpha = alpha,
                    "Loaded LoRA weight"
                );

                weights.insert(layer_name.clone(), LoraWeight {
                    layer_name: layer_name.clone(),
                    lora_down,
                    lora_up,
                    alpha,
                    rank,
                });
            } else {
                warn!(layer = %layer_name, "LoRA down tensor without matching up tensor");
            }
        }

        // Warn about any orphaned up tensors
        for layer_name in up_tensors.keys() {
            warn!(layer = %layer_name, "LoRA up tensor without matching down tensor");
        }

        debug!(
            path = %path.display(),
            weight_count = weights.len(),
            "Loaded LoRA adapter"
        );

        Ok(Self {
            id,
            name,
            weights,
            strength: 1.0,
            trigger_words: None,
            source_path: path.to_string_lossy().to_string(),
        })
    }

    /// Get the number of weight pairs in this adapter
    pub fn weight_count(&self) -> usize {
        self.weights.len()
    }

    /// Check if this LoRA has weights for a specific layer
    pub fn has_layer(&self, layer_name: &str) -> bool {
        self.weights.contains_key(layer_name)
    }

    /// Get weight for a specific layer
    pub fn get_weight(&self, layer_name: &str) -> Option<&LoraWeight> {
        self.weights.get(layer_name)
    }
}

/// Load a tensor from safetensors and convert to target dtype
fn load_tensor_from_safetensors(
    tensors: &SafeTensors,
    key: &str,
    device: &Device,
    dtype: DType,
) -> Result<Tensor> {
    let view = tensors.tensor(key)
        .with_context(|| format!("Tensor not found: {}", key))?;

    // Convert safetensors dtype to candle dtype
    let st_dtype = view.dtype();
    let candle_dtype = match st_dtype {
        safetensors::Dtype::F32 => DType::F32,
        safetensors::Dtype::F16 => DType::F16,
        safetensors::Dtype::BF16 => DType::BF16,
        _ => anyhow::bail!("Unsupported tensor dtype: {:?}", st_dtype),
    };

    // Create tensor from raw data
    let shape: Vec<usize> = view.shape().to_vec();
    let data = view.data();

    // Load as original dtype first
    let tensor = match candle_dtype {
        DType::F32 => {
            let floats: Vec<f32> = data.chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();
            Tensor::from_vec(floats, shape.as_slice(), device)?
        }
        DType::F16 => {
            let halfs: Vec<half::f16> = data.chunks_exact(2)
                .map(|b| half::f16::from_le_bytes([b[0], b[1]]))
                .collect();
            Tensor::from_vec(halfs, shape.as_slice(), device)?
        }
        DType::BF16 => {
            let bhalfs: Vec<half::bf16> = data.chunks_exact(2)
                .map(|b| half::bf16::from_le_bytes([b[0], b[1]]))
                .collect();
            Tensor::from_vec(bhalfs, shape.as_slice(), device)?
        }
        _ => unreachable!(),
    };

    // Convert to target dtype if different
    if candle_dtype != dtype {
        Ok(tensor.to_dtype(dtype)?)
    } else {
        Ok(tensor)
    }
}

/// Extract the base layer name from a LoRA key
/// e.g., "lora_unet_double_blocks_0_img_attn_qkv.lora_down.weight" -> "lora_unet_double_blocks_0_img_attn_qkv"
fn extract_lora_base_name(key: &str) -> String {
    // Remove the .lora_down.weight, .lora_up.weight, .lora_A.weight, .lora_B.weight suffix
    let key = key.strip_suffix(".weight").unwrap_or(key);

    if let Some(pos) = key.rfind(".lora_down") {
        key[..pos].to_string()
    } else if let Some(pos) = key.rfind(".lora_up") {
        key[..pos].to_string()
    } else if let Some(pos) = key.rfind(".lora_A") {
        key[..pos].to_string()
    } else if let Some(pos) = key.rfind(".lora_B") {
        key[..pos].to_string()
    } else {
        key.to_string()
    }
}

/// Validates that a safetensors file contains a valid LoRA
/// Returns (weight_pairs_count, rank, is_flux_compatible)
pub fn validate_lora_safetensors(file_data: &[u8]) -> Result<(usize, Option<usize>, bool)> {
    // 1. Deserialize safetensors
    let tensors = SafeTensors::deserialize(file_data)
        .context("Not a valid safetensors file")?;

    // 2. Find all down/up tensors
    let mut down_tensors: HashMap<String, (usize, usize)> = HashMap::new();
    let mut up_tensors: HashSet<String> = HashSet::new();
    let mut rank: Option<usize> = None;

    for (key, _) in tensors.tensors() {
        if key.contains(".lora_down.") || key.contains(".lora_A.") {
            let normalized = normalize_lora_key(&extract_lora_base_name(&key));

            let tensor_view = tensors.tensor(&key)
                .with_context(|| format!("Failed to get tensor: {}", key))?;
            let shape = tensor_view.shape();

            // Validate 2D tensor
            if shape.len() != 2 {
                anyhow::bail!("LoRA tensor {} is not a 2D matrix", key);
            }

            // Extract rank from first dimension
            if rank.is_none() {
                rank = Some(shape[0]);
            }

            down_tensors.insert(normalized, (shape[0], shape[1]));
        } else if key.contains(".lora_up.") || key.contains(".lora_B.") {
            let normalized = normalize_lora_key(&extract_lora_base_name(&key));
            up_tensors.insert(normalized);
        }
    }

    // 3. Validate at least one complete pair exists
    if down_tensors.is_empty() {
        anyhow::bail!("No LoRA weight tensors found (expected .lora_down or .lora_A)");
    }

    let mut pair_count = 0;
    for (layer_name, _) in &down_tensors {
        if up_tensors.contains(layer_name) {
            pair_count += 1;
        }
    }

    if pair_count == 0 {
        anyhow::bail!("No matching LoRA up/down pairs found");
    }

    // 4. Check FLUX compatibility
    let is_flux = check_flux_compatibility(&down_tensors);

    Ok((pair_count, rank, is_flux))
}

/// Check if LoRA layer names indicate FLUX compatibility
fn check_flux_compatibility(down_tensors: &HashMap<String, (usize, usize)>) -> bool {
    // Common FLUX layer patterns (after normalization)
    let flux_patterns = [
        "double.blocks.",
        "single.blocks.",
        "double_blocks.",
        "single_blocks.",
        "img.attn.",
        "txt.attn.",
        "img.mlp.",
        "txt.mlp.",
        "img_attn.",
        "txt_attn.",
        "img_mlp.",
        "txt_mlp.",
        "transformer.blocks.",
    ];

    // Incompatible patterns from SD1.5/SDXL
    let incompatible_patterns = [
        "down.blocks.",
        "up.blocks.",
        "mid.block.",
        "down_blocks.",
        "up_blocks.",
        "mid_block.",
        "input.blocks.",
        "output.blocks.",
        "input_blocks.",
        "output_blocks.",
    ];

    // Check for incompatible architectures first
    let has_incompatible_layers = down_tensors.keys()
        .any(|name| incompatible_patterns.iter().any(|p| name.contains(p)));

    if has_incompatible_layers {
        debug!("LoRA has incompatible layer patterns (SD1.5/SDXL)");
        return false;
    }

    // Check for FLUX patterns
    let has_flux_layers = down_tensors.keys()
        .any(|name| flux_patterns.iter().any(|p| name.contains(p)));

    // If it has LoRA pairs and no SD patterns, assume FLUX compatible
    // This is more permissive and avoids false negatives
    if has_flux_layers {
        debug!("LoRA has FLUX layer patterns");
        return true;
    }

    // If no clear FLUX patterns but also no incompatible patterns,
    // log the layer names for debugging and assume compatible
    if !down_tensors.is_empty() {
        debug!(
            "LoRA has no clear FLUX patterns but also no incompatible patterns. Sample layers: {:?}",
            down_tensors.keys().take(5).collect::<Vec<_>>()
        );
        // Assume FLUX compatible if it's a valid LoRA without SD patterns
        return true;
    }

    false
}

/// Normalize LoRA key to FLUX transformer layer name
/// Converts various LoRA naming conventions to the internal FLUX layer names
pub fn normalize_lora_key(key: &str) -> String {
    // Common FLUX LoRA patterns:
    // "lora_unet_double_blocks_0_img_attn_qkv" -> "double_blocks.0.img_attn.qkv"
    // "transformer.transformer_blocks.0.attn.to_q" -> "transformer_blocks.0.attn.to_q"

    let mut normalized = key.to_string();

    // Remove common prefixes
    for prefix in ["lora_unet_", "lora_te_", "lora_te1_", "lora_te2_", "transformer."] {
        if normalized.starts_with(prefix) {
            normalized = normalized[prefix.len()..].to_string();
        }
    }

    // Convert underscores to dots for block indices
    // double_blocks_0_img_attn -> double_blocks.0.img_attn
    let parts: Vec<&str> = normalized.split('_').collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < parts.len() {
        let part = parts[i];

        // Check if this is a block name followed by a number
        if i + 1 < parts.len() && parts[i + 1].parse::<u32>().is_ok() {
            // Combine: "double_blocks" + "0" -> "double_blocks.0"
            result.push(format!("{}.{}", part, parts[i + 1]));
            i += 2;
        } else {
            result.push(part.to_string());
            i += 1;
        }
    }

    result.join(".")
}

/// Preview info about a LoRA file without fully loading it
pub fn get_lora_file_info<P: AsRef<Path>>(path: P) -> Result<LoraFileInfo> {
    let path = path.as_ref();
    let file_data = std::fs::read(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    // Use validation function
    let (weight_count, rank, is_flux) = validate_lora_safetensors(&file_data)?;

    // Calculate total parameters
    let total_params = calculate_params(&file_data)?;

    // Get file size
    let metadata = std::fs::metadata(path)?;

    Ok(LoraFileInfo {
        path: path.to_string_lossy().to_string(),
        size_bytes: metadata.len(),
        weight_count,
        rank,
        total_params,
        is_flux_compatible: is_flux,
    })
}

/// Calculate total parameters in a safetensors file
fn calculate_params(file_data: &[u8]) -> Result<u64> {
    let tensors = SafeTensors::deserialize(file_data)?;
    let mut total = 0u64;

    for (_, tensor_view) in tensors.tensors() {
        let shape = tensor_view.shape();
        let count: u64 = shape.iter().map(|&d| d as u64).product();
        total += count;
    }

    Ok(total)
}

/// Basic info about a LoRA file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraFileInfo {
    pub path: String,
    pub size_bytes: u64,
    pub weight_count: usize,
    pub rank: Option<usize>,
    pub total_params: u64,
    pub is_flux_compatible: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_lora_key() {
        assert_eq!(
            normalize_lora_key("lora_unet_double_blocks_0_img_attn_qkv"),
            "double_blocks.0.img.attn.qkv"
        );

        assert_eq!(
            normalize_lora_key("transformer.transformer_blocks.0.attn.to_q"),
            "transformer_blocks.0.attn.to.q"
        );
    }

    #[test]
    fn test_extract_lora_base_name() {
        assert_eq!(
            extract_lora_base_name("lora_unet_double_blocks_0_img_attn_qkv.lora_down.weight"),
            "lora_unet_double_blocks_0_img_attn_qkv"
        );

        assert_eq!(
            extract_lora_base_name("some_layer.lora_A.weight"),
            "some_layer"
        );
    }
}
