# Z-Image-Turbo Reference Dataset Generation

## Purpose

This reference dataset is critical for validating the Rust/Candle implementation of Z-Image-Turbo. By generating images with the official Python implementation, we can:

1. **Verify correctness**: Compare Rust outputs pixel-by-pixel with Python outputs
2. **Debug issues**: Identify which component differs (encoder, transformer, VAE, etc.)
3. **Performance baseline**: Measure speed improvements vs Python
4. **Regression testing**: Ensure changes don't break functionality

## Prerequisites

Before running the reference generation script, ensure you have:

### 1. CUDA-enabled PyTorch
```bash
# Check if PyTorch with CUDA is installed
python -c "import torch; print(f'CUDA available: {torch.cuda.is_available()}')"
```

If not installed:
```bash
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu124
```

### 2. Latest diffusers from source
```bash
pip install git+https://github.com/huggingface/diffusers
```

**Why from source?** Z-Image support was added in PRs #12703 and #12715, which are merged but may not be in the latest PyPI release yet.

### 3. GPU with 16GB+ VRAM
```bash
# Check GPU memory
nvidia-smi --query-gpu=memory.total --format=csv,noheader
```

Z-Image-Turbo requires ~16GB VRAM in bfloat16 precision.

### 4. Other dependencies
```bash
pip install pillow transformers accelerate
```

## Running the Script

### Quick Start
```bash
python /tmp/generate_zimage_references.py
```

### What It Does

The script will:
1. Download Z-Image-Turbo model (~30GB) if not cached
2. Generate 8 reference images with different prompts:
   - Simple landscape
   - Objects with colors
   - Photorealistic portrait
   - Abstract art
   - Complex scene
   - Text rendering (English)
   - Chinese text rendering
   - Minimalist scene
3. Save images to `/tmp/zimage-reference-dataset/`
4. Save metadata JSON with prompts, seeds, and parameters

### Expected Output

```
/tmp/zimage-reference-dataset/
├── test_01_seed12345.png
├── test_02_seed42.png
├── test_03_seed999.png
├── test_04_seed2024.png
├── test_05_seed777.png
├── test_06_seed1111.png
├── test_07_seed8888.png
├── test_08_seed333.png
└── metadata.json
```

### Metadata Format

```json
[
  {
    "id": 1,
    "prompt": "A serene mountain landscape at sunset",
    "seed": 12345,
    "description": "Simple landscape scene",
    "width": 1024,
    "height": 1024,
    "num_inference_steps": 9,
    "guidance_scale": 0.0,
    "image_filename": "test_01_seed12345.png",
    "model": "Z-Image-Turbo",
    "torch_dtype": "bfloat16"
  },
  ...
]
```

## Using the Reference Dataset

### During Rust Implementation

When implementing the Rust/Candle version:

1. **Component Testing**: After implementing each component (Qwen3, Transformer, VAE), test with the same prompts and seeds
2. **Output Comparison**: Compare intermediate tensors (text embeddings, latents, decoded images)
3. **Visual Inspection**: Look for obvious artifacts or differences
4. **Quantitative Metrics**: Use SSIM, MSE, or perceptual similarity metrics

### Example Validation Code (Rust)

```rust
// Load reference metadata
let metadata: Vec<ReferenceCase> = serde_json::from_str(&metadata_json)?;

for case in metadata {
    // Generate with Rust implementation
    let rust_image = pipeline.generate(
        &case.prompt,
        8,  // steps
        1024, 1024,  // width, height
        0.0,  // guidance
        case.seed,
    )?;
    
    // Load Python reference
    let python_image = image::open(&case.image_filename)?;
    
    // Compare
    let similarity = compute_similarity(&rust_image, &python_image);
    assert!(similarity > 0.95, "Output differs significantly");
}
```

## Troubleshooting

### Issue: "CUDA out of memory"
**Solution**: Enable CPU offloading:
```python
pipe.enable_model_cpu_offload()
```

### Issue: "ZImagePipeline not found"
**Solution**: Ensure diffusers is from source:
```bash
pip uninstall diffusers
pip install git+https://github.com/huggingface/diffusers
```

### Issue: "Slow generation (>10 seconds per image)"
**Solution**: Enable Flash Attention (if supported):
```python
pipe.transformer.set_attention_backend("flash")
```

Or compile the model:
```python
pipe.transformer.compile()
```

## Notes

- **Reproducibility**: Using fixed seeds ensures deterministic outputs
- **Bilingual Testing**: Chinese prompt tests language-agnostic capabilities
- **Text Rendering**: Tests Z-Image-Turbo's text-in-image feature
- **Variety**: Different scene types stress different model capabilities

## Next Steps

After generating references:
1. ✅ Store reference dataset in a safe location
2. ⏭️ Begin Rust implementation (Phase 2)
3. ⏭️ Validate each component against references
4. ⏭️ Achieve pixel-perfect parity (or document acceptable differences)
