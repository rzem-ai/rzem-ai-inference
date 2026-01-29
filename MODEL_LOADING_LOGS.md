# Model Loading Path Logging

## Overview
Added comprehensive logging showing the actual file paths being loaded for ALL models (T5, CLIP, VAE, FLUX transformer). This addresses the issue where only T5 had loading messages, and verifies that models are loaded from the HuggingFace cache.

## Path Resolution Priority (verified in ModelPaths)

The `ModelPaths` implementation checks paths in the following order:

1. **Environment variables** (highest priority)
   - `HF_HUB_CACHE` - Direct path to HuggingFace hub cache
   - `HF_HOME` - Base HuggingFace home (appends `/hub`)

2. **Standard cache locations**
   - Linux: `~/.cache/huggingface/hub`
   - macOS: `~/Library/Caches/huggingface/hub`

3. **Snapshot resolution**
   - Reads `refs/main` file to get the actual commit hash
   - Constructs path: `cache_dir/models--{org}--{repo}/snapshots/{hash}/{file}`
   - Falls back to searching snapshot directories if refs/main is missing

## Files Modified

### 1. src/inference/flux_pipeline/loader.rs
**All model loading operations now log the actual file path:**

**T5 Encoder** (lines 134-159):
```rust
// Before: info!("Loading T5 encoder (quantized Q5_K_M ~3.3GB)");
// After:
let model_path = paths.quantized_t5_path();
let tokenizer_path = paths.t5_tokenizer_path();
info!(
    model = %model_path.display(),
    tokenizer = %tokenizer_path.display(),
    "Loading T5 encoder (quantized Q5_K_M ~3.3GB)"
);
```

**CLIP Encoder** (lines 182-195):
```rust
// Before: info!("Loading CLIP encoder");
// After:
let model_path = paths.clip_path().join("model.safetensors");
let tokenizer_path = paths.tokenizer_path();
info!(
    model = %model_path.display(),
    tokenizer = %tokenizer_path.display(),
    "Loading CLIP encoder"
);
```

**VAE Decoder** (lines 225-236):
```rust
// Before: info!("Loading VAE decoder");
// After:
let model_path = paths.vae_path();
info!(
    model = %model_path.display(),
    "Loading VAE decoder"
);
```

**FLUX Transformer** (lines 274-309):
```rust
// All three loading paths now log the actual path:
// 1. Quantized (GGUF)
let model_path = paths.quantized_transformer_path_for(self.model_type);
info!(
    model = %self.model_type,
    path = %model_path.display(),
    "Loading transformer (quantized GGUF)"
);

// 2. Full precision with LoRAs
let model_path = paths.transformer_path_for(self.model_type);
info!(
    model = %self.model_type,
    path = %model_path.display(),
    lora_count = self.active_loras.len(),
    "Loading transformer with LoRAs (full precision)"
);

// 3. Full precision without LoRAs
let model_path = paths.transformer_path_for(self.model_type);
info!(
    model = %self.model_type,
    path = %model_path.display(),
    "Loading transformer (full precision)"
);
```

### 2. src/inference/flux_pipeline/cache_integration.rs
**Updated reloading operations with path logging:**

- **T5 reloading** (lines 59-89): Shows path for both quantized and full precision
- **CLIP reloading** (lines 79-89): Shows model and tokenizer paths
- **FLUX reloading** (lines 145-162): Shows path for quantized and full precision
- **VAE reloading** (lines 165-172): Shows model path

### 3. src/models/manager.rs
**Updated shared component loading with path logging:**

- **T5** (lines 133-149): Shows model and tokenizer paths
- **CLIP** (lines 142-157): Shows model and tokenizer paths
- **VAE** (lines 151-160): Shows model path
- **FLUX transformer** (lines 181-213): Shows path for quantized and full precision variants

## Example Log Output

### Loading Schnell Model with LoRA
```
INFO Loading T5 encoder (quantized Q5_K_M ~3.3GB)
  model=/home/user/.cache/huggingface/hub/models--city96--t5-v1_1-xxl-encoder-gguf/snapshots/abc123/t5-v1_1-xxl-encoder-Q5_K_M.gguf
  tokenizer=/home/user/.cache/huggingface/hub/models--lmz--mt5-tokenizers/snapshots/def456/t5-v1_1-xxl.tokenizer.json

INFO Loading CLIP encoder
  model=/home/user/.cache/huggingface/hub/models--black-forest-labs--FLUX.1-schnell/snapshots/xyz789/text_encoder/model.safetensors
  tokenizer=/home/user/.cache/huggingface/hub/models--black-forest-labs--FLUX.1-schnell/snapshots/xyz789/tokenizer

INFO Loading VAE decoder
  model=/home/user/.cache/huggingface/hub/models--black-forest-labs--FLUX.1-schnell/snapshots/xyz789/ae.safetensors

INFO Loading transformer with LoRAs (full precision)
  model=schnell
  path=/home/user/.cache/huggingface/hub/models--black-forest-labs--FLUX.1-schnell/snapshots/xyz789/flux1-schnell.safetensors
  lora_count=1
```

### Loading Dev Model (Quantized)
```
INFO Loading transformer (quantized GGUF)
  model=dev
  path=/home/user/.cache/huggingface/hub/models--city96--FLUX.1-dev-gguf/snapshots/abc123/flux1-dev-Q8_0.gguf
```

## Benefits

1. **Transparency**: Can now verify exactly which files are being loaded
2. **HuggingFace Cache Verification**: Log messages show models come from HF cache, not arbitrary local folders
3. **Debugging**: Easier to diagnose missing model files or incorrect paths
4. **Consistency**: All model loads now have the same level of logging detail
5. **Path Resolution Visibility**: Can see the actual snapshot hash being used

## Testing Verification

To verify the logging works:

1. **Check logs during model loading**:
   ```bash
   RUST_LOG=info npm run tauri:dev
   ```

2. **Look for log messages with file paths**:
   - T5: Shows both model and tokenizer paths
   - CLIP: Shows model and tokenizer paths
   - VAE: Shows model path (ae.safetensors)
   - FLUX: Shows transformer path with model type

3. **Verify HuggingFace cache paths**:
   - All paths should start with `~/.cache/huggingface/hub` (Linux)
   - Or `~/Library/Caches/huggingface/hub` (macOS)
   - Paths should include `snapshots/{hash}/` structure

## Notes

- **Path Display**: Uses `%path.display()` in structured logging for proper path formatting
- **Backwards Compatible**: Existing log messages preserved, just enhanced with path info
- **No Performance Impact**: Path resolution happens anyway; we just log it now
- **Environment Variables**: If `HF_HUB_CACHE` or `HF_HOME` is set, logs will show those paths
