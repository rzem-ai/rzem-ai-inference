# CLAUDE.md - AI Assistant Guide (Python Backend)

## ⚠️ CRITICAL: NO BACKWARD COMPATIBILITY

**HIGHEST PRIORITY RULE:**

This application has NOT been released to users. There are NO production deployments. As such, **backward compatibility is NOT required and should NOT be implemented.**

**DO NOT:**
- ❌ Keep old code paths "for compatibility"
- ❌ Add fallback logic for legacy behavior
- ❌ Maintain deprecated fields or methods
- ❌ Write migration code for unreleased features
- ❌ Write database migration code (schema changes = delete DB)
- ❌ Preserve old APIs "just in case"
- ❌ Add conditional logic like `if old_field exists... else new_field`

**DO:**
- ✅ Delete old code completely when replacing it
- ✅ Update all references to use new approach
- ✅ Break things if needed to move forward
- ✅ Refactor aggressively
- ✅ Simplify without legacy concerns
- ✅ For database schema changes: tell user to delete the database file

**If you find backward compatibility code: REMOVE IT.**

This rule overrides all other considerations. Clean, simple code is more valuable than compatibility that no users need.

---

## Purpose
This document helps Claude Code work effectively with the rzem-ai-inference codebase using the **Python backend**. It documents architectural patterns, conventions, and critical areas specific to this project.

## Tech Stack
- **Frontend**: Vue 3 + TypeScript, Pinia stores, PrimeVue UI, TailwindCSS
- **Backend**: Python + pywebview, PyTorch + Diffusers, aiosqlite, asyncio
- **ML Pipeline**: FLUX.1 via Diffusers (text encoder → transformer → VAE decoder)
- **Features**: LoRA support, job queue, gallery with metadata

## Backend Architecture Change

**Previous**: Rust + Tauri 2 + Candle ML framework
**Current**: Python + pywebview + PyTorch/Diffusers

### Why Python?

- **Mature ML Ecosystem**: PyTorch/Diffusers have better FLUX.1 support
- **Easier Development**: Python is more accessible for ML development
- **HuggingFace Integration**: Direct model loading from HuggingFace Hub
- **Reference Implementation**: InvokeAI provides excellent patterns

### Key Differences

| Aspect | Rust/Tauri | Python/pywebview |
|--------|------------|------------------|
| Desktop Wrapper | Tauri 2 | pywebview 5+ |
| ML Framework | Candle | PyTorch + Diffusers |
| Async Runtime | Tokio | asyncio |
| IPC | Tauri commands | pywebview JS API |
| Events | WebSocket-based | Polling (100ms) |
| GPU | Feature flags | Auto-detection |

---

## Python Backend Structure

```
src-python/
├── main.py              # Entry point, CLI args, pywebview setup
├── app_state.py         # Central application state
├── api.py               # API bridge (replaces Tauri commands)
├── events.py            # Event queue for frontend polling
├── inference/           # Image generation
│   ├── device.py        # GPU/CPU device selection
│   └── flux_pipeline.py # FLUX.1 pipeline wrapper
├── queue/               # Job queue management
│   ├── types.py         # Pydantic models (Job, Params)
│   ├── manager.py       # QueueManager (add/cancel/update jobs)
│   └── processor.py     # QueueProcessor (execute jobs)
├── db/                  # Database
│   └── database.py      # InferenceDb (aiosqlite wrapper)
├── shared/              # Shared types
│   └── protocol.py      # RuntimeConfig
└── requirements.txt     # Python dependencies
```

---

## Vue 3 Coding Standards (Unchanged)

### Core Principles
- **Composition API for Components**: Always use `<script setup>` for Vue components
- **Options API for Stores**: Use Options API (`state`, `getters`, `actions`) for Pinia stores
- **Component Order**: Always order component blocks as `<template>`, `<script>`, `<style>`
- **TypeScript First**: All components must use TypeScript with proper type definitions
- **Prefer `interface` over `type`**: For object shapes (easier to extend, better errors)
- **Named Exports**: Prefer named exports over default exports
- **Named Functions**: Use named functions for methods, arrow functions only for callbacks

### Backend Communication (NEW)

**Use the backend bridge** for all backend communication:

```typescript
// ✅ Correct - works with both Tauri and pywebview
import { invoke, listen } from '@/utils/backend-bridge';

// ❌ Wrong - only works with Tauri
import { invoke } from '@tauri-apps/api/core';
```

**Invoking commands:**

```typescript
// Queue generation
const result = await invoke<{ status: string; job_id?: string }>('queue_generation', {
  prompt: 'a beautiful sunset',
  steps: 4,
  width: 1024,
  height: 1024,
  seed: -1,
  model_component_id: 'flux-schnell',
  t5_component_id: 't5-xxl',
  clip_component_id: 'clip-l',
  vae_component_id: 'vae',
});

if (result.status === 'success') {
  console.log('Job queued:', result.job_id);
}
```

