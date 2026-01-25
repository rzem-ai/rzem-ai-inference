# Z-Image-Turbo Integration - Project Status

**Last Updated**: 2026-01-25
**Overall Progress**: 40% Complete (Phases 1-3 done, Phases 4-8 remaining)

---

## Quick Summary

**Goal**: Add Z-Image-Turbo (6B single-stream DiT) as a third model option alongside FLUX Schnell and Dev.

**Status**: Foundation complete. Text encoder (Qwen3) and pipeline infrastructure ready. **Next critical task: Implement ZImageTransformer (hardest component).**

**Timeline**:
- ✅ Completed: Phases 1-3 (3 weeks equivalent)
- 🚧 Remaining: Phases 4-8 (6 weeks estimated)
- **Total**: ~9 weeks

---

## Phase-by-Phase Status

### ✅ Phase 1: Foundation & Research (COMPLETE)

**Status**: 100% Complete
**Duration**: Week 1-2 equivalent
**Documentation**: `./phase1-completion-report.md`

**Completed Tasks**:
1. ✅ Downloaded Z-Image-Turbo model (30.7GB) to `/tmp/z-image-turbo/`
2. ✅ Researched Candle ecosystem support
3. ✅ Created Python reference dataset script

**Key Findings**:
- SigLIP NOT needed for Z-Image-Turbo (only for Edit variant)
- Qwen3 natively supported in Candle
- FLUX VAE is identical to Z-Image VAE (can reuse)

**Deliverables**:
- `./z-image-turbo-analysis.md` - Complete model structure
- `./candle-ecosystem-research.md` - Component availability analysis
- `./generate_zimage_references.py` - Reference image generation script

---

### ✅ Phase 2: Backend Core Types (COMPLETE)

**Status**: 100% Complete
**Duration**: Week 2-3 equivalent
**Documentation**: `./phase2-progress.md`

**Completed Tasks**:
1. ✅ Extended ModelType enum with ZImageTurbo variant
2. ✅ Integrated Qwen3-4B text encoder
3. ✅ Verified VAE compatibility

**Files Modified/Created**:
- `src-tauri/src/models/model_type.rs` - Added ZImageTurbo variant
- `src-tauri/src/models/paths.rs` - Added 9 Z-Image path methods
- `src-tauri/src/models/downloader.rs` - Updated for ZImageTurbo
- `src-tauri/src/models/qwen3.rs` - NEW (264 lines) - Qwen3 encoder
- `src-tauri/src/models/vae.rs` - Added Z-Image test cases

**Key Accomplishments**:
- ModelType.ZImageTurbo with correct constraints (8 steps, 0.0 guidance)
- Qwen3TextEncoder using Qwen2 architecture (compatibility)
- VAE reusability confirmed (saves memory and implementation effort)

---

### ✅ Phase 3: Pipeline Scaffold (COMPLETE)

**Status**: 100% Complete
**Duration**: Week 3 equivalent
**Documentation**: `./phase3-progress.md`, `./scheduler-analysis.md`

**Completed Tasks**:
1. ✅ Updated ZIndexPipeline struct with model fields
2. ✅ Implemented model loading infrastructure
3. ✅ Researched scheduler requirements (can reuse FLUX sampling)

**Files Modified/Created**:
- `src-tauri/src/models/zimage.rs` - NEW (81 lines) - Transformer stub
- `src-tauri/src/inference/zindex_pipeline/mod.rs` - Pipeline struct
- `src-tauri/src/inference/zindex_pipeline/loader.rs` - Loading infrastructure (232 lines)
- `./scheduler-analysis.md` - NEW - Comprehensive scheduler research

**Key Discovery**:
- Z-Image uses FlowMatchEulerDiscreteScheduler (same as FLUX!)
- Can reuse `candle_transformers::models::flux::sampling` with constant shift=3.0
- No custom scheduler implementation needed

---

### 🚧 Phase 4: ZImageTransformer Implementation (NOT STARTED)

**Status**: 0% Complete - **CRITICAL PATH**
**Estimated Duration**: Week 5-7 (3 weeks)
**Complexity**: HIGH - No Candle implementation exists

**Required Tasks**:

