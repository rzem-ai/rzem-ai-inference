//! FLUX VAE decoder for latent → RGB conversion

use anyhow::Result;
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::flux::autoencoder::{AutoEncoder, Config};
use std::path::Path;

/// FLUX VAE decoder for converting latents to RGB images
pub struct VaeDecoder {
    model: AutoEncoder,
    device: Device,
}

impl VaeDecoder {
    /// Load FLUX VAE from ae.safetensors file
    pub fn load<P: AsRef<Path>>(model_path: P, device: Device) -> Result<Self> {
        // Use bf16 on CUDA for efficiency, f32 on CPU
        let dtype = if device.is_cuda() {
            DType::BF16
        } else {
            DType::F32
        };

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[model_path.as_ref()], dtype, &device)?
        };

        // FLUX Schnell VAE config
        let config = Config::schnell();
        let model = AutoEncoder::new(&config, vb)?;

        Ok(Self { model, device })
    }

    /// Decode latent tensor to RGB image
    ///
    /// # Arguments
    /// * `latents` - Latent tensor from flux::sampling::unpack [1, 16, H/8, W/8]
    ///
    /// # Returns
    /// RGB tensor [1, 3, H, W] with values in range [-1, 1]
    pub fn decode(&self, latents: &Tensor) -> Result<Tensor> {
        // FLUX autoencoder handles the scaling/shifting internally
        let image = self.model.decode(latents)?;
        Ok(image)
    }

    /// Convert decoded tensor to RGB image buffer
    ///
    /// # Arguments
    /// * `tensor` - Image tensor [1, 3, H, W] with values in [-1, 1]
    ///
    /// # Returns
    /// Vec<u8> with RGB pixel data (H * W * 3 bytes)
    pub fn tensor_to_rgb(&self, tensor: &Tensor) -> Result<Vec<u8>> {
        // Clamp to [-1, 1], scale to [0, 255]
        let image = tensor.clamp(-1f32, 1f32)?;
        let image = ((image + 1.0)? * 127.5)?;
        let image = image.to_dtype(DType::U8)?;

        // Remove batch dimension and permute to HWC
        let image = image.squeeze(0)?.permute((1, 2, 0))?;

        // Flatten to Vec<u8>
        Ok(image.flatten_all()?.to_vec1()?)
    }

    /// Get the device this decoder is on
    pub fn device(&self) -> &Device {
        &self.device
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires downloaded FLUX model
    fn test_vae_loading() {
        use crate::models::ModelPaths;

        let paths = ModelPaths::new().unwrap();
        let device = Device::cuda_if_available(0).unwrap();

        let _vae = VaeDecoder::load(paths.vae_path().unwrap(), device).unwrap();
    }

    #[test]
    #[ignore] // Requires downloaded Z-Image-Turbo model
    fn test_zimage_vae_loading() {
        use crate::models::ModelPaths;

        let paths = ModelPaths::new().unwrap();
        let device = Device::cuda_if_available(0).unwrap();

        // Z-Image uses FLUX's VAE - verify it loads with existing decoder
        // TODO: Update test for bundle system
        let vae_file = paths.vae_path().unwrap().join("diffusion_pytorch_model.safetensors");

        let vae = VaeDecoder::load(&vae_file, device);
        assert!(vae.is_ok(), "Failed to load Z-Image VAE: {:?}", vae.err());

        println!("✓ Z-Image VAE loaded successfully using FLUX VaeDecoder");
    }

    #[test]
    #[ignore] // Requires downloaded Z-Image-Turbo model
    fn test_zimage_vae_decode() {
        use crate::models::ModelPaths;
        use candle_core::Tensor;

        let paths = ModelPaths::new().unwrap();
        let device = Device::cuda_if_available(0).unwrap();

        // Load Z-Image VAE
        // TODO: Update test for bundle system
        let vae_file = paths.vae_path().unwrap().join("diffusion_pytorch_model.safetensors");
        let vae = VaeDecoder::load(&vae_file, device.clone()).unwrap();

        // Create dummy latent tensor [1, 16, 128, 128] (for 1024x1024 image)
        let latents = Tensor::zeros((1, 16, 128, 128), DType::F32, &device).unwrap();

        // Test decode
        let decoded = vae.decode(&latents);
        assert!(decoded.is_ok(), "Failed to decode: {:?}", decoded.err());

        let image = decoded.unwrap();
        let shape = image.dims();

        // Should produce [1, 3, 1024, 1024]
        assert_eq!(shape[0], 1, "Batch size should be 1");
        assert_eq!(shape[1], 3, "Should have 3 RGB channels");
        assert_eq!(shape[2], 1024, "Height should be 1024");
        assert_eq!(shape[3], 1024, "Width should be 1024");

        println!("✓ Z-Image VAE decode successful: {:?}", shape);
    }
}
