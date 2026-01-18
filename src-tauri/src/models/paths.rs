//! Path management for model files

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Manages paths for FLUX model files
pub struct ModelPaths {
    /// Base cache directory (~/.cache/huggingface)
    pub cache_dir: PathBuf,
    /// FLUX Schnell model directory
    pub schnell_dir: PathBuf,
}

impl ModelPaths {
    /// Create new ModelPaths using HuggingFace cache structure
    /// Checks multiple possible cache locations in order of priority
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        // Check for HF_HUB_CACHE or HF_HOME environment variables first
        let cache_dir = if let Ok(hf_cache) = std::env::var("HF_HUB_CACHE") {
            std::path::PathBuf::from(hf_cache)
        } else if let Ok(hf_home) = std::env::var("HF_HOME") {
            std::path::PathBuf::from(hf_home).join("hub")
        } else {
            // Check possible cache locations
            let possible_locations = vec![
                home.join(".cache").join("huggingface").join("hub"),
                // macOS alternative location
                #[cfg(target_os = "macos")]
                home.join("Library").join("Caches").join("huggingface").join("hub"),
            ];

            // Find the first location that exists and has the model, or default to first
            let schnell_model = "models--black-forest-labs--FLUX.1-schnell";
            possible_locations
                .iter()
                .find(|p| p.join(schnell_model).exists())
                .cloned()
                .unwrap_or_else(|| possible_locations[0].clone())
        };

        let schnell_dir = cache_dir.join("models--black-forest-labs--FLUX.1-schnell");

