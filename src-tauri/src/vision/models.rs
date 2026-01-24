//! Moondream vision model integration for local image tagging
//!
//! Uses the candle-transformers quantized_moondream module for local inference.
//! This uses the GGUF quantized model from santiagomed/candle-moondream.

use anyhow::{Context, Result};
use candle_core::{Device, Module, Tensor};
use candle_transformers::models::{moondream, quantized_moondream::Model as QModel};
use candle_transformers::quantized_var_builder::VarBuilder;
use std::path::Path;
use tokenizers::Tokenizer;
use tracing::{debug, info};

use super::taxonomy::TagWithConfidence;

/// Quantized Moondream model repo (compatible with candle)
const MOONDREAM_REPO: &str = "santiagomed/candle-moondream";
const IMAGE_SIZE: u32 = 378;

/// Moondream-based image tagger for local inference using quantized model
pub struct MoondreamTagger {
    model: QModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl MoondreamTagger {
    /// Create a new MoondreamTagger with quantized model
    pub fn new(device: Device) -> Result<Self> {
        info!("Initializing Moondream tagger (quantized)");

        let api = hf_hub::api::sync::Api::new().context("Failed to create HF API")?;
        let repo = api.model(MOONDREAM_REPO.to_string());

        // Load quantized model weights (GGUF format)
        let model_path = repo
            .get("model-q4_0.gguf")
            .context("Failed to get quantized model weights. Run download first.")?;

        // Load tokenizer
        let tokenizer_path = repo
            .get("tokenizer.json")
            .context("Failed to get tokenizer")?;

        info!(model_path = %model_path.display(), "Loading quantized Moondream model");

        let tokenizer =
            Tokenizer::from_file(&tokenizer_path).map_err(|e| anyhow::anyhow!("{}", e))?;

        // Load quantized model from GGUF using quantized_var_builder
        let vb = VarBuilder::from_gguf(&model_path, &device)
            .context("Failed to load GGUF weights")?;

        // Use Moondream v2 configuration
        let config = moondream::Config::v2();

        let model = QModel::new(&config, vb)
            .context("Failed to create quantized Moondream model")?;

        info!("Quantized Moondream model loaded successfully");

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    /// Load and preprocess an image for the vision encoder
    fn load_image(&self, path: &Path) -> Result<Tensor> {
        debug!(path = %path.display(), "Loading image for Moondream");

        let img = image::open(path)
            .with_context(|| format!("Failed to open image: {}", path.display()))?;

        // Resize to 378x378 as required by Moondream
        let img = img.resize_exact(
            IMAGE_SIZE,
            IMAGE_SIZE,
            image::imageops::FilterType::Triangle,
        );

        let img = img.to_rgb8();

        // Convert to tensor with shape (3, 378, 378)
        let data: Vec<f32> = img
            .pixels()
            .flat_map(|p| {
                let r = p[0] as f32 / 255.0;
                let g = p[1] as f32 / 255.0;
                let b = p[2] as f32 / 255.0;
                [r, g, b]
            })
            .collect();

        // Reshape to (3, H, W) - CHW format
        let tensor = Tensor::from_vec(
            data,
            (IMAGE_SIZE as usize, IMAGE_SIZE as usize, 3),
            &self.device,
        )?
        .permute((2, 0, 1))?;

        // Normalize: (x - 0.5) / 0.5
        let mean = Tensor::new(&[0.5f32], &self.device)?.broadcast_as(tensor.shape())?;
        let std = Tensor::new(&[0.5f32], &self.device)?.broadcast_as(tensor.shape())?;
        let tensor = ((&tensor - &mean)? / &std)?;

        // Add batch dimension
        let tensor = tensor.unsqueeze(0)?;

        debug!(shape = ?tensor.dims(), "Image tensor created");
        Ok(tensor)
    }

    /// Generate text from an image and prompt
    fn generate(&mut self, image: &Tensor, prompt: &str, max_tokens: usize) -> Result<String> {
        // Clear KV cache from any previous generation
        self.model.text_model().clear_kv_cache();

        // Encode the image using the vision encoder (Module trait)
        let image_embeds = image.apply(self.model.vision_encoder())?;
        debug!(image_embeds_shape = ?image_embeds.dims(), "Image embeddings computed");

        // Encode the prompt
        let prompt_with_format = format!("\n\nQuestion: {}\n\nAnswer:", prompt);
        let encoding = self
            .tokenizer
            .encode(prompt_with_format.as_str(), true)
            .map_err(|e| anyhow::anyhow!("Tokenizer error: {}", e))?;
        let prompt_tokens: Vec<u32> = encoding.get_ids().to_vec();

        // BOS and EOS tokens
        let bos_token = 1u32;
        let eos_token = self
            .tokenizer
            .token_to_id("<|endoftext|>")
            .unwrap_or(50256);

        let mut generated = Vec::new();

        for i in 0..max_tokens {
            let logits = if i == 0 {
                // First iteration: BOS token + prompt tokens + image embeddings
                let bos_tensor = Tensor::new(&[bos_token], &self.device)?.unsqueeze(0)?;
                let prompt_tensor = Tensor::new(&prompt_tokens[..], &self.device)?.unsqueeze(0)?;
                debug!(
                    bos_shape = ?bos_tensor.dims(),
                    prompt_shape = ?prompt_tensor.dims(),
                    img_shape = ?image_embeds.dims(),
                    "First forward pass tensors"
                );
                self.model.text_model().forward_with_img(&bos_tensor, &prompt_tensor, &image_embeds)?
            } else {
                // Subsequent iterations: just the last generated token
                let last_token = Tensor::new(&[*generated.last().unwrap()], &self.device)?.unsqueeze(0)?;
                self.model.text_model().forward(&last_token)?
            };

            // Greedy sampling (argmax) on the vocab dimension
            // Logits shape: (batch_size, vocab_size) -> squeeze to (vocab_size) -> argmax
            let logits = logits.squeeze(0)?.to_dtype(candle_core::DType::F32)?;
            let next_token = logits.argmax(0)?.to_scalar::<u32>()?;

            if next_token == eos_token {
                break;
            }

            // Check for <END> sequence
            if generated.len() >= 2
                && generated[generated.len() - 2] == 27
                && generated[generated.len() - 1] == 10619
                && next_token == 29
            {
                // Remove the partial <END> sequence
                generated.pop();
                generated.pop();
                break;
            }

            generated.push(next_token);
        }

        // Decode generated tokens
        let text = self
            .tokenizer
            .decode(&generated, true)
            .map_err(|e| anyhow::anyhow!("Decode error: {}", e))?;

        Ok(text.trim().to_string())
    }

    /// Extract tags from an image using structured prompts
    pub fn extract_tags(&mut self, image_path: &Path) -> Result<Vec<TagWithConfidence>> {
        debug!(path = %image_path.display(), "Extracting tags with Moondream");

        let image = self.load_image(image_path)?;

        // Use a structured prompt to get tags
        let prompt = "List keywords describing this image, separated by commas.";

        let response = self.generate(&image, prompt, 200)?;

        info!(response_len = response.len(), "Generated description");
        debug!(response = %response, "Moondream response");

        // Parse the response into tags
        let tags = self.parse_tags_from_response(&response);

        info!(tag_count = tags.len(), "Extracted tags");
        Ok(tags)
    }

    /// Parse comma-separated tags from Moondream's response
    fn parse_tags_from_response(&self, response: &str) -> Vec<TagWithConfidence> {
        let mut tags = Vec::new();

        // Split by commas and "and" conjunctions, then clean up each tag
        for tag_str in response.split(',') {
            // Further split on " and " to handle "blue, green, and yellow" -> ["blue", "green", "yellow"]
            for part in tag_str.split(" and ") {
                let tag = part
                    .trim()
                    .trim_start_matches("and ")
                    .trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != ' ')
                    .to_lowercase();

                if tag.is_empty() || tag.len() < 2 || tag.len() > 50 {
                    continue;
                }

                // Skip common filler words that aren't useful as tags
                if matches!(tag.as_str(), "and" | "the" | "a" | "an" | "with" | "of") {
                    continue;
                }

                // Categorize the tag based on keywords
                let category = categorize_tag(&tag);

                // Moondream doesn't provide confidence scores, so use 0.8 as default
                tags.push(TagWithConfidence::new(&tag, category, 0.8));
            }
        }

        // Deduplicate
        tags.sort_by(|a, b| a.tag.cmp(&b.tag));
        tags.dedup_by(|a, b| a.tag == b.tag);

        tags
    }

    /// Extract tags from multiple images (batch processing)
    pub fn extract_tags_batch(&mut self, paths: &[&Path]) -> Result<Vec<Vec<TagWithConfidence>>> {
        info!(count = paths.len(), "Batch extracting tags with Moondream");

        let mut results = Vec::with_capacity(paths.len());

        for path in paths {
            match self.extract_tags(path) {
                Ok(tags) => results.push(tags),
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "Failed to extract tags");
                    results.push(Vec::new());
                }
            }
        }

