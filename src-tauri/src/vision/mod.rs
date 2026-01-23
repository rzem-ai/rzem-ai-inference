//! Vision module for image analysis and auto-tagging
//!
//! Provides both local (Moondream) and cloud (Claude) backends for
//! extracting structured tags from generated images.

pub mod downloader;
pub mod models;
pub mod tagger;
pub mod taxonomy;

// Re-exports for convenience
pub use downloader::{
    download_moondream, ensure_moondream_weights, force_cleanup_locks, get_model_status,
    is_moondream_downloaded, VisionModelStatus, MOONDREAM_SIZE_BYTES,
};
pub use models::{is_moondream_available, moondream_repo_id, MoondreamTagger};
pub use tagger::{ClaudeTagger, TaggerBackend};
pub use taxonomy::{get_tagging_prompt, TagCategory, TagWithConfidence};

use serde::{Deserialize, Serialize};

/// Backend preference for auto-tagging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaggingBackend {
    /// Use local Moondream model (no API costs, works offline)
    #[default]
    Local,
    /// Use Claude Vision API (higher quality, requires API key)
    Claude,
}

impl std::fmt::Display for TaggingBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaggingBackend::Local => write!(f, "local"),
            TaggingBackend::Claude => write!(f, "claude"),
        }
    }
}

impl std::str::FromStr for TaggingBackend {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "local" | "moondream" => Ok(TaggingBackend::Local),
            "claude" | "api" => Ok(TaggingBackend::Claude),
            _ => Err(format!("Unknown tagging backend: {}", s)),
        }
    }
}

/// Settings for auto-tagging feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTagSettings {
    /// Whether auto-tagging is enabled
    pub enabled: bool,

    /// Whether to automatically tag images after generation
    pub auto_tag_on_generation: bool,

    /// Preferred backend for tagging
    pub preferred_backend: TaggingBackend,

    /// Minimum confidence threshold for tags (0.0 - 1.0)
    pub min_confidence: f32,

    /// Claude API key (stored encrypted in production)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude_api_key: Option<String>,
}

impl Default for AutoTagSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_tag_on_generation: false,
            preferred_backend: TaggingBackend::Local,
            min_confidence: 0.6,
            claude_api_key: None,
        }
    }
}

impl AutoTagSettings {
    /// Load settings from the gallery database
    pub fn load(db: &crate::gallery::GalleryDb) -> Self {
        db.get_auto_tag_settings().unwrap_or_default()
    }

    /// Save settings to the gallery database
    pub fn save(&self, db: &crate::gallery::GalleryDb) -> anyhow::Result<()> {
        db.set_auto_tag_settings(self)
    }

    /// Check if Claude backend is available (has API key)
    pub fn is_claude_available(&self) -> bool {
        self.claude_api_key
            .as_ref()
            .map(|k| !k.is_empty())
            .unwrap_or(false)
    }

    /// Check if local backend is available (model downloaded)
    pub fn is_local_available(&self) -> bool {
        is_moondream_downloaded()
    }

    /// Get the effective backend to use (considering availability)
    pub fn effective_backend(&self) -> Option<TaggingBackend> {
        match self.preferred_backend {
            TaggingBackend::Local => {
                if self.is_local_available() {
                    Some(TaggingBackend::Local)
                } else if self.is_claude_available() {
                    Some(TaggingBackend::Claude)
                } else {
                    None
                }
            }
            TaggingBackend::Claude => {
                if self.is_claude_available() {
                    Some(TaggingBackend::Claude)
                } else if self.is_local_available() {
                    Some(TaggingBackend::Local)
                } else {
                    None
                }
            }
        }
    }
}

/// Result of an auto-tagging operation
#[derive(Debug, Clone, Serialize)]
pub struct TaggingResult {
    /// Image ID that was tagged
    pub image_id: String,
    /// Tags extracted from the image
    pub tags: Vec<TagWithConfidence>,
    /// Backend that was used
    pub backend: TaggingBackend,
    /// Whether tagging was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tagging_backend_parse() {
        assert_eq!(
            "local".parse::<TaggingBackend>().unwrap(),
            TaggingBackend::Local
        );
        assert_eq!(
            "claude".parse::<TaggingBackend>().unwrap(),
            TaggingBackend::Claude
        );
        assert!("unknown".parse::<TaggingBackend>().is_err());
    }

    #[test]
    fn test_default_settings() {
        let settings = AutoTagSettings::default();
        assert!(!settings.enabled);
        assert!(!settings.auto_tag_on_generation);
        assert_eq!(settings.preferred_backend, TaggingBackend::Local);
        assert!(settings.min_confidence > 0.5);
    }
}