#### 4.1 Research Python Implementation
- [ ] Study diffusers `ZImageTransformer2DModel` source code
- [ ] Document architecture:
  - 30 transformer layers
  - Hidden dim: 3840
  - Attention heads: 32
  - FFN dim: 10,240
- [ ] Identify all sub-components (attention, MLP, normalization)
- [ ] Document tensor shapes at each stage

#### 4.2 Implement Transformer Components
- [ ] Create `ZImageAttention` module (self-attention with GQA)
- [ ] Create `ZImageMLP` module (feed-forward network)
- [ ] Create `ZImageTransformerBlock` module (combines attention + MLP)
- [ ] Add layer normalization (AdaLayerNorm with timestep conditioning)
- [ ] Implement positional embeddings

#### 4.3 Implement Single-Stream Architecture
- [ ] Token concatenation logic (Qwen3 hidden states + image latents)
- [ ] Timestep embedding (sinusoidal + MLP projection)
- [ ] Input projection (project concatenated tokens to hidden_dim=3840)
- [ ] Output projection (hidden_dim → latent space)

#### 4.4 Implement Full Transformer
- [ ] Stack 30 transformer blocks
- [ ] Add final layer norm
- [ ] Implement complete `forward()` method:
  ```rust
  pub fn forward(
      &self,
      img: &Tensor,               // [batch, img_seq_len, 64]
      qwen3_hidden_states: &Tensor, // [batch, text_seq_len, 2560]
      timestep: &Tensor,          // [batch] current timestep
  ) -> Result<Tensor>
  ```

#### 4.5 Weight Loading
- [ ] Load sharded safetensors files (3 shards, ~23GB total)
- [ ] Map Python weight keys to Rust VarBuilder paths
- [ ] Handle dtype conversion (BF16 on CUDA, F32 on CPU)
- [ ] Memory-mapped loading for efficiency

#### 4.6 Testing & Validation
- [ ] Unit tests for individual components
- [ ] Integration test: load model weights
- [ ] Validation test: compare output with Python reference
- [ ] Shape verification at each layer

**Estimated LOC**: ~800-1200 lines

**Blockers/Risks**:
- ⚠️ No Candle reference implementation (must port from Python)
- ⚠️ Complex attention mechanism with GQA
- ⚠️ Timestep conditioning integration
- ⚠️ Potential tensor shape mismatches during porting

**Files to Create/Modify**:
- `src-tauri/src/models/zimage.rs` - Replace 81-line stub with full implementation

---

### 🔲 Phase 5: Pipeline Integration (NOT STARTED)

**Status**: 0% Complete
**Estimated Duration**: Week 7-8 (1 week)
**Complexity**: MEDIUM - Scaffolding already done

**Required Tasks**:

#### 5.1 Implement Generation Method
- [ ] Update `src-tauri/src/inference/zindex_pipeline/generation.rs`
- [ ] Implement `generate()` method:
  ```rust
  pub fn generate(
      &mut self,
      prompt: &str,
      steps: usize,     // Force to 8
      width: usize,
      height: usize,
      guidance: f64,    // Force to 0.0
      seed: u64,
  ) -> Result<GenerationResult>
  ```

#### 5.2 Integrate FLUX Sampling
- [ ] Import `candle_transformers::models::flux::sampling`
- [ ] Generate initial noise: `get_noise(1, height, width, device)`
- [ ] Create timestep schedule: `get_schedule(8, Some((seq_len, 3.0, 3.0)))`
- [ ] Implement Euler denoising loop:
  ```rust
  for window in timesteps.windows(2) {
      let (t_curr, t_prev) = (window[0], window[1]);
      let t_vec = Tensor::full(t_curr as f32, 1, device)?;
      let pred = self.zimage.forward(&img, &qwen3_hidden_states, &t_vec)?;
      img = (img + pred * (t_prev - t_curr))?;
  }
  ```
- [ ] Unpack latents: `unpack(&img, height, width)`

#### 5.3 Complete Pipeline Flow
- [ ] Encode text with Qwen3
- [ ] Denoise with ZImageTransformer
- [ ] Decode with VAE
- [ ] Create ImageMetadata with correct model info
- [ ] Return GenerationResult

