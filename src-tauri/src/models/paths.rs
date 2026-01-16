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

    /// Get path to VAE decoder
    pub fn vae_path(&self) -> PathBuf {
        self.snapshot_path()
            .map(|p| p.join("vae"))
            .unwrap_or_else(|_| self.schnell_dir.join("snapshots").join("main").join("vae"))
    }

    /// Get path to FLUX transformer
    pub fn transformer_path(&self) -> PathBuf {
        self.snapshot_path()
            .map(|p| p.join("transformer"))
            .unwrap_or_else(|_| self.schnell_dir.join("snapshots").join("main").join("transformer"))
    }

    /// Get path to tokenizer
    pub fn tokenizer_path(&self) -> PathBuf {
        self.snapshot_path()
            .map(|p| p.join("tokenizer"))
            .unwrap_or_else(|_| self.schnell_dir.join("snapshots").join("main").join("tokenizer"))
    }

    /// Check if all required files exist
    pub fn all_files_exist(&self) -> bool {
        // Check for key model files, not just directories
        let has_clip = self.clip_path().join("model.safetensors").exists();
        let has_vae = self.vae_path().join("diffusion_pytorch_model.safetensors").exists();

        // Transformer model is split into 3 parts - check for first part and index
        let transformer_dir = self.transformer_path();
        let has_transformer = transformer_dir
            .join("diffusion_pytorch_model-00001-of-00003.safetensors")
            .exists()
            && transformer_dir
                .join("diffusion_pytorch_model.safetensors.index.json")
                .exists();

        has_clip && has_vae && has_transformer
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