**Listening to events:**

```typescript
const unlisten = await listen('job-progress', (payload) => {
  console.log('Progress:', payload.progress);
  console.log('Stage:', payload.stage);
});

// Cleanup
onUnmounted(() => unlisten());
```

**Event polling (automatic):**
The backend-bridge automatically polls for events when using pywebview. No code changes needed!

---

## Python Coding Standards

### Type Hints

Always use type hints:

```python
from typing import Optional, List, Dict, Any

async def add_job(self, params: GenerationParams) -> str:
    """Add a new job to the queue"""
    job = GenerationJob(params=params)
    # ...
    return job.id
```

### Pydantic Models

Use Pydantic for data validation:

```python
from pydantic import BaseModel, Field

class GenerationParams(BaseModel):
    """Parameters for image generation"""
    prompt: str
    steps: int = 4
    width: int = 1024
    height: int = 1024
    seed: int = -1
    # ...
```

### Async/Await

Use asyncio for async operations:

```python
async def load_jobs(self) -> List[GenerationJob]:
    """Load all jobs from database"""
    async with self.lock:
        # async operations
        return jobs
```

### Logging

Use loguru for logging:

```python
from loguru import logger

logger.info("Job queued: {}", job_id)
logger.error("Failed to load model: {}", error)
logger.debug("Event pushed: {}", event_name)
```

### Error Handling

Always handle errors gracefully:

```python
try:
    result = await self.db.insert_image(image_data)
    return result
except Exception as e:
    logger.error(f"Failed to save image: {e}")
    raise
```

---

## Architecture Overview

### System Layers

```
Vue 3 Frontend (src/)
  - Components, Views, Pinia Stores
  - Backend Bridge (abstracts Tauri vs pywebview)
  ↕ pywebview JS API (Python methods exposed to JS)
Python Backend (src-python/)
  - API Bridge (api.py)
  - Queue Manager (async job scheduling)
  - Inference Engine (FluxPipeline, PyTorch/Diffusers)
  - Gallery (aiosqlite database)
  - Event Queue (polling-based)
```

### Image Generation Flow

```
User prompt → invoke('queue_generation') → QueueManager.add_job()
→ QueueProcessor picks up → FluxPipeline.generate()
  1. load_pipeline() - lazy load FLUX model from HuggingFace
  2. apply_loras() - if specified
  3. generate() - diffusion steps with progress callbacks
  4. save_image() - PNG + gallery DB
→ push_event('job-completed') → Frontend polls via poll_events()
→ UI updates (QueueView, GalleryView)
```

---

## Critical Areas

### State Management (Frontend)

**Pinia Stores** (`src/stores/`) - Use backend-bridge:

```typescript
// src/stores/queue.ts
import { invoke } from '@/utils/backend-bridge';

export const useQueueStore = defineStore('queue', {
  state: () => ({
    jobs: [] as GenerationJob[],
  }),

  actions: {
    async loadJobs() {
      this.jobs = await invoke('get_all_jobs');
    },

    async queueGeneration(params: GenerationParams) {
      const result = await invoke('queue_generation', params);
      if (result.status === 'success') {
        await this.loadJobs();
      }
    },
  },
});
```

### GPU Handling (Python)

**Device Selection** (`src-python/inference/device.py`):

```python
def select_device(device_str: Optional[str] = None) -> torch.device:
    """Select best device: CUDA > MPS > CPU"""
    if torch.cuda.is_available():
        return torch.device("cuda")
    if torch.backends.mps.is_available():
        return torch.device("mps")
    return torch.device("cpu")
```

**Optimization:**

```python
# Enable memory optimizations
if device.type == "cuda":
    pipeline.enable_attention_slicing()
    pipeline.enable_vae_slicing()
```

### Model Pipeline (Python)

**Lazy Loading:**

```python
def load_pipeline(self, force_reload: bool = False) -> None:
    """Lazy load the FLUX pipeline"""
    if self.is_loaded and not force_reload:
        return

    self.pipeline = DiffusersFluxPipeline.from_pretrained(
        self.model_path,  # e.g., "black-forest-labs/FLUX.1-schnell"
        torch_dtype=self.dtype,
    ).to(self.device)

    self.is_loaded = True
```

**LoRA Support:**

```python
def apply_loras(self, loras: List[LoraConfig]) -> None:
    """Apply LoRA adapters"""
    for lora in loras:
        self.pipeline.load_lora_weights(
            lora.path,
            adapter_name=lora.name,
        )
```

### Event System (Python)

**Polling-based Events:**

