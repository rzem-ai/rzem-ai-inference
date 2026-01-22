//! Tag taxonomy definitions for auto-tagging images
//!
//! Provides structured categories and vocabulary for consistent tagging
//! across both local (Moondream) and cloud (Claude) vision models.

use serde::{Deserialize, Serialize};

/// Categories for organizing tags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TagCategory {
    /// Main subjects: person, animal, landscape, object, abstract
    Subject,
    /// Art style: photorealistic, anime, oil-painting, digital-art, sketch
    Style,
    /// Emotional tone: serene, dramatic, cheerful, dark, mysterious
    Mood,
    /// Light characteristics: natural, studio, golden-hour, neon, low-key
    Lighting,
    /// Color palette: vibrant, muted, monochrome, warm, cool
    Color,
    /// Framing: portrait, wide-shot, close-up, symmetrical, dynamic
    Composition,
    /// Environment: indoor, outdoor, urban, nature, fantasy
    Setting,
    /// Technical quality: detailed, minimalist, textured, smooth, stylized
    Quality,
}

impl TagCategory {
    /// Get all categories as a slice
    pub fn all() -> &'static [TagCategory] {
        &[
            TagCategory::Subject,
            TagCategory::Style,
            TagCategory::Mood,
            TagCategory::Lighting,
            TagCategory::Color,
            TagCategory::Composition,
            TagCategory::Setting,
            TagCategory::Quality,
        ]
    }

    /// Get display name for the category
    pub fn display_name(&self) -> &'static str {
        match self {
            TagCategory::Subject => "Subject",
            TagCategory::Style => "Style",
            TagCategory::Mood => "Mood",
            TagCategory::Lighting => "Lighting",
            TagCategory::Color => "Color",
            TagCategory::Composition => "Composition",
            TagCategory::Setting => "Setting",
            TagCategory::Quality => "Quality",
        }
    }

    /// Get the vocabulary (valid tags) for this category
    pub fn vocabulary(&self) -> &'static [&'static str] {
        match self {
            TagCategory::Subject => &[
                "person", "people", "portrait", "face", "animal", "wildlife", "pet",
                "landscape", "nature", "building", "architecture", "vehicle", "food",
                "object", "still-life", "abstract", "pattern", "text", "logo",
                "fantasy-creature", "robot", "character",
            ],
            TagCategory::Style => &[
                "photorealistic", "photograph", "realistic", "anime", "manga",
                "cartoon", "illustration", "digital-art", "3d-render", "oil-painting",
                "watercolor", "sketch", "pencil", "ink", "pixel-art", "vector",
                "concept-art", "surreal", "impressionist", "pop-art",
            ],
            TagCategory::Mood => &[
                "serene", "peaceful", "calm", "dramatic", "intense", "epic",
                "cheerful", "happy", "joyful", "dark", "moody", "somber",
                "mysterious", "ethereal", "nostalgic", "romantic", "melancholic",
                "energetic", "whimsical", "eerie",
            ],
            TagCategory::Lighting => &[
                "natural-light", "daylight", "sunlight", "studio-lighting",
                "golden-hour", "sunset", "sunrise", "blue-hour", "neon",
                "low-key", "high-key", "dramatic-lighting", "backlit", "silhouette",
                "soft-light", "harsh-light", "rim-light", "ambient", "candlelight",
            ],
            TagCategory::Color => &[
                "vibrant", "saturated", "colorful", "muted", "desaturated", "pastel",
                "monochrome", "black-and-white", "sepia", "warm-tones", "cool-tones",
                "earthy", "neon-colors", "gradient", "duotone", "high-contrast",
                "low-contrast", "complementary", "analogous",
            ],
            TagCategory::Composition => &[
                "portrait", "headshot", "full-body", "wide-shot", "panoramic",
                "close-up", "macro", "detail", "symmetrical", "asymmetrical",
                "centered", "rule-of-thirds", "dynamic", "diagonal", "leading-lines",
                "frame-within-frame", "negative-space", "minimalist-composition",
                "busy", "layered",
            ],
            TagCategory::Setting => &[
                "indoor", "interior", "room", "outdoor", "exterior", "urban",
                "city", "street", "rural", "nature", "forest", "mountain",
                "ocean", "beach", "desert", "space", "underwater", "fantasy-world",
                "sci-fi", "historical", "futuristic",
            ],
            TagCategory::Quality => &[
                "highly-detailed", "intricate", "fine-detail", "minimalist", "simple",
                "textured", "smooth", "sharp", "soft-focus", "bokeh", "stylized",
                "realistic", "hyper-realistic", "cinematic", "professional",
                "amateur", "artistic", "raw", "polished", "rough",
            ],
        }
    }
}

/// A tag with its category and confidence score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagWithConfidence {
    /// The tag string (from vocabulary)
    pub tag: String,
    /// The category this tag belongs to
    pub category: TagCategory,
    /// Confidence score from 0.0 to 1.0
    pub confidence: f32,
}

