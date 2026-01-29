//! Model management and downloading

pub mod bundle_builder;
mod clip;
mod downloader;
mod flux;
pub mod lora;
pub mod lora_manager;
mod manager;
mod model_config;
mod model_config_cache;
mod model_type;
mod paths;
mod qwen3;
pub mod scanner;
mod t5;
mod vae;
mod zimage;

pub use bundle_builder::{BundleBuilder, BundleDefinition, to_component_record};
pub use clip::ClipTextEncoder;
pub use downloader::ModelDownloader;
pub use flux::FluxTransformer;
pub use lora::{LoraAdapter, LoraConfig, LoraInfo, LoraFileInfo, LoraWeight};
pub use lora_manager::LoraManager;
pub use manager::{create_shared_manager, ModelManager, Precision, SharedModelManager};
pub use model_config::ModelConfig;
pub use model_config_cache::ModelConfigCache;
pub use model_type::ModelType;
pub use paths::{ComponentRole, ModelPaths};
pub use qwen3::Qwen3TextEncoder;
pub use scanner::{ComponentType, DiscoveredComponent, DiscoveredModel, ModelFormat, scan_all_components, scan_cache_for_models};
pub use t5::T5TextEncoder;
pub use vae::VaeDecoder;
pub use zimage::ZImageTransformer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_module() {
        let _paths = ModelPaths::new().unwrap();
    }
}
