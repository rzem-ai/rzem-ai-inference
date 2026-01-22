//! Moondream vision model integration for local image tagging
//!
//! NOTE: This is currently a stub implementation. The full Moondream integration
//! requires additional work on the candle model API. For now, use the Claude
//! backend for auto-tagging.

use anyhow::Result;
use candle_core::Device;
use std::path::Path;
use tracing::{debug, info};

use super::taxonomy::TagWithConfidence;

/// Moondream model configuration
const MOONDREAM_REPO: &str = "vikhyatk/moondream2";

/// Moondream-based image tagger for local inference
///
/// NOTE: This is currently a stub. Full implementation pending.
pub struct MoondreamTagger {
    device: Device,
}

impl MoondreamTagger {
    /// Create a new MoondreamTagger
    ///
    /// NOTE: This currently returns an error as the full implementation is pending.
    pub fn new(device: Device) -> Result<Self> {
        info!("Initializing Moondream tagger (stub implementation)");

        // Check if model files exist
        if !is_moondream_available() {
            anyhow::bail!("Moondream model not downloaded. Please download it first.");
        }

        Ok(Self { device })
    }

    /// Extract tags from an image
    ///
    /// NOTE: This is a stub that returns a not-implemented error.
    /// Use ClaudeTagger for actual tagging functionality.
    pub fn extract_tags(&mut self, image_path: &Path) -> Result<Vec<TagWithConfidence>> {
        debug!(path = %image_path.display(), "Extract tags called (stub)");

        // Return error indicating this is not yet implemented
        anyhow::bail!(
            "Local Moondream tagging is not yet fully implemented. \
             Please use Claude backend for auto-tagging, or wait for a future update. \
             The model files are downloaded and ready for when this feature is complete."
        )
    }

    /// Extract tags from multiple images (batch processing)
    pub fn extract_tags_batch(&mut self, paths: &[&Path]) -> Result<Vec<Vec<TagWithConfidence>>> {
        info!(count = paths.len(), "Batch extract tags called (stub)");

        // Return error for each path
        paths.iter().map(|p| self.extract_tags(p)).collect()
    }
}

/// Check if Moondream model is available in cache
pub fn is_moondream_available() -> bool {
    let api = match hf_hub::api::sync::Api::new() {
        Ok(api) => api,
        Err(_) => return false,
    };

    let repo = api.model(MOONDREAM_REPO.to_string());

    // Check if model file exists in cache
    repo.get("model.safetensors").is_ok()
}

/// Get the Moondream model repository ID
pub fn moondream_repo_id() -> &'static str {
    MOONDREAM_REPO
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moondream_repo_id() {
        assert_eq!(moondream_repo_id(), "vikhyatk/moondream2");
    }
}
