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

    // ===== FLUX Dev Methods =====

    /// Get the FLUX Dev model directory
    pub fn dev_dir(&self) -> PathBuf {
        self.cache_dir.join("models--black-forest-labs--FLUX.1-dev")
    }

    /// Get snapshot hash for Dev model
    fn get_dev_snapshot_hash(&self) -> Result<String> {
        let refs_main = self.dev_dir().join("refs").join("main");

        if refs_main.exists() {
            let hash = std::fs::read_to_string(&refs_main)
                .context("Failed to read Dev refs/main")?
                .trim()
                .to_string();
            Ok(hash)
        } else {
            let snapshots_dir = self.dev_dir().join("snapshots");
            if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            return Ok(name.to_string());
                        }
                    }
                }
            }
            anyhow::bail!("Could not find Dev snapshot directory")
        }
    }

    /// Get path to FLUX Dev transformer
    pub fn dev_transformer_path(&self) -> PathBuf {
        self.get_dev_snapshot_hash()
            .map(|hash| self.dev_dir().join("snapshots").join(hash).join("flux1-dev.safetensors"))
            .unwrap_or_else(|_| self.dev_dir().join("snapshots").join("main").join("flux1-dev.safetensors"))
    }

    /// Get path to quantized FLUX Dev transformer
    /// Checks multiple sources: lmz/candle-flux and city96/FLUX.1-dev-gguf
    pub fn quantized_dev_transformer_path(&self) -> PathBuf {
        // First, check city96/FLUX.1-dev-gguf (Q8_0 quantization)
        let city96_dir = self.cache_dir.join("models--city96--FLUX.1-dev-gguf");
        if let Ok(refs_main) = std::fs::read_to_string(city96_dir.join("refs").join("main")) {
            let hash = refs_main.trim();
            let path = city96_dir.join("snapshots").join(hash).join("flux1-dev-Q8_0.gguf");
            if path.exists() {
                return path;
            }
        }
        if let Ok(entries) = std::fs::read_dir(city96_dir.join("snapshots")) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let path = entry.path().join("flux1-dev-Q8_0.gguf");
                    if path.exists() {
                        return path;
                    }
                }
            }
        }

        // Fallback: check lmz/candle-flux
        let lmz_dir = self.cache_dir.join("models--lmz--candle-flux");
        if let Ok(refs_main) = std::fs::read_to_string(lmz_dir.join("refs").join("main")) {
            let hash = refs_main.trim();
            return lmz_dir.join("snapshots").join(hash).join("flux1-dev.gguf");
        }

        if let Ok(entries) = std::fs::read_dir(lmz_dir.join("snapshots")) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    return entry.path().join("flux1-dev.gguf");
                }
            }
        }

        // Default fallback path
        city96_dir.join("snapshots").join("main").join("flux1-dev-Q8_0.gguf")
    }

    /// Check if Dev model is downloaded
    pub fn is_dev_downloaded(&self) -> bool {
        self.dev_transformer_path().exists() || self.quantized_dev_transformer_path().exists()
    }

    /// Check if quantized Dev transformer is available
    pub fn has_quantized_dev(&self) -> bool {
        self.quantized_dev_transformer_path().exists()
    }

    // ===== Z-Image-Turbo Paths =====

    /// Get Z-Image-Turbo base directory
    pub fn zimage_dir(&self) -> PathBuf {
        self.cache_dir.join("models--Tongyi-MAI--Z-Image-Turbo")
    }

    /// Get Z-Image-Turbo snapshot hash
    fn get_zimage_snapshot_hash(&self) -> Result<String> {
        let refs_main = self.zimage_dir().join("refs").join("main");
        if refs_main.exists() {
            let hash = std::fs::read_to_string(&refs_main)
                .context("Failed to read Z-Image refs/main")?
                .trim()
                .to_string();
            Ok(hash)
        } else {
            // Fallback: find first directory in snapshots/
            let snapshots_dir = self.zimage_dir().join("snapshots");
            if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            return Ok(name.to_string());
                        }
                    }
                }
            }
            anyhow::bail!("Could not find Z-Image snapshot directory")
        }
    }

    /// Get path to Qwen3 text encoder
    pub fn qwen3_path(&self) -> PathBuf {
        self.get_zimage_snapshot_hash()
            .map(|hash| self.zimage_dir().join("snapshots").join(hash).join("text_encoder"))
            .unwrap_or_else(|_| self.zimage_dir().join("snapshots").join("main").join("text_encoder"))
    }

    /// Get path to Qwen3 tokenizer
    pub fn qwen3_tokenizer_path(&self) -> PathBuf {
        self.get_zimage_snapshot_hash()
            .map(|hash| self.zimage_dir().join("snapshots").join(hash).join("tokenizer"))
            .unwrap_or_else(|_| self.zimage_dir().join("snapshots").join("main").join("tokenizer"))
    }

    /// Get path to Z-Image-Turbo transformer
    pub fn zimage_transformer_path(&self) -> PathBuf {
        self.get_zimage_snapshot_hash()
            .map(|hash| self.zimage_dir().join("snapshots").join(hash).join("transformer"))
            .unwrap_or_else(|_| self.zimage_dir().join("snapshots").join("main").join("transformer"))
    }

    /// Get path to Z-Image-Turbo VAE (shared with FLUX)
    pub fn zimage_vae_path(&self) -> PathBuf {
        self.get_zimage_snapshot_hash()
            .map(|hash| self.zimage_dir().join("snapshots").join(hash).join("vae"))
            .unwrap_or_else(|_| self.zimage_dir().join("snapshots").join("main").join("vae"))
    }

    /// Get path to quantized Z-Image-Turbo transformer (to be created)
    pub fn quantized_zimage_transformer_path(&self) -> PathBuf {
        // Placeholder for future quantized Z-Image model
        // Format: zimage-turbo.gguf (similar to FLUX GGUF)
        self.cache_dir
            .join("models--lmz--candle-zimage") // Future quantized repo
            .join("snapshots")
            .join("main")
            .join("zimage-turbo.gguf")
    }

    /// Check if Z-Image-Turbo model is downloaded
    pub fn is_zimage_downloaded(&self) -> bool {
        // Check if transformer directory exists and has the sharded safetensors files
        let transformer_dir = self.zimage_transformer_path();
        transformer_dir.join("diffusion_pytorch_model-00001-of-00003.safetensors").exists()
            && transformer_dir.join("diffusion_pytorch_model-00002-of-00003.safetensors").exists()
            && transformer_dir.join("diffusion_pytorch_model-00003-of-00003.safetensors").exists()
    }

    /// Check if quantized Z-Image-Turbo transformer is available
    pub fn has_quantized_zimage(&self) -> bool {
        self.quantized_zimage_transformer_path().exists()
    }

    // ===== Model Type Helpers =====

    /// Get transformer path for a given model type
    pub fn transformer_path_for(&self, model_type: super::ModelType) -> PathBuf {
        match model_type {
            super::ModelType::Schnell => self.transformer_path(),
            super::ModelType::Dev => self.dev_transformer_path(),
            super::ModelType::ZImageTurbo => self.zimage_transformer_path(),
        }
    }

    /// Get quantized transformer path for a given model type
    pub fn quantized_transformer_path_for(&self, model_type: super::ModelType) -> PathBuf {
        match model_type {
            super::ModelType::Schnell => self.quantized_transformer_path(),
            super::ModelType::Dev => self.quantized_dev_transformer_path(),
            super::ModelType::ZImageTurbo => self.quantized_zimage_transformer_path(),
        }
    }

    /// Check if quantized version exists for model type
    pub fn has_quantized_for(&self, model_type: super::ModelType) -> bool {
        match model_type {
            super::ModelType::Schnell => self.has_quantized_transformer(),
            super::ModelType::Dev => self.has_quantized_dev(),
            super::ModelType::ZImageTurbo => self.has_quantized_zimage(),
        }
    }

    // ===== Validation =====

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
        let dev_transformer_path = self.dev_transformer_path();
        let quantized_dev_path = self.quantized_dev_transformer_path();
        let t5_path = self.t5_path();
        let t5_model_path = t5_path.join("model-00001-of-00002.safetensors");
        let t5_config_path = t5_path.join("config.json");
        let quantized_t5_path = self.quantized_t5_path();
        let t5_tokenizer_path = self.t5_tokenizer_path();

        vec![
            ("CLIP text encoder".to_string(), clip_path.exists(), clip_path.display().to_string()),
            ("VAE (ae.safetensors)".to_string(), vae_path.exists(), vae_path.display().to_string()),
            ("Schnell transformer (full)".to_string(), transformer_path.exists(), transformer_path.display().to_string()),
            ("Schnell transformer (quantized)".to_string(), quantized_transformer_path.exists(), quantized_transformer_path.display().to_string()),
            ("Dev transformer (full)".to_string(), dev_transformer_path.exists(), dev_transformer_path.display().to_string()),
            ("Dev transformer (quantized)".to_string(), quantized_dev_path.exists(), quantized_dev_path.display().to_string()),
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
