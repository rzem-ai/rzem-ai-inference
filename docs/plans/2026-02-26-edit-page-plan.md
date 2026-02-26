# Edit Page (Flux.1 Kontext) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an `/edit` page that uses Flux.1 Kontext [dev] for image-to-image manipulation — input image + prompt produces a modified output.

**Architecture:** New `pages/edit/` directory with `Main.vue` + `Menu.vue` pattern. Dedicated `useEditStore` (Pinia Options API) manages input image, generation params, job lifecycle, and output history. Cross-store reading of engine state from `useInferenceStore`. Backend gets a `browse_input_image()` API method and a Kontext bundle. Engine repo gets a new `Flux1KontextPipeline`.

**Tech Stack:** Vue 3 + TypeScript + Pinia, PrimeVue 4, Tiptap, Lucide icons, Python pywebview API, diffusers (Kontext pipeline)

**Design doc:** `docs/plans/2026-02-26-kontext-page-design.md`

**No test framework** is configured — verification is manual (type-check, dev server, visual inspection).

---

## Task 1: Engine — Add Kontext type and input_image_path to JobParams

**Repo:** `../rzem-ai-inference-engine`

**Files:**
- Modify: `src/rzem_ai_inference_engine/types.py:15-20` (TransformerType enum)
- Modify: `src/rzem_ai_inference_engine/types.py:70-115` (JobParams model)

**Step 1: Add enum value**

In `types.py`, add `FLUX1_KONTEXT` to the `TransformerType` enum:

```python
class TransformerType(str, Enum):
    FLUX1_DEV = "flux1_dev"
    FLUX1_KONTEXT = "flux1_kontext"   # <-- new
    FLUX2_DEV = "flux2_dev"
    Z_IMAGE = "z_image"
    QWEN_IMAGE = "qwen_image"
    FAL_CLOUD = "fal_cloud"
```

**Step 2: Add input_image_path field**

In the `JobParams` class, add after the `prompt` field:

```python
input_image_path: str | None = None
```

**Step 3: Verify**

```bash
cd ../rzem-ai-inference-engine && uv run python -c "from rzem_ai_inference_engine.types import TransformerType, JobParams; print(TransformerType.FLUX1_KONTEXT); p = JobParams(prompt='test', transformer_model='x', transformer_type='flux1_kontext', vae_model='x'); print(p.input_image_path)"
```

Expected: `flux1_kontext` then `None`

**Step 4: Commit**

```bash
cd ../rzem-ai-inference-engine
git add src/rzem_ai_inference_engine/types.py
git commit -m "feat: add FLUX1_KONTEXT type and input_image_path to JobParams"
```

---

## Task 2: Engine — Create Flux1KontextPipeline

**Repo:** `../rzem-ai-inference-engine`

**Files:**
- Create: `src/rzem_ai_inference_engine/pipeline/flux1_kontext.py`

**Step 1: Create the pipeline**

Create `pipeline/flux1_kontext.py`. The Kontext pipeline is similar to `flux1.py` but:
- Requires `input_image_path` (validates it exists)
- Uses the same CLIP + T5 text encoding stage
- Loads the input image, encodes it through the VAE to get latents
- Uses `FluxTransformer2DModel` with image conditioning (Kontext-specific: the input latents are concatenated/injected into the denoising process)
- Output dimensions match the input image (no width/height from params)

The Kontext model from Black Forest Labs uses a modified Flux architecture where the input image latents are packed alongside noise latents. The key difference from txt2img Flux is:

```python
"""FLUX.1 Kontext pipeline — image-to-image editing via text-conditioned denoising."""

from __future__ import annotations

import math
from pathlib import Path
from typing import TYPE_CHECKING, Callable

import torch
from loguru import logger

from rzem_ai_inference_engine.models.loader import ModelLoader
from rzem_ai_inference_engine.models.memory import preferred_dtype
from rzem_ai_inference_engine.pipeline.base import BasePipeline
from rzem_ai_inference_engine.types import ModelSpec, ProgressEvent, TransformerType

if TYPE_CHECKING:
    import PIL.Image

    from rzem_ai_inference_engine.models.cache import ModelCache
    from rzem_ai_inference_engine.types import JobParams, PreviewConfig
```

The `validate_params` method should check for `input_image_path` existence. The `run` method should:
1. Load and encode text (CLIP + T5) — same as Flux1DevPipeline
2. Load input image from `params.input_image_path`, resize if needed, encode through VAE
3. Run the Kontext denoising loop with image latents as conditioning
4. Decode output latents through VAE
5. Return (output_image, seed)

**Important:** The exact Kontext pipeline implementation depends on how `diffusers` exposes the model. Check the diffusers docs/source for `FluxKontextPipeline` or `FluxImg2ImgPipeline` support. The pipeline class should follow the same stage-based VRAM management pattern as `Flux1DevPipeline` (load encoders → encode → release → load transformer → denoise → release → load VAE → decode → release).

**Step 2: Verify syntax**

```bash
cd ../rzem-ai-inference-engine && uv run python -c "from rzem_ai_inference_engine.pipeline.flux1_kontext import Flux1KontextPipeline; print('OK')"
```

**Step 3: Commit**

