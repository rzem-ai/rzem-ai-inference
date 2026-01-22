//! PNG Metadata Embedding for Generation Parameters
//!
//! Embeds generation parameters into PNG files using tEXt chunks,
//! following a format compatible with other AI image generation tools.

use anyhow::Result;
use png::{Encoder, Decoder, BitDepth, ColorType};
use std::io::{Cursor, Read};
use serde::{Deserialize, Serialize};

/// Generation metadata to embed in images
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub steps: u32,
    pub cfg_scale: f64,
    pub width: u32,
    pub height: u32,
    pub seed: i64,
    pub model: String,
    pub sampler: Option<String>,
    pub scheduler: Option<String>,
}

impl ImageMetadata {
    /// Format metadata as A1111-compatible parameters string
    /// This format is widely recognized by other AI image tools
    pub fn to_parameters_string(&self) -> String {
        let mut params = self.prompt.clone();

        if let Some(ref neg) = self.negative_prompt {
            if !neg.is_empty() {
                params.push_str(&format!("\nNegative prompt: {}", neg));
            }
        }

        params.push_str(&format!(
            "\nSteps: {}, CFG scale: {}, Seed: {}, Size: {}x{}, Model: {}",
            self.steps,
            self.cfg_scale,
            self.seed,
            self.width,
            self.height,
            self.model
        ));

        if let Some(ref sampler) = self.sampler {
            params.push_str(&format!(", Sampler: {}", sampler));
        }

        if let Some(ref scheduler) = self.scheduler {
            params.push_str(&format!(", Scheduler: {}", scheduler));
        }

        params
    }

    /// Parse metadata from A1111-compatible parameters string
    pub fn from_parameters_string(params: &str) -> Option<Self> {
        let lines: Vec<&str> = params.lines().collect();
        if lines.is_empty() {
            return None;
        }

        // First line(s) until "Negative prompt:" or settings line is the prompt
        let mut prompt_lines = Vec::new();
        let mut negative_prompt = None;
        let mut settings_line = None;

        for line in &lines {
            if line.starts_with("Negative prompt:") {
                negative_prompt = Some(line.trim_start_matches("Negative prompt:").trim().to_string());
            } else if line.starts_with("Steps:") {
                settings_line = Some(*line);
            } else if negative_prompt.is_none() && settings_line.is_none() {
                prompt_lines.push(*line);
            }
        }

        let prompt = prompt_lines.join("\n");
        let settings = settings_line?;

        // Parse settings line
        let mut steps = 4;
        let mut cfg_scale = 1.0;
        let mut seed: i64 = -1;
        let mut width = 1024;
        let mut height = 1024;
        let mut model = String::from("unknown");
        let mut sampler = None;
        let mut scheduler = None;

        for part in settings.split(',') {
            let part = part.trim();
            if let Some(val) = part.strip_prefix("Steps:") {
                steps = val.trim().parse().unwrap_or(4);
            } else if let Some(val) = part.strip_prefix("CFG scale:") {
                cfg_scale = val.trim().parse().unwrap_or(1.0);
            } else if let Some(val) = part.strip_prefix("Seed:") {
                seed = val.trim().parse().unwrap_or(-1);
            } else if let Some(val) = part.strip_prefix("Size:") {
                if let Some((w, h)) = val.trim().split_once('x') {
                    width = w.parse().unwrap_or(1024);
                    height = h.parse().unwrap_or(1024);
                }
            } else if let Some(val) = part.strip_prefix("Model:") {
                model = val.trim().to_string();
            } else if let Some(val) = part.strip_prefix("Sampler:") {
                sampler = Some(val.trim().to_string());
            } else if let Some(val) = part.strip_prefix("Scheduler:") {
                scheduler = Some(val.trim().to_string());
            }
        }

        Some(ImageMetadata {
            prompt,
            negative_prompt,
            steps,
            cfg_scale,
            width,
            height,
            seed,
            model,
            sampler,
            scheduler,
        })
    }
}

/// Encode RGB data to PNG with embedded metadata
pub fn encode_png_with_metadata(
    rgb_data: &[u8],
    width: u32,
    height: u32,
    metadata: &ImageMetadata,
) -> Result<Vec<u8>> {
    let mut png_data = Vec::new();

    {
        let mut encoder = Encoder::new(&mut png_data, width, height);
        encoder.set_color(ColorType::Rgb);
        encoder.set_depth(BitDepth::Eight);

        // Add metadata as tEXt chunks
        // "parameters" is the standard key used by A1111 and other tools
        encoder.add_text_chunk(
            "parameters".to_string(),
            metadata.to_parameters_string(),
        )?;

        // Also add as JSON for easier programmatic access
        encoder.add_text_chunk(
            "rzem-metadata".to_string(),
            serde_json::to_string(metadata)?,
        )?;

        let mut writer = encoder.write_header()?;
        writer.write_image_data(rgb_data)?;
    }

    Ok(png_data)
}

/// Read metadata from a PNG file
pub fn read_png_metadata(png_data: &[u8]) -> Result<Option<ImageMetadata>> {
    let decoder = Decoder::new(Cursor::new(png_data));
    let reader = decoder.read_info()?;
    let info = reader.info();

    // First try to read our JSON metadata
    for chunk in &info.uncompressed_latin1_text {
        if chunk.keyword == "rzem-metadata" {
            if let Ok(metadata) = serde_json::from_str(&chunk.text) {
                return Ok(Some(metadata));
            }
        }
    }

    // Fall back to parsing A1111-style parameters
    for chunk in &info.uncompressed_latin1_text {
        if chunk.keyword == "parameters" {
            if let Some(metadata) = ImageMetadata::from_parameters_string(&chunk.text) {
                return Ok(Some(metadata));
            }
        }
    }

    Ok(None)
}

/// Read metadata from a PNG file path
pub fn read_png_metadata_from_file(path: &std::path::Path) -> Result<Option<ImageMetadata>> {
    let mut file = std::fs::File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    read_png_metadata(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameters_roundtrip() {
        let metadata = ImageMetadata {
            prompt: "A beautiful sunset over mountains".to_string(),
            negative_prompt: Some("blurry, low quality".to_string()),
            steps: 20,
            cfg_scale: 7.5,
            width: 1024,
            height: 768,
            seed: 12345,
            model: "flux-schnell".to_string(),
            sampler: Some("euler".to_string()),
            scheduler: Some("normal".to_string()),
        };

        let params_str = metadata.to_parameters_string();
        let parsed = ImageMetadata::from_parameters_string(&params_str).unwrap();

        assert_eq!(parsed.prompt, metadata.prompt);
        assert_eq!(parsed.steps, metadata.steps);
        assert_eq!(parsed.seed, metadata.seed);
    }
}
