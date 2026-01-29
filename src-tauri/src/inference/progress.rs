//! Progress tracking for generation pipeline

use serde::{Deserialize, Serialize};

/// Pipeline stages with their weight in overall progress
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStage {
    LoadingModels,
    Denoising,
    DecodingVae,
    EncodingPng,
}

impl PipelineStage {
    /// Get the start percentage for this stage (0.0-1.0)
    pub fn start_percent(&self) -> f32 {
        match self {
            Self::LoadingModels => 0.0,
            Self::Denoising => 0.5,
            Self::DecodingVae => 0.95,
            Self::EncodingPng => 0.98,
        }
    }

    /// Get the end percentage for this stage (0.0-1.0)
    pub fn end_percent(&self) -> f32 {
        match self {
            Self::LoadingModels => 0.5,
            Self::Denoising => 0.95,
            Self::DecodingVae => 0.98,
            Self::EncodingPng => 1.0,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::LoadingModels => "Loading models",
            Self::Denoising => "Drawing",
            Self::DecodingVae => "Decoding image",
            Self::EncodingPng => "Saving image",
        }
    }
}

/// Progress update during generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationProgress {
    /// Current pipeline stage
    pub stage: PipelineStage,
    /// Progress within current stage (0.0-1.0)
    pub stage_progress: f32,
    /// Overall generation progress (0.0-1.0)
    pub overall_progress: f32,
    /// Human-readable status message
    pub message: String,
    /// Estimated seconds remaining
    pub eta_seconds: Option<f32>,
    /// For batch jobs: current image index (1-indexed)
    pub batch_index: Option<usize>,
    /// For batch jobs: total image count
    pub batch_total: Option<usize>,
    /// Current denoising step (if in denoising stage)
    pub current_step: Option<usize>,
    /// Total denoising steps (if in denoising stage)
    pub total_steps: Option<usize>,
}

impl GenerationProgress {
    /// Create a new progress update
    pub fn new(stage: PipelineStage, stage_progress: f32) -> Self {
        let stage_start = stage.start_percent();
        let stage_end = stage.end_percent();
        let stage_range = stage_end - stage_start;
        let overall = stage_start + (stage_range * stage_progress.clamp(0.0, 1.0));

        Self {
            stage,
            stage_progress: stage_progress.clamp(0.0, 1.0),
            overall_progress: overall,
            message: stage.display_name().to_string(),
            eta_seconds: None,
            batch_index: None,
            batch_total: None,
            current_step: None,
            total_steps: None,
        }
    }

    /// Create progress for a denoising step
    pub fn denoising_step(current_step: usize, total_steps: usize) -> Self {
        let stage_progress = current_step as f32 / total_steps as f32;
        let mut progress = Self::new(PipelineStage::Denoising, stage_progress);
        progress.message = format!("Drawing step {}/{}", current_step, total_steps);
        progress.current_step = Some(current_step);
        progress.total_steps = Some(total_steps);
        progress
    }

    /// Set batch information
    pub fn with_batch(mut self, index: usize, total: usize) -> Self {
        self.batch_index = Some(index);
        self.batch_total = Some(total);
        if total > 1 {
            self.message = format!("Image {}/{}: {}", index, total, self.message);
        }
        self
    }

    /// Set ETA
    pub fn with_eta(mut self, seconds: f32) -> Self {
        self.eta_seconds = Some(seconds);
        self
    }
}

/// Callback type for progress updates
pub type ProgressCallback = Box<dyn Fn(GenerationProgress) + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_percentages() {
        assert_eq!(PipelineStage::LoadingModels.start_percent(), 0.0);
        assert_eq!(PipelineStage::EncodingPng.end_percent(), 1.0);
    }

    #[test]
    fn test_progress_calculation() {
        let progress = GenerationProgress::new(PipelineStage::Denoising, 0.5);
        // Denoising is 25-85%, so 50% through = 25 + (60 * 0.5) = 55%
        assert!((progress.overall_progress - 0.55).abs() < 0.01);
    }

    #[test]
    fn test_denoising_step() {
        let progress = GenerationProgress::denoising_step(2, 4);
        assert_eq!(progress.message, "Denoising step 2/4");
        assert!((progress.stage_progress - 0.5).abs() < 0.01);
    }
}
