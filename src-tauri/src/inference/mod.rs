//! Inference engine for running Flux models with Candle

mod engine;
mod pipeline;
mod stats;

pub use engine::InferenceEngine;
pub use pipeline::FluxPipeline;
pub use stats::{GenerationStats, GenerationResult, Timer};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_module() {
        let _engine = InferenceEngine::new().unwrap();
    }
}
