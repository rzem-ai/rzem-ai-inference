# Performance Optimization & FLUX Dev Support - Design Document

**Date:** 2026-01-18
**Status:** Approved
**Author:** Claude + Alex

## Overview

This document describes the design for two related features:
1. **Performance Optimization** - Persistent model loading, batch generation, real-time progress tracking
2. **FLUX Dev Support** - Higher-quality model with smart VRAM management

## Goals

- Reduce generation time by keeping models loaded between generations
- Support batch generation (multiple images from one prompt)
- Provide granular progress feedback to UI and CLI
- Add FLUX.1 [dev] model support with automatic quality/memory tradeoffs
- Create a full-featured CLI for scripting and headless generation

---

## Section 1: Architecture - Model Manager

### Current Problem

The current `FluxPipeline` creates fresh model instances per generation and unloads them after each image. This causes:
- ~6-10 second model loading overhead per generation
- No ability to batch efficiently
- Wasted VRAM headroom

### Solution: Persistent Model Manager

Introduce a `ModelManager` that lives for the application's lifetime and manages model lifecycle intelligently.

```
ModelManager
├── device: Device (CUDA/Metal/CPU)
├── vram_total: u64
├── vram_available: u64 (monitored)
├── loaded_models: HashMap<ModelId, LoadedModel>
│   ├── "t5" → T5TextEncoder
│   ├── "clip" → ClipTextEncoder
│   ├── "vae" → VaeDecoder
│   ├── "flux-schnell" → FluxTransformer
│   └── "flux-dev" → FluxTransformer (when loaded)
└── current_pipeline: PipelineType (Schnell | Dev)
```

### Shared vs Model-Specific Components

| Component | Shared? | Size (VRAM) | Notes |
|-----------|---------|-------------|-------|
| VAE | Yes | ~160MB | Identical for Schnell & Dev |
| CLIP | Yes | ~250MB | Identical for Schnell & Dev |
| T5 | Yes | ~9GB (full) / ~3GB (quantized) | Same architecture |
| FLUX Transformer | No | ~12-23GB | Different weights per model |

### VRAM Budget Logic

```rust
fn select_precision(&self, model: ModelType) -> Precision {
    let available = self.available_vram();
    let full_precision_requirement = match model {
        ModelType::Schnell => 23_000, // MB
        ModelType::Dev => 24_000,
    };

    if available > full_precision_requirement + 2_000 {
        Precision::Full
    } else {
        Precision::Quantized
    }
}
```

Decision thresholds:
- Full-precision Dev (~24GB) if VRAM > 26GB free
- Quantized Dev (~12GB) otherwise
- Always prefer quantized Schnell for speed

---

## Section 2: Progress Tracking System

### Progress Event Structure

```rust
pub struct GenerationProgress {
    /// Current pipeline stage
    pub stage: PipelineStage,
    /// Progress within current stage (0.0-1.0)
    pub stage_progress: f32,
    /// Overall generation progress (0.0-1.0)
    pub overall_progress: f32,
    /// Human-readable status message
    pub message: String,
    /// Estimated seconds remaining (if calculable)
    pub eta_seconds: Option<f32>,
    /// For batch jobs: which image (1-indexed)
    pub batch_index: Option<usize>,
    /// For batch jobs: total count
    pub batch_total: Option<usize>,
}

pub enum PipelineStage {
    LoadingModels,    // 0-10% of total
    EncodingT5,       // 10-20%
    EncodingClip,     // 20-25%
    Denoising,        // 25-85% (bulk of time)
    DecodingVae,      // 85-95%
    EncodingPng,      // 95-100%
}
```

### Stage Weight Distribution

Stages are weighted by typical execution time:

| Stage | Weight | Cumulative |
|-------|--------|------------|
| LoadingModels | 10% | 0-10% |
| EncodingT5 | 10% | 10-20% |
| EncodingClip | 5% | 20-25% |
| Denoising | 60% | 25-85% |
| DecodingVae | 10% | 85-95% |
| EncodingPng | 5% | 95-100% |

### Callback Pattern

```rust
impl FluxPipeline {
    pub fn generate_with_progress<F>(
        &mut self,
        params: GenerationParams,
        on_progress: F,
    ) -> Result<GenerationResult>
    where
        F: Fn(GenerationProgress) + Send + 'static,
    {
        // Called at each stage transition and each denoising step
        on_progress(GenerationProgress {
            stage: PipelineStage::Denoising,
            stage_progress: step as f32 / total_steps as f32,
            overall_progress: 0.25 + (0.60 * step as f32 / total_steps as f32),
            message: format!("Denoising step {}/{}", step + 1, total_steps),
            eta_seconds: self.estimate_remaining(step, total_steps),
            batch_index: None,
            batch_total: None,
        });
    }
}
```

### ETA Calculation