        Ok(Self {
            cache_dir,
            schnell_dir,
        })
    }

    /// Get the actual snapshot commit hash from refs/main
    fn get_snapshot_hash(&self) -> Result<String> {
        let refs_main = self.schnell_dir.join("refs").join("main");

        if refs_main.exists() {
            // Read commit hash from refs/main file
            let hash = std::fs::read_to_string(&refs_main)
                .context("Failed to read refs/main")?
                .trim()
                .to_string();
            Ok(hash)
        } else {
            // Fallback: find first directory in snapshots/
            let snapshots_dir = self.schnell_dir.join("snapshots");
            if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        if entry.path().is_dir() {
                            if let Some(name) = entry.file_name().to_str() {
                                return Ok(name.to_string());
                            }
                        }
                    }
                }
            }
            anyhow::bail!("Could not find snapshot directory")
        }
    }

    /// Get base path to snapshot directory
    fn snapshot_path(&self) -> Result<PathBuf> {
        let hash = self.get_snapshot_hash()?;
        Ok(self.schnell_dir.join("snapshots").join(hash))
    }

    /// Get path to CLIP text encoder
    pub fn clip_path(&self) -> PathBuf {
        self.snapshot_path()
            .map(|p| p.join("text_encoder"))
            .unwrap_or_else(|_| self.schnell_dir.join("snapshots").join("main").join("text_encoder"))
    }

    /// Get path to VAE decoder (ae.safetensors in native FLUX format)
    pub fn vae_path(&self) -> PathBuf {
        self.snapshot_path()
            .map(|p| p.join("ae.safetensors"))
            .unwrap_or_else(|_| self.schnell_dir.join("snapshots").join("main").join("ae.safetensors"))
    }

    /// Get path to FLUX transformer (native format single file)
    pub fn transformer_path(&self) -> PathBuf {
        self.snapshot_path()
            .map(|p| p.join("flux1-schnell.safetensors"))
            .unwrap_or_else(|_| self.schnell_dir.join("snapshots").join("main").join("flux1-schnell.safetensors"))
    }

    /// Get path to quantized FLUX transformer (GGUF format - ~12GB vs 23GB)
    /// From lmz/candle-flux repository
    pub fn quantized_transformer_path(&self) -> PathBuf {
        let lmz_dir = self.cache_dir.join("models--lmz--candle-flux");

        // Try to find the snapshot
        if let Ok(refs_main) = std::fs::read_to_string(lmz_dir.join("refs").join("main")) {
            let hash = refs_main.trim();
            return lmz_dir.join("snapshots").join(hash).join("flux1-schnell.gguf");
        }

        // Fallback: look for any snapshot
        if let Ok(entries) = std::fs::read_dir(lmz_dir.join("snapshots")) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    return entry.path().join("flux1-schnell.gguf");
                }
            }
        }

        // Default path
        lmz_dir.join("snapshots").join("main").join("flux1-schnell.gguf")
    }

    /// Check if quantized transformer is available
    pub fn has_quantized_transformer(&self) -> bool {
        self.quantized_transformer_path().exists()
    }

    /// Get path to CLIP tokenizer
    pub fn tokenizer_path(&self) -> PathBuf {
        self.snapshot_path()
            .map(|p| p.join("tokenizer"))
            .unwrap_or_else(|_| self.schnell_dir.join("snapshots").join("main").join("tokenizer"))
    }

    /// Get path to T5 text encoder (text_encoder_2)
    pub fn t5_path(&self) -> PathBuf {
        self.snapshot_path()
            .map(|p| p.join("text_encoder_2"))
            .unwrap_or_else(|_| self.schnell_dir.join("snapshots").join("main").join("text_encoder_2"))
    }

    /// Get path to quantized T5 encoder (GGUF format - ~3.3GB vs 9GB)
    /// From city96/t5-v1_1-xxl-encoder-gguf repository
    /// Uses Q5_K_M quantization as recommended
    pub fn quantized_t5_path(&self) -> PathBuf {
        let t5_gguf_dir = self.cache_dir.join("models--city96--t5-v1_1-xxl-encoder-gguf");

        // Try to find the snapshot
        if let Ok(refs_main) = std::fs::read_to_string(t5_gguf_dir.join("refs").join("main")) {
            let hash = refs_main.trim();
            return t5_gguf_dir.join("snapshots").join(hash).join("t5-v1_1-xxl-encoder-Q5_K_M.gguf");
        }

        // Fallback: look for any snapshot
        if let Ok(entries) = std::fs::read_dir(t5_gguf_dir.join("snapshots")) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    return entry.path().join("t5-v1_1-xxl-encoder-Q5_K_M.gguf");
                }
            }
        }

        // Default path
        t5_gguf_dir.join("snapshots").join("main").join("t5-v1_1-xxl-encoder-Q5_K_M.gguf")
    }

    /// Check if quantized T5 encoder is available
    pub fn has_quantized_t5(&self) -> bool {
        self.quantized_t5_path().exists()
    }

    /// Get path to T5 tokenizer (from lmz/mt5-tokenizers - compatible format)
    pub fn t5_tokenizer_path(&self) -> PathBuf {
        // Use the compatible tokenizer from lmz/mt5-tokenizers
        let mt5_dir = self.cache_dir.join("models--lmz--mt5-tokenizers");

        // Try to find the snapshot
        if let Ok(refs_main) = std::fs::read_to_string(mt5_dir.join("refs").join("main")) {
            let hash = refs_main.trim();
            return mt5_dir.join("snapshots").join(hash).join("t5-v1_1-xxl.tokenizer.json");
        }

        // Fallback: look for any snapshot
        if let Ok(entries) = std::fs::read_dir(mt5_dir.join("snapshots")) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    return entry.path().join("t5-v1_1-xxl.tokenizer.json");
                }
            }
        }

        // Default path
        mt5_dir.join("snapshots").join("main").join("t5-v1_1-xxl.tokenizer.json")
    }

    /// Check if all required files exist
    /// Accepts either full precision or quantized versions
    pub fn all_files_exist(&self) -> bool {
        // Check for key model files, not just directories
        let has_clip = self.clip_path().join("model.safetensors").exists();
        let has_vae = self.vae_path().exists(); // ae.safetensors (single file)

        // Transformer model - either full precision or quantized
        let has_transformer = self.transformer_path().exists()
            || self.has_quantized_transformer();

        // T5 model - either split safetensors or quantized GGUF
        let t5_dir = self.t5_path();
        let has_t5_full = t5_dir.join("model-00001-of-00002.safetensors").exists()
            && t5_dir.join("config.json").exists();
        let has_t5 = has_t5_full || self.has_quantized_t5();

        // T5 tokenizer (single file from lmz/mt5-tokenizers)
        let has_t5_tokenizer = self.t5_tokenizer_path().exists();

        has_clip && has_vae && has_transformer && has_t5 && has_t5_tokenizer
    }

    /// Get detailed status of which model files exist (for debugging)
    pub fn get_status(&self) -> Vec<(String, bool, String)> {
        let clip_path = self.clip_path().join("model.safetensors");
        let vae_path = self.vae_path();
        let transformer_path = self.transformer_path();
        let quantized_transformer_path = self.quantized_transformer_path();
        let t5_path = self.t5_path();
        let t5_model_path = t5_path.join("model-00001-of-00002.safetensors");
        let t5_config_path = t5_path.join("config.json");
        let quantized_t5_path = self.quantized_t5_path();
        let t5_tokenizer_path = self.t5_tokenizer_path();

        vec![
            ("CLIP text encoder".to_string(), clip_path.exists(), clip_path.display().to_string()),
            ("VAE (ae.safetensors)".to_string(), vae_path.exists(), vae_path.display().to_string()),
            ("Transformer (full)".to_string(), transformer_path.exists(), transformer_path.display().to_string()),
            ("Transformer (quantized)".to_string(), quantized_transformer_path.exists(), quantized_transformer_path.display().to_string()),
            ("T5 model".to_string(), t5_model_path.exists(), t5_model_path.display().to_string()),
            ("T5 config".to_string(), t5_config_path.exists(), t5_config_path.display().to_string()),
            ("T5 (quantized)".to_string(), quantized_t5_path.exists(), quantized_t5_path.display().to_string()),
            ("T5 tokenizer".to_string(), t5_tokenizer_path.exists(), t5_tokenizer_path.display().to_string()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths_creation() {
        let paths = ModelPaths::new().unwrap();
        assert!(paths.cache_dir.to_string_lossy().contains("huggingface"));
    }
}