impl TagWithConfidence {
    /// Create a new tag with confidence
    pub fn new(tag: impl Into<String>, category: TagCategory, confidence: f32) -> Self {
        Self {
            tag: tag.into(),
            category,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }

    /// Check if this tag is above a confidence threshold
    pub fn is_confident(&self, threshold: f32) -> bool {
        self.confidence >= threshold
    }
}

/// Get the structured prompt for extracting tags from an image
///
/// This prompt is designed to work with both Moondream VQA and Claude Vision
pub fn get_tagging_prompt() -> &'static str {
    r#"Analyze this image and provide tags in the following categories. For each category, list the most relevant tags (1-3 tags per category) with confidence scores from 0 to 1.

Categories:
1. Subject: What is the main subject? (person, animal, landscape, object, abstract, etc.)
2. Style: What artistic style is this? (photorealistic, anime, digital-art, oil-painting, sketch, etc.)
3. Mood: What is the emotional tone? (serene, dramatic, cheerful, dark, mysterious, etc.)
4. Lighting: What type of lighting? (natural, studio, golden-hour, neon, low-key, etc.)
5. Color: What is the color palette? (vibrant, muted, monochrome, warm, cool, etc.)
6. Composition: How is it composed? (portrait, wide-shot, close-up, symmetrical, dynamic, etc.)
7. Setting: What is the environment? (indoor, outdoor, urban, nature, fantasy, etc.)
8. Quality: What are the quality characteristics? (detailed, minimalist, textured, stylized, etc.)

Respond in JSON format:
{
  "subject": [{"tag": "...", "confidence": 0.9}],
  "style": [{"tag": "...", "confidence": 0.8}],
  "mood": [{"tag": "...", "confidence": 0.7}],
  "lighting": [{"tag": "...", "confidence": 0.8}],
  "color": [{"tag": "...", "confidence": 0.9}],
  "composition": [{"tag": "...", "confidence": 0.8}],
  "setting": [{"tag": "...", "confidence": 0.7}],
  "quality": [{"tag": "...", "confidence": 0.8}]
}"#
}

/// Get a simpler prompt for Moondream (which works better with direct questions)
pub fn get_moondream_prompts() -> &'static [(&'static str, TagCategory)] {
    &[
        ("What is the main subject of this image? Answer with one or two words.", TagCategory::Subject),
        ("What artistic style is this image? Answer with one or two words like: photorealistic, anime, digital-art, painting, sketch.", TagCategory::Style),
        ("What is the mood or emotional tone of this image? Answer with one word like: serene, dramatic, cheerful, dark, mysterious.", TagCategory::Mood),
        ("What type of lighting is in this image? Answer with one or two words like: natural, studio, golden-hour, neon, dramatic.", TagCategory::Lighting),
        ("Describe the color palette in one or two words: vibrant, muted, monochrome, warm, cool, colorful.", TagCategory::Color),
        ("What type of composition or framing? Answer: portrait, landscape, close-up, wide-shot, or similar.", TagCategory::Composition),
        ("What is the setting or environment? Answer: indoor, outdoor, urban, nature, fantasy, or similar.", TagCategory::Setting),
        ("Describe the image quality in one or two words: detailed, minimalist, textured, stylized, realistic.", TagCategory::Quality),
    ]
}

/// Parse a simple text response into a tag
///
/// Used for Moondream responses which are simpler than JSON
pub fn parse_simple_response(response: &str, category: TagCategory) -> Option<TagWithConfidence> {
    let response = response.trim().to_lowercase();

    // Skip empty or unhelpful responses
    if response.is_empty() || response.contains("i cannot") || response.contains("i can't") {
        return None;
    }

    // Try to match against vocabulary
    let vocabulary = category.vocabulary();

    // First, try exact match
    for tag in vocabulary {
        if response == *tag || response.contains(tag) {
            return Some(TagWithConfidence::new(*tag, category, 0.8));
        }
    }

    // If no vocabulary match, use the first few words as a custom tag
    let words: Vec<&str> = response.split_whitespace().take(2).collect();
    if !words.is_empty() {
        let tag = words.join("-");
        // Lower confidence for non-vocabulary tags
        return Some(TagWithConfidence::new(tag, category, 0.6));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_category_vocabulary() {
        for category in TagCategory::all() {
            let vocab = category.vocabulary();
            assert!(!vocab.is_empty(), "{:?} should have vocabulary", category);
        }
    }

    #[test]
    fn test_tag_with_confidence() {
        let tag = TagWithConfidence::new("portrait", TagCategory::Subject, 0.85);
        assert_eq!(tag.tag, "portrait");
        assert_eq!(tag.category, TagCategory::Subject);
        assert!(tag.is_confident(0.8));
        assert!(!tag.is_confident(0.9));
    }

    #[test]
    fn test_parse_simple_response() {
        let result = parse_simple_response("portrait", TagCategory::Subject);
        assert!(result.is_some());
        assert_eq!(result.unwrap().tag, "portrait");

        let result = parse_simple_response("A beautiful landscape", TagCategory::Subject);
        assert!(result.is_some());
        assert_eq!(result.unwrap().tag, "landscape");
    }
}