```bash
cd ../rzem-ai-inference-engine
git add src/rzem_ai_inference_engine/pipeline/flux1_kontext.py
git commit -m "feat: add Flux1KontextPipeline for image-to-image editing"
```

---

## Task 3: Engine — Register Kontext pipeline

**Repo:** `../rzem-ai-inference-engine`

**Files:**
- Modify: `src/rzem_ai_inference_engine/engine.py:64-73` (pipeline registration)

**Step 1: Add import and registration**

Add to imports at top of `engine.py`:

```python
from rzem_ai_inference_engine.pipeline.flux1_kontext import Flux1KontextPipeline
```

Add to `self._pipelines` dict in `__init__`:

```python
TransformerType.FLUX1_KONTEXT: Flux1KontextPipeline(),
```

**Step 2: Verify**

```bash
cd ../rzem-ai-inference-engine && uv run python -c "from rzem_ai_inference_engine.engine import InferenceEngine; print('OK')"
```

**Step 3: Commit**

```bash
cd ../rzem-ai-inference-engine
git add src/rzem_ai_inference_engine/engine.py
git commit -m "feat: register Flux1KontextPipeline in engine"
```

---

## Task 4: Backend — Add Kontext bundle type and default bundle

**Repo:** This repo (`rzem-ai-inference`)

**Files:**
- Modify: `backend/bundles.py:248-363` (DEFAULT_BUNDLE_TYPES — add entry)
- Modify: `backend/bundles.py:47-245` (DEFAULT_BUNDLES — add entry)

**Step 1: Add bundle type**

Add a new entry to `DEFAULT_BUNDLE_TYPES` list (after the `flux1_dev` entry, with `sort_order: 1` — shift existing sort orders up):

```python
{
    "id": "flux1_kontext",
    "label": "FLUX.1 Kontext",
    "icon": "gpu",
    "sort_order": 1,
    "guide": "## FLUX.1 Kontext [dev]\n\nImage-to-image editing model from Black Forest Labs. Provide an input image and a text prompt describing the desired edit.\n\n**Requirements:** ~24GB VRAM (BF16)\n\n**Best for:** Editing existing images — changing styles, adding/removing elements, modifying attributes.",
},
```

**Step 2: Add default bundle**

Add to `DEFAULT_BUNDLES` list (after the existing FLUX.1 Dev entries):

```python
ModelBundle(
    id="flux1_kontext_dev_quality",
    label="FLUX.1 Kontext [dev] BF16",
    description="Full precision Kontext model for image editing",
    transformer_type="flux1_kontext",
    tier="quality",
    transformer_model="black-forest-labs/FLUX.1-Kontext-dev",
    vae_model="black-forest-labs/FLUX.1-dev",
    clip_tokenizer="openai/clip-vit-large-patch14",
    clip_encoder="openai/clip-vit-large-patch14",
    t5_tokenizer="google/t5-v1_1-xxl",
    t5_encoder="google/t5-v1_1-xxl",
    steps=28,
    cfg_scale=3.5,
    sampler="euler",
    scheduler="simple",
    vram_estimate_gb=24.0,
),
```

**Step 3: Verify**

```bash
uv run python -c "from backend.bundles import DEFAULT_BUNDLES, DEFAULT_BUNDLE_TYPES; kt = [t for t in DEFAULT_BUNDLE_TYPES if t['id'] == 'flux1_kontext']; kb = [b for b in DEFAULT_BUNDLES if b.transformer_type == 'flux1_kontext']; print(f'Types: {len(kt)}, Bundles: {len(kb)}')"
```

Expected: `Types: 1, Bundles: 1`

**Note:** Users with existing databases will get the new bundle type and bundle on next app start via the seeding logic. No schema migration needed — the seeding in `database.py` inserts missing rows.

**Step 4: Commit**

```bash
git add backend/bundles.py
git commit -m "feat: add FLUX.1 Kontext bundle type and default bundle"
```

---

## Task 5: Backend — Add browse_input_image API method

**Files:**
- Modify: `backend/api/inference.py` (add new method)

**Step 1: Add the method**

Add `browse_input_image` to the `InferenceAPI` class. Pattern follows `browse_image_file` in `backend/api/styles.py:363-383` but simpler — no copy, just return the selected path:

```python
def browse_input_image(self, **kwargs) -> dict[str, Any]:
    """Open a native file dialog to select an input image. Returns the absolute path."""
    try:
        import webview
        window = webview.windows[0]
        file_filter = "Images (*.png;*.jpg;*.jpeg;*.webp;*.bmp;*.tiff)"
        result = window.create_file_dialog(
            webview.FileDialog.OPEN,
            allow_multiple=False,
            file_types=(file_filter,),
        )
        if not result:
            return {"status": "success", "path": None}
        path = str(result[0]) if isinstance(result, (list, tuple)) else str(result)
        return {"status": "success", "path": path}
    except Exception as e:
        logger.error("browse_input_image failed: %s", e)
        return {"status": "error", "message": str(e)}
```

**Step 2: Add save_clipboard_image method**

This method receives base64 image data from the frontend, saves to a temp file, returns the path:

