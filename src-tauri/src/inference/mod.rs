//! Inference engine for running Flux models with Candle

mod engine;
mod pipeline;

pub use engine::InferenceEngine;
pub use pipeline::FluxPipeline;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_module() {
        let _engine = InferenceEngine::new().unwrap();
    }
}
