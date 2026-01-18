//! Model management and downloading

mod clip;
mod downloader;
mod flux;
mod manager;
mod model_type;
mod paths;
mod t5;
mod vae;

pub use clip::ClipTextEncoder;
pub use downloader::ModelDownloader;
pub use flux::FluxTransformer;
pub use manager::{create_shared_manager, ModelManager, Precision, SharedModelManager};
pub use model_type::ModelType;
pub use paths::ModelPaths;
pub use t5::T5TextEncoder;
pub use vae::VaeDecoder;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_module() {
        let _paths = ModelPaths::new().unwrap();
    }
}
