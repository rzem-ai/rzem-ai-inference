//! Model type definitions for FLUX variants

use serde::{Deserialize, Serialize};

/// Available FLUX model types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    /// FLUX.1 [schnell] - Fast, 4 steps
    Schnell,
    /// FLUX.1 [dev] - Higher quality, 28+ steps
    Dev,
}

impl ModelType {
    /// Default number of denoising steps
    pub fn default_steps(&self) -> usize {
        match self {
            Self::Schnell => 4,
            Self::Dev => 28,
        }
    }

    /// Default guidance scale
    pub fn default_guidance(&self) -> f64 {
        match self {
            Self::Schnell => 4.0,
            Self::Dev => 3.5,
        }
    }

    /// Valid step range (min, max)
    pub fn step_range(&self) -> (usize, usize) {
        match self {
            Self::Schnell => (1, 8),
            Self::Dev => (20, 100),
        }
    }

    /// Approximate VRAM usage in MB (full precision)
    pub fn vram_full_precision(&self) -> usize {
        match self {
            Self::Schnell => 23_000,
            Self::Dev => 24_000,
        }
    }

    /// Approximate VRAM usage in MB (quantized)
    pub fn vram_quantized(&self) -> usize {
        match self {
            Self::Schnell => 12_000,
            Self::Dev => 12_000,
        }
    }

    /// HuggingFace repository ID
    pub fn repo_id(&self) -> &'static str {
        match self {
            Self::Schnell => "black-forest-labs/FLUX.1-schnell",
            Self::Dev => "black-forest-labs/FLUX.1-dev",
        }
    }

    /// Transformer filename
    pub fn transformer_filename(&self) -> &'static str {
        match self {
            Self::Schnell => "flux1-schnell.safetensors",
            Self::Dev => "flux1-dev.safetensors",
        }
    }

    /// Quantized transformer filename
    pub fn quantized_filename(&self) -> &'static str {
        match self {
            Self::Schnell => "flux1-schnell.gguf",
            Self::Dev => "flux1-dev.gguf",
        }
    }

    /// Display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Schnell => "FLUX.1 [schnell]",
            Self::Dev => "FLUX.1 [dev]",
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
            _ => Err(format!("Unknown model type: {}. Use 'schnell' or 'dev'", s)),
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
    }

    #[test]
    fn test_parse() {
        assert_eq!("schnell".parse::<ModelType>().unwrap(), ModelType::Schnell);
        assert_eq!("dev".parse::<ModelType>().unwrap(), ModelType::Dev);
        assert_eq!("SCHNELL".parse::<ModelType>().unwrap(), ModelType::Schnell);
    }

    #[test]
    fn test_vram() {
        assert!(ModelType::Dev.vram_full_precision() > ModelType::Dev.vram_quantized());
    }
}
