//! Unified image tagger with support for local (Moondream) and cloud (Claude) backends
//!
//! Provides a consistent interface for extracting tags from images using
//! either local inference or cloud API.

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use candle_core::Device;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::time::{sleep, Duration};
use tracing::{debug, info, warn};

use super::models::MoondreamTagger;
use super::taxonomy::{get_tagging_prompt, TagCategory, TagWithConfidence};

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const CLAUDE_MODEL: &str = "claude-sonnet-4-20250514";

/// Claude API request structures (reused from claude module pattern)
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: Vec<ClaudeContent>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ClaudeContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ImageSource },
}

#[derive(Debug, Serialize)]
struct ImageSource {
    #[serde(rename = "type")]
    source_type: String,
    media_type: String,
    data: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ResponseContent>,
}

#[derive(Debug, Deserialize)]
struct ResponseContent {
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClaudeError {
    error: ClaudeErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ClaudeErrorDetail {
    message: String,
}

/// Claude Vision tagger using API calls
pub struct ClaudeTagger {
    api_key: String,
    client: reqwest::Client,
}

impl ClaudeTagger {
    /// Create a new Claude tagger with an API key
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Load image from disk and encode as base64
    fn load_image_base64(path: &Path) -> Result<(String, String)> {
        let data = std::fs::read(path)
            .with_context(|| format!("Failed to read image: {}", path.display()))?;

        let base64_data = BASE64.encode(&data);

        // Determine media type from extension
        let media_type = match path.extension().and_then(|e| e.to_str()) {
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("webp") => "image/webp",
            _ => "image/png", // default
        };

        Ok((base64_data, media_type.to_string()))
    }

    /// Extract tags from an image using Claude Vision API
    pub async fn extract_tags(&self, image_path: &Path) -> Result<Vec<TagWithConfidence>> {
        debug!(path = %image_path.display(), "Extracting tags with Claude Vision");

        let (image_data, media_type) = Self::load_image_base64(image_path)?;

        let request = ClaudeRequest {
            model: CLAUDE_MODEL.to_string(),
            max_tokens: 1024,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: vec![
                    ClaudeContent::Image {
                        source: ImageSource {
                            source_type: "base64".to_string(),
                            media_type,
                            data: image_data,
                        },
                    },
                    ClaudeContent::Text {
                        text: get_tagging_prompt().to_string(),
                    },
                ],
            }],
        };

        let response = self.client
            .post(CLAUDE_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Claude API")?;

        let status = response.status();
        let body = response.text().await.context("Failed to read response body")?;

        if !status.is_success() {
            if let Ok(error) = serde_json::from_str::<ClaudeError>(&body) {
                anyhow::bail!("Claude API error: {}", error.error.message);
            }
            anyhow::bail!("Claude API returned status {}: {}", status, body);
        }

        let claude_response: ClaudeResponse =
            serde_json::from_str(&body).context("Failed to parse Claude response")?;

        let response_text = claude_response
            .content
            .into_iter()
            .filter_map(|c| c.text)
            .collect::<Vec<_>>()
            .join("");

        // Parse JSON response into tags
        let tags = self.parse_tag_response(&response_text)?;

        info!(tag_count = tags.len(), "Extracted tags with Claude");
        Ok(tags)
    }

    /// Parse Claude's JSON response into TagWithConfidence
    fn parse_tag_response(&self, response: &str) -> Result<Vec<TagWithConfidence>> {
        // Try to extract JSON from response (Claude may include explanation text)
        let json_start = response.find('{').unwrap_or(0);
        let json_end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[json_start..json_end];

        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .with_context(|| format!("Failed to parse JSON: {}", json_str))?;

        let mut tags = Vec::new();

        // Parse each category
        for (category_name, category) in [
            ("subject", TagCategory::Subject),
            ("style", TagCategory::Style),
            ("mood", TagCategory::Mood),
            ("lighting", TagCategory::Lighting),
            ("color", TagCategory::Color),
            ("composition", TagCategory::Composition),
            ("setting", TagCategory::Setting),
            ("quality", TagCategory::Quality),
        ] {
            if let Some(category_tags) = parsed.get(category_name).and_then(|v| v.as_array()) {
                for item in category_tags {
                    if let (Some(tag), Some(confidence)) = (
                        item.get("tag").and_then(|t| t.as_str()),
                        item.get("confidence").and_then(|c| c.as_f64()),
                    ) {
                        tags.push(TagWithConfidence::new(tag, category, confidence as f32));
                    }
                }
            }
        }

        Ok(tags)
    }

    /// Extract tags from multiple images with rate limiting
    pub async fn extract_tags_batch(&self, paths: &[&Path]) -> Result<Vec<Vec<TagWithConfidence>>> {
        info!(count = paths.len(), "Batch extracting tags with Claude");

        let mut results = Vec::with_capacity(paths.len());

        for (idx, path) in paths.iter().enumerate() {
            // Rate limiting: ~1 request per second to be safe
            if idx > 0 {
                sleep(Duration::from_millis(1000)).await;
            }

            match self.extract_tags(path).await {
                Ok(tags) => results.push(tags),
                Err(e) => {
                    warn!(path = %path.display(), error = %e, "Failed to extract tags");
                    results.push(Vec::new());
                }
            }
        }

        Ok(results)
    }
}

/// Unified tagger backend enum
pub enum TaggerBackend {
    /// Local Moondream model
    Local(MoondreamTagger),
    /// Claude Vision API
    Claude(ClaudeTagger),
}

impl TaggerBackend {
    /// Create a local (Moondream) tagger
    pub fn local(device: Device) -> Result<Self> {
        Ok(Self::Local(MoondreamTagger::new(device)?))
    }

