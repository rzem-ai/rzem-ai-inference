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
            .ok_or_else(|| anyhow::anyhow!("Missing default_settings for model '{}'", record.id))?;

        let default_steps = default_settings.get("steps")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'steps' in default_settings for model '{}'", record.id))? as usize;

        let default_guidance = default_settings.get("guidance")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'guidance' in default_settings for model '{}'", record.id))?;

        let quantized_repos = record.quantized_repos.as_ref()
            .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
            .unwrap_or_default();

        Ok(Self {
            id: record.id.clone(),
            name: record.name.clone(),
            display_name: record.name.clone(),
            repo_id: record.repo_id.clone()
                .ok_or_else(|| anyhow::anyhow!("Missing 'repo_id' for model '{}'", record.id))?,
            transformer_filename: record.transformer_filename.clone()
                .ok_or_else(|| anyhow::anyhow!("Missing 'transformer_filename' for model '{}'", record.id))?,
            quantized_filename: record.quantized_filename.clone(),
            quantized_repos,
            default_steps,
            default_guidance,
            step_min: record.step_min
                .ok_or_else(|| anyhow::anyhow!("Missing 'step_min' for model '{}'", record.id))? as usize,
            step_max: record.step_max
                .ok_or_else(|| anyhow::anyhow!("Missing 'step_max' for model '{}'", record.id))? as usize,
            vram_full_mb: record.vram_full
                .ok_or_else(|| anyhow::anyhow!("Missing 'vram_full' for model '{}'", record.id))? as usize,
            vram_quantized_mb: record.vram_quantized
                .ok_or_else(|| anyhow::anyhow!("Missing 'vram_quantized' for model '{}'", record.id))? as usize,
            model_family: record.model_family.clone()
                .ok_or_else(|| anyhow::anyhow!("Missing 'model_family' for model '{}'", record.id))?,
            component_type: record.component_type.clone()
                .ok_or_else(|| anyhow::anyhow!("Missing 'component_type' for model '{}'", record.id))?,
        })
    }
}
