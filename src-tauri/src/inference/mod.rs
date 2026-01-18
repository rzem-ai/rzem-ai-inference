//! Inference engine for running Flux models with Candle

mod engine;
mod pipeline;
mod progress;
mod stats;

pub use engine::InferenceEngine;
pub use pipeline::FluxPipeline;
pub use progress::{GenerationProgress, PipelineStage, ProgressCallback};
pub use stats::{GenerationStats, GenerationResult, Timer};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_module() {
        let _engine = InferenceEngine::new().unwrap();
    }
}
