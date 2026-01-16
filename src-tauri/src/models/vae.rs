//! VAE decoder for FLUX latent → RGB conversion

use anyhow::Result;
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::stable_diffusion;
use std::path::Path;

/// VAE decoder for converting latents to RGB images
pub struct VaeDecoder {
    vae: stable_diffusion::vae::AutoEncoderKL,
    device: Device,
}

impl VaeDecoder {
    /// Load VAE from safetensors file
    pub fn load<P: AsRef<Path>>(
        model_path: P,
        device: Device,
    ) -> Result<Self> {
        // SAFETY: from_mmaped_safetensors uses memory-mapped IO which is safe
        // because safetensors format is designed to be safely memory-mapped
        // without requiring trust of the file contents.
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(
                &[model_path.as_ref()],
                candle_core::DType::F32,
                &device,
            )?
        };

        // Load VAE configuration for FLUX
        // FLUX uses 16-channel latents (vs 4 for Stable Diffusion)
        // Configuration from vae/config.json:
        // - latent_channels: 16
        // - in_channels: 3 (RGB)
        // - out_channels: 3 (RGB)
        // - block_out_channels: [128, 256, 512, 512]
        let mut config = stable_diffusion::vae::AutoEncoderKLConfig::default();
        config.block_out_channels = vec![128, 256, 512, 512];
        config.latent_channels = 16;

        // Parameters: in_channels=3 (RGB input), out_channels=3 (RGB output)
        let vae = stable_diffusion::vae::AutoEncoderKL::new(vb, 3, 3, config)?;

        Ok(Self { vae, device })
    }

    /// Decode latent tensor to RGB image
    ///
    /// # Arguments
    /// * `latents` - Latent tensor [1, 16, H/8, W/8] (FLUX uses 16-channel latents)
    ///
    /// # Returns
    /// RGB tensor [1, 3, H, W] with values in range [0, 1]
    pub fn decode(&self, latents: &Tensor) -> Result<Tensor> {
        // VAE decode
        let image = self.vae.decode(latents)?;

        // Scale from [-1, 1] to [0, 1]
        let image = ((image + 1.0)? * 0.5)?;

        // Clamp to valid range
        let image = image.clamp(0.0, 1.0)?;

        Ok(image)
    }

    /// Convert tensor to RGB image buffer
    ///
    /// # Arguments
    /// * `tensor` - Image tensor [1, 3, H, W]
    ///
    /// # Returns
    /// Vec<u8> with RGB pixel data (H * W * 3 bytes)
    pub fn tensor_to_rgb(&self, tensor: &Tensor) -> Result<Vec<u8>> {
        // Remove batch dimension and permute to HWC
        let image = tensor.squeeze(0)?.permute((1, 2, 0))?;

        // Convert to u8 (0-255 range)
        let image = (image * 255.0)?.to_dtype(candle_core::DType::U8)?;

        // Flatten to Vec<u8>
        Ok(image.flatten_all()?.to_vec1()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires downloaded model
    fn test_vae_loading() {
        use crate::models::ModelPaths;

        let paths = ModelPaths::new().unwrap();
        let device = Device::cuda_if_available(0).unwrap();

        let _vae = VaeDecoder::load(
            paths.vae_path().join("diffusion_pytorch_model.safetensors"),
            device,
        ).unwrap();
    }
}
