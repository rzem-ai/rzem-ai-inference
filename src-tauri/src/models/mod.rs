//! Model management and downloading

mod clip;
mod downloader;
mod paths;

pub use clip::ClipTextEncoder;
pub use downloader::ModelDownloader;
pub use paths::ModelPaths;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_module() {
        let _paths = ModelPaths::new().unwrap();
    }
}
