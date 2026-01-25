//! Model type definitions for image generation models

use serde::{Deserialize, Serialize};

/// Available image generation model types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    /// FLUX.1 [schnell] - Fast, 4 steps
    Schnell,
    /// FLUX.1 [dev] - Higher quality, 28+ steps
    Dev,
    /// Z-Image-Turbo - Single-stream DiT, 8 fixed steps
    #[serde(rename = "zimage-turbo")]
    ZImageTurbo,
}

impl ModelType {
    /// Default number of denoising steps
    pub fn default_steps(&self) -> usize {
        match self {
            Self::Schnell => 4,
            Self::Dev => 28,
            Self::ZImageTurbo => 8,
        }
    }

    /// Default guidance scale
    pub fn default_guidance(&self) -> f64 {
        match self {
            Self::Schnell => 4.0,
            Self::Dev => 3.5,
            Self::ZImageTurbo => 0.0, // Distilled model, no CFG
        }
    }

    /// Valid step range (min, max)
    pub fn step_range(&self) -> (usize, usize) {
        match self {
            Self::Schnell => (1, 8),
            Self::Dev => (20, 100),
            Self::ZImageTurbo => (8, 8), // Fixed 8 steps
        }
    }

    /// Approximate VRAM usage in MB (full precision)
    pub fn vram_full_precision(&self) -> usize {
        match self {
            Self::Schnell => 23_000,
            Self::Dev => 24_000,
            Self::ZImageTurbo => 16_000,
        }
    }

    /// Approximate VRAM usage in MB (quantized)
    pub fn vram_quantized(&self) -> usize {
        match self {
            Self::Schnell => 12_000,
            Self::Dev => 12_000,
            Self::ZImageTurbo => 10_000, // TBD - estimated
        }
    }

    /// HuggingFace repository ID
    pub fn repo_id(&self) -> &'static str {
        match self {
            Self::Schnell => "black-forest-labs/FLUX.1-schnell",
            Self::Dev => "black-forest-labs/FLUX.1-dev",
            Self::ZImageTurbo => "Tongyi-MAI/Z-Image-Turbo",
        }
    }

    /// Transformer filename
    pub fn transformer_filename(&self) -> &'static str {
        match self {
            Self::Schnell => "flux1-schnell.safetensors",
            Self::Dev => "flux1-dev.safetensors",
            Self::ZImageTurbo => "diffusion_pytorch_model.safetensors",
        }
    }

    /// Quantized transformer filename
    pub fn quantized_filename(&self) -> &'static str {
        match self {
            Self::Schnell => "flux1-schnell.gguf",
            Self::Dev => "flux1-dev.gguf",
            Self::ZImageTurbo => "zimage-turbo.gguf", // TBD - to be created
        }
    }

    /// Display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Schnell => "FLUX.1 [schnell]",
            Self::Dev => "FLUX.1 [dev]",
            Self::ZImageTurbo => "Z-Image-Turbo",
        }
    }
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for ModelType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "schnell" | "flux-schnell" | "flux.1-schnell" => Ok(Self::Schnell),
            "dev" | "flux-dev" | "flux.1-dev" => Ok(Self::Dev),
            "zimage-turbo" | "z-image-turbo" | "zimage" => Ok(Self::ZImageTurbo),
            _ => Err(format!(
                "Unknown model type: {}. Use 'schnell', 'dev', or 'zimage-turbo'",
                s
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_steps() {
        assert_eq!(ModelType::Schnell.default_steps(), 4);
        assert_eq!(ModelType::Dev.default_steps(), 28);
        assert_eq!(ModelType::ZImageTurbo.default_steps(), 8);
    }

    #[test]
    fn test_default_guidance() {
        assert_eq!(ModelType::Schnell.default_guidance(), 4.0);
        assert_eq!(ModelType::Dev.default_guidance(), 3.5);
        assert_eq!(ModelType::ZImageTurbo.default_guidance(), 0.0);
    }

    #[test]
    fn test_step_range() {
        assert_eq!(ModelType::Schnell.step_range(), (1, 8));
        assert_eq!(ModelType::Dev.step_range(), (20, 100));
        assert_eq!(ModelType::ZImageTurbo.step_range(), (8, 8));
    }

    #[test]
    fn test_parse() {
        assert_eq!("schnell".parse::<ModelType>().unwrap(), ModelType::Schnell);
        assert_eq!("dev".parse::<ModelType>().unwrap(), ModelType::Dev);
        assert_eq!("SCHNELL".parse::<ModelType>().unwrap(), ModelType::Schnell);
        assert_eq!("zimage-turbo".parse::<ModelType>().unwrap(), ModelType::ZImageTurbo);
        assert_eq!("z-image-turbo".parse::<ModelType>().unwrap(), ModelType::ZImageTurbo);
        assert_eq!("ZIMAGE".parse::<ModelType>().unwrap(), ModelType::ZImageTurbo);
    }

    #[test]
    fn test_vram() {
        assert!(ModelType::Dev.vram_full_precision() > ModelType::Dev.vram_quantized());
        assert!(ModelType::ZImageTurbo.vram_full_precision() > ModelType::ZImageTurbo.vram_quantized());
        assert_eq!(ModelType::ZImageTurbo.vram_full_precision(), 16_000);
    }

    #[test]
    fn test_repo_ids() {
        assert_eq!(ModelType::Schnell.repo_id(), "black-forest-labs/FLUX.1-schnell");
        assert_eq!(ModelType::Dev.repo_id(), "black-forest-labs/FLUX.1-dev");
        assert_eq!(ModelType::ZImageTurbo.repo_id(), "Tongyi-MAI/Z-Image-Turbo");
    }

    #[test]
    fn test_display_names() {
        assert_eq!(ModelType::Schnell.display_name(), "FLUX.1 [schnell]");
        assert_eq!(ModelType::Dev.display_name(), "FLUX.1 [dev]");
        assert_eq!(ModelType::ZImageTurbo.display_name(), "Z-Image-Turbo");
    }
}
