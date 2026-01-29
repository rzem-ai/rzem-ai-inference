# Progress Reporting Update - Per-Step Drawing

## Summary
Updated progress reporting to match user specifications with **per-step drawing callbacks**.

## User-Facing Language
Changed technical term "Denoising" to "Drawing" - more intuitive for lay users.

## Progress Values (as specified)

### Full Pipeline
- **0.0** - Loading models starts
- **0.25** - T5 encoder loaded
- **0.375** - CLIP encoder loaded
- **0.5** - Models ready, drawing starts
- **0.5 - 0.95** - Each drawing step reported individually
- **0.95** - Drawing complete
- **0.98** - VAE decoding complete
- **1.0** - Generation complete

### Key Requirement Met
✅ **Each step in the drawing phase is now reported back** via progress callbacks

## Technical Changes

### 1. Progress Stages Simplified (`src-tauri/src/inference/progress.rs`)
Removed `EncodingT5` and `EncodingClip` stages - now part of `LoadingModels`:
- `LoadingModels`: 0.0 → 0.5 (includes T5/CLIP encoding)
- `Denoising`: 0.5 → 0.95 (**per-step reporting**)
- `DecodingVae`: 0.95 → 0.98
- `EncodingPng`: 0.98 → 1.0

### 2. Denoise Method (`src-tauri/src/models/flux.rs`)
Added optional progress callback parameter to all denoising functions:
```rust
pub fn denoise<F>(
    &self,
    // ... existing parameters ...
    on_step: Option<F>,
) -> Result<Tensor>
where
    F: Fn(usize, usize),
```

Updated **all 6 sampler implementations**:
- `denoise_euler_impl` / `denoise_euler_impl_q`
- `denoise_euler_ancestral_impl` / `denoise_euler_ancestral_impl_q`
- `denoise_dpm_pp_2m_impl` / `denoise_dpm_pp_2m_impl_q`

Each sampler now calls the callback **on every step**:
```rust
for (i, window) in timesteps.windows(2).enumerate() {
    // ... denoising computation ...

    // Report progress for this step
    if let Some(callback) = on_step {
        callback(i + 1, total_steps);  // ← Called EVERY step
    }
}
```

### 3. Generation Pipeline (`src-tauri/src/inference/flux_pipeline/generation.rs`)
Updated `generate_with_progress()`:
- Models loading + encoding report as LoadingModels stage (0.0 → 0.5)
- Passes lambda callback to `denoise()` that reports each step
- Each step emits `GenerationProgress::denoising_step(current, total)`

### 4. Cache Integration (`src-tauri/src/inference/flux_pipeline/cache_integration.rs`)
Updated both cached generation methods to use the new progress system.

## Example: 4-Step Generation (FLUX Schnell)

1. **0.0** - Loading T5 encoder
2. **0.25** - T5 loaded, encoding prompt
3. **0.375** - CLIP loaded, encoding prompt
4. **0.5** - Models ready, starting drawing
5. **~0.61** - Drawing step 1/4 ✅
6. **~0.72** - Drawing step 2/4 ✅
7. **~0.84** - Drawing step 3/4 ✅
8. **0.95** - Drawing step 4/4 ✅
9. **0.98** - VAE decoding complete
10. **1.0** - PNG encoding complete

## Testing
- ✅ Code compiles successfully
- ✅ All sampler implementations updated (Euler, EulerA, DPM++ 2M)
- ✅ Both regular and quantized models supported
- ✅ Backward compatible (callback is optional)

## Files Modified
1. `src-tauri/src/inference/progress.rs` - Stage percentages
2. `src-tauri/src/models/flux.rs` - Denoise method + samplers
3. `src-tauri/src/inference/flux_pipeline/generation.rs` - Progress callbacks
4. `src-tauri/src/inference/flux_pipeline/cache_integration.rs` - Cache generation
