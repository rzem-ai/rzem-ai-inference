//! Inference engine for running Flux models with Candle

mod engine;

pub use engine::InferenceEngine;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_module() {
        let _engine = InferenceEngine::new().unwrap();
    }
}
