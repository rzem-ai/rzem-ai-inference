# Candle Ecosystem Research for Z-Image-Turbo

## Executive Summary

✅ **Good News**: Both Qwen3 and SigLIP have native Candle implementations!
✅ **Even Better News**: Z-Image-Turbo does NOT need SigLIP (only Z-Image-Edit does)!

## Required Components for Z-Image-Turbo

### Components We Need:
1. **Qwen3-4B Text Encoder** - ✅ Supported in Candle
2. **FLUX VAE Decoder** - ✅ Already implemented in our codebase
3. **ZImageTransformer2DModel** - ❌ Needs custom implementation
4. **FlowMatchEulerDiscreteScheduler** - ⚠️ May need custom implementation

### Components We DON'T Need:
1. **SigLIP** - ❌ Only for Z-Image-Edit (image editing variant)

## Critical Architecture Finding

From the [Z-Image architecture article](https://medium.com/@akdemir_bahadir/what-makes-z-image-so-efficient-part-2-architecture-training-9bae9d7d947e) and [arXiv paper](https://arxiv.org/html/2511.22699v2):

> "Exclusively for editing tasks, the architecture is augmented with SigLIP 2 to capture abstract visual semantics from reference images. The visual semantic token components are disabled in the base (Turbo) model rather than completely removed."

**Z-Image-Turbo** (text-to-image):
- Qwen3-4B text tokens
- FLUX VAE image tokens
- NO SigLIP tokens

**Z-Image-Edit** (image-to-image):
- Qwen3-4B text tokens
- FLUX VAE image tokens
- SigLIP 2 visual semantic tokens

## Candle Implementation Status

### 1. Qwen3 - ✅ FULLY SUPPORTED

**Location**: `candle-transformers/src/models/qwen3.rs`

**Key Structs**:
- `Config` - Model parameters (vocab_size, hidden_size, etc.)
- `Qwen3RotaryEmbedding` - RoPE implementation
- `Qwen3MLP` - Feed-forward layer
- `Qwen3Attention` - Attention mechanism with GQA support
- `DecoderLayer` - Combined self-attention and MLP
- `Model` - Base transformer
- `ModelForCausalLM` - Language modeling head

**Features**:
- ✅ Grouped Query Attention (GQA)
- ✅ Per-head RMS normalization
- ✅ Rotary Position Embeddings (RoPE)
- ✅ KV cache management
- ✅ Tied word embeddings support

**Configuration Match**:
Our Z-Image Qwen3 config:
```json
{
  "hidden_size": 2560,
  "num_hidden_layers": 36,
  "num_attention_heads": 32,
  "num_key_value_heads": 8,
  "intermediate_size": 9728,
  "vocab_size": 151936,
  "head_dim": 128
}
```

✅ All parameters supported by Candle's Qwen3 implementation!

**Sources**:
- [Candle Repository](https://github.com/huggingface/candle)
- [Candle Qwen3 implementation](https://github.com/huggingface/candle/blob/main/candle-transformers/src/models/qwen3.rs)
- [Qwen3 issue tracker](https://github.com/huggingface/candle/issues/3004)

### 2. SigLIP - ✅ SUPPORTED (But Not Needed for Turbo)

**Location**: `candle-transformers/src/models/siglip.rs`

**Features**:
- ✅ Text and vision encoders
- ✅ Dynamic position encoding interpolation
- ✅ Variable image size support
- ✅ Zero-shot classification

**Example Usage**:
```bash
cargo run --features cuda --example siglip
```

**Notes**:
- SigLIP 2 support requested but not yet implemented ([Issue #2799](https://github.com/huggingface/candle/issues/2799))
- Only needed for Z-Image-Edit (future work)
- Can be ignored for initial Z-Image-Turbo implementation

**Sources**:
- [SigLIP implementation](https://github.com/huggingface/candle/blob/main/candle-transformers/src/models/siglip.rs)
- [Dynamic position encoding PR](https://github.com/huggingface/candle/pull/2770)

### 3. VAE Decoder - ✅ ALREADY IMPLEMENTED

**Status**: Our codebase already has FLUX VAE implementation

**Reusability**: Z-Image uses FLUX's VAE with identical configuration!

```json
{
  "_name_or_path": "flux-dev",
  "latent_channels": 16,
  "scaling_factor": 0.3611,
  "shift_factor": 0.1159
}
```

✅ No additional work needed for VAE!

### 4. ZImageTransformer2DModel - ❌ CUSTOM IMPLEMENTATION REQUIRED

**Status**: No Candle implementation exists

**Complexity**: High - requires porting from diffusers

**Architecture**: Single-stream DiT
- 30 transformer layers + 2 refiner layers
- Hidden dim: 3840
- 30 attention heads (no GQA)
- Caption feature dim: 2560 (matches Qwen3)
- RoPE theta: 256.0

**Source Material**:
- Python implementation in diffusers (merged PRs #12703, #12715)
- Must port from PyTorch to Candle/Rust

### 5. FlowMatchEulerDiscreteScheduler - ⚠️ INVESTIGATION NEEDED

**Status**: Check if Candle has flow matching scheduler

**Configuration**:
```json
{
  "num_train_timesteps": 1000,
  "shift": 3.0,
  "use_dynamic_shifting": false
}
```

**Fallback**: Implement custom scheduler (similar to FLUX's Euler sampler)

## Implementation Complexity Assessment

### Easy (Can Reuse):
1. ✅ **VAE Decoder** - Already implemented (FLUX's VAE)
2. ✅ **Qwen3 Encoder** - Use Candle's implementation with minor integration work

### Medium (Requires Integration):
1. ⚠️ **Scheduler** - May need custom implementation, but similar to FLUX
2. ⚠️ **Pipeline Orchestration** - New ZImagePipeline to coordinate components

### Hard (Requires Full Port):
1. ❌ **ZImageTransformer2DModel** - Complete port from Python/PyTorch to Rust/Candle
   - 32 transformer layers
   - Single-stream architecture (different from FLUX)
   - RoPE, attention, MLP, normalization layers
   - Token concatenation logic

## Revised Implementation Plan

### Phase 1: Foundation (Week 1-2)
✅ Model downloaded and documented
✅ Candle ecosystem researched
⏭️ Create Python test dataset

### Phase 2: Easy Components (Week 2-3)
- Integrate Candle Qwen3 encoder
- Verify VAE reusability
- Test Qwen3 encoding with sample prompts

### Phase 3: Medium Components (Week 3-4)
- Implement/adapt FlowMatchEulerDiscreteScheduler
- Create ZImagePipeline scaffold
- Set up model loading infrastructure

### Phase 4: Hard Component (Week 4-7)
- Port ZImageTransformer2DModel from diffusers
- Implement single-stream token concatenation
- Test transformer forward pass with reference outputs
- Debug and optimize

### Phase 5: Integration & Testing (Week 7-9)
- Complete end-to-end pipeline
- Verify against Python reference
- Optimize memory usage
- Create quantized version

## Risk Mitigation Updates

### Original Risk: Candle Implementation Complexity
**Status**: ✅ MITIGATED
- Qwen3 is fully supported in Candle
- SigLIP not needed for Turbo variant
- Only ZImageTransformer requires porting

### Original Risk: No SigLIP Support
**Status**: ✅ RESOLVED
- SigLIP only needed for Edit variant
- Turbo variant uses Qwen3 + VAE tokens only
- Can defer SigLIP integration to future Z-Image-Edit work

### Remaining Risk: ZImageTransformer Port
**Probability**: High | **Impact**: Critical

**Mitigation**:
- Study Python implementation thoroughly
- Port layer-by-layer with unit tests
- Compare intermediate outputs with Python
- Engage Candle community if blocked

## Next Steps

1. ✅ Phase 1.1: Model downloaded
2. ✅ Phase 1.2: Candle research complete
3. ⏭️ Phase 1.3: Create Python test dataset
4. ⏭️ Phase 2: Integrate Qwen3 encoder

## References

- [Z-Image Architecture (Medium)](https://medium.com/@akdemir_bahadir/what-makes-z-image-so-efficient-part-2-architecture-training-9bae9d7d947e)
- [Z-Image arXiv Paper](https://arxiv.org/html/2511.22699v2)
- [Candle Repository](https://github.com/huggingface/candle)
- [Candle Qwen3 Implementation](https://github.com/huggingface/candle/blob/main/candle-transformers/src/models/qwen3.rs)
- [Candle SigLIP Implementation](https://github.com/huggingface/candle/blob/main/candle-transformers/src/models/siglip.rs)
- [HuggingFace Transformers SigLIP](https://github.com/huggingface/transformers/blob/main/src/transformers/models/siglip/modeling_siglip.py)
- [Candle-VLLM Qwen3 Support](https://github.com/EricLBuehler/candle-vllm)
