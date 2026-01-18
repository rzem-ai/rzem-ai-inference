//! Claude API client for image analysis
//!
//! Uses Claude's vision capabilities to analyze images and generate prompts.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const CLAUDE_MODEL: &str = "claude-sonnet-4-20250514";

/// Request body for Claude API
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

/// Response from Claude API
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ResponseContent>,
}

#[derive(Debug, Deserialize)]
struct ResponseContent {
    text: Option<String>,
}

/// Claude API error response
#[derive(Debug, Deserialize)]
struct ClaudeError {
    error: ClaudeErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ClaudeErrorDetail {
    message: String,
}

/// Analyzes an image and generates a prompt to recreate it.
///
/// # Arguments
/// * `api_key` - Claude API key
/// * `image_data` - Base64-encoded image data (without data URL prefix)
/// * `media_type` - MIME type (e.g., "image/png", "image/jpeg")
///
/// # Returns
/// A detailed prompt string that can be used to recreate the image.
pub async fn analyze_image_for_prompt(
    api_key: &str,
    image_data: &str,
    media_type: &str,
) -> Result<String> {
    let client = reqwest::Client::new();

    let analysis_prompt = r#"Analyze this image in detail and create a comprehensive text prompt that could be used to recreate it with an AI image generator like FLUX.

Your prompt should:
1. Start with the main subject and its key characteristics
2. Describe the setting, background, and environment
3. Include lighting details (direction, color, intensity, style)
4. Specify the artistic style, medium, or aesthetic (e.g., photorealistic, cinematic, anime, oil painting)
5. Mention camera perspective and composition if relevant
6. Include any notable colors, textures, or materials
7. Add atmospheric or mood descriptors

Write the prompt as a single, detailed paragraph in natural language. Focus on visual elements that would help recreate the image accurately. Do not include any preamble or explanation - output ONLY the prompt text itself."#;

    let request = ClaudeRequest {
        model: CLAUDE_MODEL.to_string(),
        max_tokens: 1024,
        messages: vec![ClaudeMessage {
            role: "user".to_string(),
            content: vec![
                ClaudeContent::Image {
                    source: ImageSource {
                        source_type: "base64".to_string(),
                        media_type: media_type.to_string(),
                        data: image_data.to_string(),
                    },
                },
                ClaudeContent::Text {
                    text: analysis_prompt.to_string(),
                },
            ],
        }],
    };

    let response = client
        .post(CLAUDE_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
        .context("Failed to send request to Claude API")?;

    let status = response.status();
    let body = response.text().await.context("Failed to read response body")?;

    if !status.is_success() {
        // Try to parse error message
        if let Ok(error) = serde_json::from_str::<ClaudeError>(&body) {
            anyhow::bail!("Claude API error: {}", error.error.message);
        }
        anyhow::bail!("Claude API returned status {}: {}", status, body);
    }

    let claude_response: ClaudeResponse =
        serde_json::from_str(&body).context("Failed to parse Claude response")?;

    // Extract the text from the response
    let prompt = claude_response
        .content
        .into_iter()
        .filter_map(|c| c.text)
        .collect::<Vec<_>>()
        .join("");

    if prompt.is_empty() {
        anyhow::bail!("Claude returned empty response");
    }

    Ok(prompt.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = ClaudeRequest {
            model: "test".to_string(),
            max_tokens: 100,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: vec![ClaudeContent::Text {
                    text: "Hello".to_string(),
                }],
            }],
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"test\""));
    }
}
