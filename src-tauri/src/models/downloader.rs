//! Model downloading from HuggingFace Hub

use anyhow::Result;
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

    /// Download FLUX Schnell model (stub - will implement in Task 3)
    pub async fn download_schnell(&self) -> Result<()> {
        // TODO: Implement actual download in Task 3
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
