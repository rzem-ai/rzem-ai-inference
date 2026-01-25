# Project Resume Status

**Date:** January 18, 2025
**Project:** Flux Generator (Tauri + Vue + Rust)

## Summary

Local FLUX Schnell image generation app using Candle ML framework. Successfully generates images from text prompts with ~21GB VRAM usage.

## Current State: Working

- ✅ Image generation working end-to-end
- ✅ Images display on Generate tab after completion
- ✅ Images saved to gallery database
- ✅ Gallery view displays all generated images
- ✅ Local fonts (Instrument Sans) configured
- ✅ Custom theme (AuraPlus) in use
- ✅ Splitter-based UI layout

## Architecture

### Backend (Rust/Tauri)
```
src-tauri/src/
├── lib.rs              # Tauri commands (generate_image, get_gallery_images, etc.)
├── inference/
│   ├── mod.rs          # InferenceEngine (device selection)
│   └── pipeline.rs     # FluxPipeline (orchestrates generation)
├── models/
│   ├── mod.rs          # Model exports
│   ├── paths.rs        # HuggingFace cache path management
│   ├── clip.rs         # CLIP text encoder (pooled embeddings)
│   ├── t5.rs           # T5-XXL text encoder (main conditioning)
│   ├── vae.rs          # VAE decoder (latents → image)
│   └── flux.rs         # FLUX transformer (denoising)
├── gallery/
│   └── mod.rs          # SQLite database for images
└── queue/
    ├── mod.rs          # Job queue management
    └── processor.rs    # Background job processing
```

### Frontend (Vue/TypeScript)
```
src/
├── main.ts             # App entry, PrimeVue config
├── style.css           # Global styles, Tailwind, CSS vars
├── assets/
│   ├── fonts.css       # @font-face declarations
│   └── theme.ts        # AuraPlus PrimeVue theme
├── stores/
│   ├── queue.ts        # Job queue state, event listeners
│   ├── gallery.ts      # Gallery images state
│   └── generation.ts   # Generation parameters
├── views/
│   ├── GenerateView.vue # Main generation UI with canvas
│   └── GalleryView.vue  # Image gallery grid
└── components/
    ├── generation/
    │   └── ImageCanvas.vue  # Displays generated images
    └── gallery/
        └── ImageGrid.vue    # Gallery image grid
```

## Key Technical Details

### Model Loading (pipeline.rs)
1. **T5-XXL**: Full precision (~9GB VRAM), encodes prompt to [1, 256, 4096]
2. **CLIP**: 768-dim pooled embeddings for FLUX's `vec` parameter
3. **FLUX Transformer**: Prefers quantized GGUF (~12GB) over full precision (~23GB)
4. **VAE**: Decodes 16-channel latents to RGB image

### Memory Management
Models are unloaded after use to fit in 32GB VRAM:
```rust
// After T5 encoding
self.t5 = None;  // Frees ~9GB

// After FLUX denoising
self.flux = None;
self.clip = None;  // Frees ~12GB

// VAE decode now has headroom
```

### CLIP Tokenization Fix (clip.rs:131-150)
Candle's `ClipTextTransformer.forward()` has a bug with `argmax`. Fixed by:
1. Finding EOS position manually from token IDs
2. Using `forward_with_mask()` instead of `forward()`
3. Extracting pooled output at known EOS position

### Image Display Flow
1. Job completes with `result_path` in queue processor
2. Backend emits `job-update` event with path
3. Frontend queue store updates job state
4. GenerateView watcher detects new completed job
5. Calls `canvasRef.value.setImage(path)`
6. ImageCanvas uses `convertFileSrc()` for Tauri asset protocol

### Database (SQLite)
- Location: `~/.flux-generator/gallery.db`
- Tables: `images`, `tags`, `image_tags`, `images_fts`, `models`, `loras`, `presets`
- `GalleryImage` struct uses `#[serde(rename_all = "camelCase")]` for frontend

## Required Models

Download via HuggingFace CLI:
```bash
# Main FLUX Schnell model
huggingface-cli download black-forest-labs/FLUX.1-schnell

# Quantized FLUX transformer (saves ~11GB VRAM)
huggingface-cli download lmz/candle-flux flux1-schnell.gguf

# T5 tokenizer (compatible format)
huggingface-cli download lmz/mt5-tokenizers t5-v1_1-xxl.tokenizer.json
```

Models stored in: `~/.cache/huggingface/hub/`

## Running the App

```bash
# Frontend dev server
npm run dev

# In another terminal, run Tauri
cd src-tauri
cargo run
```

Or use Tauri CLI:
```bash
npm run tauri dev
```

## Known Issues / Notes

1. **First generation is slow**: Models load on first use (~30-60 seconds)
2. **CUDA required**: CPU inference not tested, likely very slow
3. **32GB VRAM recommended**: With quantized FLUX, uses ~21GB peak
4. **Quantized T5 incompatible**: city96's GGUF uses llama.cpp tensor naming, not supported by Candle

## Files Modified This Session

### Backend
- `src-tauri/src/models/clip.rs` - Fixed CLIP argmax overflow, manual EOS extraction
- `src-tauri/src/lib.rs` - Added database insertion to generate_image
- `src-tauri/src/gallery/mod.rs` - Added `GalleryImage` struct with camelCase, `tags` field

### Frontend
- `src/views/GenerateView.vue` - Added watcher for completed jobs, Splitter layout
- `src/views/GalleryView.vue` - Debug logging (removed)
- `src/stores/gallery.ts` - Debug logging (removed)
- `src/components/gallery/ImageGrid.vue` - Debug logging (removed)
- `src/main.ts` - AuraPlus theme, fonts import
- `src/style.css` - Local fonts import, CSS variables, Tailwind classes
- `src/assets/fonts.css` - Created: @font-face for Instrument Sans

## Build Commands

```bash
# Check Rust compilation
cd src-tauri && cargo check

# Build Rust backend
cd src-tauri && cargo build

# Build full app
npm run tauri build
```

## Environment

- OS: Linux (Ubuntu/Debian)
- Rust: Edition 2021
- Tauri: 2.x
- Node: (check with `node -v`)
- CUDA: Required for GPU inference
