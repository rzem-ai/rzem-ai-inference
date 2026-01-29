//! Model type definitions for image generation models

use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// Model type identifier (replaces enum)
/// Uses Arc<str> for cheap cloning while supporting dynamic model IDs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModelType(Arc<str>);

impl ModelType {
    pub fn new(id: impl Into<String>) -> Self {
        Self(Arc::from(id.into()))
    }

    pub fn id(&self) -> &str {
        &self.0
    }

    // Known model constants for convenience
    pub fn schnell() -> Self {
        Self(Arc::from("schnell"))
    }

    pub fn dev() -> Self {
        Self(Arc::from("dev"))
    }

    pub fn zimage_turbo() -> Self {
        Self(Arc::from("zimage-turbo"))
    }

    // Temporary methods for backward compatibility
    // TODO: Replace with ModelConfig lookups
    pub fn vram_full_precision(&self) -> usize {
        match self.id() {
            "schnell" => 23_000,
            "dev" => 24_000,
            "zimage-turbo" => 16_000,
            _ => 24_000,
        }
    }

    pub fn vram_quantized(&self) -> usize {
        match self.id() {
            "schnell" => 12_000,
            "dev" => 12_000,
            "zimage-turbo" => 10_000,
            _ => 12_000,
        }
    }

    pub fn default_steps(&self) -> usize {
        match self.id() {
            "schnell" => 4,
            "dev" => 28,
            "zimage-turbo" => 8,
            _ => 4,
        }
    }

    pub fn default_guidance(&self) -> f64 {
        match self.id() {
            "schnell" => 4.0,
            "dev" => 3.5,
            "zimage-turbo" => 0.0,
            _ => 4.0,
        }
    }

    pub fn step_range(&self) -> (usize, usize) {
        match self.id() {
            "schnell" => (1, 8),
            "dev" => (20, 100),
            "zimage-turbo" => (8, 8),
            _ => (1, 100),
        }
    }

    pub fn repo_id(&self) -> &'static str {
        match self.id() {
            "schnell" => "black-forest-labs/FLUX.1-schnell",
            "dev" => "black-forest-labs/FLUX.1-dev",
            "zimage-turbo" => "Tongyi-MAI/Z-Image-Turbo",
            _ => "black-forest-labs/FLUX.1-schnell",
        }
    }

    pub fn transformer_filename(&self) -> &'static str {
        match self.id() {
            "schnell" => "flux1-schnell.safetensors",
            "dev" => "flux1-dev.safetensors",
            "zimage-turbo" => "diffusion_pytorch_model.safetensors",
            _ => "flux1-schnell.safetensors",
        }
    }

    pub fn quantized_filename(&self) -> &'static str {
        match self.id() {
            "schnell" => "flux1-schnell.gguf",
            "dev" => "flux1-dev.gguf",
            "zimage-turbo" => "zimage-turbo.gguf",
            _ => "flux1-schnell.gguf",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self.id() {
            "schnell" => "FLUX.1 [schnell]",
            "dev" => "FLUX.1 [dev]",
            "zimage-turbo" => "Z-Image-Turbo",
            _ => "Unknown Model",
        }
    }
}

impl FromStr for ModelType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Normalize common aliases
        let lowercased = s.to_lowercase();
        let normalized = match lowercased.as_str() {
            "flux-schnell" | "flux.1-schnell" => "schnell",
            "flux-dev" | "flux.1-dev" => "dev",
            "zimage" | "z-image-turbo" => "zimage-turbo",
            _ => lowercased.as_str(),
        };
        Ok(Self(Arc::from(normalized)))
    }
}

impl fmt::Display for ModelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ModelType {
    fn from(s: String) -> Self {
        Self(Arc::from(s))
    }
}

impl From<&str> for ModelType {
    fn from(s: &str) -> Self {
        Self(Arc::from(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!("schnell".parse::<ModelType>().unwrap(), ModelType::schnell());
        assert_eq!("dev".parse::<ModelType>().unwrap(), ModelType::dev());
        assert_eq!("SCHNELL".parse::<ModelType>().unwrap(), ModelType::schnell());
        assert_eq!("zimage-turbo".parse::<ModelType>().unwrap(), ModelType::zimage_turbo());
        assert_eq!("z-image-turbo".parse::<ModelType>().unwrap(), ModelType::zimage_turbo());
        assert_eq!("ZIMAGE".parse::<ModelType>().unwrap(), ModelType::zimage_turbo());
    }

    #[test]
    fn test_constants() {
        assert_eq!(ModelType::schnell().id(), "schnell");
        assert_eq!(ModelType::dev().id(), "dev");
        assert_eq!(ModelType::zimage_turbo().id(), "zimage-turbo");
    }

    #[test]
    fn test_display() {
        assert_eq!(ModelType::schnell().to_string(), "schnell");
        assert_eq!(ModelType::dev().to_string(), "dev");
    }

    #[test]
    fn test_clone_cheap() {
        let model1 = ModelType::schnell();
        let model2 = model1.clone();
        assert_eq!(model1, model2);
        // Arc makes this a cheap pointer copy
        assert_eq!(Arc::strong_count(&model1.0), Arc::strong_count(&model2.0));
    }
}
