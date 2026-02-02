//! Chatbot for prompt refinement using Claude API
//!
//! Provides conversational AI assistance to help users craft better
//! image generation prompts.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const CLAUDE_MODEL: &str = "claude-sonnet-4-20250514";

/// Chat message role
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

/// A single chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    #[serde(default)]
    pub timestamp: i64,
}

/// Context about the current generation settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenerationContext {
    pub current_prompt: String,
    pub width: u32,
    pub height: u32,
    pub model: String,
    #[serde(default)]
    pub style_loras: Vec<String>,
    #[serde(default)]
    pub steps: u32,
    #[serde(default)]
    pub cfg_scale: f32,
}

/// Request to refine a prompt
#[derive(Debug, Serialize, Deserialize)]
pub struct RefineRequest {
    pub user_message: String,
    #[serde(default)]
    pub history: Vec<ChatMessage>,
    #[serde(default)]
    pub context: GenerationContext,
}

/// Response from the chatbot
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: String,
    pub suggested_prompt: Option<String>,
}

/// Claude API request structure
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

/// Claude API response structure
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
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

/// System prompt for the chatbot
const SYSTEM_PROMPT: &str = r#"You are a helpful AI assistant specialized in crafting prompts for FLUX image generation. Your role is to help users refine and improve their image generation prompts through conversation.

## Your Capabilities
- Help users articulate their creative vision
- Suggest specific details: lighting, style, mood, composition, camera angles
- Recommend artistic styles that match their intent (photorealistic, anime, oil painting, digital art, etc.)
- Improve prompt structure and clarity
- Warn about common issues: conflicting styles, too many subjects, unclear focus

## Current Generation Settings
- Canvas size: {width}x{height}
- Model: {model}
- Steps: {steps}
- CFG Scale: {cfg_scale}
- Active style LoRAs: {loras}

## How to Help
1. If the user's prompt is vague, ask clarifying questions about their vision
2. Suggest specific improvements one at a time, explaining why each helps
3. Consider the aspect ratio when suggesting compositions
4. When you have a refined prompt ready, include it in a special format

## Prompt Format
When suggesting a refined prompt, format it like this:
```prompt
your refined prompt here
```

This allows the user to easily copy and apply it.

## Response Formatting
Your conversational response is rendered as markdown, so use it naturally:
- Use **bold** to highlight key terms or important points
- Use bullet or numbered lists when offering multiple suggestions
- Use *italic* for style names or subtle emphasis
- Everything outside the ```prompt block is rendered as markdown, so structure your response accordingly

## Guidelines
- Keep responses concise and actionable
- Focus on visual elements that FLUX can render well
- Be encouraging but honest about limitations
- Don't overwhelm with too many suggestions at once
- Remember previous context in the conversation"#;

/// Build the system prompt with generation context
fn build_system_prompt(context: &GenerationContext) -> String {
    let loras = if context.style_loras.is_empty() {
        "None".to_string()
    } else {
        context.style_loras.join(", ")
    };

    SYSTEM_PROMPT
        .replace("{width}", &context.width.to_string())
        .replace("{height}", &context.height.to_string())
        .replace("{model}", &context.model)
        .replace("{steps}", &context.steps.to_string())
        .replace("{cfg_scale}", &format!("{:.1}", context.cfg_scale))
        .replace("{loras}", &loras)
}

/// Extract suggested prompt from Claude's response
///
/// Looks for prompts in markdown code blocks with the "prompt" language tag
fn extract_suggested_prompt(text: &str) -> Option<String> {
    // Look for ```prompt ... ``` blocks
    let start_marker = "```prompt";
    let end_marker = "```";

    if let Some(start_idx) = text.find(start_marker) {
        let after_marker = start_idx + start_marker.len();
        let remaining = &text[after_marker..];

        // Skip any whitespace/newline after the marker
        let content_start = remaining
            .find(|c: char| !c.is_whitespace())
            .unwrap_or(0);

        if let Some(end_idx) = remaining[content_start..].find(end_marker) {
            let prompt = &remaining[content_start..content_start + end_idx];
            return Some(prompt.trim().to_string());
        }
    }

    None
}

/// Refine a prompt using Claude
pub async fn refine_prompt(api_key: &str, request: RefineRequest) -> Result<ChatResponse> {
    let client = reqwest::Client::new();

    // Build system prompt with context
    let system = build_system_prompt(&request.context);

    // Convert history to Claude format
    let mut messages: Vec<ClaudeMessage> = request
        .history
        .iter()
        .map(|msg| ClaudeMessage {
            role: match msg.role {
                MessageRole::User => "user".to_string(),
                MessageRole::Assistant => "assistant".to_string(),
            },
            content: msg.content.clone(),
        })
        .collect();

    // Build the current user message with context
    let user_content = if request.context.current_prompt.is_empty() {
        request.user_message.clone()
    } else {
        format!(
            "My current prompt is: \"{}\"\n\n{}",
            request.context.current_prompt, request.user_message
        )
    };

    messages.push(ClaudeMessage {
        role: "user".to_string(),
        content: user_content,
    });

    let claude_request = ClaudeRequest {
        model: CLAUDE_MODEL.to_string(),
        max_tokens: 1024,
        system,
        messages,
        stream: None,
    };

    debug!("Sending request to Claude API");

    let response = client
        .post(CLAUDE_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&claude_request)
        .send()
        .await
        .context("Failed to send request to Claude API")?;

    let status = response.status();
    let body = response
        .text()
        .await
        .context("Failed to read response body")?;

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
    let message = claude_response
        .content
        .into_iter()
        .filter_map(|c| c.text)
        .collect::<Vec<_>>()
        .join("");

    if message.is_empty() {
        anyhow::bail!("Claude returned empty response");
    }

    // Extract suggested prompt if present
    let suggested_prompt = extract_suggested_prompt(&message);

    info!(
        has_suggestion = suggested_prompt.is_some(),
        "Chat response received"
    );

    Ok(ChatResponse {
        message: message.trim().to_string(),
        suggested_prompt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_suggested_prompt_markdown() {
        let text = r#"Here's my suggestion:

```prompt
A majestic dragon flying over mountains at sunset, detailed scales, dramatic lighting
```

This prompt focuses on the key elements."#;

        let result = extract_suggested_prompt(text);
        assert_eq!(
            result,
            Some(
                "A majestic dragon flying over mountains at sunset, detailed scales, dramatic lighting"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_extract_suggested_prompt_none() {
        let text = "Here's some advice without a specific prompt suggestion.";
        let result = extract_suggested_prompt(text);
        assert!(result.is_none());
    }

    #[test]
    fn test_build_system_prompt() {
        let context = GenerationContext {
            current_prompt: "test".to_string(),
            width: 1024,
            height: 768,
            model: "flux-schnell".to_string(),
            style_loras: vec!["anime".to_string(), "cinematic".to_string()],
            steps: 4,
            cfg_scale: 3.5,
        };

        let prompt = build_system_prompt(&context);

        assert!(prompt.contains("1024x768"));
        assert!(prompt.contains("flux-schnell"));
        assert!(prompt.contains("anime, cinematic"));
    }
}
