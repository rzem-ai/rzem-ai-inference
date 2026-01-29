//! Model downloading from HuggingFace Hub

use anyhow::{Context, Result};
use hf_hub::api::tokio::Api;
use super::{ModelPaths, ModelType};
use serde::Deserialize;
use tracing::info;

/// Response from HuggingFace API for repository file tree
#[derive(Debug, Deserialize)]
struct RepoFile {
    #[serde(rename = "type")]
    file_type: String,
    path: String,
    size: Option<u64>,
}

/// Repository tree response
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RepoTreeItem {
    File(RepoFile),
    Directory {
        #[serde(rename = "type")]
        item_type: String,
        path: String,
    },
}

/// Downloads and manages FLUX models from HuggingFace Hub
pub struct ModelDownloader {
    paths: ModelPaths,
}

impl ModelDownloader {
    /// Create new downloader
    pub fn new() -> Result<Self> {
        let db_path = ModelPaths::get_db_path()?;
        let db = crate::gallery::GalleryDb::new(&db_path)?;
        Ok(Self {
            paths: ModelPaths::new(&db)?,
        })
    }

    /// Check if FLUX Schnell is already downloaded
    pub fn is_schnell_downloaded(&self) -> bool {
        self.paths.all_files_exist()
    }

    /// Check if FLUX Dev is already downloaded
    /// DEPRECATED: Legacy downloader - use bundle system instead
    pub fn is_dev_downloaded(&self) -> bool {
        false // Legacy method - dev model is now managed via bundles
    }

    /// Check if a specific model is downloaded
    /// DEPRECATED: Legacy downloader - use bundle system instead
    pub fn is_model_downloaded(&self, model_type: ModelType) -> bool {
        match model_type.id() {
            "schnell" => self.is_schnell_downloaded(),
            "dev" => self.is_dev_downloaded(),
            _ => false, // Z-Image removed - use bundle system
        }
    }

    /// Fetch list of files in FLUX Schnell repository from HuggingFace API
    async fn fetch_repo_files(repo_id: &str, token: &str) -> Result<Vec<String>> {
        let url = format!(
            "https://huggingface.co/api/models/{}/tree/main?recursive=true",
            repo_id
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", "rzem-ai-inference/0.1.0")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to query HuggingFace API")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "HuggingFace API returned status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        let items: Vec<RepoTreeItem> = response
            .json()
            .await
            .context("Failed to parse API response")?;

        // Extract file paths (not directories)
        let files: Vec<String> = items
            .into_iter()
            .filter_map(|item| match item {
                RepoTreeItem::File(file) => Some(file.path),
                RepoTreeItem::Directory { .. } => None,
            })
            .collect();

        Ok(files)
    }

    /// Filter files to only include model files we need for Schnell
    fn filter_required_files(files: Vec<String>) -> Vec<String> {
        Self::filter_required_files_for_model(files, ModelType::schnell())
    }

    /// Filter files to only include model files we need for a specific model
    fn filter_required_files_for_model(files: Vec<String>, model_type: ModelType) -> Vec<String> {
        files
            .into_iter()
            .filter(|path| {
                // Include files from specific directories (shared between models)
                let include_dirs = [
                    "text_encoder/",      // CLIP text encoder
                    "text_encoder_2/",    // T5 text encoder
                    "tokenizer/",
                    "tokenizer_2/",
                    "scheduler/",
                ];

                // Root-level model files vary by model type
                let transformer_file = match model_type.id() {
                    "schnell" => "flux1-schnell.safetensors",
                    "dev" => "flux1-dev.safetensors",
                    "zimage-turbo" => "transformer/", // Z-Image uses directory with sharded files
                    _ => "flux1-schnell.safetensors", // Fallback
                };

                let root_files: Vec<&str> = vec![
                    "ae.safetensors",     // VAE (shared)
                    transformer_file,      // Transformer (model-specific)
                    "model_index.json",
                ];

                let in_target_dir = include_dirs.iter().any(|dir| path.starts_with(dir));
                let is_root_file = root_files.iter().any(|f| path == *f);

                // Exclude certain file types we don't need
                let is_unwanted = path.ends_with(".md")
                    || path.ends_with(".txt")
                    || path.ends_with(".gitattributes")
                    || path.contains("/.git/");

                (in_target_dir || is_root_file) && !is_unwanted
            })
            .collect()
    }

    /// Load HuggingFace token from settings, .env file, or environment variable
    fn load_hf_token() -> Result<String> {
        // First, try to load from app settings
        if let Ok(settings) = crate::settings::AppSettings::load() {
            if let Some(token) = settings.get_hf_token() {
                if !token.is_empty() {
                    return Ok(token.to_string());
                }
            }
        }

        // Try to load from .env file in project root
        if let Ok(cwd) = std::env::current_dir() {
            let env_path = cwd.join(".env");
            if env_path.exists() {
                let _ = dotenvy::from_path(&env_path);
            }
        }

        // Also try parent directory (for tauri dev mode)
        if let Ok(cwd) = std::env::current_dir() {
            if let Some(parent) = cwd.parent() {
                let env_path = parent.join(".env");
                if env_path.exists() {
                    let _ = dotenvy::from_path(&env_path);
                }
            }
        }

        // Try HF_API_KEY first (from .env), then HF_TOKEN (standard env var)
        std::env::var("HF_API_KEY")
            .or_else(|_| std::env::var("HF_TOKEN"))
            .context("HuggingFace token not found. Please set it in Settings or as HF_TOKEN environment variable")
    }

