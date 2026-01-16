//! Model downloading from HuggingFace Hub

use anyhow::Result;
use hf_hub::api::tokio::Api;
use super::ModelPaths;

/// Downloads and manages FLUX models from HuggingFace Hub
pub struct ModelDownloader {
    paths: ModelPaths,
}

impl ModelDownloader {
    /// Create new downloader
    pub fn new() -> Result<Self> {
        Ok(Self {
            paths: ModelPaths::new()?,
        })
    }

    /// Check if FLUX Schnell is already downloaded
    pub fn is_schnell_downloaded(&self) -> bool {
        self.paths.all_files_exist()
    }

    /// Download FLUX Schnell model from HuggingFace Hub
    ///
    /// Downloads to ~/.cache/huggingface/hub/models--black-forest-labs--FLUX.1-schnell/
    /// Model files: ~12GB total
    pub async fn download_schnell(&self) -> Result<()> {
        if self.is_schnell_downloaded() {
            println!("FLUX Schnell already downloaded");
            return Ok(());
        }

        println!("Downloading FLUX Schnell from HuggingFace Hub...");
        println!("This will download ~12GB of model files");

        let api = Api::new()?;
        let repo = api.model("black-forest-labs/FLUX.1-schnell".to_string());

        // Download required files
        let files = vec![
            "text_encoder/model.safetensors",
            "text_encoder/config.json",
            "vae/diffusion_pytorch_model.safetensors",
            "vae/config.json",
            "transformer/diffusion_pytorch_model.safetensors",
            "transformer/config.json",
            "scheduler/scheduler_config.json",
            "tokenizer/vocab.json",
            "tokenizer/merges.txt",
            "tokenizer/special_tokens_map.json",
            "tokenizer/tokenizer_config.json",
        ];

        for file in files {
            println!("Downloading {}", file);
            repo.get(file).await?;
        }

        println!("FLUX Schnell download complete!");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downloader_creation() {
        let _downloader = ModelDownloader::new().unwrap();
    }
}

#[tokio::test]
#[ignore] // Ignore by default (downloads 12GB)
async fn test_download_schnell() {
    let downloader = ModelDownloader::new().unwrap();
    // This test is marked ignore - run with: cargo test -- --ignored
    // Only run this if you want to actually download the model
    downloader.download_schnell().await.unwrap();
    assert!(downloader.is_schnell_downloaded());
}
