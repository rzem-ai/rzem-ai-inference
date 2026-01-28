# Sketch-to-Image Feature Research

This document outlines the research findings and implementation recommendations for adding sketch-to-image functionality to rzem-ai-inference.

## Table of Contents

1. [Overview](#overview)
2. [Model Options](#model-options)
3. [Architecture Approaches](#architecture-approaches)
4. [Implementation Recommendations](#implementation-recommendations)
5. [Client-Side Requirements](#client-side-requirements)
6. [Server-Side Requirements](#server-side-requirements)
7. [Migration Path](#migration-path)

---

## Overview

Sketch-to-image generation allows users to provide rough sketches, line drawings, or doodles as conditioning input, which the model uses to guide the structure and composition of the generated image while the text prompt controls style, subject, and details.

### Current Architecture State

The codebase already has:
- **FLUX.1 pipeline** (Schnell 4-step, Dev 28-step)
- **LoRA support** with merge-at-load-time pattern
- **Vision module** with image preprocessing (Moondream for tagging)
- **Model cache system** with embedding caching
- **TypeScript types** for `img2img` and `inpainting` modes (not yet implemented in Rust)

Missing for sketch-to-image:
- ControlNet/conditioning model loading
- Image → latent encoding (VAE encoder)
- Sketch preprocessing (edge detection, lineart extraction)
- Drawing canvas UI component

---

## Model Options

### Recommended: FLUX ControlNet Models

| Model | Parameters | Type | Best For | License |
|-------|------------|------|----------|---------|
| **MistoControlNet-Flux-dev** | ~1.4B | Lineart/Sketch | Sketches, line drawings | Check repo |
| **XLabs flux-controlnet-collections** | Various | Multi-purpose | Canny, HED, depth | Apache 2.0 |
| **InstantX FLUX.1-dev-ControlNet-Union** | ~1B | Multi-mode | Versatility (canny, depth, pose) | Non-commercial |
| **EasyControl** | ~15M | Lightweight | Minimal VRAM overhead | Research |

#### MistoControlNet-Flux-dev (Top Recommendation for Sketches)

- **Purpose-built** for lineart and outline sketches
- Uses dual-stream Transformer architecture
- No additional inference time over base FLUX
- Recommended strength: 0.65-0.8
- Source: [GitHub - TheMistoAI/MistoControlNet-Flux-dev](https://github.com/TheMistoAI/MistoControlNet-Flux-dev)

#### XLabs flux-controlnet-collections

- Multiple control types: Canny, HED, depth, pose
- v3 versions available (improved quality)
- Good for hand-drawn sketches
- Source: [HuggingFace - XLabs-AI/flux-controlnet-collections](https://huggingface.co/XLabs-AI/flux-controlnet-collections)

### Alternative: T2I-Adapter

| Aspect | T2I-Adapter | ControlNet |
|--------|-------------|------------|
| Parameters | ~77M (SD1.5), ~79M (SDXL) | ~567M (SD1.5), ~1.25B (SDXL) |
| Speed Impact | Near-zero (runs once) | Significant (runs each step) |
| Control Precision | Good (slightly less precise) | Excellent |
| VRAM Usage | Much lower | Higher |

**Note:** T2I-Adapter for FLUX is not yet widely available. ControlNet is the recommended path.

### Alternative: IP-Adapter (Image Prompting)

- **InstantX FLUX.1-dev IP-Adapter**: Uses SigLIP image encoder
- Better for style transfer than structural control
- Not ideal for sketch-to-image (content leakage issues)
- Source: [HuggingFace - InstantX/FLUX.1-dev-IP-Adapter](https://huggingface.co/InstantX/FLUX.1-dev-IP-Adapter)

---

## Architecture Approaches

### Approach 1: FLUX ControlNet (Recommended)

```
User Sketch → Preprocessor → ControlNet Encoder → Conditioning Signal
                                                         ↓
Text Prompt → T5/CLIP Encoders → FLUX Transformer ← Inject Conditioning
                                        ↓
                                    VAE Decoder → Output Image
```

**How FLUX ControlNet Works:**
1. ControlNet has matching transformer blocks to FLUX
2. Sketch image is encoded through ControlNet
3. `controlnet_block_samples` and `controlnet_single_block_samples` are extracted
4. These residuals are added to FLUX transformer blocks during denoising
5. Text embeddings still control style/content, sketch controls structure

**FLUX ControlNet Architecture Details:**
- Uses `FluxControlNetConfig` with `num_single_layers` and `num_joint_layers`
- Transformer blocks can be initialized from FLUX weights
- Supports dual-stream architecture for better alignment

### Approach 2: Channel-wise Concatenation (FLUX.1 Native)

Used by official FLUX.1 Canny/Depth:
1. Preprocess sketch to control signal
2. Concatenate with noise latents (channel dimension)
3. Transformer learns to condition on the additional channels

**Pros:** No separate ControlNet needed, simpler architecture
**Cons:** Requires retraining, less flexible

### Approach 3: EasyControl (Lightweight)

- Only 15M parameters (vs. ControlNet's 1-3B)
- Condition Injection LoRA Module
- Plug-and-play, doesn't modify base weights
- Compatible with existing LoRAs
- Source: [arXiv - EasyControl](https://arxiv.org/html/2503.07027v1)

---

## Implementation Recommendations

### Phase 1: Core Infrastructure (MVP)

1. **Extend `GenerationParams`** with conditioning fields:
   ```rust
   pub struct GenerationParams {
       // ... existing fields ...

       /// Conditioning image (base64 or file path)
       pub control_image: Option<String>,

       /// Control type: "sketch", "canny", "lineart", "depth"
       pub control_type: Option<ControlType>,

       /// Control strength (0.0-1.0, default 0.75)
       pub control_strength: Option<f32>,

       /// Preprocessor to apply: "none", "canny", "hed", "lineart"
       pub preprocessor: Option<PreprocessorType>,
   }
   ```

2. **Add edge detection preprocessing** (Rust crates):
   - `edge-detection` - Canny edge detection
   - `imageproc` - Additional image processing
   - For HED/Lineart: ONNX Runtime with pre-trained models

3. **Implement ControlNet loader** in `src-tauri/src/models/`:
   ```rust
   pub struct FluxControlNet {
       transformer_blocks: Vec<TransformerBlock>,
       single_transformer_blocks: Vec<SingleTransformerBlock>,
       device: Device,
   }
   ```

4. **Modify `FluxPipeline`** to accept conditioning:
   ```rust
   pub fn generate_with_control(
       &mut self,
       params: &GenerationParams,
       control_image: &Tensor,
       on_progress: impl Fn(GenerationProgress),
   ) -> Result<GenerationResult>
   ```

### Phase 2: Client-Side Drawing Canvas

1. **Vue 3 drawing component** using `vue-drawing-canvas`:
   ```vue
   <template>
     <SketchCanvas
       v-model:image="sketchData"
       :width="generationStore.width"
       :height="generationStore.height"
       :brush-size="brushSize"
       :brush-color="brushColor"
       @update:image="handleSketchUpdate"
     />
   </template>
   ```

2. **Features required:**
   - Freehand drawing with adjustable brush size
   - Eraser tool
   - Undo/redo history
   - Clear canvas
   - Import existing image as reference
   - Export to base64 PNG

3. **Integration with generation form:**
   ```typescript
   // src/stores/generation.ts
   export interface GenerationParams {
     // ... existing ...
     controlImage?: string;     // base64 PNG
     controlType?: ControlType;
     controlStrength?: number;
     preprocessor?: PreprocessorType;
   }
   ```

### Phase 3: Preprocessing Pipeline

| Preprocessor | Use Case | Rust Implementation |
|--------------|----------|---------------------|
| None | User provides clean lineart | Pass through |
| Canny | Photos → edge detection | `edge-detection` crate |
| HED | Soft edges, preserve detail | ONNX model via `ort` crate |
| Lineart | Extract clean lines | ONNX model via `ort` crate |
| Scribble | Rough sketches | Threshold + dilation |

**Preprocessing Pipeline:**
```rust
pub fn preprocess_control_image(
    image: &DynamicImage,
    preprocessor: PreprocessorType,
    device: &Device,
) -> Result<Tensor> {
    match preprocessor {
        PreprocessorType::None => image_to_tensor(image, device),
        PreprocessorType::Canny => {
            let edges = canny_edge_detection(image, 100.0, 200.0);
            image_to_tensor(&edges, device)
        }
        PreprocessorType::Hed => {
            let hed_model = load_hed_onnx()?;
            run_hed_inference(&hed_model, image, device)
        }
        PreprocessorType::Lineart => {
            let lineart_model = load_lineart_onnx()?;
            run_lineart_inference(&lineart_model, image, device)
        }
    }
}
```

---

## Client-Side Requirements

### Drawing Canvas Component

**Recommended package:** `vue-drawing-canvas` (Vue 3 compatible)

```bash
npm install --save-dev vue-drawing-canvas
```

**Alternative:** Custom implementation with HTML5 Canvas:
```typescript
// src/components/generation/SketchCanvas.vue
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

defineProps<{
  width: number;
  height: number;
}>();

const emit = defineEmits<{
  'update:image': [data: string];
}>();

const canvas = ref<HTMLCanvasElement>();
const isDrawing = ref(false);
const ctx = ref<CanvasRenderingContext2D>();

function startDrawing(e: MouseEvent) {
  isDrawing.value = true;
  draw(e);
}

function draw(e: MouseEvent) {
  if (!isDrawing.value || !ctx.value) return;

  ctx.value.lineTo(e.offsetX, e.offsetY);
  ctx.value.stroke();
}

function stopDrawing() {
  isDrawing.value = false;
  ctx.value?.beginPath();
  emitImage();
}

function emitImage() {
  const dataUrl = canvas.value?.toDataURL('image/png');
  if (dataUrl) emit('update:image', dataUrl);
}
</script>
```

### UI Integration

```vue
<!-- src/views/GenerateView.vue -->
<template>
  <div class="generation-container">
    <!-- Mode selector -->
    <TabView v-model:activeIndex="modeIndex">
      <TabPanel header="Text to Image">
        <PromptEditor />
      </TabPanel>
      <TabPanel header="Sketch to Image">
        <SketchCanvas
          v-model:image="generationStore.controlImage"
          :width="generationStore.width"
          :height="generationStore.height"
        />
        <PromptEditor />
        <div class="control-options">
          <Slider v-model="generationStore.controlStrength" :min="0" :max="1" :step="0.05" />
          <Dropdown v-model="generationStore.preprocessor" :options="preprocessorOptions" />
        </div>
      </TabPanel>
    </TabView>
  </div>
</template>
```

### TypeScript Types

```typescript
// src/types/index.ts
export type ControlType = 'sketch' | 'canny' | 'lineart' | 'depth' | 'pose';
export type PreprocessorType = 'none' | 'canny' | 'hed' | 'lineart' | 'scribble';

export interface GenerationParams {
  // ... existing fields ...

  // Control/conditioning
  controlImage?: string;      // base64 data URL
  controlType?: ControlType;
  controlStrength?: number;   // 0.0-1.0
  preprocessor?: PreprocessorType;
}
```

---

## Server-Side Requirements

### New Rust Dependencies

```toml
# src-tauri/Cargo.toml
[dependencies]
edge-detection = "0.2"        # Canny edge detection
imageproc = "0.24"            # Image processing utilities
ort = "2.0"                   # ONNX Runtime for HED/Lineart models
```

### ControlNet Model Structure

```rust
// src-tauri/src/models/controlnet.rs
use candle_core::{Device, Tensor, Module};
use candle_nn::VarBuilder;

pub struct FluxControlNet {
    /// Input projection for control image
    input_hint_block: candle_nn::Conv2d,

    /// Transformer blocks matching FLUX architecture
    transformer_blocks: Vec<JointTransformerBlock>,
    single_transformer_blocks: Vec<SingleTransformerBlock>,

    device: Device,
}

impl FluxControlNet {
    pub fn load(model_path: &Path, device: &Device) -> Result<Self> {
        // Load from safetensors format
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[model_path], DType::BF16, device)? };

        // Build model architecture
        // ...
    }

    pub fn forward(
        &self,
        control_image: &Tensor,  // [B, 3, H, W]
        timestep: &Tensor,
        encoder_hidden_states: &Tensor,
    ) -> Result<ControlNetOutput> {
        // Process through transformer blocks
        // Return block samples for injection into FLUX
    }
}

pub struct ControlNetOutput {
    pub block_samples: Vec<Tensor>,
    pub single_block_samples: Vec<Tensor>,
}
```

### Modified Pipeline

```rust
// src-tauri/src/inference/flux_pipeline/generation.rs

impl FluxPipeline {
    pub fn generate_with_control(
        &mut self,
        params: &GenerationParams,
        control_image: Option<&Tensor>,
        control_strength: f32,
        on_progress: impl Fn(GenerationProgress),
    ) -> Result<GenerationResult> {
        self.ensure_models_loaded()?;

        // Load ControlNet if needed
        if control_image.is_some() {
            self.ensure_controlnet_loaded()?;
        }

        // Encode text
        let (t5_emb, clip_emb) = self.encode_prompt(&params.prompt)?;

        // Initialize latents
        let mut latents = self.init_latents(params)?;

        // Denoising loop
        for step in 0..params.steps {
            let timestep = self.get_timestep(step, params.steps)?;

            // Get ControlNet conditioning if available
            let control_conditioning = if let (Some(img), Some(controlnet)) =
                (control_image, &self.controlnet)
            {
                let output = controlnet.forward(img, &timestep, &t5_emb)?;
                Some((output, control_strength))
            } else {
                None
            };

            // FLUX forward with optional conditioning
            let noise_pred = self.flux.as_ref().unwrap().forward_with_control(
                &latents,
                &timestep,
                &t5_emb,
                &clip_emb,
                control_conditioning,
            )?;

            // Sampler step
            latents = self.sampler_step(&latents, &noise_pred, step)?;

            on_progress(GenerationProgress {
                stage: GenerationStage::Denoising,
                progress: (step + 1) as f32 / params.steps as f32,
                ..Default::default()
            });
        }

        // Decode
        let image = self.decode_latents(&latents)?;

        Ok(GenerationResult { image, stats: self.collect_stats() })
    }
}
```

### Preprocessing Module

```rust
// src-tauri/src/inference/preprocessing.rs

use edge_detection::canny;
use image::{DynamicImage, GrayImage};
use ort::{Session, GraphOptimizationLevel};

pub enum PreprocessorType {
    None,
    Canny { low: f32, high: f32 },
    Hed,
    Lineart,
    Scribble,
}

pub struct ImagePreprocessor {
    hed_session: Option<Session>,
    lineart_session: Option<Session>,
    device: Device,
}

impl ImagePreprocessor {
    pub fn new(device: Device) -> Result<Self> {
        Ok(Self {
            hed_session: None,
            lineart_session: None,
            device,
        })
    }

    pub fn preprocess(
        &mut self,
        image: &DynamicImage,
        preprocessor: PreprocessorType,
    ) -> Result<Tensor> {
        match preprocessor {
            PreprocessorType::None => self.to_tensor(image),

            PreprocessorType::Canny { low, high } => {
                let gray = image.to_luma8();
                let edges = canny(&gray, low, high);
                self.to_tensor(&DynamicImage::ImageLuma8(edges))
            }

            PreprocessorType::Hed => {
                self.ensure_hed_loaded()?;
                self.run_hed(image)
            }

            PreprocessorType::Lineart => {
                self.ensure_lineart_loaded()?;
                self.run_lineart(image)
            }

            PreprocessorType::Scribble => {
                // Simple threshold + dilation for rough sketches
                let gray = image.to_luma8();
                let binary = imageproc::contrast::threshold(&gray, 128);
                let dilated = imageproc::morphology::dilate(&binary,
                    imageproc::distance_transform::Norm::LInf, 2);
                self.to_tensor(&DynamicImage::ImageLuma8(dilated))
            }
        }
    }

    fn ensure_hed_loaded(&mut self) -> Result<()> {
        if self.hed_session.is_none() {
            let model_path = get_hed_model_path()?;
            self.hed_session = Some(Session::builder()?
                .with_optimization_level(GraphOptimizationLevel::Level3)?
                .commit_from_file(model_path)?);
        }
        Ok(())
    }

    fn to_tensor(&self, image: &DynamicImage) -> Result<Tensor> {
        // Resize to match generation dimensions
        // Normalize to [-1, 1] or [0, 1] depending on model requirements
        // Convert to Tensor [1, 3, H, W]
    }
}
```

### Model Downloads

Add ControlNet to the model manager:

```rust
// src-tauri/src/models/manager.rs

pub enum ModelType {
    Schnell,
    Dev,
    ZImageTurbo,
    // New
    ControlNetCanny,
    ControlNetLineart,
    ControlNetSketch,
}

impl ModelManager {
    pub async fn download_controlnet(&self, control_type: ControlType) -> Result<()> {
        let (repo_id, filename) = match control_type {
            ControlType::Sketch => (
                "TheMistoAI/MistoControlNet-Flux-dev",
                "controlnet.safetensors"
            ),
            ControlType::Canny => (
                "XLabs-AI/flux-controlnet-canny",
                "controlnet.safetensors"
            ),
            // ...
        };

        self.downloader.download(repo_id, filename).await
    }
}
```

---

## Migration Path

### Phase 1: Foundation (Week 1-2)

1. **Backend types and params**
   - Add `ControlType`, `PreprocessorType` enums
   - Extend `GenerationParams` with control fields
   - Update TypeScript types to match

2. **Basic preprocessing**
   - Implement Canny edge detection using `edge-detection` crate
   - Add image loading and tensor conversion utilities

3. **Frontend canvas**
   - Add `vue-drawing-canvas` dependency
   - Create `SketchCanvas.vue` component
   - Add sketch mode to GenerateView

### Phase 2: ControlNet Integration (Week 3-4)

1. **Model loading**
   - Implement `FluxControlNet` struct
   - Add safetensors loading for ControlNet weights
   - Integrate with `ModelManager`

2. **Pipeline modification**
   - Add `generate_with_control()` method
   - Implement conditioning injection in denoising loop
   - Test with XLabs Canny ControlNet

3. **Download UI**
   - Add ControlNet models to ModelsView
   - Show download progress

### Phase 3: Advanced Features (Week 5-6)

1. **HED/Lineart preprocessors**
   - Add ONNX Runtime dependency
   - Download and integrate preprocessing models
   - Add preprocessor selection UI

2. **MistoControlNet**
   - Adapt loader for Misto architecture
   - Test with sketch inputs

3. **Polish**
   - Add reference image import to canvas
   - Undo/redo for drawing
   - Preset control strengths

---

## VRAM Considerations

| Component | VRAM (BF16) | Notes |
|-----------|-------------|-------|
| FLUX Transformer | 12-23 GB | Quantized: ~12GB |
| ControlNet | ~2-3 GB | Runs in parallel |
| VAE Decoder | ~335 MB | Shared |
| T5 Encoder | 3.3-9 GB | Quantized: ~3.3GB |
| CLIP Encoder | ~1 GB | Unloaded after use |
| **Total with ControlNet** | ~18-37 GB | Requires high VRAM |

**Recommendations:**
- Use quantized FLUX (GGUF) when possible
- Unload ControlNet when not in use
- Consider EasyControl (15M params) for low-VRAM systems

---

## Key Resources

### Models
- [MistoControlNet-Flux-dev](https://github.com/TheMistoAI/MistoControlNet-Flux-dev) - Best for sketches
- [XLabs flux-controlnet-collections](https://huggingface.co/XLabs-AI/flux-controlnet-collections) - Multi-purpose
- [InstantX ControlNet-Union](https://huggingface.co/InstantX/FLUX.1-dev-Controlnet-Union) - Versatile

### Libraries
- [edge-detection (Rust)](https://docs.rs/edge-detection/latest/edge_detection/) - Canny implementation
- [imageproc (Rust)](https://docs.rs/imageproc/latest/imageproc/) - Image processing
- [vue-drawing-canvas](https://github.com/razztyfication/vue-drawing-canvas) - Vue 3 drawing component

### References
- [Diffusers FLUX ControlNet](https://huggingface.co/docs/diffusers/en/api/pipelines/controlnet_flux)
- [EasyControl Paper](https://arxiv.org/html/2503.07027v1)
- [ControlNet Comparison Study](https://stable-diffusion-art.com/controlnet/)

---

## Conclusion

The recommended approach is:

1. **Model:** Start with **XLabs flux-controlnet-canny** for broad compatibility, then add **MistoControlNet-Flux-dev** for sketch-specific optimization

2. **Architecture:** Use standard FLUX ControlNet injection (block sample residuals)

3. **Preprocessing:** Implement Canny in Rust natively, use ONNX for HED/Lineart

4. **Client:** Use `vue-drawing-canvas` for the sketch UI, with options for brush size, color, and eraser

5. **VRAM:** Plan for ~20-25GB total, recommend users use quantized models

The existing codebase architecture (LoRA system, model cache, progress events) provides a solid foundation. The main work is implementing the ControlNet transformer blocks in Candle and creating the frontend drawing interface.
