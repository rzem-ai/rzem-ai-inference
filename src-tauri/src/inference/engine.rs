//! Core inference engine using Candle

use anyhow::Result;
use candle_core::{Device, Tensor};

pub struct InferenceEngine {
    device: Device,
}

impl InferenceEngine {
    pub fn new() -> Result<Self> {
        let device = Self::select_device()?;
        Ok(Self { device })
    }

    /// Select the best available compute device based on platform
    fn select_device() -> Result<Device> {
        // macOS: Try Metal first (Apple Silicon)
        #[cfg(target_os = "macos")]
        {
            match Device::new_metal(0) {
                Ok(device) => {
                    println!("[InferenceEngine] Using Metal device for inference");
                    return Ok(device);
                }
                Err(e) => {
                    println!("[InferenceEngine] Metal not available: {}, falling back to CPU", e);
                }
            }
        }

        // Linux: Try CUDA first
        #[cfg(target_os = "linux")]
        {
            match Device::new_cuda(0) {
                Ok(device) => {
                    println!("[InferenceEngine] Using CUDA device for inference");
                    return Ok(device);
                }
                Err(e) => {
                    println!("[InferenceEngine] CUDA not available: {}, falling back to CPU", e);
                }
            }
        }

        // Fallback to CPU for all platforms
        println!("[InferenceEngine] Using CPU for inference");
        Ok(Device::Cpu)
    }

    pub fn get_device(&self) -> &Device {
        &self.device
    }

    pub fn test_tensor_ops(&self) -> Result<Vec<f32>> {
        let tensor = Tensor::new(&[1.0f32, 2.0, 3.0, 4.0], &self.device)?;
        let result = (tensor * 2.0)?;
        Ok(result.to_vec1()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = InferenceEngine::new().unwrap();
        let _device = engine.get_device();
    }

    #[test]
    fn test_tensor_operations() {
        let engine = InferenceEngine::new().unwrap();
        let result = engine.test_tensor_ops().unwrap();
        assert_eq!(result, vec![2.0, 4.0, 6.0, 8.0]);
    }
}
