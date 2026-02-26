# Edit Page (Flux.1 Kontext) — Design

## Overview

New page for image manipulation using Flux.1 Kontext [dev]. Users provide an input image and a text prompt describing the desired edit; the model produces a modified version.

## Page Structure

Route: `/edit` with `Main.vue` + `Menu.vue` named views.

```
frontend/src/pages/edit/
├── Main.vue            # Side-by-side comparison + history strip
├── Menu.vue            # Sidebar: image input, prompt, params, generate
├── History.vue         # Horizontal thumbnail strip
├── ImageInput.vue      # Drop zone + file picker + gallery pick + paste
├── CompareView.vue     # Left (input) / Right (output) layout
└── ProgressOverlay.vue # Generation progress
```

NavBar: New icon between Create and Gallery.

## Main Area — Side-by-Side Compare

- Two equal columns: input image (left), output image (right), labels above each
- Both images maintain native aspect ratio, scale to fit within their column
- Empty states when no image is loaded

### History Strip

- Horizontal scrollable thumbnails below the compare view, most recent on left
- Clicking a thumbnail: loads it as the output comparison, restores prompt and params
- Does NOT change the input image

### "Use as Input" Action

- Button on the output image
- Copies output image path to become the new input
- Output side clears for next generation

## Menu Sidebar

### Image Input Zone (top)

Thumbnail preview of current input image, or dashed-border drop zone when empty.

Four input methods (all converge to `editStore.setInputImage(path)`):

1. **File picker**: Calls `api.browse_input_image()` — native OS dialog, returns absolute path
2. **Gallery pick**: PrimeVue Dialog with grid of existing gallery images, returns `file_path` from DB
3. **Drag & drop**: `@drop` handler on the drop zone, extracts file path, validates image extension
4. **Clipboard paste**: Global `Ctrl+V` listener, reads clipboard image, saves to temp file via `api.save_clipboard_image()`. Best-effort — clipboard API in pywebview WebKit may be unreliable.

Small "Clear" button to remove the input image.

### Controls (below image input)

1. Prompt input — Tiptap editor (reuse from Create page)
2. Model select — filtered to `flux1_kontext` bundles only
3. Quality sliders — steps, CFG scale (collapsed by default)
4. Advanced — sampler, scheduler dropdowns
5. Seed — number input with randomize button

### Footer

- Generate button (disabled until input image + prompt provided)
- Cancel button when generating

## Kontext Store

Separate `useKontextStore` (Pinia Options API).

### State

- `inputImagePath: string | null` — local file path of source image
- `inputImageDataUrl: string | null` — base64 for display
- `generatedImages: GeneratedImage[]` — output history
- `selectedImageIndex: number` — which history item is shown
- `currentJobId`, `isGenerating`, `progress`, `error` — job lifecycle
- `params: KontextParams` — prompt, steps, cfg_scale, seed, sampler, scheduler, transformer fields
- `bundles: ModelBundle[]` — Kontext-type bundles only
- `selectedBundleId: string | null`

### Key Actions

- `setInputImage(path)` — sets path, loads base64 via `get_image_base64`
- `useOutputAsInput()` — copies selected output's imagePath to inputImagePath
- `submitJob()` — builds params including inputImagePath, calls `api.submit_job()`
- `selectImage(index)` — shows history image on output side, restores prompt/params
- `clearInputImage()` — resets input state

### Engine/Polling

Reads `engineReady` from `useInferenceStore()` and calls its `startEngine()`. Event polling is global on the inference store — the Kontext store filters polled events for its own `currentJobId`.

## Backend Changes (this repo)

- `backend/api/inference.py`: Add `browse_input_image()` — native file dialog returning path, and `save_clipboard_image()` for paste support
- `backend/bundles.py`: Add `"flux1_kontext"` to `DEFAULT_BUNDLE_TYPES` + default bundle entry for `black-forest-labs/FLUX.1-Kontext-dev`
- `_persist_image()`: Store `input_image_path` in `model_config` JSON snapshot
- No `submit_job()` changes needed — `ApiMeta` bridge auto-converts `inputImagePath` to `input_image_path`
- No database schema changes — `model_config` JSON handles the new field

## Engine Changes (rzem-ai-inference-engine)

- Add `FLUX1_KONTEXT = "flux1_kontext"` to `TransformerType` enum in `types.py`
- Add `input_image_path: str | None = None` to `JobParams`
- New `Flux1KontextPipeline` class: loads Kontext model via diffusers, reads input image from disk, runs inference with prompt + image conditioning
- Register in `InferenceEngine.__init__` under the new type