    /// Create a Claude API tagger
    pub fn claude(api_key: String) -> Self {
        Self::Claude(ClaudeTagger::new(api_key))
    }

    /// Extract tags from an image using the configured backend
    pub async fn tag_image(&mut self, path: &Path) -> Result<Vec<TagWithConfidence>> {
        match self {
            TaggerBackend::Local(_tagger) => {
                // Local backend requires sync context due to model architecture
                // Use tag_image_sync() instead, or wrap in spawn_blocking at call site
                Err(anyhow::anyhow!(
                    "Local tagger requires sync context. Use tag_image_sync() or \
                     wrap in tokio::task::spawn_blocking at the call site."
                ))
            }
            TaggerBackend::Claude(tagger) => {
                tagger.extract_tags(path).await
            }
        }
    }

    /// Extract tags from an image synchronously (for local backend)
    pub fn tag_image_sync(&mut self, path: &Path) -> Result<Vec<TagWithConfidence>> {
        match self {
            TaggerBackend::Local(tagger) => tagger.extract_tags(path),
            TaggerBackend::Claude(_) => {
                Err(anyhow::anyhow!("Claude tagger requires async context"))
            }
        }
    }

    /// Check if this is a local backend
    pub fn is_local(&self) -> bool {
        matches!(self, TaggerBackend::Local(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_tagger_creation() {
        let tagger = ClaudeTagger::new("test-key".to_string());
        assert!(!tagger.api_key.is_empty());
    }

    #[test]
    fn test_parse_tag_response() {
        let tagger = ClaudeTagger::new("test".to_string());
        let response = r#"{
            "subject": [{"tag": "portrait", "confidence": 0.9}],
            "style": [{"tag": "photorealistic", "confidence": 0.8}],
            "mood": [],
            "lighting": [{"tag": "natural-light", "confidence": 0.7}],
            "color": [],
            "composition": [],
            "setting": [],
            "quality": []
        }"#;

        let tags = tagger.parse_tag_response(response).unwrap();
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0].tag, "portrait");
        assert_eq!(tags[0].category, TagCategory::Subject);
    }
}