#### 5.4 Progress Callbacks
- [ ] Implement `generate_with_progress()` method
- [ ] Report step progress (1/8, 2/8, ..., 8/8)
- [ ] Track timing for each step

#### 5.5 Cache Integration
- [ ] Update `cache_integration.rs` if needed
- [ ] Ensure pipeline caching works with Z-Image

**Estimated LOC**: ~200 lines

**Files to Modify**:
- `src-tauri/src/inference/zindex_pipeline/generation.rs`
- `src-tauri/src/inference/zindex_pipeline/cache_integration.rs` (maybe)

---

### 🔲 Phase 6: UI Integration (NOT STARTED)

**Status**: 0% Complete
**Estimated Duration**: Week 8 (1 week)
**Complexity**: LOW - UI patterns already exist

**Required Tasks**:

#### 6.1 Update Model Store
- [ ] `src/stores/models.ts` - Add Z-Image-Turbo model entry:
  ```typescript
  {
    id: 'zimage-turbo',
    name: 'Z-Image-Turbo',
    type: 'z-image-turbo',
    isDownloaded: false,
    isActive: false,
    description: 'Single-stream DiT, 8 steps (no CFG)',
    defaultSteps: 8,
    defaultGuidance: 0.0,
    vramRequirement: 16000,  // 16GB
  }
  ```

#### 6.2 Add UI Constraints
- [ ] `src/stores/generation.ts` - Add computed properties:
  ```typescript
  const isZImageTurbo = computed(() =>
    modelsStore.activeModel?.id === 'zimage-turbo'
  )
  const canAdjustSteps = computed(() => !isZImageTurbo.value)
  const canAdjustGuidance = computed(() => !isZImageTurbo.value)
  const canSelectSampler = computed(() => !isZImageTurbo.value)
  ```

#### 6.3 Update Parameter Controls
- [ ] `src/components/generation/actions/GenerationSettings.vue`
  - Disable steps slider when `!canAdjustSteps`
  - Hide/disable guidance scale when `!canAdjustGuidance`
  - Add tooltip: "Z-Image-Turbo uses fixed 8-step schedule with guidance=0.0"

- [ ] `src/components/generation/actions/AdvancedSettings.vue`
  - Disable sampler dropdown when `!canSelectSampler`
  - Disable scheduler dropdown

#### 6.4 Update Model Selector
- [ ] `src/components/generation/actions/ModelSelector.vue`
  - Add visual distinction (badges: "Single-Stream", "8 steps fixed")
  - Show VRAM: "~16GB" (or "~8-12GB" for quantized when available)

**Estimated LOC**: ~100 lines (mostly UI template changes)

**Files to Modify**:
- `src/stores/models.ts`
- `src/stores/generation.ts`
- `src/components/generation/actions/GenerationSettings.vue`
- `src/components/generation/actions/AdvancedSettings.vue`
- `src/components/generation/actions/ModelSelector.vue`

---

### 🔲 Phase 7: Testing & Quantization (NOT STARTED)

**Status**: 0% Complete
**Estimated Duration**: Week 9 (1 week)
**Complexity**: MEDIUM

**Required Tasks**:

#### 7.1 Integration Testing
- [ ] Generate test images with Z-Image-Turbo
- [ ] Compare with Python reference outputs (from Phase 1.3)
- [ ] Test model switching: FLUX Schnell → Z-Image → FLUX Dev
- [ ] Verify memory management (FLUX components freed when switching)
- [ ] Test various prompts (English, Chinese, long prompts)
- [ ] Test different resolutions (512x512, 1024x1024, 1024x1536)
- [ ] Verify metadata embedded correctly

#### 7.2 Memory Profiling
- [ ] Profile VRAM usage with `nvidia-smi`
- [ ] Measure peak VRAM during generation
- [ ] Verify VAE sharing works (not loaded twice)
- [ ] Check for memory leaks during repeated generations
- [ ] Document actual VRAM requirements

#### 7.3 Performance Benchmarking
- [ ] Measure model loading time
- [ ] Measure generation time per image
- [ ] Compare with FLUX Schnell/Dev
- [ ] Create performance comparison table

