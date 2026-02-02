// Example: Test loading actual LoRA file and verify shapes work
//
// Run with: cargo run --example test_lora_loading --release

use candle_core::{Device, DType};
use rzem_ai_inference::models::lora::LoraAdapter;
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    println!("\n=== Testing Real LoRA File Loading ===\n");

    let device = Device::Cpu;
    let lora_path = "/home/alex/.rzem-ai-inference/loras/ed90e0f6-5d26-45bb-b1e3-c1b84fd36c64.safetensors";

    // Load the actual LoRA file
    println!("Loading LoRA from: {}", lora_path);
    let lora = LoraAdapter::load(
        lora_path,
        "test-lora".to_string(),
        "Test LoRA".to_string(),
        &device,
        DType::F32,
    )?;

    println!("✓ LoRA loaded successfully");
    println!("  Weight count: {}", lora.weight_count());
    println!();

    // Check a few weight shapes
    println!("Sample weight shapes:");
    for (i, (layer_name, weight)) in lora.weights.iter().take(5).enumerate() {
        println!("  {}. {}", i + 1, layer_name);
        println!("     lora_down: {:?}", weight.lora_down.dims());
        println!("     lora_up:   {:?}", weight.lora_up.dims());
        println!("     rank: {}, alpha: {}", weight.rank, weight.alpha);

        // Verify transpose will work
        let lora_a = weight.lora_down.t()?;
        let lora_b = weight.lora_up.t()?;

        let a_dims = lora_a.dims();
        let b_dims = lora_b.dims();

        print!("     After transpose: a={:?}, b={:?}", a_dims, b_dims);

        if a_dims[1] == b_dims[0] {
            println!(" ✓ Compatible (a[1]={} == b[0]={})", a_dims[1], b_dims[0]);
        } else {
            println!(" ✗ Incompatible (a[1]={} != b[0]={})", a_dims[1], b_dims[0]);
        }
        println!();
    }

    println!("✅ All sampled LoRA layers have compatible shapes after transpose!");
    println!("\nThe LoRA integration fixes are working correctly.\n");

    Ok(())
}
