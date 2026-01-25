//! Model management and downloading

mod clip;
mod downloader;
mod flux;
pub mod lora;
pub mod lora_manager;
mod manager;
mod model_type;
mod paths;
mod qwen3;
mod t5;
mod vae;
mod zimage;

pub use clip::ClipTextEncoder;
pub use downloader::ModelDownloader;
pub use flux::FluxTransformer;
pub use lora::{LoraAdapter, LoraConfig, LoraInfo, LoraFileInfo, LoraWeight};
pub use lora_manager::LoraManager;
pub use manager::{create_shared_manager, ModelManager, Precision, SharedModelManager};
pub use model_type::ModelType;
pub use paths::ModelPaths;
pub use qwen3::Qwen3TextEncoder;
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