        Ok(results)
    }
}

/// Categorize a tag based on keywords
fn categorize_tag(tag: &str) -> super::taxonomy::TagCategory {
    use super::taxonomy::TagCategory;

    let tag_lower = tag.to_lowercase();

    // Style keywords
    if tag_lower.contains("style")
        || tag_lower.contains("realistic")
        || tag_lower.contains("abstract")
        || tag_lower.contains("anime")
        || tag_lower.contains("cartoon")
        || tag_lower.contains("painting")
        || tag_lower.contains("photograph")
        || tag_lower.contains("digital")
        || tag_lower.contains("3d")
        || tag_lower.contains("illustration")
    {
        return TagCategory::Style;
    }

    // Mood keywords
    if tag_lower.contains("mood")
        || tag_lower.contains("feeling")
        || tag_lower.contains("emotion")
        || tag_lower.contains("serene")
        || tag_lower.contains("dramatic")
        || tag_lower.contains("peaceful")
        || tag_lower.contains("dark")
        || tag_lower.contains("bright")
        || tag_lower.contains("mysterious")
        || tag_lower.contains("joyful")
    {
        return TagCategory::Mood;
    }

    // Lighting keywords
    if tag_lower.contains("light")
        || tag_lower.contains("shadow")
        || tag_lower.contains("sunset")
        || tag_lower.contains("sunrise")
        || tag_lower.contains("golden hour")
        || tag_lower.contains("backlit")
        || tag_lower.contains("ambient")
        || tag_lower.contains("studio")
    {
        return TagCategory::Lighting;
    }

    // Color keywords
    if tag_lower.contains("color")
        || tag_lower.contains("red")
        || tag_lower.contains("blue")
        || tag_lower.contains("green")
        || tag_lower.contains("yellow")
        || tag_lower.contains("orange")
        || tag_lower.contains("purple")
        || tag_lower.contains("pink")
        || tag_lower.contains("black")
        || tag_lower.contains("white")
        || tag_lower.contains("monochrome")
        || tag_lower.contains("vibrant")
        || tag_lower.contains("pastel")
    {
        return TagCategory::Color;
    }

    // Composition keywords
    if tag_lower.contains("composition")
        || tag_lower.contains("close-up")
        || tag_lower.contains("wide")
        || tag_lower.contains("portrait")
        || tag_lower.contains("landscape")
        || tag_lower.contains("symmetr")
        || tag_lower.contains("rule of thirds")
        || tag_lower.contains("centered")
    {
        return TagCategory::Composition;
    }

    // Setting keywords
    if tag_lower.contains("indoor")
        || tag_lower.contains("outdoor")
        || tag_lower.contains("nature")
        || tag_lower.contains("urban")
        || tag_lower.contains("forest")
        || tag_lower.contains("beach")
        || tag_lower.contains("mountain")
        || tag_lower.contains("city")
        || tag_lower.contains("room")
        || tag_lower.contains("studio")
    {
        return TagCategory::Setting;
    }

    // Quality keywords
    if tag_lower.contains("quality")
        || tag_lower.contains("detailed")
        || tag_lower.contains("sharp")
        || tag_lower.contains("blurry")
        || tag_lower.contains("high resolution")
        || tag_lower.contains("4k")
        || tag_lower.contains("8k")
    {
        return TagCategory::Quality;
    }

    // Default to Subject
    TagCategory::Subject
}

/// Check if Moondream model is available in cache
pub fn is_moondream_available() -> bool {
    let api = match hf_hub::api::sync::Api::new() {
        Ok(api) => api,
        Err(_) => return false,
    };

    let repo = api.model(MOONDREAM_REPO.to_string());

    // Check if quantized model file exists in cache
    repo.get("model-q4_0.gguf").is_ok()
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
        assert_eq!(moondream_repo_id(), "santiagomed/candle-moondream");
    }

    #[test]
    fn test_categorize_tag() {
        use super::super::taxonomy::TagCategory;

        assert_eq!(categorize_tag("realistic style"), TagCategory::Style);
        assert_eq!(categorize_tag("serene mood"), TagCategory::Mood);
        assert_eq!(categorize_tag("natural light"), TagCategory::Lighting);
        assert_eq!(categorize_tag("vibrant colors"), TagCategory::Color);
        assert_eq!(categorize_tag("person"), TagCategory::Subject);
    }
}
