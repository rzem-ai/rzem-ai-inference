//! Path management for model files

use anyhow::Result;
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
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        let cache_dir = home.join(".cache").join("huggingface").join("hub");
        let schnell_dir = cache_dir.join("models--black-forest-labs--FLUX.1-schnell");

        Ok(Self {
            cache_dir,
            schnell_dir,
        })
    }

    /// Get path to CLIP text encoder
    pub fn clip_path(&self) -> PathBuf {
        self.schnell_dir.join("snapshots").join("main").join("text_encoder")
    }

    /// Get path to VAE decoder
    pub fn vae_path(&self) -> PathBuf {
        self.schnell_dir.join("snapshots").join("main").join("vae")
    }

    /// Get path to FLUX transformer
    pub fn transformer_path(&self) -> PathBuf {
        self.schnell_dir.join("snapshots").join("main").join("transformer")
    }

    /// Get path to tokenizer
    pub fn tokenizer_path(&self) -> PathBuf {
        self.schnell_dir.join("snapshots").join("main").join("tokenizer")
    }

    /// Check if all required files exist
    pub fn all_files_exist(&self) -> bool {
        // Check for key model files, not just directories
        self.clip_path().join("model.safetensors").exists()
            && self.vae_path().join("diffusion_pytorch_model.safetensors").exists()
            && self.transformer_path().join("diffusion_pytorch_model.safetensors").exists()
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
