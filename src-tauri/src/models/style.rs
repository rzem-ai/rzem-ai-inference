//! Style management types and template rendering
//!
//! Styles are reusable generation templates combining:
//! - Prompt templates with {{prompt}} placeholders
//! - One or more LoRAs with configured strengths
//! - Example prompts and generated images
//! - Metadata (category, favorites, usage tracking)

use serde::{Deserialize, Serialize};

/// Render template with user prompt
///
/// Replaces all {{prompt}} placeholders with the provided user_prompt.
/// This is intentionally simple - no complex template logic needed.
///
/// # Example
/// ```
/// let template = "cinematic, {{prompt}}, highly detailed";
/// let rendered = render_template(template, "sunset over mountains");
/// assert_eq!(rendered, "cinematic, sunset over mountains, highly detailed");
/// ```
pub fn render_template(template: &str, user_prompt: &str) -> String {
    template.replace("{{prompt}}", user_prompt)
}

/// Style metadata for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub prompt_template: String,
    pub default_strength: f32,
    pub strength_min: f32,
    pub strength_max: f32,
    pub category: Option<String>,
    pub thumbnail_path: Option<String>,
    pub is_favorite: bool,
    pub usage_count: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

/// LoRA within a style with its configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleLoraWithInfo {
    pub lora_id: String,
    pub lora_name: String,
    pub lora_trigger_words: Option<String>,
    pub strength: f32,
    pub priority: i32,
}

/// Example prompt or image reference for a style
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleExample {
    pub id: String,
    pub style_id: String,
    pub example_type: String,  // "prompt" or "image"
    pub content: String,       // prompt text or image_id
    pub generation_params: Option<String>,  // JSON string
    pub created_at: i64,
}

/// Complete style with all associated data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleDetail {
    #[serde(flatten)]
    pub info: StyleInfo,
    pub loras: Vec<StyleLoraWithInfo>,
    pub examples: Vec<StyleExample>,
}

/// Request to create or update a style
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleRequest {
    pub name: String,
    pub description: Option<String>,
    pub prompt_template: String,
    pub default_strength: f32,
    pub strength_min: f32,
    pub strength_max: f32,
    pub category: Option<String>,
    pub is_favorite: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_template_basic() {
        let template = "{{prompt}}";
        assert_eq!(render_template(template, "hello"), "hello");
    }

    #[test]
    fn test_render_template_with_context() {
        let template = "cinematic, {{prompt}}, 4k";
        assert_eq!(
            render_template(template, "sunset"),
            "cinematic, sunset, 4k"
        );
    }

    #[test]
    fn test_render_template_multiple_placeholders() {
        let template = "{{prompt}} and {{prompt}}";
        assert_eq!(
            render_template(template, "cat"),
            "cat and cat"
        );
    }

    #[test]
    fn test_render_template_no_placeholder() {
        let template = "fixed prompt";
        assert_eq!(render_template(template, "ignored"), "fixed prompt");
    }

    #[test]
    fn test_render_template_empty_prompt() {
        let template = "prefix, {{prompt}}, suffix";
        assert_eq!(render_template(template, ""), "prefix, , suffix");
    }
}
