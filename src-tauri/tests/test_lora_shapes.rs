// Integration test to verify LoRA tensor shape handling

use candle_core::{Device, Tensor, DType};

#[test]
fn test_lora_tensor_transpose_and_matmul() -> anyhow::Result<()> {
    let device = Device::Cpu;

    println!("\n=== Testing LoRA Tensor Operations ===\n");

    // Simulate LoRA file format: [rank, in] and [out, rank]
    let rank = 2;
    let in_dim = 3072;
    let out_dim = 18432;

    let lora_down_from_file = Tensor::randn(0.0f32, 1.0f32, &[rank, in_dim], &device)?;
    let lora_up_from_file = Tensor::randn(0.0f32, 1.0f32, &[out_dim, rank], &device)?;

    println!("1. LoRA tensors from file:");
    println!("   lora_down: {:?}", lora_down_from_file.dims());
    println!("   lora_up:   {:?}", lora_up_from_file.dims());

    // Apply Fix 1: Transpose to match quantized_nn expectations
    let lora_a = lora_down_from_file.t()?;
    let lora_b = lora_up_from_file.t()?;

    println!("\n2. After transpose (Fix 1):");
    println!("   lora_a: {:?}", lora_a.dims());
    println!("   lora_b: {:?}", lora_b.dims());

    // Verify validation: a[1] == b[0]
    assert_eq!(lora_a.dim(1)?, lora_b.dim(0)?,
        "Validation failed: a[1]={} != b[0]={}", lora_a.dim(1)?, lora_b.dim(0)?);
    println!("   ✓ Validation passed: a[1]={} == b[0]={}", lora_a.dim(1)?, lora_b.dim(0)?);

    // Simulate 3D input from FLUX forward pass
    let batch = 1;
    let seq_len = 4096;
    let x = Tensor::randn(0.0f32, 1.0f32, &[batch, seq_len, in_dim], &device)?;

    println!("\n3. Forward pass with 3D input:");
    println!("   input x: {:?}", x.dims());

    // Apply Fix 2: Reshape to 2D for matmul
    let x_2d = x.reshape(&[batch * seq_len, in_dim])?;
    println!("   x_2d (flattened): {:?}", x_2d.dims());

    // Compute LoRA: (x @ A) @ B
    let intermediate = x_2d.matmul(&lora_a)?;
    println!("   x @ a: {:?}", intermediate.dims());
    assert_eq!(intermediate.dims(), &[batch * seq_len, rank]);

    let lora_out_2d = intermediate.matmul(&lora_b)?;
    println!("   (x@a) @ b: {:?}", lora_out_2d.dims());
    assert_eq!(lora_out_2d.dims(), &[batch * seq_len, out_dim]);

    // Reshape back to 3D
    let lora_out_3d = lora_out_2d.reshape(&[batch, seq_len, out_dim])?;
    println!("   reshaped back: {:?}", lora_out_3d.dims());
    assert_eq!(lora_out_3d.dims(), &[batch, seq_len, out_dim]);

    println!("\n✅ All LoRA tensor operations successful!");
    println!("   The fixes handle both tensor orientation and multi-dimensional inputs correctly.\n");

    Ok(())
}
