# Z-Image-Turbo Model Analysis

## Download Summary

**Repository**: `Tongyi-MAI/Z-Image-Turbo`
**Total Size**: ~30.7 GB
**Status**: ✅ Successfully downloaded to `/tmp/z-image-turbo/`

## Model Components

### 1. Text Encoder - Qwen3
**Path**: `text_encoder/`
**Size**: ~7.5 GB (3 sharded safetensors files)
**Architecture**: Qwen3ForCausalLM

**Configuration**:
- Model type: `qwen3`
- Hidden size: 2560
- Layers: 36
- Attention heads: 32
- Key-value heads: 8 (Grouped Query Attention)
- Max position embeddings: 40960
- Vocab size: 151936
- dtype: bfloat16

**Files**:
- `model-00001-of-00003.safetensors` (3.7 GB)
- `model-00002-of-00003.safetensors` (3.7 GB)
- `model-00003-of-00003.safetensors` (95 MB)
- `config.json`
- `generation_config.json`
- `model.safetensors.index.json`

### 2. Tokenizer - Qwen2Tokenizer
**Path**: `tokenizer/`
**Size**: ~15 MB total

**Files**:
- `tokenizer.json` (11 MB)
- `vocab.json` (2.6 MB)
- `merges.txt` (1.6 MB)
- `tokenizer_config.json` (9.5 KB)

### 3. Transformer - ZImageTransformer2DModel
**Path**: `transformer/`
**Size**: ~23 GB (3 sharded safetensors files)

**Configuration**:
- Class: `ZImageTransformer2DModel`
- Hidden dimension: 3840
- Layers: 30 transformer + 2 refiner = 32 total
- Attention heads: 30 (n_heads = n_kv_heads, no GQA)
- Caption feature dim: 2560 (matches Qwen3 hidden_size)
- Input channels: 16 (VAE latent channels)
- RoPE theta: 256.0
- QK normalization: enabled

**Files**:
- `diffusion_pytorch_model-00001-of-00003.safetensors` (9.3 GB)
- `diffusion_pytorch_model-00002-of-00003.safetensors` (9.3 GB)
- `diffusion_pytorch_model-00003-of-00003.safetensors` (4.4 GB)
- `config.json`
- `diffusion_pytorch_model.safetensors.index.json`

### 4. VAE - AutoencoderKL
**Path**: `vae/`
**Size**: ~160 MB

**Configuration**:
- Class: `AutoencoderKL`
- Source: `flux-dev` (FLUX's VAE!)
- Latent channels: 16
- Scaling factor: 0.3611
- Shift factor: 0.1159
- Sample size: 1024

**Files**:
- `diffusion_pytorch_model.safetensors` (160 MB)
- `config.json`

**NOTE**: This is FLUX's VAE, so we can reuse our existing VAE implementation!

### 5. Scheduler - FlowMatchEulerDiscreteScheduler
**Path**: `scheduler/`

**Configuration**:
- Class: `FlowMatchEulerDiscreteScheduler`
- Num train timesteps: 1000
- Shift: 3.0
- Use dynamic shifting: false

**Files**:
- `scheduler_config.json`

## Model Index

```json
{
    "_class_name": "ZImagePipeline",
    "_diffusers_version": "0.36.0.dev0",
    "scheduler": ["diffusers", "FlowMatchEulerDiscreteScheduler"],
    "text_encoder": ["transformers", "Qwen3Model"],
    "tokenizer": ["transformers", "Qwen2Tokenizer"],
    "transformer": ["diffusers", "ZImageTransformer2DModel"],
    "vae": ["diffusers", "AutoencoderKL"]
}
```

## Architecture: Single-Stream DiT (S3-DiT)

According to the README:
> "text, visual semantic tokens, and image VAE tokens are concatenated at the sequence level to serve as a unified input stream"

**Key difference from FLUX**: 
- FLUX uses dual-stream (separate text and image paths)
- Z-Image uses single-stream (all tokens concatenated)

## Generation Parameters

**Fixed parameters** (per README and plan):
- **Steps**: 8 (actually 9 num_inference_steps in code, which results in 8 DiT forwards)
- **Guidance scale**: 0.0 (distilled model, no CFG)
- **Sampler**: FlowMatchEulerDiscreteScheduler (built-in, not user-selectable)

**Adjustable parameters**:
- Prompt
- Width/Height
- Seed

## VRAM Requirements

From README:
- **Full precision (bfloat16)**: Fits within **16GB VRAM** consumer devices
- **With CPU offloading**: Can work on lower VRAM devices
- **Optimizations**: Flash Attention, model compilation supported

## Comparison with Plan Assumptions

| Aspect | Plan Assumption | Reality |
|--------|----------------|---------|
| Text Encoder 1 | Qwen3-4B | ✅ Qwen3ForCausalLM (~7.5GB) |
| Text Encoder 2 | SigLIP-2 | ❓ Not found in model files |
| Transformer | Single-stream DiT | ✅ ZImageTransformer2DModel |
| VAE | TBD | ✅ FLUX's VAE (can reuse!) |
| Steps | 8 fixed | ✅ Confirmed (9 steps → 8 forwards) |
| Guidance | 0.0 only | ✅ Confirmed |
| VRAM | 16GB full / 8-12GB quantized | ✅ 16GB confirmed |
| Total size | Not specified | ⚠️ ~30.7GB on disk |

## Critical Finding: No SigLIP Encoder?

The plan assumes Z-Image uses both Qwen3 and SigLIP encoders, but:
- `model_index.json` only lists Qwen3Model as text_encoder
- No SigLIP files found in downloaded model
- README mentions "visual semantic tokens" but doesn't specify source

**Hypothesis**: The "visual semantic tokens" may be:
1. Generated internally by ZImageTransformer2DModel
2. Part of Qwen3's multimodal capabilities
3. Or the plan's assumption about SigLIP was incorrect

**Action needed**: Investigate ZImagePipeline source code in diffusers to understand how visual semantic tokens are generated.

## Implementation Path Forward

### What we CAN reuse:
✅ **VAE**: Existing FLUX VAE implementation works directly
✅ **Scheduler concept**: Flow matching similar to FLUX

### What we need to implement:
🔨 **Qwen3TextEncoder**: Port from HuggingFace transformers (or check candle-transformers support)
🔨 **ZImageTransformer2DModel**: Port from diffusers (no Candle equivalent exists)
🔨 **ZImagePipeline**: New pipeline orchestration

### What we need to investigate:
❓ **Visual semantic tokens**: How are they generated?
❓ **Candle support**: Does candle-transformers have Qwen3? (different from Qwen2)
❓ **Performance**: Can we achieve sub-second inference in Rust/Candle?

## Next Steps

1. ✅ Task 1 complete: Model downloaded and documented
2. ⏭️ Task 2: Research Candle ecosystem (Qwen3, potential SigLIP)
3. ⏭️ Task 3: Create Python test dataset with reference outputs
