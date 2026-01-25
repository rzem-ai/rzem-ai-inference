# Phase 3 Progress Report
## Z-Image-Turbo Integration - Pipeline Scaffold

**Status**: ✅ COMPLETE
**Date**: 2026-01-25

---

## Overview

Phase 3 prepared the pipeline infrastructure for Z-Image-Turbo before implementing the complex transformer (Phase 4). All foundational components are in place.

**Note**: Original plan's "Phase 3: Text Encoders" was completed early:
- Qwen3: ✅ Implemented in Phase 2
- SigLIP: ❌ Not needed for Z-Image-Turbo (only for Edit variant)

This phase focused on pipeline scaffolding and scheduler research.

---

## Completed Tasks

### ✅ Task 1: Update ZIndexPipeline Struct with Model Fields

**Files Modified**:
- `src-tauri/src/models/zimage.rs` (CREATED - 81 lines)
- `src-tauri/src/models/mod.rs` (added ZImageTransformer export)
- `src-tauri/src/inference/zindex_pipeline/mod.rs` (updated struct)

**Changes**:

1. **Created ZImageTransformer stub**:
   ```rust
   pub struct ZImageTransformer {
       device: Device,
   }

   impl ZImageTransformer {
       pub fn load<P: AsRef<Path>>(_model_dir: P, device: Device) -> Result<Self>
       pub fn forward(&self, ...) -> Result<Tensor> // Stub - Phase 4 task
   }
   ```

2. **Updated ZIndexPipeline struct**:
   ```rust
   pub struct ZIndexPipeline {
       pub(crate) device: Device,
       pub(crate) qwen3: Option<Qwen3TextEncoder>,
       pub(crate) vae: Option<VaeDecoder>,
       pub(crate) zimage: Option<ZImageTransformer>,
       pub(crate) models_loaded_this_session: bool,
   }
   ```

3. **Added helper methods**:
   - `models_loaded()` → Returns `(qwen3_loaded, vae_loaded, zimage_loaded)`
   - `all_models_loaded()` → Checks if all components ready
   - `unload_models()` → Frees VRAM

**Compilation Status**: ✅ Successful

---

### ✅ Task 2: Implement Model Loading Infrastructure

**File Modified**:
- `src-tauri/src/inference/zindex_pipeline/loader.rs` (232 lines)

**Implementation**:

Followed FluxPipeline pattern with Z-Image-specific adaptations:

```rust
impl ZIndexPipeline {
    pub(crate) fn ensure_models_loaded(&mut self, stats: &mut GenerationStats) -> Result<()>
    pub fn ensure_ready_for_generation(&mut self, stats: &mut GenerationStats) -> Result<()>

    fn load_qwen3_encoder(&mut self, paths: &ModelPaths, stats: &mut GenerationStats) -> Result<()>
    fn load_vae_decoder(&mut self, paths: &ModelPaths, stats: &mut GenerationStats) -> Result<()>
    fn load_zimage_transformer(&mut self, paths: &ModelPaths, stats: &mut GenerationStats) -> Result<()>
}
```

**Features**:
- Individual model loading with skipping for already-loaded components
- GPU memory tracking via `nvidia-smi`
- Loading time statistics in `GenerationStats`
- Comprehensive logging with memory deltas
- Reuses FLUX VAE from Z-Image directory

**Key Design Decision**:
- VAE loaded from Z-Image's `vae/diffusion_pytorch_model.safetensors` (proven compatible in Phase 2)
- Timing stats reuse existing GenerationStats fields (`t5_load_ms` for Qwen3, `flux_load_ms` for transformer)

**Compilation Status**: ✅ Successful

---

### ✅ Task 3: Research and Document Z-Image Scheduler Requirements

**File Created**:
- `docs/z-image-turbo/scheduler-analysis.md` (complete analysis)

**Findings**:

1. **Scheduler Configuration**:
   ```json
   {
     "_class_name": "FlowMatchEulerDiscreteScheduler",
     "num_train_timesteps": 1000,
     "use_dynamic_shifting": false,
     "shift": 3.0
   }
   ```

2. **FLUX Compatibility**: ✅ Near-identical approach
   - Both use flow matching with Euler discretization
   - Same integration formula: `img = img + pred * (t_prev - t_curr)`
   - Z-Image uses constant shift (3.0), FLUX uses dynamic shifting

3. **Implementation Strategy**: **Reuse FLUX sampling**
   - Available in `candle_transformers::models::flux::sampling`
   - Key functions:
     - `get_schedule(num_steps, shift)` → Timestep schedule
     - `get_noise(...)` → Initial noise generation
     - `unpack(...)` → Latent unpacking
     - Euler integration loop (adapt for single-stream)

4. **No Custom Scheduler Needed**: ✅
   - FLUX's FlowMatch sampling is sufficient
   - Modifications:
     - Use `get_schedule(8, Some((seq_len, 3.0, 3.0)))` for constant shift
     - Simplify denoising loop (no `img_ids`/`txt_ids` complexity)
     - Single-stream forward: `transformer.forward(img, qwen3_hidden_states, timestep)`

**Compilation Status**: N/A (documentation only)

---

## Phase 3 Summary

**Accomplishments**:
1. ✅ Created ZImageTransformer stub for pipeline integration
2. ✅ Fully implemented model loading infrastructure (Qwen3, VAE, ZImage stub)
3. ✅ Researched and documented scheduler approach (reuse FLUX sampling)
4. ✅ All code compiles successfully
5. ✅ Comprehensive documentation for scheduler implementation

**Files Created/Modified**: 5
- `src-tauri/src/models/zimage.rs` - NEW (transformer stub)
- `src-tauri/src/models/mod.rs` - Export
- `src-tauri/src/inference/zindex_pipeline/mod.rs` - Pipeline struct
- `src-tauri/src/inference/zindex_pipeline/loader.rs` - Loading infrastructure
- `docs/z-image-turbo/scheduler-analysis.md` - NEW (scheduler research)

**Lines of Code**: ~350 lines

---

## Key Insights

### 🎯 Scheduler Simplification

The discovery that Z-Image uses nearly the same FlowMatch sampling as FLUX significantly reduces Phase 4 complexity:
- No custom scheduler implementation needed
- Can leverage battle-tested FLUX sampling code
- Only need to adapt the forward pass for single-stream architecture

### 🏗️ Pipeline Architecture

The ZIndexPipeline structure mirrors FluxPipeline:
- Same loading pattern (individual model checks)
- Same memory tracking approach
- Same statistics integration
- Easier to maintain and understand

### 🔄 Single-Stream Advantage

Z-Image's single-stream architecture eliminates complexity:
- No `img_ids`, `txt_ids`, `vec` tensors
- Just `(img, qwen3_hidden_states, timestep)` → `pred`
- Simpler denoising loop than FLUX

---

## Next Steps - Phase 4: ZImageTransformer Implementation

**Priority**: Implement the actual transformer (hardest component)

**Tasks**:
1. Port ZImageTransformer2DModel from diffusers
2. Implement single-stream token concatenation
3. Add timestep embedding
4. Implement 30 transformer layers
5. Test forward pass with reference outputs

**Estimated Complexity**: High - requires careful porting from Python to Rust

**Files to Create/Modify**:
- `src-tauri/src/models/zimage.rs` - Replace stub with full implementation
- Test cases with dummy inputs

**Note**: Phase 4 is the critical path. Once the transformer works, Phase 5 (pipeline integration) will be straightforward thanks to the scheduler research completed in this phase.

---

## Compilation Verification

Final check:
```bash
cargo check --quiet
```

**Result**: ✅ No errors (only pre-existing unused import warnings)
