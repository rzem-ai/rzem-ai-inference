# Rzem AI Inference - Development Resume Guide

## Current State (2026-01-19)

### What's Working
- FLUX image generation with Schnell/Dev models (quantized GGUF support)
- Vue 3 + Tauri 2 desktop app with dark theme UI
- Queue-based generation with real-time progress events
- Seed control with lock/unlock toggle (reproducible generations)
- Image drag-and-drop analysis via Claude Vision API
- Dual progress bars showing pipeline stages and denoising steps

### Recent Changes (Uncommitted)
1. **Seed Bug Fix** - Seeds now actually affect generation output
2. **Dual Progress Bars** - Pipeline progress + step counter in BottomPanel
3. **Component Cleanup** - Removed unused QueueList.vue and QueuePanel.vue
4. **Tooltip Directive** - Registered globally in main.ts

## Key Files

| Area | Files |
|------|-------|
| Generation Pipeline | `src-tauri/src/inference/pipeline.rs`, `src-tauri/src/models/flux.rs` |
| Queue Processing | `src-tauri/src/queue/processor.rs`, `src/stores/queue.ts` |
| Progress UI | `src/components/generation/BottomPanel.vue` |
| Seed Control | `src/components/generation/LeftSidebar.vue`, `src/stores/generation.ts` |
| Main View | `src/views/GenerateView.vue` |

## How to Resume

### 1. Check Current Status
```bash
cd /home/alex/Dev/Work/rzem-ai-inference
git status
git diff --stat
```

### 2. Run the App
```bash
npm run tauri dev
```

### 3. Test Changes
- Generate an image and verify dual progress bars appear
- Lock the seed, generate twice, verify identical images
- Unlock seed, generate twice, verify different images

### 4. Commit When Ready
```bash
git add -A
git commit -m "feat: seed control and dual progress bars

- Fix seed bug: now properly seeds noise generation via device.set_seed()
- Add dual progress bars: pipeline (teal) + steps (blue) in BottomPanel
- Add seed lock/unlock toggle in LeftSidebar
- Register Tooltip directive globally
- Remove unused QueueList.vue and QueuePanel.vue components"
```

## Pending Tasks
- [ ] Test dual progress bars with actual generation
- [ ] Remove debug console.log from GenerationInput.vue
- [ ] Test image drag-drop from web browsers

## Architecture Notes

### Progress Event Flow
```
Rust Pipeline → on_progress callback → Tauri emit("job-progress") →
Vue queue store listener → BottomPanel reactive update
```

### Seed Flow
```
Frontend (LeftSidebar) → generationStore.currentParams.seed →
GenerateView.handleGenerate() → queueStore.addToQueue() →
Rust processor → pipeline.generate_with_progress(seed) →
flux.denoise(seed) → device.set_seed(seed) → get_noise()
```

## Reference
- Detailed notes: `PROGRESS.md`
- Planning docs: `docs/plans/`
