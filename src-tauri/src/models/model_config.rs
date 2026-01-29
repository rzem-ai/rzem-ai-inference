use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Complete model configuration loaded from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub display_name: String,

    // Repository information
    pub repo_id: String,
    pub transformer_filename: String,
    pub quantized_filename: Option<String>,
    pub quantized_repos: Vec<String>,

    // Generation parameters
    pub default_steps: usize,
    pub default_guidance: f64,
    pub step_min: usize,
    pub step_max: usize,

    // Resource requirements
    pub vram_full_mb: usize,
    pub vram_quantized_mb: usize,

    // Classification
    pub model_family: String,
    pub component_type: String,
}

impl ModelConfig {
    /// Create from database ModelRecord
    pub fn from_record(record: &crate::gallery::ModelRecord) -> Result<Self> {
        let default_settings = record.default_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing default_settings"))?;

        let default_steps = default_settings.get("steps")
            .and_then(|v| v.as_u64())
            .unwrap_or(4) as usize;

        let default_guidance = default_settings.get("guidance")
            .and_then(|v| v.as_f64())
            .unwrap_or(4.0);

        let quantized_repos = record.quantized_repos.as_ref()
            .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
            .unwrap_or_default();

        Ok(Self {
            id: record.id.clone(),
            name: record.name.clone(),
            display_name: record.name.clone(),
            repo_id: record.repo_id.clone().unwrap_or_default(),
            transformer_filename: record.transformer_filename.clone().unwrap_or_default(),
            quantized_filename: record.quantized_filename.clone(),
            quantized_repos,
            default_steps,
            default_guidance,
            step_min: record.step_min.unwrap_or(1) as usize,
            step_max: record.step_max.unwrap_or(100) as usize,
            vram_full_mb: record.vram_full.unwrap_or(24000) as usize,
            vram_quantized_mb: record.vram_quantized.unwrap_or(12000) as usize,
            model_family: record.model_family.clone().unwrap_or_else(|| "flux".to_string()),
            component_type: record.component_type.clone().unwrap_or_else(|| "transformer".to_string()),
        })
    }

    /// Fallback config for known models (if DB unavailable)
    pub fn fallback(model_id: &str) -> Self {
        match model_id {
            "schnell" => Self {
                id: "schnell".to_string(),
                name: "FLUX.1 [schnell]".to_string(),
                display_name: "FLUX.1 [schnell]".to_string(),
                repo_id: "black-forest-labs/FLUX.1-schnell".to_string(),
                transformer_filename: "flux1-schnell.safetensors".to_string(),
                quantized_filename: Some("flux1-schnell.gguf".to_string()),
                quantized_repos: vec!["lmz/candle-flux".to_string()],
                default_steps: 4,
                default_guidance: 4.0,
                step_min: 1,
                step_max: 8,
                vram_full_mb: 23000,
                vram_quantized_mb: 12000,
                model_family: "flux".to_string(),
                component_type: "transformer".to_string(),
            },
            "dev" => Self {
                id: "dev".to_string(),
                name: "FLUX.1 [dev]".to_string(),
                display_name: "FLUX.1 [dev]".to_string(),
                repo_id: "black-forest-labs/FLUX.1-dev".to_string(),
                transformer_filename: "flux1-dev.safetensors".to_string(),
                quantized_filename: Some("flux1-dev.gguf".to_string()),
                quantized_repos: vec!["city96/FLUX.1-dev-gguf".to_string(), "lmz/candle-flux".to_string()],
                default_steps: 28,
                default_guidance: 3.5,
                step_min: 20,
                step_max: 100,
                vram_full_mb: 24000,
                vram_quantized_mb: 12000,
                model_family: "flux".to_string(),
                component_type: "transformer".to_string(),
            },
            "zimage-turbo" => Self {
                id: "zimage-turbo".to_string(),
                name: "Z-Image-Turbo".to_string(),
                display_name: "Z-Image-Turbo".to_string(),
                repo_id: "Tongyi-MAI/Z-Image-Turbo".to_string(),
                transformer_filename: "diffusion_pytorch_model.safetensors".to_string(),
                quantized_filename: Some("zimage-turbo.gguf".to_string()),
                quantized_repos: vec![],
                default_steps: 8,
                default_guidance: 0.0,
                step_min: 8,
                step_max: 8,
                vram_full_mb: 16000,
                vram_quantized_mb: 10000,
                model_family: "zindex".to_string(),
                component_type: "transformer".to_string(),
            },
            _ => Self::default_flux(),
        }
    }

    fn default_flux() -> Self {
        Self::fallback("schnell")
    }
}