```python
def save_clipboard_image(self, data_url: str, **kwargs) -> dict[str, Any]:
    """Save a base64 data URL image to a temp file. Returns the absolute path."""
    try:
        import base64
        import tempfile
        # Strip data URL prefix: "data:image/png;base64,..."
        header, b64data = data_url.split(",", 1)
        ext = ".png"
        if "jpeg" in header or "jpg" in header:
            ext = ".jpg"
        elif "webp" in header:
            ext = ".webp"
        raw = base64.b64decode(b64data)
        tmp = tempfile.NamedTemporaryFile(suffix=ext, delete=False, prefix="kontext_input_")
        tmp.write(raw)
        tmp.close()
        return {"status": "success", "path": tmp.name}
    except Exception as e:
        logger.error("save_clipboard_image failed: %s", e)
        return {"status": "error", "message": str(e)}
```

**Step 3: Verify**

```bash
uv run python -c "from backend.api.inference import InferenceAPI; print([m for m in dir(InferenceAPI) if 'browse_input' in m or 'clipboard' in m])"
```

Expected: `['browse_input_image', 'save_clipboard_image']`

**Step 4: Commit**

```bash
git add backend/api/inference.py
git commit -m "feat: add browse_input_image and save_clipboard_image API methods"
```

---

## Task 6: Backend — Store input_image_path in model_config

**Files:**
- Modify: `backend/services/inference_service.py` (find `_persist_image` method)

**Step 1: Pass input_image_path through**

In `_persist_image`, the `model_config` dict is built from `job_params`. Ensure `input_image_path` is included. Since `job_params` is stored as a dict and `model_config` is built from it, check if `input_image_path` is already captured. If `model_config` is built by cherry-picking fields, add `input_image_path` to the picked fields.

**Step 2: Commit**

```bash
git add backend/services/inference_service.py
git commit -m "feat: persist input_image_path in model_config for edit jobs"
```

---

## Task 7: Frontend — Update TypeScript types

**Files:**
- Modify: `frontend/src/types/inference.ts:5` (TransformerType)
- Modify: `frontend/src/types/inference.ts:78-104` (SubmitJobParams)
- Modify: `frontend/src/types/pywebview.d.ts` (PywebviewAPI interface)

**Step 1: Add flux1_kontext to TransformerType**

```typescript
export type TransformerType = "flux1_dev" | "flux1_kontext" | "flux2_dev" | "z_image" | "qwen_image" | "fal_cloud";
```

**Step 2: Add input_image_path to SubmitJobParams**

Add to the `SubmitJobParams` interface:

```typescript
input_image_path?: string;
```

**Step 3: Add API methods to PywebviewAPI**

Add to the `PywebviewAPI` interface (in the Jobs or Inference section):

```typescript
browse_input_image(): Promise<ApiResponse<{ path?: string | null }>>;
save_clipboard_image(args: { data_url: string }): Promise<ApiResponse<{ path?: string }>>;
```

**Step 4: Update mock API in bridge.ts**

Add mock implementations for `browse_input_image` and `save_clipboard_image` in the mock API object in `frontend/src/bridge.ts` so browser-only dev works:

```typescript
browse_input_image: async () => ({ status: 'success' as const, path: null }),
save_clipboard_image: async () => ({ status: 'success' as const, path: '/tmp/mock_clipboard.png' }),
```

**Step 5: Verify**

```bash
cd frontend && npm run type-check
```

**Step 6: Commit**

```bash
git add frontend/src/types/inference.ts frontend/src/types/pywebview.d.ts frontend/src/bridge.ts
git commit -m "feat: add Kontext types and API declarations to frontend"
```

---

## Task 8: Frontend — Guard inference store event processing

**Files:**
- Modify: `frontend/src/stores/inference.ts:532-680` (processEvents method)

**Step 1: Add job ownership check**

The inference store's `processEvents` currently updates `isGenerating`, `progress`, etc. for ANY job event. When the edit store submits jobs, those events must not mutate the inference store's state.

Add a helper method and guard job-specific event cases:

```typescript
_ownsJob(jobId: string): boolean {
  if (jobId === this.currentJobId) return true;
  if (this.batchJobIds.includes(jobId)) return true;
  if (this.gridJobMap.has(jobId)) return true;
  return false;
},
```

In `processEvents`, for cases `job_started`, `job_progress`, `job_completed`, `job_failed`, `job_cancelled` — add at the top of each case:

```typescript
case 'job_started': {
  const jobId = event.data.job_id;
  if (jobId && !this._ownsJob(jobId)) break;
  // ... existing logic
}
```

Global events (`model_loading`, `model_loaded`, `model_unloaded`, `server_connected`, `server_disconnected`) remain unguarded — both stores care about these.

**Important:** The `this.events.push(event)` line (before the switch) must remain for ALL events, so the edit store can watch the events array and process its own.

**Step 2: Verify**

```bash
cd frontend && npm run type-check
```

**Step 3: Commit**

```bash
git add frontend/src/stores/inference.ts
git commit -m "feat: guard inference store event processing to only owned jobs"
```

---

## Task 9: Frontend — Create useEditStore

**Files:**
- Create: `frontend/src/stores/edit.ts`

**Step 1: Create the store**