```python
# src-python/events.py
async def push_event(event_name: str, payload: Dict[str, Any]) -> None:
    """Push event to queue for frontend polling"""
    await _event_queue.push_event(event_name, payload)

# src-python/api.py
def poll_events(self, max_events: int = 50) -> List[Dict[str, Any]]:
    """Poll for events (called by frontend every 100ms)"""
    return self._run_async(events.pop_events(max_events))
```

### API Bridge (Python → JavaScript)

**Exposing Methods:**

```python
# src-python/api.py
class Api:
    """All methods are auto-exposed to JavaScript"""

    def queue_generation(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """Queue a generation job"""
        try:
            gen_params = GenerationParams(**params)
            job_id = self._run_async(self.app_state.queue_manager.add_job(gen_params))
            return {"status": "success", "job_id": job_id}
        except Exception as e:
            return {"status": "error", "message": str(e)}
```

**Frontend calls:**

```javascript
const result = await window.pywebview.api.queue_generation(params);
```

---

## Development Workflows

### Running the Application

**Production mode:**
```bash
npm run build          # Build frontend
npm run python         # Run Python backend
```

**Development mode:**
```bash
npm run python:dev     # Build + run with debug logging
```

**Manual:**
```bash
npm run build
python src-python/main.py --debug
```

### Adding a New API Command

1. **Add method to `api.py`:**

```python
def my_new_command(self, arg1: str, arg2: int) -> Dict[str, Any]:
    """Description"""
    try:
        # Implementation
        return {"status": "success", "result": "..."}
    except Exception as e:
        return {"status": "error", "message": str(e)}
```

2. **Call from frontend:**

```typescript
const result = await invoke('my_new_command', { arg1: 'test', arg2: 42 });
```

3. **No registration needed** - all `Api` methods are auto-exposed!

### Adding Event Support

1. **Push event from backend:**

```python
import events
await events.push_event('my-event', {'key': 'value'})
```

2. **Listen in frontend:**

```typescript
const unlisten = await listen('my-event', (payload) => {
  console.log('Received:', payload.key);
});
```

### Debugging

**Python backend:**
```bash
python src-python/main.py --debug
```

**Check GPU:**
```python
import torch
print(torch.cuda.is_available())  # CUDA
print(torch.backends.mps.is_available())  # Apple Silicon
```

**Check model loading:**
```
# First run will download models (~20GB)
# Check ~/.cache/huggingface/hub/
```

---

## Common Pitfalls

### Python

- ❌ Forgetting `async`/`await` → runtime errors
- ❌ Not using type hints → harder to debug
- ❌ Blocking operations in async code → use `loop.run_in_executor()`
- ❌ Not handling exceptions → crashes the backend

### Frontend

- ❌ Importing from `@tauri-apps` directly → use `backend-bridge`
- ❌ Assuming instant events → polling has ~100ms latency
- ❌ Not cleaning up listeners → memory leaks

### Events

- ❌ Pushing too many events → queue overflow (max 1000)
- ❌ Not polling from frontend → events pile up
- ✅ Keep event payloads small

---

## File Organization

### Python Structure (Flat)

```
src-python/
├── *.py files (top-level modules)
└── [module]/
    └── *.py (submodules)
```

No deep nesting. Keep it simple.

### Frontend Structure (Unchanged)

```
src/
├── views/
├── components/
├── stores/
├── composables/
├── utils/
│   └── backend-bridge.ts  # NEW
└── types/
```

---

## Summary: Key Principles (Python Backend)

1. **Backend Bridge**: Always use `backend-bridge.ts` for backend calls
2. **Type Safety**: Use Pydantic models in Python, TypeScript interfaces in frontend
3. **Async Everything**: Python uses asyncio, handle with `async`/`await`
4. **Lazy Loading**: Load models only when needed, cache in memory
5. **Event Polling**: Frontend polls every 100ms, keep payloads small
6. **Error Handling**: Return `{"status": "error", "message": "..."}` dicts
7. **Logging**: Use loguru with structured logging
8. **GPU Auto-Detection**: PyTorch handles device selection automatically

---

## Migration Notes

See `MIGRATION_GUIDE.md` for details on porting from Rust to Python.

**Key files to reference:**
- `src-python/README.md` - Python backend docs
- `MIGRATION_GUIDE.md` - Migration guide
- `src/utils/backend-bridge.ts` - Frontend compatibility layer

---

## Questions or Issues?

When working on this codebase:
- Unclear patterns → Check this document first
- Python-specific questions → Check `src-python/README.md`
- Migration questions → Check `MIGRATION_GUIDE.md`
- Missing information → Ask the user for clarification
- New patterns → Document them here for future reference

This is a living document. Update it as the codebase evolves.