```rust
fn estimate_remaining(&self, current_step: usize, total_steps: usize) -> Option<f32> {
    if current_step == 0 {
        return None;
    }

    let elapsed = self.step_timer.elapsed();
    let avg_per_step = elapsed.as_secs_f32() / current_step as f32;
    let remaining_steps = total_steps - current_step;

    Some(avg_per_step * remaining_steps as f32)
}
```

### Frontend Integration

Tauri emits `generation-progress` events:

```typescript
// Vue composable
const { progress, stage, eta } = useGenerationProgress();

listen('generation-progress', (event) => {
  progress.value = event.payload.overall_progress;
  stage.value = event.payload.message;
  eta.value = event.payload.eta_seconds;
});
```

---

## Section 3: Batch Generation

### Batch Parameters

```rust
pub struct BatchParams {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub count: usize,
    pub seeds: Option<Vec<i64>>,  // Auto-generate if None
    pub width: usize,
    pub height: usize,
    pub steps: usize,
    pub guidance: f64,
    pub model: ModelType,
}
```

### Execution Flow

1. **Encode once** - T5 and CLIP encode the prompt (reused for all images)
2. **Generate noise** - Create `count` noise tensors with different seeds
3. **Denoise** - Process latents (sequential or batched based on VRAM)
4. **Decode** - VAE decode each latent to RGB
5. **Return** - `Vec<GenerationResult>` with individual stats

### VRAM-Aware Batch Sizing

```rust
fn max_batch_size(&self, width: usize, height: usize) -> usize {
    let available_mb = self.available_vram() / 1024 / 1024;
    let latent_size_mb = (width / 8) * (height / 8) * 16 * 4 / 1024 / 1024;

    // Reserve 2GB headroom
    let usable_mb = available_mb.saturating_sub(2048);

    // Each batch item needs latent + intermediate activations (~2x)
    let per_item_mb = latent_size_mb * 2;

    (usable_mb / per_item_mb).max(1).min(8)
}
```

Approximate limits for 1024x1024:

| Available VRAM | Max Batch |
|----------------|-----------|
| < 16GB | 1 (sequential) |
| 16-24GB | 2 |
| 24-32GB | 4 |
| > 32GB | 8 |

### Batch Progress

For batch jobs, progress includes batch context:

```
"Generating image 2/4 - Denoising step 3/4 - 52%"
```

---

## Section 4: CLI Design

### Binary Structure

The CLI shares code with the Tauri app via the library crate:

```
src-tauri/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Shared: models, pipeline, progress
│   ├── main.rs             # Tauri desktop app
│   └── bin/
│       └── flux-cli.rs     # CLI binary
```

In `Cargo.toml`:

```toml
[[bin]]
name = "flux-cli"
path = "src/bin/flux-cli.rs"
```

### Commands

#### Generate

```bash
flux-cli generate --prompt "a cat wearing a hat" --output cat.png

# Full options
flux-cli generate \
  --prompt "a beautiful sunset" \
  --output sunset.png \
  --model dev \
  --steps 50 \
  --width 1344 \
  --height 768 \
  --seed 42 \
  --batch 4 \
  --guidance 5.0

# Batch from file
flux-cli generate \
  --prompts prompts.txt \
  --output-dir ./outputs/ \
  --model schnell
```

#### Models

```bash
# List all models with status
flux-cli models list
# Output:
# MODEL          STATUS       SIZE      VRAM (full/quant)
# schnell        downloaded   23.8 GB   23 GB / 12 GB
# dev            not downloaded  24.2 GB   24 GB / 12 GB

# Download a model
flux-cli models download dev

# Show model info
flux-cli models info schnell
```

#### System Info

```bash
flux-cli info
# Output:
# GPU: NVIDIA RTX 5090
# VRAM: 32,607 MB total, 31,594 MB free
# CUDA: 12.8
# Loaded models: none
# Cache: ~/.cache/huggingface/hub (47.3 GB)
```

