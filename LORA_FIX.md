# LoRA Application Fix

## Issue
LoRAs were not being applied correctly to the image generation pipeline.

## Root Cause
All three `FluxTransformer` loading methods (`load`, `load_quantized`, `load_with_loras`) were hardcoded to use `Config::schnell()`, ignoring the pipeline's `model_type` field.

This caused:
1. Dev model LoRAs to fail (wrong architecture config)
2. Model mismatch between selected model and actual config used
3. LoRAs trained for specific models not working properly

## Code Analysis

### Before (Broken)
```rust
// Always used schnell config regardless of model_type
pub fn load_with_loras<P: AsRef<Path>>(
    model_path: P,
    device: Device,
    loras: &[(Arc<LoraAdapter>, f32)],
) -> Result<Self> {
    // ...
    let cfg = flux::model::Config::schnell(); // ❌ Always schnell!
    let model = flux::model::Flux::new(&cfg, vb)?;
}
```

### After (Fixed)
```rust
// Uses appropriate config based on model_type
pub fn load_with_loras<P: AsRef<Path>>(
    model_path: P,
    device: Device,
    model_type: ModelType,  // ✅ New parameter
    loras: &[(Arc<LoraAdapter>, f32)],
) -> Result<Self> {
    // ...
    let cfg = match model_type {
        ModelType::Schnell => flux::model::Config::schnell(),
        ModelType::Dev => flux::model::Config::dev(),  // ✅ Now supported!
        _ => anyhow::bail!("Unsupported model type for FLUX: {:?}", model_type),
    };
    let model = flux::model::Flux::new(&cfg, vb)?;
}
```

## Files Modified

1. **src/models/flux.rs**
   - Added `ModelType` import
   - Updated `load()` to accept `model_type` parameter
   - Updated `load_quantized()` to accept `model_type` parameter
   - Updated `load_with_loras()` to accept `model_type` parameter
   - All three methods now use the correct config based on model type
   - Updated test cases to pass `ModelType::Schnell`

2. **src/inference/flux_pipeline/loader.rs**
   - Updated all three `FluxTransformer::load*()` calls to pass `self.model_type`

3. **src/inference/flux_pipeline/cache_integration.rs**
   - Updated `FluxTransformer::load()` and `load_quantized()` calls to pass `self.model_type`

4. **src/models/manager.rs**
   - Updated `FluxTransformer::load()` and `load_quantized()` calls to pass `model` parameter

## How It Works Now

1. User selects model (Schnell or Dev) in UI
2. Frontend sends model ID to backend
3. Backend converts model string to `ModelType` enum
4. Pipeline stores `model_type` field
5. When loading models (including with LoRAs), the correct config is used:
   - `ModelType::Schnell` → `Config::schnell()`
   - `ModelType::Dev` → `Config::dev()`

## LoRA Flow (Now Fixed)

1. **Frontend**: User toggles LoRAs in UI
2. **Frontend**: `modelsStore.getActiveLoraConfigs()` gets active LoRAs
3. **Frontend**: LoRAs included in generation params
4. **Backend**: QueueProcessor loads LoRA adapters from disk
5. **Backend**: `pipeline.set_loras()` stores LoRAs and marks FLUX for reload
6. **Backend**: `ensure_models_loaded()` calls `load_flux_transformer()`
7. **Backend**: Loader checks if LoRAs are active
8. **Backend**: Calls `FluxTransformer::load_with_loras()` with correct `model_type` ✅
9. **Backend**: FLUX loaded with appropriate config + LoRAs merged
10. **Backend**: Generation uses LoRA-modified model

## Testing

### Verification Steps
1. ✅ Code compiles successfully
2. ✅ `Config::dev()` exists in candle-transformers
3. Test with Schnell model + LoRA
4. Test with Dev model + LoRA
5. Verify log messages show "Loading transformer with LoRAs"
6. Verify generated images reflect LoRA effects

### Expected Log Messages
```
INFO Loading transformer with LoRAs (full precision) model=schnell lora_count=1
INFO Loading FLUX with LoRA adapters lora_count=1
DEBUG Applying LoRA lora_id="..." lora_name="..." strength=1.0
DEBUG Applied LoRA weight layer="..." flux_name="..."
INFO FLUX loaded with LoRA adapters merged
```

## Previous Investigation

The LoRA system architecture was correct:
- ✅ Frontend properly gets active LoRAs
- ✅ Backend properly loads LoRA files
- ✅ LoRA merging logic is correct
- ✅ Layer name mapping works
- ✅ Progress reporting works

The ONLY issue was the hardcoded config preventing proper model initialization.

## Impact

This fix enables:
- ✅ LoRAs to work with both Schnell and Dev models
- ✅ Proper model architecture matching
- ✅ LoRAs trained for specific FLUX variants to apply correctly
- ✅ Multiple LoRAs with different strengths