Pinia Options API store. Key design points:
- Own state for `inputImagePath`, `inputImageDataUrl`, `generatedImages`, `selectedImageIndex`, `currentJobId`, `isGenerating`, `progress`, `error`, `params`, `bundles`, `selectedBundleId`
- `params` shape: `{ prompt, transformer_model, transformer_type, vae_model, steps, cfg_scale, seed, sampler, scheduler, clip_*, t5_* }` — same as `SubmitJobParams` but with `input_image_path`
- `submitJob()` builds job params including `input_image_path: this.inputImagePath`
- `setInputImage(path)` loads base64 via `api.get_image_base64`
- `useOutputAsInput()` copies selected output path to input
- `selectImage(index)` loads image as comparison, restores prompt/params
- Watches `inferenceStore.events` array length for new events, filters by own `currentJobId`

```typescript
import { defineStore } from 'pinia';
import { getApiAsync } from '@/bridge';
import { useInferenceStore } from '@/stores/inference';
import { useModelsStore } from '@/stores/models';
import type { ModelBundle, GeneratedImage, SubmitJobParams } from '@/types/inference';

export const useEditStore = defineStore('edit', {
  state: () => ({
    // Input image
    inputImagePath: null as string | null,
    inputImageDataUrl: null as string | null,

    // Job state
    currentJobId: null as string | null,
    isGenerating: false,
    progress: null as { step: number; totalSteps: number } | null,
    error: null as string | null,

    // Results
    generatedImages: [] as GeneratedImage[],
    selectedImageIndex: 0,
    previewDataUrl: null as string | null,

    // Bundles & params
    bundles: [] as ModelBundle[],
    selectedBundleId: null as string | null,

    params: {
      prompt: '',
      transformer_model: 'black-forest-labs/FLUX.1-Kontext-dev',
      transformer_type: 'flux1_kontext',
      vae_model: 'black-forest-labs/FLUX.1-dev',
      clip_tokenizer: 'openai/clip-vit-large-patch14',
      clip_encoder: 'openai/clip-vit-large-patch14',
      t5_tokenizer: 'google/t5-v1_1-xxl',
      t5_encoder: 'google/t5-v1_1-xxl',
      steps: 28,
      cfg_scale: 3.5,
      width: 1024,
      height: 1024,
      seed: -1,
      sampler: 'euler',
      scheduler: 'simple',
      loras: [],
    } as SubmitJobParams,

    // Track last processed event index
    _lastEventIndex: 0,
  }),

  getters: {
    selectedImage(state): GeneratedImage | null {
      return state.generatedImages[state.selectedImageIndex] ?? null;
    },
    engineReady(): boolean {
      return useInferenceStore().engineReady;
    },
    modelStatus(): string | null {
      return useInferenceStore().modelStatus;
    },
  },

  actions: {
    async setInputImage(path: string) { ... },
    clearInputImage() { ... },
    useOutputAsInput() { ... },
    async loadBundles() { ... },
    applyBundle(bundle: ModelBundle) { ... },
    async submitJob() { ... },
    selectImage(index: number) { ... },
    async cancelJob() { ... },
    processNewEvents() { ... },
    async loadCompletedImage(img: GeneratedImage) { ... },
  },
});
```

**Step 2: Implement all actions**

`setInputImage`:
```typescript
async setInputImage(path: string) {
  this.inputImagePath = path;
  const api = await getApiAsync();
  const res = await api.get_image_base64({ image_path: path });
  if (res.status === 'success' && res.data_url) {
    this.inputImageDataUrl = res.data_url;
  }
},
```

`clearInputImage`:
```typescript
clearInputImage() {
  this.inputImagePath = null;
  this.inputImageDataUrl = null;
},
```

`useOutputAsInput`:
```typescript
useOutputAsInput() {
  const img = this.selectedImage;
  if (img?.imagePath) {
    this.setInputImage(img.imagePath);
  }
},
```

`loadBundles`:
```typescript
async loadBundles() {
  const modelsStore = useModelsStore();
  if (!modelsStore.bundleTypes.length) {
    await modelsStore.loadBundleTypes();
  }
  const api = await getApiAsync();
  const res = await api.get_bundles();
  if (res.status === 'success' && res.bundles) {
    this.bundles = res.bundles.filter((b: ModelBundle) => b.transformer_type === 'flux1_kontext');
  }
},
```

`applyBundle` — same pattern as inference store's `applyBundle`.

`submitJob`:
```typescript
async submitJob() {
  if (!this.engineReady || this.isGenerating) return;
  if (!this.params.prompt.trim()) { this.error = 'Prompt is required'; return; }
  if (!this.inputImagePath) { this.error = 'Input image is required'; return; }

  this.error = null;
  this.isGenerating = true;
  this.progress = null;

  const api = await getApiAsync();
  const jobParams: Record<string, any> = {
    ...this.params,
    input_image_path: this.inputImagePath,
  };
  if (this.selectedBundleId) jobParams.bundle_id = this.selectedBundleId;

  for (const key of Object.keys(jobParams)) {
    if (jobParams[key] === undefined || jobParams[key] === '') delete jobParams[key];
  }

  const res = await api.submit_job(jobParams);
  if (res.status === 'error') {
    this.error = res.message ?? 'Failed to submit job';
    this.isGenerating = false;
    return;
  }
  this.currentJobId = res.job_id ?? null;
},
```

`processNewEvents` — called by a watcher in the edit page's `Menu.vue` on mount. Watches `inferenceStore.events.length` and processes any events where `event.data.job_id === this.currentJobId`:

```typescript
processNewEvents() {
  const inferenceStore = useInferenceStore();
  const events = inferenceStore.events;
  while (this._lastEventIndex < events.length) {
    const event = events[this._lastEventIndex++];
    const jobId = event.data?.job_id;

    // Process global events
    if (['model_loading', 'model_loaded', 'model_unloaded'].includes(event.type)) {
      continue; // Handled by inference store, we read via getters
    }

    // Only process our own job events
    if (jobId && jobId !== this.currentJobId) continue;

    switch (event.type) {
      case 'job_started':
        this.isGenerating = true;
        this.progress = { step: 0, totalSteps: this.params.steps };
        break;
      case 'job_progress':
        this.progress = {
          step: event.data.step ?? 0,
          totalSteps: event.data.total_steps ?? this.params.steps,
        };
        if (event.data.preview_path) {
          this.loadPreview(event.data.preview_path);
        }
        break;
      case 'job_completed':
        // Build GeneratedImage, load data URL, unshift to history
        this.handleJobCompleted(event);
        break;
      case 'job_failed':
        this.progress = null;
        this.previewDataUrl = null;
        this.error = event.data.error ?? 'Generation failed';
        this.isGenerating = false;
        this.currentJobId = null;
        break;
      case 'job_cancelled':
        this.progress = null;
        this.previewDataUrl = null;
        this.isGenerating = false;
        this.currentJobId = null;
        break;
    }
  }
},
```

**Step 3: Verify**

```bash
cd frontend && npm run type-check
```

**Step 4: Commit**

```bash
git add frontend/src/stores/edit.ts
git commit -m "feat: add useEditStore for Kontext image editing"
```

---

## Task 10: Frontend — Create ImageInput.vue component

**Files:**
- Create: `frontend/src/pages/edit/ImageInput.vue`

**Step 1: Create the component**

Handles all four image input methods. Layout:
- When empty: dashed border drop zone with icon, "Drop image here" text, and two buttons (Browse File, Pick from Gallery)
- When loaded: thumbnail preview with a small X button to clear

```vue
<template>
  <div
    class="relative rounded-xl border-2 border-dashed transition-colors"
    :class="isDragging ? 'border-blue-400 bg-blue-950/20' : inputImageDataUrl ? 'border-transparent' : 'border-surface-300'"
    @dragover.prevent="isDragging = true"
    @dragleave="isDragging = false"
    @drop.prevent="onDrop">

    <!-- Loaded state -->
    <div v-if="inputImageDataUrl" class="relative">
      <img :src="inputImageDataUrl" alt="Input image" class="w-full rounded-xl object-contain max-h-48" />
      <Button
        class="absolute top-1 right-1"
        severity="danger"
        size="small"
        rounded
        text
        @click="store.clearInputImage()">
        <X :size="14" />
      </Button>
    </div>

    <!-- Empty state -->
    <div v-else class="flex flex-col items-center gap-3 py-6 px-4">
      <ImagePlus :size="32" class="text-surface-400" />
      <span class="text-sm text-surface-400">Drop image here or</span>
      <div class="flex gap-2">
        <Button size="small" severity="secondary" variant="outlined" @click="browseFile">
          <Upload :size="14" class="mr-1" /> Browse
        </Button>
        <Button size="small" severity="secondary" variant="outlined" @click="showGalleryPicker = true">
          <Images :size="14" class="mr-1" /> Gallery
        </Button>
      </div>
    </div>
  </div>

  <GalleryPickerDialog v-model:visible="showGalleryPicker" @pick="onGalleryPick" />
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useEditStore } from '@/stores/edit';
import { getApiAsync } from '@/bridge';
import GalleryPickerDialog from './GalleryPickerDialog.vue';

const store = useEditStore();
const isDragging = ref(false);
const showGalleryPicker = ref(false);

const inputImageDataUrl = computed(() => store.inputImageDataUrl);

async function browseFile() {
  const api = await getApiAsync();
  const res = await api.browse_input_image();
  if (res.status === 'success' && res.path) {
    store.setInputImage(res.path);
  }
}

function onGalleryPick(imagePath: string) {
  store.setInputImage(imagePath);
  showGalleryPicker.value = false;
}

function onDrop(e: DragEvent) {
  isDragging.value = false;
  // Check for image path from drag data (e.g. dragged from gallery)
  const imagePath = e.dataTransfer?.getData('text/image-path');
  if (imagePath) {
    store.setInputImage(imagePath);
    return;
  }
  // Check for dropped files
  const file = e.dataTransfer?.files?.[0];
  if (file && /\.(png|jpe?g|webp|bmp|tiff?)$/i.test(file.name)) {
    // In pywebview, file.path gives the absolute path
    if ((file as any).path) {
      store.setInputImage((file as any).path);
    }
  }
}
</script>
```

**Step 2: Verify**

```bash
cd frontend && npm run type-check
```

**Step 3: Commit**

```bash
git add frontend/src/pages/edit/ImageInput.vue
git commit -m "feat: add ImageInput component with file picker, gallery, and drag & drop"
```

---

## Task 11: Frontend — Create GalleryPickerDialog.vue

**Files:**
- Create: `frontend/src/pages/edit/GalleryPickerDialog.vue`