#### 7.4 GGUF Quantization (Optional)
- [ ] Research GGUF quantization tools for transformer models
- [ ] Quantize ZImageTransformer to GGUF format
- [ ] Test multiple quantization levels (Q4, Q5, Q8)
- [ ] Compare quality vs VRAM tradeoff
- [ ] Implement `load_quantized()` method if quality acceptable

**Deliverables**:
- Performance benchmark report
- Memory profiling results
- Optional: Quantized GGUF model (~8-12GB vs 16GB)

---

### 🔲 Phase 8: Documentation & Polish (NOT STARTED)

**Status**: 0% Complete
**Estimated Duration**: Week 9 (concurrent with Phase 7)
**Complexity**: LOW

**Required Tasks**:

#### 8.1 User Documentation
- [ ] Create user guide: `./USER-GUIDE.md`
  - How to download Z-Image-Turbo
  - How to switch to Z-Image model
  - Parameter constraints (8 steps, no guidance)
  - Expected VRAM requirements
  - Troubleshooting common issues

- [ ] Create model comparison chart
  - FLUX Schnell vs Dev vs Z-Image-Turbo
  - Steps, guidance, VRAM, speed, quality

- [ ] Update main README with Z-Image support

#### 8.2 Developer Documentation
- [ ] Document architecture decisions
- [ ] Document single-stream token concatenation approach
- [ ] Document why Qwen2 was used for Qwen3 weights
- [ ] Document scheduler reuse from FLUX
- [ ] Create contribution guide for future models

#### 8.3 Code Polish
- [ ] Add missing documentation comments
- [ ] Clean up debug logging
- [ ] Add helpful error messages
- [ ] Review code for consistency

#### 8.4 Known Limitations
- [ ] Document known issues
- [ ] LoRA support status (not initially supported)
- [ ] Multi-resolution limitations
- [ ] Performance considerations

**Deliverables**:
- User guide
- Developer documentation
- Updated README
- Known limitations document

---

## Current State of Codebase

### ✅ Implemented Components

**Backend (Rust)**:
1. `src-tauri/src/models/model_type.rs` - ZImageTurbo variant with correct defaults
2. `src-tauri/src/models/paths.rs` - Z-Image path management (9 methods)
3. `src-tauri/src/models/qwen3.rs` - Qwen3-4B text encoder (264 lines, complete)
4. `src-tauri/src/models/vae.rs` - VAE compatibility tests
5. `src-tauri/src/models/zimage.rs` - Transformer stub (81 lines, needs full implementation)
6. `src-tauri/src/inference/zindex_pipeline/mod.rs` - Pipeline struct
7. `src-tauri/src/inference/zindex_pipeline/loader.rs` - Model loading (232 lines, complete)

**Documentation**:
1. `docs/z-image-turbo/z-image-turbo-analysis.md` - Model structure analysis
2. `docs/z-image-turbo/candle-ecosystem-research.md` - Component availability
3. `docs/z-image-turbo/phase1-completion-report.md` - Phase 1 report
4. `docs/z-image-turbo/phase2-progress.md` - Phase 2 report
5. `docs/z-image-turbo/phase3-progress.md` - Phase 3 report
6. `docs/z-image-turbo/scheduler-analysis.md` - Scheduler research
7. `docs/z-image-turbo/generate_zimage_references.py` - Reference generation script
8. `docs/z-image-turbo/PROJECT-STATUS.md` - This file

### 🚧 Stub/Incomplete Components

**Backend (Rust)**:
1. `src-tauri/src/models/zimage.rs` - **STUB**: Needs 800-1200 line implementation
2. `src-tauri/src/inference/zindex_pipeline/generation.rs` - **STUB**: Returns error
3. `src-tauri/src/models/downloader.rs` - Z-Image download not implemented (manual for now)

**Frontend (Vue/TypeScript)**:
1. All UI components unchanged - Phase 6 work needed

---

## Critical Path to Completion

### Immediate Next Step: Phase 4 (ZImageTransformer)

**Why Critical**: Everything else depends on this. The transformer is the core component.

**Recommended Approach**:
1. **Week 1-2**: Research and component implementation
   - Study diffusers ZImageTransformer2DModel thoroughly
   - Implement attention, MLP, transformer blocks
   - Unit test each component

2. **Week 2-3**: Full transformer assembly
   - Stack transformer blocks
   - Implement single-stream token concatenation
   - Load model weights from safetensors
   - Integration test with dummy inputs

