# Z-Image-Turbo Scheduler Analysis

**Date**: 2026-01-25
**Status**: ✅ Complete - Can reuse FLUX sampling with minor modifications

---

## Scheduler Configuration

Z-Image-Turbo uses `FlowMatchEulerDiscreteScheduler` with the following config:

```json
{
  "_class_name": "FlowMatchEulerDiscreteScheduler",
  "_diffusers_version": "0.36.0.dev0",
  "num_train_timesteps": 1000,
  "use_dynamic_shifting": false,
  "shift": 3.0
}
```

**Location**: `/tmp/z-image-turbo/scheduler/scheduler_config.json`

---

## Key Findings

### ✅ FLUX Compatibility

Z-Image-Turbo's scheduler is **nearly identical** to FLUX's FlowMatch sampling!

**Both use**:
- Flow matching paradigm (continuous normalizing flows)
- Euler discretization method
- Timestep scheduling with optional shifting
- Simple Euler integration: `img = img + pred * (t_prev - t_curr)`

**Differences**:
| Parameter | FLUX | Z-Image-Turbo |
|-----------|------|---------------|
| Dynamic shifting | Optional (image_seq_len dependent) | **Disabled** (`use_dynamic_shifting: false`) |
| Shift value | Calculated: `(image_seq_len, base_shift, max_shift)` | **Constant: 3.0** |
| Steps | Variable (4 for Schnell, 28 for Dev) | **Fixed: 8** |

---

## Implementation Strategy

### Option 1: Reuse FLUX Sampling (Recommended ✅)

**Approach**: Adapt `candle_transformers::models::flux::sampling` for Z-Image

**Files to leverage**:
- `~/.cargo/registry/src/.../candle-transformers-0.8.4/src/models/flux/sampling.rs`

**Key functions**:
```rust
pub fn get_schedule(num_steps: usize, shift: Option<(usize, f64, f64)>) -> Vec<f64>
pub fn denoise<M: super::WithForward>(model, img, ..., timesteps, guidance) -> Result<Tensor>
```

**Modifications needed**:
1. **Timestep schedule**: Call `get_schedule(8, Some((..., 3.0, 3.0)))` to apply constant shift of 3.0
2. **State struct**: Simplify - Z-Image only needs `(img, qwen3_hidden_states)`, not `(img, img_ids, txt, txt_ids, vec)`
3. **Forward pass**: Single-stream architecture - concatenate Qwen3 tokens + image tokens before calling transformer

**Code pattern**:
```rust
// Z-Image scheduler
let timesteps = get_schedule(8, Some((image_seq_len, 3.0, 3.0))); // Constant shift

// Z-Image denoising loop (simplified)
for window in timesteps.windows(2) {
    let (t_curr, t_prev) = (window[0], window[1]);
    let t_vec = Tensor::full(t_curr as f32, batch_size, device)?;

    // Single-stream forward (no img_ids, txt_ids, vec like FLUX)
    let pred = transformer.forward(&img, &qwen3_hidden_states, &t_vec)?;

    img = (img + pred * (t_prev - t_curr))?; // Euler step
}
```

**Advantages**:
- ✅ Minimal implementation effort
- ✅ Proven sampling approach (same as FLUX)
- ✅ Already in Candle ecosystem
- ✅ Well-tested and optimized

**Disadvantages**:
- ⚠️ Need to adapt State struct for single-stream architecture
- ⚠️ FLUX's `denoise()` expects `WithForward` trait that Z-Image transformer won't implement identically

---

### Option 2: Implement FlowMatchEulerDiscreteScheduler from Scratch

**Approach**: Port diffusers' `FlowMatchEulerDiscreteScheduler`

**Reference**:
- https://github.com/huggingface/diffusers/blob/main/src/diffusers/schedulers/scheduling_flow_match_euler_discrete.py

**Advantages**:
- ✅ Exact match with Z-Image training scheduler
- ✅ Full control over implementation

**Disadvantages**:
- ❌ More implementation work
- ❌ Needs testing/validation
- ❌ Duplicates similar code already in Candle

**Verdict**: Not recommended - Option 1 is simpler and sufficient.

---

## Recommended Implementation Plan

### Phase 3 (Current): Document findings ✅

### Phase 4 (Transformer): Implement forward pass compatible with Euler sampling

The Z-Image transformer's `forward()` method should match this signature:
```rust
impl ZImageTransformer {
    pub fn forward(
        &self,
        img: &Tensor,               // [batch, img_seq_len, hidden_dim]
        qwen3_hidden_states: &Tensor, // [batch, text_seq_len, 2560]
        timestep: &Tensor,          // [batch] current timestep
    ) -> Result<Tensor> {
        // 1. Concatenate qwen3 + img tokens (single-stream)
        // 2. Add timestep embedding
        // 3. Run through transformer layers
        // 4. Return predicted noise/velocity
    }
}
```

### Phase 5 (Pipeline): Implement generation with adapted FLUX sampling

```rust
// In ZIndexPipeline::generate()
use candle_transformers::models::flux::sampling::{get_schedule, get_noise, unpack};

// 1. Encode text with Qwen3
let qwen3_hidden_states = self.qwen3.encode(prompt)?;

// 2. Generate initial noise
let img = get_noise(1, height, width, &self.device)?;

// 3. Create timestep schedule (8 steps, constant shift 3.0)
let image_seq_len = (height / 16) * (width / 16);
let timesteps = get_schedule(8, Some((image_seq_len, 3.0, 3.0)));

// 4. Denoise (Euler integration)
let mut img = img;
for window in timesteps.windows(2) {
    let (t_curr, t_prev) = (window[0], window[1]);
    let t_vec = Tensor::full(t_curr as f32, 1, &self.device)?;

    let pred = self.zimage.forward(&img, &qwen3_hidden_states, &t_vec)?;
    img = (img + pred * (t_prev - t_curr))?;
}

// 5. Unpack latents
let latents = unpack(&img, height, width)?;

// 6. Decode with VAE
let decoded = self.vae.decode(&latents)?;
```

---

## Key Insights

### 🎯 Architecture Similarity

Z-Image-Turbo and FLUX share the same core diffusion approach:
- **Flow matching**: Both use continuous normalizing flows (rectified flows)
- **Euler discretization**: Simple one-step integration
- **No classifier guidance**: Z-Image is distilled (guidance=0.0), FLUX supports it optionally

This is why Z-Image chose 8 steps - it's the sweet spot for distilled flow matching models.

### 📊 Timestep Shifting

The `shift: 3.0` parameter controls how timesteps are distributed:
- **Higher shift** = More steps near t=1.0 (early denoising)
- **Lower shift** = More uniform distribution

Z-Image uses constant shift (simpler than FLUX's dynamic shifting based on image resolution).

### 🔄 Single-Stream Advantage

Z-Image's single-stream architecture (text + image tokens concatenated) simplifies the denoising loop:
- No separate `img_ids`, `txt_ids`, `vec` tensors like FLUX
- Just `(img, qwen3_hidden_states, timestep)` → `pred`

This makes adaptation straightforward.

---

## Conclusion

**✅ No custom scheduler implementation needed**

We can reuse FLUX's sampling infrastructure with minimal changes:
1. Use `get_schedule(8, Some((seq_len, 3.0, 3.0)))` for timesteps
2. Implement simplified denoising loop (no img_ids/txt_ids complexity)
3. Leverage existing `get_noise()` and `unpack()` utilities

**Next Steps**: Proceed with Phase 4 (ZImageTransformer implementation) knowing the sampling approach is validated.
