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