3. **Week 3**: Validation
   - Compare outputs with Python reference
   - Fix shape mismatches
   - Optimize performance

### Then: Phase 5 (Pipeline Integration)

Once transformer works, pipeline integration is straightforward:
- Reuse FLUX sampling (already researched)
- Connect Qwen3 → Transformer → VAE
- Should take ~1 week

### Finally: Phases 6-8 (UI, Testing, Docs)

These are relatively simple once core inference works:
- UI constraints (disable controls)
- Integration tests
- Documentation
- Should take ~2 weeks total

---

## Risk Assessment

### High Risk
- **ZImageTransformer complexity**: Porting 6B transformer from Python to Rust is non-trivial
  - Mitigation: Break into small components, test incrementally
  - Fallback: Consider Python bridge if Rust porting proves too difficult

### Medium Risk
- **Performance**: Rust implementation might be slower than Python diffusers
  - Mitigation: Profile and optimize critical paths
  - Fallback: Accept some performance cost for Rust safety

- **Memory management**: 16GB VRAM requirement is tight on some GPUs
  - Mitigation: Implement GGUF quantization in Phase 7
  - Target: 8-12GB quantized version

### Low Risk
- **UI integration**: Straightforward based on existing patterns
- **Pipeline integration**: Scheduler research de-risked this
- **Documentation**: Time-consuming but no blockers

---

## Success Criteria

### Minimum Viable Product (MVP)

✅ **Must Have**:
1. Generate 1024×1024 images with Z-Image-Turbo
2. Output quality matches Python implementation
3. Model switching FLUX ↔ Z-Image works without crashes
4. VRAM managed correctly (components unloaded when switching)
5. UI constraints applied (8 steps, 0 guidance, fixed sampler)
6. Full precision support (16GB VRAM)

### Additional Goals

🎯 **Should Have**:
1. GGUF quantized version (8-12GB VRAM)
2. Generation speed competitive with FLUX
3. Comprehensive error handling
4. Memory profiling integrated

### Future Enhancements

🚀 **Could Have** (post-MVP):
1. LoRA support for Z-Image
2. Multi-resolution support beyond 1024×1024
3. Z-Image-Edit variant (requires SigLIP)
4. Batch generation optimizations

---

## Resource Requirements

### Development Time
- **MVP**: ~6 weeks remaining (Phases 4-6)
- **Full completion**: ~7 weeks (including testing & docs)

### Compute Requirements
- **Development**: GPU with 16GB+ VRAM for testing
- **Testing**: Access to Python diffusers environment for reference outputs

### External Dependencies
- Python diffusers library (for reference outputs)
- HuggingFace model hub (model already downloaded)

---

## Questions & Decisions Needed

1. **Quantization Priority**: Should GGUF quantization be MVP or nice-to-have?
   - Recommendation: Nice-to-have, focus on full precision first

2. **LoRA Support**: When should this be implemented?
   - Recommendation: Post-MVP (Phase 9+)

3. **Download Integration**: Implement Z-Image downloader or keep manual?
   - Recommendation: Manual for MVP, implement later

4. **Performance Target**: What's acceptable generation time?
   - Recommendation: Within 2x of Python diffusers

---

## Contact & Handoff

**Current State**: Foundation complete, ready for Phase 4 (transformer implementation)

**To Continue**:
1. Review this document
2. Study `docs/z-image-turbo/scheduler-analysis.md` for sampling approach
3. Begin Phase 4.1: Research diffusers `ZImageTransformer2DModel`
4. Reference `src-tauri/src/models/qwen3.rs` as example of porting transformer architecture

**Key Files**:
- Plan: `/home/alex/.claude/plans/glimmering-stargazing-reddy.md`
- Progress: `./docs/z-image-turbo/phase*-progress.md`
- Status: `./docs/z-image-turbo/PROJECT-STATUS.md` (this file)

**Notes**:
- All Phase 1-3 code compiles successfully
- VAE compatibility verified (can reuse FLUX VAE)
- Scheduler approach validated (reuse FLUX sampling)
- Qwen3 encoder tested and working
- Main blocker: ZImageTransformer implementation (Phase 4)
