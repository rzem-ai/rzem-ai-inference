//! Core inference engine using Candle

use anyhow::Result;
use candle_core::{Device, Tensor};

pub struct InferenceEngine {
    device: Device,
}

impl InferenceEngine {
    pub fn new() -> Result<Self> {
        let device = Device::cuda_if_available(0)?;
        Ok(Self { device })
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
