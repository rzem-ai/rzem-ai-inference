"""
Generate reference images with Z-Image-Turbo for validation.
This script creates a test dataset to verify Rust implementation correctness.
"""

import torch
import json
from pathlib import Path
from diffusers import ZImagePipeline
from PIL import Image
import sys

# Test prompts with varying complexity
TEST_CASES = [
    {
        "prompt": "A serene mountain landscape at sunset",
        "seed": 12345,
        "description": "Simple landscape scene"
    },
    {
        "prompt": "A red cat sitting on a blue chair",
        "seed": 42,
        "description": "Object with colors"
    },
    {
        "prompt": "Photorealistic portrait of a young woman with long brown hair",
        "seed": 999,
        "description": "Photorealistic human"
    },
    {
        "prompt": "Abstract geometric shapes in vibrant colors",
        "seed": 2024,
        "description": "Abstract art"
    },
    {
        "prompt": "A futuristic city with flying cars at night",
        "seed": 777,
        "description": "Complex scene"
    },
    {
        "prompt": "Sign that says 'HELLO WORLD' in bold letters",
        "seed": 1111,
        "description": "Text rendering"
    },
    {
        "prompt": "一个美丽的中国传统建筑",  # Chinese text
        "seed": 8888,
        "description": "Chinese prompt"
    },
    {
        "prompt": "Minimalist white room with a single chair",
        "seed": 333,
        "description": "Minimalist scene"
    }
]

def main():
    output_dir = Path("/tmp/zimage-reference-dataset")
    output_dir.mkdir(exist_ok=True)
    
    print("Loading Z-Image-Turbo pipeline...")
    print("This will download the model if not cached (~30GB)")
    
    try:
        # Load pipeline with bfloat16 for optimal performance
        pipe = ZImagePipeline.from_pretrained(
            "Tongyi-MAI/Z-Image-Turbo",
            torch_dtype=torch.bfloat16,
            low_cpu_mem_usage=False,
        )
        pipe.to("cuda")
        print("✓ Pipeline loaded successfully")
    except Exception as e:
        print(f"✗ Failed to load pipeline: {e}")
        print("\nPlease ensure:")
        print("  1. PyTorch with CUDA is installed")
        print("  2. Latest diffusers installed: pip install git+https://github.com/huggingface/diffusers")
        print("  3. GPU with at least 16GB VRAM available")
        sys.exit(1)
    
    # Optional: Enable Flash Attention if supported
    try:
        pipe.transformer.set_attention_backend("flash")
        print("✓ Flash Attention enabled")
    except:
        print("⚠ Flash Attention not available, using SDPA")
    
    # Generation parameters (fixed for Z-Image-Turbo)
    width = 1024
    height = 1024
    num_inference_steps = 9  # Results in 8 DiT forwards
    guidance_scale = 0.0      # Must be 0.0 for Turbo models
    
    metadata_list = []
    
    print(f"\nGenerating {len(TEST_CASES)} reference images...")
    print("=" * 60)
    
    for idx, test_case in enumerate(TEST_CASES, 1):
        prompt = test_case["prompt"]
        seed = test_case["seed"]
        description = test_case["description"]
        
        print(f"\n[{idx}/{len(TEST_CASES)}] {description}")
        print(f"  Prompt: {prompt}")
        print(f"  Seed: {seed}")
        
        # Set random seed for reproducibility
        generator = torch.Generator("cuda").manual_seed(seed)
        
        # Generate image
        try:
            result = pipe(
                prompt=prompt,
                height=height,
                width=width,
                num_inference_steps=num_inference_steps,
                guidance_scale=guidance_scale,
                generator=generator,
            )
            image = result.images[0]
            
            # Save image
            image_filename = f"test_{idx:02d}_seed{seed}.png"
            image_path = output_dir / image_filename
            image.save(image_path)
            print(f"  ✓ Saved: {image_filename}")
            
            # Save metadata
            metadata = {
                "id": idx,
                "prompt": prompt,
                "seed": seed,
                "description": description,
                "width": width,
                "height": height,
                "num_inference_steps": num_inference_steps,
                "guidance_scale": guidance_scale,
                "image_filename": image_filename,
                "model": "Z-Image-Turbo",
                "torch_dtype": "bfloat16"
            }
            metadata_list.append(metadata)
            
        except Exception as e:
            print(f"  ✗ Generation failed: {e}")
            continue
    
    # Save metadata JSON
    metadata_path = output_dir / "metadata.json"
    with open(metadata_path, 'w', encoding='utf-8') as f:
        json.dump(metadata_list, f, indent=2, ensure_ascii=False)
    
    print("\n" + "=" * 60)
    print(f"✓ Complete! Generated {len(metadata_list)} reference images")
    print(f"  Output directory: {output_dir}")
    print(f"  Metadata: {metadata_path}")
    print("\nThese images will be used to validate the Rust implementation.")

if __name__ == "__main__":
    main()
