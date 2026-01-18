//! Model management and downloading

mod clip;
mod downloader;
mod flux;
mod paths;
mod t5;
mod vae;

pub use clip::ClipTextEncoder;
pub use downloader::ModelDownloader;
pub use flux::FluxTransformer;
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