**Step 1: Create the dialog**

A PrimeVue Dialog that shows a grid of gallery images for selection:

```vue
<template>
  <Dialog
    v-model:visible="visible"
    header="Pick an image"
    modal
    :style="{ width: '640px', maxHeight: '80vh' }">
    <div class="grid grid-cols-4 gap-2 overflow-y-auto max-h-96">
      <div
        v-for="img in images"
        :key="img.id"
        class="cursor-pointer rounded overflow-hidden border-2 border-transparent hover:border-blue-400 transition-colors"
        @click="emit('pick', img.file_path)">
        <img :src="img.thumbnailDataUrl" alt="" class="w-full aspect-square object-cover" />
      </div>
    </div>
    <div v-if="!images.length" class="text-center text-surface-400 py-8">
      No images in gallery yet
    </div>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { getApiAsync } from '@/bridge';

const visible = defineModel<boolean>('visible');
const emit = defineEmits<{ pick: [imagePath: string] }>();

const images = ref<any[]>([]);

watch(visible, async (v) => {
  if (!v) return;
  const api = await getApiAsync();
  const res = await api.get_gallery_images({ limit: 50, offset: 0 });
  if (res.status === 'success' && res.images) {
    // Load thumbnail data URLs
    for (const img of res.images) {
      if (img.thumbnail_path) {
        const thumbRes = await api.get_image_base64({ image_path: img.thumbnail_path });
        if (thumbRes.status === 'success') {
          img.thumbnailDataUrl = thumbRes.data_url;
        }
      }
    }
    images.value = res.images;
  }
});
</script>
```

**Step 2: Verify**

```bash
cd frontend && npm run type-check
```

**Step 3: Commit**

```bash
git add frontend/src/pages/edit/GalleryPickerDialog.vue
git commit -m "feat: add GalleryPickerDialog for image selection from gallery"
```

---

## Task 12: Frontend — Create CompareView.vue

**Files:**
- Create: `frontend/src/pages/edit/CompareView.vue`

**Step 1: Create side-by-side comparison component**

```vue
<template>
  <div ref="wrapper" class="flex gap-4 h-full w-full items-center justify-center p-4">
    <!-- Input side -->
    <div class="flex-1 flex flex-col items-center gap-2 h-full">
      <span class="text-sm text-surface-400 font-medium">Input</span>
      <div class="flex-1 flex items-center justify-center w-full overflow-hidden">
        <img v-if="store.inputImageDataUrl" :src="store.inputImageDataUrl" alt="Input" class="max-w-full max-h-full object-contain rounded-xl" />
        <div v-else class="border border-surface-200 rounded-xl bg-surface-100 flex flex-col items-center justify-center w-full h-full max-w-md max-h-96 gap-2">
          <ImageIcon :size="48" class="text-slate-500" />
          <span class="text-lg text-slate-500">Select an input image</span>
          <span class="text-base text-slate-400">Use the sidebar to pick or drop an image</span>
        </div>
      </div>
    </div>

    <!-- Output side -->
    <div class="flex-1 flex flex-col items-center gap-2 h-full">
      <span class="text-sm text-surface-400 font-medium">Output</span>
      <div class="flex-1 flex items-center justify-center w-full overflow-hidden relative">
        <!-- Preview during generation -->
        <img v-if="store.isGenerating && store.previewDataUrl" :src="store.previewDataUrl" alt="Preview" class="max-w-full max-h-full object-contain rounded-xl" style="filter: blur(1px)" />
        <!-- Completed output -->
        <template v-else-if="outputDataUrl">
          <img :src="outputDataUrl" alt="Output" class="max-w-full max-h-full object-contain rounded-xl" />
          <Button
            class="absolute bottom-3 right-3"
            severity="secondary"
            size="small"
            raised
            title="Use as input"
            @click="store.useOutputAsInput()">
            <ArrowLeftToLine :size="14" class="mr-1" /> Use as Input
          </Button>
        </template>
        <!-- Empty state -->
        <div v-else class="border border-surface-200 rounded-xl bg-surface-100 flex flex-col items-center justify-center w-full h-full max-w-md max-h-96 gap-2">
          <Sparkles :size="48" class="text-slate-500" />
          <span class="text-lg text-slate-500">Output will appear here</span>
          <span class="text-base text-slate-400">Enter a prompt and click Generate</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useEditStore } from '@/stores/edit';

const store = useEditStore();
const outputDataUrl = computed(() => store.selectedImage?.dataUrl ?? null);
</script>
```

**Step 2: Commit**

```bash
git add frontend/src/pages/edit/CompareView.vue
git commit -m "feat: add CompareView side-by-side comparison component"
```

---

## Task 13: Frontend — Create edit page History.vue

**Files:**
- Create: `frontend/src/pages/edit/History.vue`

**Step 1: Create the history strip**

Same pattern as `pages/create/History.vue` but uses `useEditStore`:

```vue
<template>
  <Card class="min-h-30">
    <template #content>
      <div class="flex gap-1 overflow-x-auto" v-if="store.generatedImages.length">
        <div
          v-for="(img, index) in store.generatedImages"
          :key="img.jobId"
          class="shrink-0 w-20 h-20 rounded cursor-pointer overflow-hidden border-2 transition-colors"
          :class="index === store.selectedImageIndex ? 'border-blue-500' : 'border-transparent hover:border-slate-300'"
          @click="store.selectImage(index)">
          <img v-if="img.dataUrl" :src="img.dataUrl" alt="" draggable="false" class="w-full h-full object-cover" />
          <div v-else class="w-full h-full bg-slate-200 flex items-center justify-center">
            <ImageIcon :size="10" class="text-slate-400" />
          </div>
        </div>
      </div>
    </template>
  </Card>
</template>

<script setup lang="ts">
import { useEditStore } from '@/stores/edit';
const store = useEditStore();
</script>
```

**Step 2: Commit**

```bash
git add frontend/src/pages/edit/History.vue
git commit -m "feat: add edit page History strip component"
```

---

## Task 14: Frontend — Create edit page Main.vue

**Files:**
- Create: `frontend/src/pages/edit/Main.vue`

**Step 1: Create the main view**

Combines CompareView and History:

```vue
<template>
  <div class="flex flex-col h-full px-2 py-4 gap-2">
    <CompareView class="flex-1" />
    <History />
  </div>
</template>

<script setup lang="ts">
import CompareView from './CompareView.vue';
import History from './History.vue';
</script>
```

**Step 2: Commit**

```bash
git add frontend/src/pages/edit/Main.vue
git commit -m "feat: add edit page Main.vue"
```

---

## Task 15: Frontend — Create edit page PromptInput.vue

**Files:**
- Create: `frontend/src/pages/edit/PromptInput.vue`

**Step 1: Create the prompt input**

Adapted from `pages/create/PromptInput.vue` but uses `useEditStore`. Same Tiptap setup, reads/writes `store.params.prompt`:

```vue
<template>
  <div class="prompt-editor rounded-xl bg-surface-0 border border-surface-200 px-3 py-2 min-h-24 max-h-48 overflow-y-auto cursor-text" @click="editor?.commands.focus()">
    <EditorContent :editor="editor" />
  </div>
</template>

<script setup lang="ts">
import { watch, onBeforeUnmount } from 'vue';
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import { useEditStore } from '@/stores/edit';

const emit = defineEmits<{ submit: [] }>();
const store = useEditStore();

const editor = useEditor({
  content: store.params.prompt,
  extensions: [
    StarterKit.configure({
      heading: false, codeBlock: false, blockquote: false,
      bold: false, italic: false, strike: false, code: false,
      bulletList: false, orderedList: false, listItem: false,
      horizontalRule: false,
    }),
    Placeholder.configure({ placeholder: 'Describe the edit you want to make...' }),
  ],
  editorProps: {
    handleKeyDown(view, event) {
      if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) {
        emit('submit');
        return true;
      }
      return false;
    },
  },
  onUpdate({ editor: e }) {
    const text = e.getText();
    store.params.prompt = text;
  },
});

watch(() => store.params.prompt, (newVal) => {
  if (!editor.value) return;
  const current = editor.value.getText();
  if (current !== newVal) {
    editor.value.commands.setContent(newVal || '');
  }
});

onBeforeUnmount(() => editor.value?.destroy());
</script>
```

**Step 2: Commit**

```bash
git add frontend/src/pages/edit/PromptInput.vue
git commit -m "feat: add edit page PromptInput component"
```

---

## Task 16: Frontend — Create edit page Menu.vue

**Files:**
- Create: `frontend/src/pages/edit/Menu.vue`

**Step 1: Create the sidebar**

Follows the same `MenuPanel` + slot pattern as `pages/create/Menu.vue`:

```vue
<template>
  <MenuPanel title="Edit" icon="PenToolIcon">
    <template #content>
      <div class="flex-1 overflow-y-auto px-4 pb-2 flex flex-col gap-4">
        <ImageInput />
        <PromptInput @submit="onSubmit" />
        <ModelSelect />
        <QualitySection />
      </div>
      <div v-if="store.error" class="px-4 py-1">
        <span class="text-sm text-red-500">{{ store.error }}</span>
      </div>
    </template>
    <template #footer>
      <ProgressOverlay v-if="store.isGenerating" :progress="store.progress" />
      <div v-if="!store.isGenerating" class="flex gap-2">
        <Button
          class="flex-1 transition-colors flex items-center justify-center gap-2"
          severity="primary"
          raised
          :disabled="!canGenerate"
          @click="onSubmit">
          <Sparkles :size="16" /> Generate
        </Button>
      </div>
      <Button
        v-else
        class="transition-colors flex items-center justify-center gap-2"
        severity="danger"
        fluid
        raised
        @click="store.cancelJob()">
        <Square :size="14" /> Cancel
      </Button>
    </template>
  </MenuPanel>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, watch } from 'vue';
import { useEditStore } from '@/stores/edit';
import { useInferenceStore } from '@/stores/inference';
import ImageInput from './ImageInput.vue';
import PromptInput from './PromptInput.vue';
import ModelSelect from './ModelSelect.vue';
import QualitySection from './QualitySection.vue';
import ProgressOverlay from './ProgressOverlay.vue';
import MenuPanel from '@/components/MenuPanel.vue';

const store = useEditStore();
const inferenceStore = useInferenceStore();

const canGenerate = computed(() =>
  store.engineReady && store.inputImagePath && store.params.prompt.trim() && !store.isGenerating
);

function onSubmit() {
  store.submitJob();
}

// Watch inference store events for our job updates
const stopWatch = watch(
  () => inferenceStore.events.length,
  () => store.processNewEvents(),
);

// Clipboard paste handler
function onPaste(e: ClipboardEvent) {
  const items = e.clipboardData?.items;
  if (!items) return;
  for (const item of items) {
    if (item.type.startsWith('image/')) {
      const blob = item.getAsFile();
      if (!blob) continue;
      const reader = new FileReader();
      reader.onload = async () => {
        const dataUrl = reader.result as string;
        const api = await (await import('@/bridge')).getApiAsync();
        const res = await api.save_clipboard_image({ data_url: dataUrl });
        if (res.status === 'success' && res.path) {
          store.setInputImage(res.path);
        }
      };
      reader.readAsDataURL(blob);
      break;
    }
  }
}

onMounted(async () => {
  window.addEventListener('paste', onPaste);
  await store.loadBundles();
  if (store.bundles.length && !store.selectedBundleId) {
    store.applyBundle(store.bundles[0]);
  }
  // Ensure engine is running
  if (!inferenceStore.engineReady && !inferenceStore.engineStarting) {
    await inferenceStore.startEngine();
  }
});

onUnmounted(() => {
  window.removeEventListener('paste', onPaste);
  stopWatch();
});
</script>
```