### CLI Flags Reference

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--prompt` | `-p` | required | Text prompt |
| `--output` | `-o` | `flux_{timestamp}.png` | Output file path |
| `--model` | `-m` | `schnell` | Model: `schnell` or `dev` |
| `--steps` | `-s` | 4 / 28 | Denoising steps (default varies by model) |
| `--width` | `-W` | 1024 | Image width |
| `--height` | `-H` | 1024 | Image height |
| `--seed` | | -1 | Random seed (-1 = random) |
| `--batch` | `-b` | 1 | Number of images to generate |
| `--guidance` | `-g` | 4.0 | CFG guidance scale |
| `--negative` | `-n` | none | Negative prompt |
| `--json` | | false | Output results as JSON |
| `--quiet` | `-q` | false | Suppress progress output |
| `--prompts` | | none | File with prompts (one per line) |
| `--output-dir` | | `.` | Output directory for batch |

### JSON Output Mode

For scripting integration:

```bash
flux-cli generate -p "a cat" --json
```

```json
{
  "success": true,
  "output_path": "/home/user/.flux-generator/outputs/flux_1234567890_42.png",
  "seed": 42,
  "model": "schnell",
  "steps": 4,
  "generation_time_ms": 13058,
  "stats": {
    "t5_encode_ms": 147,
    "clip_encode_ms": 108,
    "denoise_ms": 5773,
    "vae_decode_ms": 846
  }
}
```

---

## Section 5: FLUX Dev Integration

### Model Comparison

| Aspect | Schnell | Dev |
|--------|---------|-----|
| Recommended steps | 4 | 28-50 |
| Guidance range | 3.5-4.0 | 3.5-7.0 |
| Quality | Good, fast | Higher detail, better composition |
| Generation time | ~13s | ~60-120s (depending on steps) |
| VRAM (full precision) | ~23GB | ~24GB |
| VRAM (quantized) | ~12GB | ~12GB |
| License | Apache 2.0 | Non-commercial |

### File Structure

```
~/.cache/huggingface/hub/
├── models--black-forest-labs--FLUX.1-schnell/
│   └── snapshots/{hash}/
│       ├── flux1-schnell.safetensors    # Transformer
│       ├── ae.safetensors               # VAE (shared)
│       ├── text_encoder/                 # CLIP (shared)
│       └── text_encoder_2/               # T5 (shared)
│
├── models--black-forest-labs--FLUX.1-dev/
│   └── snapshots/{hash}/
│       └── flux1-dev.safetensors        # Transformer only
│
└── models--lmz--candle-flux/            # Quantized versions
    └── snapshots/{hash}/
        ├── flux1-schnell.gguf
        └── flux1-dev.gguf
```

### Smart Model Switching

```rust
impl ModelManager {
    pub fn switch_to_model(&mut self, target: ModelType) -> Result<()> {
        if self.current_model == Some(target) {
            return Ok(()); // Already loaded
        }

        // Keep shared components loaded
        // - VAE: always keep (small, always needed)
        // - CLIP: always keep (small, always needed)
        // - T5: keep (large but shared between models)

        // Unload current transformer only
        if let Some(current) = self.current_model {
            println!("Unloading {} transformer...", current);
            self.unload_transformer(current)?;
        }

        // Determine precision based on available VRAM
        let precision = self.select_precision(target);
        println!("Loading {} transformer ({:?})...", target, precision);

        self.load_transformer(target, precision)?;
        self.current_model = Some(target);

        Ok(())
    }

    fn unload_transformer(&mut self, model: ModelType) -> Result<()> {
        match model {
            ModelType::Schnell => self.flux_schnell = None,
            ModelType::Dev => self.flux_dev = None,
        }
        // Force CUDA memory cleanup
        if self.device.is_cuda() {
            // Trigger garbage collection
        }
        Ok(())
    }
}
```

### Default Parameters by Model

```rust
impl ModelType {
    pub fn default_steps(&self) -> usize {
        match self {
            ModelType::Schnell => 4,
            ModelType::Dev => 28,
        }
    }

    pub fn default_guidance(&self) -> f64 {
        match self {
            ModelType::Schnell => 4.0,
            ModelType::Dev => 3.5,
        }
    }

    pub fn step_range(&self) -> (usize, usize) {
        match self {
            ModelType::Schnell => (1, 8),
            ModelType::Dev => (20, 100),
        }
    }
}
```

---

## Implementation Plan

### Phase 1: Model Manager & Persistent Loading
1. Create `ModelManager` struct with VRAM monitoring
2. Refactor `FluxPipeline` to use `ModelManager`
3. Implement keep-alive between generations
4. Add smart unload for model switching

### Phase 2: Progress Tracking
1. Define `GenerationProgress` and `PipelineStage` types
2. Add callback support to pipeline methods
3. Integrate with Tauri event system
4. Update frontend to display progress

### Phase 3: Batch Generation
1. Add `BatchParams` and batch execution logic
2. Implement VRAM-aware batch sizing
3. Add batch progress tracking
4. Update queue processor for batch jobs

### Phase 4: FLUX Dev Support
1. Add `ModelType` enum and path resolution for Dev
2. Implement model download for Dev
3. Add precision auto-selection
4. Update UI model selector

### Phase 5: CLI
1. Create `flux-cli` binary with clap
2. Implement `generate` command
3. Implement `models` subcommands
4. Implement `info` command
5. Add JSON output mode

---

## Success Criteria

- [ ] Models stay loaded between generations (no reload overhead)
- [ ] Batch generation produces N images efficiently
- [ ] Progress updates show in UI with ETA
- [ ] FLUX Dev generates higher quality images
- [ ] Auto-selects quantized when VRAM is tight
- [ ] CLI can generate images headlessly
- [ ] CLI supports all generation parameters
- [ ] Switching models keeps shared components loaded
