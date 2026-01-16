//! Model downloading from HuggingFace Hub

use anyhow::{Context, Result};
use hf_hub::api::tokio::Api;
use super::ModelPaths;
use serde::Deserialize;

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
        Ok(Self {
            paths: ModelPaths::new()?,
        })
    }

    /// Check if FLUX Schnell is already downloaded
    pub fn is_schnell_downloaded(&self) -> bool {
        self.paths.all_files_exist()
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
            .header("User-Agent", "flux-generator/0.1.0")
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

    /// Filter files to only include model files we need
    fn filter_required_files(files: Vec<String>) -> Vec<String> {
        files
            .into_iter()
            .filter(|path| {
                // Include files from specific directories
                let include_dirs = [
                    "text_encoder/",
                    "vae/",
                    "transformer/",
                    "tokenizer/",
                    "scheduler/",
                ];

                // Only include if in one of the target directories
                let in_target_dir = include_dirs.iter().any(|dir| path.starts_with(dir));

                // Exclude certain file types we don't need
                let is_unwanted = path.ends_with(".md")
                    || path.ends_with(".txt.md")
                    || path.ends_with(".gitattributes")
                    || path.contains("/.git/");

                in_target_dir && !is_unwanted
            })
            .collect()
    }

    /// Load HuggingFace token from .env file
    fn load_hf_token() -> Result<String> {
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
            .context("HuggingFace token not found. Please set HF_API_KEY in .env file or HF_TOKEN environment variable")
    }

    /// Download FLUX Schnell model from HuggingFace Hub
    ///
    /// Downloads to ~/.cache/huggingface/hub/models--black-forest-labs--FLUX.1-schnell/
    /// Model files: ~24GB total (dynamically fetched from HuggingFace API)
    pub async fn download_schnell(&self) -> Result<()> {
        if self.is_schnell_downloaded() {
            println!("FLUX Schnell already downloaded");
            return Ok(());
        }

        // Load HuggingFace token for gated model access
        let hf_token = Self::load_hf_token()?;
        println!("Loaded HuggingFace token from .env");

        // Set HF_TOKEN env var so hf-hub uses it
        std::env::set_var("HF_TOKEN", &hf_token);

        let repo_id = "black-forest-labs/FLUX.1-schnell";

        println!("Downloading FLUX Schnell from HuggingFace Hub...");
        println!("Fetching file list from repository...");

        // Fetch and filter file list dynamically from HuggingFace API
        let all_files = Self::fetch_repo_files(repo_id, &hf_token).await?;
        let files = Self::filter_required_files(all_files);

        println!("Found {} files to download", files.len());
        println!("This will download ~24GB of model files");
        println!("NOTE: If download is interrupted, delete ~/.cache/huggingface/hub/models--black-forest-labs--FLUX.1-schnell and restart");

        let api = Api::new()?;
        let repo = api.model(repo_id.to_string());

        // Download each file
        for (idx, file) in files.iter().enumerate() {
            println!("Downloading [{}/{}] {}", idx + 1, files.len(), file);
            repo.get(file).await
                .with_context(|| format!("Failed to download {}", file))?;
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