**Step 2: Create supporting sub-components**

Create `ModelSelect.vue` and `QualitySection.vue` for the edit page, or reuse from create page if they can be parameterized. Given the "may diverge" requirement, create thin edit-specific versions:

- `pages/edit/ModelSelect.vue` — Dropdown of `store.bundles` (already filtered to Kontext type)
- `pages/edit/QualitySection.vue` — Collapsible section with steps, CFG, seed, sampler, scheduler sliders
- `pages/edit/ProgressOverlay.vue` — Progress bar, same pattern as create page

**Step 3: Verify**

```bash
cd frontend && npm run type-check
```

**Step 4: Commit**

```bash
git add frontend/src/pages/edit/
git commit -m "feat: add edit page Menu.vue with all sub-components"
```

---

## Task 17: Frontend — Router and NavBar integration

**Files:**
- Modify: `frontend/src/router/index.ts` (add route)
- Modify: `frontend/src/components/NavBar.vue` (add icon)

**Step 1: Add route**

Add after the `/create` route in `router/index.ts`:

```typescript
{
  path: '/edit',
  name: 'edit',
  components: {
    default: () => import('@/pages/edit/Main.vue'),
    menu: () => import('@/pages/edit/Menu.vue'),
  },
},
```

**Step 2: Add NavBar icon**

In `NavBar.vue`, add a new `RouterLink` between the Create (`ImagePlus`) and Gallery (`Images`) links. Use `PenTool` from Lucide:

```vue
<RouterLink :to="{ name: 'edit' }" v-slot="{ isActive }">
  <div
    class="p-3 transition-all duration-300 ease-in-out"
    :class="isActive ? 'text-white' : 'text-surface-400 hover:text-white'">
    <PenTool :size="20" />
  </div>
</RouterLink>
```

Add `PenTool` to the Lucide imports in `NavBar.vue`'s `<script>`.

**Step 3: Verify**

```bash
cd frontend && npm run type-check
```

**Step 4: Manual test**

```bash
cd frontend && npm run dev
# Open http://localhost:1978 in browser
# Verify: new PenTool icon appears in NavBar between Create and Gallery
# Click it — should navigate to /edit with sidebar and empty compare view
# All four states visible: empty input, empty output, sidebar controls
```

**Step 5: Commit**

```bash
git add frontend/src/router/index.ts frontend/src/components/NavBar.vue
git commit -m "feat: add /edit route and NavBar icon"
```

---

## Task 18: Integration — End-to-end verification

**Step 1: Frontend type-check**

```bash
cd frontend && npm run type-check
```

**Step 2: Build check**

```bash
cd frontend && npm run build
```

**Step 3: Manual dev test (browser-only, mock API)**

```bash
cd frontend && npm run dev
```

Verify in browser:
- [ ] Navigate to `/edit` via NavBar icon
- [ ] Empty state shows correctly (input placeholder, output placeholder)
- [ ] Browse button opens (mock returns null, that's OK)
- [ ] Gallery picker dialog opens and closes
- [ ] Prompt input accepts text, Ctrl+Enter triggers submit
- [ ] Generate button disabled when no input image or empty prompt
- [ ] Error message displays when required fields missing

**Step 4: Manual dev test (full Python backend)**

```bash
bash scripts/dev.sh
```

Verify:
- [ ] Browse File opens native OS dialog
- [ ] Selecting an image shows preview in ImageInput zone
- [ ] Drag & drop from gallery page works
- [ ] Clipboard paste works (best-effort)
- [ ] Generate submits job with input_image_path
- [ ] Progress bar animates during generation
- [ ] Completed image appears on output side
- [ ] History strip populates
- [ ] Clicking history item loads that output and restores prompt
- [ ] "Use as Input" swaps output to input side

**Step 5: Final commit**

```bash
git add -A
git commit -m "feat: complete Edit page for Flux.1 Kontext image editing"
```