    /// Download FLUX Schnell model from HuggingFace Hub
    ///
    /// Downloads to ~/.cache/huggingface/hub/models--black-forest-labs--FLUX.1-schnell/
    /// Model files: ~24GB total (dynamically fetched from HuggingFace API)
    pub async fn download_schnell(&self) -> Result<()> {
        if self.is_schnell_downloaded() {
            info!("FLUX Schnell already downloaded");
            return Ok(());
        }

        // Load HuggingFace token for gated model access
        let hf_token = Self::load_hf_token()?;
        info!("Loaded HuggingFace token");

        // Set HF_TOKEN env var so hf-hub uses it
        std::env::set_var("HF_TOKEN", &hf_token);

        let api = Api::new()?;

        // Download main FLUX Schnell model
        let repo_id = "black-forest-labs/FLUX.1-schnell";
        info!("Downloading FLUX Schnell from HuggingFace Hub");
        info!("Fetching file list from repository");

        let all_files = Self::fetch_repo_files(repo_id, &hf_token).await?;
        let files = Self::filter_required_files(all_files);

        info!(file_count = files.len(), "Found files to download from FLUX.1-schnell (~24GB)");

        let repo = api.model(repo_id.to_string());
        for (idx, file) in files.iter().enumerate() {
            info!(progress = format!("{}/{}", idx + 1, files.len()), file = %file, "Downloading");
            repo.get(file).await
                .with_context(|| format!("Failed to download {}", file))?;
        }

        // Download T5 tokenizer from lmz/mt5-tokenizers (required for text encoding)
        info!("Downloading T5 tokenizer");
        let t5_tokenizer_repo = api.model("lmz/mt5-tokenizers".to_string());
        t5_tokenizer_repo.get("t5-v1_1-xxl.tokenizer.json").await
            .context("Failed to download T5 tokenizer")?;

        info!("FLUX Schnell download complete");
        Ok(())
    }

    /// Download FLUX Dev model from HuggingFace Hub
    ///
    /// FLUX Dev is a higher quality model that requires more steps (28+)
    /// Downloads to ~/.cache/huggingface/hub/models--black-forest-labs--FLUX.1-dev/
    /// Model files: ~24GB total
    ///
    /// Note: If Schnell is already downloaded, this will only download
    /// the Dev-specific transformer file, saving ~11GB of bandwidth.
    pub async fn download_dev(&self) -> Result<()> {
        if self.is_dev_downloaded() {
            info!("FLUX Dev already downloaded");
            return Ok(());
        }

        // Load HuggingFace token for gated model access
        let hf_token = Self::load_hf_token()?;
        info!("Loaded HuggingFace token");

        // Set HF_TOKEN env var so hf-hub uses it
        std::env::set_var("HF_TOKEN", &hf_token);

        let api = Api::new()?;
        let repo_id = "black-forest-labs/FLUX.1-dev";

        info!("Downloading FLUX Dev from HuggingFace Hub");
        info!("Fetching file list from repository");

        let all_files = Self::fetch_repo_files(repo_id, &hf_token).await?;
        let mut files = Self::filter_required_files_for_model(all_files, ModelType::dev());

        // Optimization: If Schnell is already downloaded, skip shared components
        // Only download the Dev-specific transformer file
        if self.is_schnell_downloaded() {
            info!("FLUX Schnell already downloaded - reusing shared components");
            files.retain(|f| f == "flux1-dev.safetensors");
            info!("Only downloading Dev transformer (~12GB)");
        } else {
            info!(file_count = files.len(), "Found files to download from FLUX.1-dev (~24GB)");
        }

        let repo = api.model(repo_id.to_string());
        for (idx, file) in files.iter().enumerate() {
            info!(progress = format!("{}/{}", idx + 1, files.len()), file = %file, "Downloading");
            repo.get(file).await
                .with_context(|| format!("Failed to download {}", file))?;
        }

        // Download T5 tokenizer if not already present (shared component)
        if !self.is_schnell_downloaded() {
            info!("Downloading T5 tokenizer");
            let t5_tokenizer_repo = api.model("lmz/mt5-tokenizers".to_string());
            t5_tokenizer_repo.get("t5-v1_1-xxl.tokenizer.json").await
                .context("Failed to download T5 tokenizer")?;
        }

        info!("FLUX Dev download complete");
        Ok(())
    }

    /// Download a specific model
    pub async fn download_model(&self, model_type: ModelType) -> Result<()> {
        match model_type.id() {
            "schnell" => self.download_schnell().await,
            "dev" => self.download_dev().await,
            "zimage-turbo" => {
                // TODO: Implement Z-Image-Turbo download
                // For now, model should be manually downloaded to HF cache
                anyhow::bail!("Z-Image-Turbo download not yet implemented. Please download manually using: huggingface-cli download Tongyi-MAI/Z-Image-Turbo")
            }
            _ => {
                anyhow::bail!("Unknown model type: {}", model_type.id())
            }
        }
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
