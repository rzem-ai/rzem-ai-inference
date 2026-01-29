//! Inference engine for running diffusion models with Candle

pub mod cache;
mod embedding_cache;
mod engine;
pub mod flux_pipeline;
pub mod metadata;
mod progress;
pub mod samplers;
mod stats;
mod zindex_pipeline;

pub use cache::{CacheStats, ModelCache, ModelCacheConfig, ModelsLoaded};
pub use embedding_cache::{CachedEmbedding, EmbeddingCache};
pub use engine::InferenceEngine;
pub use flux_pipeline::{BundleContext, FluxPipeline};
pub use metadata::{ImageMetadata, encode_png_with_metadata, read_png_metadata};
pub use progress::{GenerationProgress, PipelineStage, ProgressCallback};
pub use samplers::{SamplerType, SchedulerType, get_timesteps};
pub use zindex_pipeline::ZIndexPipeline;
pub use stats::{GenerationStats, GenerationResult, Timer};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_module() {
        let _engine = InferenceEngine::new().unwrap();
    }
}
