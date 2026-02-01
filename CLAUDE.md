# CLAUDE.md - AI Assistant Guide

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
This document helps Claude Code work effectively with the rzem-ai-inference codebase. It documents architectural patterns, conventions, and critical areas specific to this project.

## Tech Stack
- **Frontend**: Vue 3 + TypeScript, Pinia stores, PrimeVue UI, TailwindCSS
- **Backend**: Rust + Tauri 2, Candle ML framework, SQLite, Tokio async runtime
- **ML Pipeline**: CLIP text encoder → FLUX.1-schnell transformer → VAE decoder
- **Features**: LoRA support, job queue, gallery with metadata, server/client modes

---

## Vue 3 Coding Standards

### Core Principles
- **Composition API for Components**: Always use `<script setup>` for Vue components
- **Options API for Stores**: Use Options API (`state`, `getters`, `actions`) for Pinia stores
- **Component Order**: Always order component blocks as `<template>`, `<script>`, `<style>`
- **TypeScript First**: All components must use TypeScript with proper type definitions
- **Prefer `interface` over `type`**: For object shapes (easier to extend, better errors)
- **Named Exports**: Prefer named exports over default exports
- **Named Functions**: Use named functions for methods, arrow functions only for callbacks
- **Meaningful Comments**: Only explain WHY, not WHAT (code should be self-documenting)

### Component Patterns

```vue
<template>
  <!-- Use kebab-case in templates -->
  <ImageCard @image-click="handleImageClick" :image-id />
</template>

<script setup lang="ts">
// ✅ Props: TypeScript interface, no `const props =`
defineProps<{
  imageId: string;
  width: number;
  height?: number;
}>();

// ✅ Emits: Type-safe with payload types
const emit = defineEmits<{
  imageClick: [imageId: string];
  update: [id: string, value: number];
}>();

// ✅ v-model: Use defineModel
const prompt = defineModel<string>();
const width = defineModel<number>('width');

// ✅ Named functions for handlers
function handleClick() {
  emit('imageClick', 'img-123');
}
</script>

<style scoped>
/* Component-specific styles */
</style>
```

### Styling
- **Primary**: PrimeVue components (Button, DataTable, Dialog)
- **Layout**: TailwindCSS utility classes
- **Custom**: Scoped CSS only when PrimeVue/Tailwind insufficient

### State Management (Pinia)

**Use Options API for all Pinia stores:**

```typescript
// src/stores/queue.ts
export const useQueueStore = defineStore('queue', {
  state: () => ({
    jobs: [] as GenerationJob[],
  }),

  getters: {
    pendingJobs(state): GenerationJob[] {
      return state.jobs.filter(j => j.status === 'pending')
    },
  },

  actions: {
    async loadJobs() {
      this.jobs = await invoke('get_all_jobs')
    },
  },
})
```

**Usage:**
```vue
<script setup lang="ts">
const queueStore = useQueueStore();
const { jobs } = storeToRefs(queueStore);  // Reactive refs
await queueStore.loadJobs();  // Call actions directly
</script>
```

**Key Points:**
- Use `state()` function returning an object for state
- Use `getters` object for computed properties (receive `state` as first param)
- Use `actions` object for methods (access state via `this`)
- Event listeners should be initialized/cleaned up in actions (call from components)

### TypeScript Patterns

```typescript
// ✅ Map Rust types to TypeScript
/**
 * Generation job from backend
 * Maps to: src-tauri/src/queue/mod.rs::GenerationJob
 */
export interface GenerationJob {
  id: string;
  status: JobStatus;      // Rust enum with serde lowercase
  progress: number;       // f32 → number (0.0-1.0)
  result_path?: string;   // Option<String> → string | undefined
}

export type JobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

// ✅ Type Tauri invoke calls
async function queueGeneration(params: GenerationParams): Promise<string> {
  try {
    return await invoke<string>('queue_generation', { params });
  } catch (error) {
    console.error('Failed:', error);
    throw error;
  }
}
```

### Composables

```typescript
// src/composables/useJobUpdates.ts
export function useJobUpdates() {
  const unlisteners: UnlistenFn[] = [];

  function onJobProgress(callback: (update: ProgressUpdate) => void) {
    listen<ProgressUpdate>('job-progress', (event) => {
      callback(event.payload);
    }).then(unlisten => unlisteners.push(unlisten));
  }

  // Auto cleanup
  onUnmounted(() => unlisteners.forEach(fn => fn()));

  return { onJobProgress };
}
```

### Common Anti-Patterns

**❌ DON'T:**
- Call `invoke()` in computed properties → infinite loops
- Forget to unsubscribe from events → memory leaks
- Mutate props directly → use emits
- Mutate store state from components → use actions
- Use `const props =` unless needed in script

**✅ DO:**
- Load once, use reactive state
- Use composables with automatic cleanup
- Emit update events for two-way binding
- Call store actions for state changes

### Project-Specific Event Flow

```typescript
// Backend emits
app.emit("job-progress", ProgressUpdate { job_id, progress, stage });

// Frontend receives
const { onJobProgress } = useJobUpdates();
onJobProgress((update) => {
  const job = jobs.value.find(j => j.id === update.job_id);
  if (job) job.progress = update.progress;
});
```

**State Sync Pattern:**
1. Initial load from backend (`invoke()` in actions)
2. Initialize event listeners via explicit actions (call from components on mount)
3. Store maintains single source of truth
4. Components use reactive refs from store (`storeToRefs()`)
5. Cleanup listeners when store/component unmounts

---

## Architecture Overview

### System Layers

```
Vue 3 Frontend (src/)
  - Components, Views, Pinia Stores, Composables
  ↕ Tauri IPC Commands + Events
Rust Backend (src-tauri/src/)
  - Tauri Commands (entry points)
  - Queue Manager (job scheduling)
  - Inference Engine (FluxPipeline, ZIndexPipeline, caches)
  - Model Management (ModelManager, Downloader, LoRA)
  - Gallery (SQLite database)
```

### Operation Modes

1. **Local** (default): Desktop app with local GPU inference
2. **Server**: REST API + WebSocket on specified port (`RZEM_SERVER_MODE=1 RZEM_PORT=8080`)
3. **Client**: Connects to remote server (`RZEM_CLIENT_MODE=1 RZEM_SERVER_URL=http://...`)

**Key files:** `src-tauri/src/main.rs`, `src-tauri/src/shared/protocol.rs`, `src-tauri/src/server/`, `src-tauri/src/client/`

### Image Generation Flow

```
User prompt → invoke('queue_generation') → QueueManager.add_job()
→ QueueProcessor picks up → Pipeline.generate()
  1. ensure_models_loaded() - lazy load T5, CLIP, VAE, FLUX
  2. encode() - text → embeddings (cache check)
  3. denoise() - diffusion steps
  4. decode() - latent → RGB
  5. save_image() - PNG + gallery DB
→ emit 'job-completed' → Frontend updates via useJobUpdates()
→ UI updates (QueueView, GalleryView)
```

---

## Critical Areas

### State Management

**Pinia Stores** (`src/stores/`):
- `queue.ts` - Generation jobs, progress
- `gallery.ts` - Images, folders, tags
- `models.ts` - Model downloads
- `generation.ts` - Form state
- `settings.ts` - User preferences

**Pattern: Initial Load + Event Updates**

```typescript
export const useQueueStore = defineStore('queue', {
  state: () => ({
    jobs: [] as GenerationJob[],
    jobUpdates: null as ReturnType<typeof useJobUpdates> | null,
  }),

  actions: {
    async loadJobs() {
      this.jobs = await invoke('get_all_jobs')
    },

    initializeEventListeners() {
      if (this.jobUpdates) return // Already initialized

      this.jobUpdates = useJobUpdates()
      this.jobUpdates.onJobProgress((update) => {
        const job = this.jobs.find(j => j.id === update.job_id)
        if (job) {
          job.progress = update.progress
          job.currentStage = update.stage
        }
      })
    },

    cleanupEventListeners() {
      if (this.jobUpdates) {
        this.jobUpdates.cleanup()
        this.jobUpdates = null
      }
    },
  },
})
```

**Common Issues:**
- ❌ Calling `invoke()` in getters → load data in actions, store in state
- ❌ Forgetting to cleanup event listeners → create explicit cleanup actions
- ❌ Mutating store state from components → always use store actions
- ❌ Initializing listeners in state → initialize in actions, call from components

**Type Safety: Rust ↔ TypeScript**

Rust:
```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus { Pending, Running, Completed }  // → "pending", "running", "completed"
```

TypeScript:
```typescript
export type JobStatus = 'pending' | 'running' | 'completed';
```

**Mapping rules:**
- `#[serde(rename_all = "lowercase")]` for enums
- `f32`/`f64` → `number`
- `Option<T>` → `T | undefined`
- `Vec<T>` → `T[]`

### GPU Handling

**Feature Flags** (`src-tauri/Cargo.toml`):
```toml
# Linux: CUDA 12.8
[target.'cfg(target_os = "linux")'.dependencies]
candle-core = { version = "0.8.4", features = ["cuda"] }

# macOS: Metal
[target.'cfg(target_os = "macos")'.dependencies]
candle-core = { version = "0.8.4", features = ["metal"] }
```

**Device Selection:**
```rust
fn select_device() -> Result<Device> {
    #[cfg(feature = "cuda")]
    if let Ok(device) = Device::cuda_if_available(0) {
        return Ok(device);
    }

    #[cfg(feature = "metal")]
    if let Ok(device) = Device::new_metal(0) {
        return Ok(device);
    }

    Ok(Device::Cpu)  // Always graceful fallback
}
```

**Common Issues:**
- ❌ `Device::cuda_if_available(0).unwrap()` → panics if unavailable
- ❌ Loading all models unconditionally → wastes VRAM
- ❌ Mixing CPU and GPU tensors → runtime error

**Best Practices:**
- ✅ Handle device creation failures gracefully
- ✅ Use lazy loading with state tracking
- ✅ Ensure all tensors on same device
- ✅ Log GPU memory stats before/after loading
- ✅ Check available VRAM before model loads

### Model Pipeline

**Structure** (`src-tauri/src/inference/flux_pipeline/`):
```rust
pub struct FluxPipeline {
    t5: Option<T5TextEncoder>,      // Lazy loaded
    clip: Option<ClipTextEncoder>,
    vae: Option<VaeDecoder>,
    flux: Option<FluxTransformer>,
    device: Device,
    embedding_cache: Option<Arc<EmbeddingCache>>,
}
```

**Lazy Loading:** Models are ~24GB total. Load only when needed.

```rust
fn ensure_models_loaded(&mut self) -> Result<()> {
    if self.t5.is_some() && self.clip.is_some() && self.vae.is_some() && self.flux.is_some() {
        return Ok(());  // Already loaded
    }
    // Load only missing models
}
```

**Singleton Pattern:**
```rust
pub struct QueueProcessor {
    pipeline: Arc<Mutex<FluxPipeline>>,  // Shared across jobs
}
```

**Embedding Cache:**
- Same prompt → same embeddings
- Cache T5/CLIP output (saves ~2-5 seconds per generation)
- Located: `src-tauri/src/inference/embedding_cache.rs`

**LoRA Integration:**
```rust
flux.apply_loras(&[(lora_weights, strength)]);
let image = pipeline.generate(&params)?;
flux.remove_loras();  // CRITICAL: Always cleanup
```

---

## Tauri IPC Patterns

### Command Registration

`src-tauri/src/lib.rs`:
```rust
.invoke_handler(tauri::generate_handler![
    queue_generation,
    get_all_jobs,
    gallery_get_all_images,
    download_model,
])
```

### Command Implementation

```rust
#[tauri::command]
async fn queue_generation(
    params: GenerationParams,
    queue_manager: State<'_, Arc<QueueManager>>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    if params.prompt.is_empty() {
        return Err("Prompt cannot be empty".to_string());
    }

    let job_id = queue_manager.add_job(params).await;

    app.emit("job-queued", JobQueuedEvent { job_id: job_id.clone() })
        .map_err(|e| format!("Failed to emit: {}", e))?;

    Ok(job_id)
}
```

**Rules:**
1. Return `Result<T, String>` (error messages for frontend)
2. Use `State<'_, Arc<T>>` for shared state
3. Use `tauri::AppHandle` for emitting events
4. Validate inputs early
5. Keep commands thin, delegate to modules

### Events

**Common events:**
- `job-queued`, `job-progress`, `job-completed`, `job-failed`
- `model-download-progress`

**Backend:**
```rust
app.emit("job-progress", ProgressUpdate { job_id, progress: 0.5, stage }).ok();
```

**Frontend:**
```typescript
const unlisten = await listen<ProgressUpdate>('job-progress', (event) => {
  console.log('Progress:', event.payload);
});
onUnmounted(() => unlisten());
```

---

## Code Organization

### Rust Structure

```
src-tauri/src/
├── main.rs, lib.rs
├── inference/
│   ├── flux_pipeline/ (mod.rs, loader.rs, generation.rs)
│   ├── zindex_pipeline/
│   ├── engine.rs, stats.rs, samplers.rs
│   └── embedding_cache.rs
├── models/ (clip.rs, t5.rs, vae.rs, flux.rs, lora.rs, manager.rs, downloader.rs)
├── queue/ (mod.rs, processor.rs)
├── gallery/ (mod.rs)
├── server/ (mod.rs, router.rs, websocket.rs, handlers/)
├── client/ (mod.rs, api.rs)
├── shared/ (protocol.rs)
├── settings/, vision/, utils/, logging.rs
```

### Vue Structure

```
src/
├── main.ts, App.vue
├── router/
├── views/ (GenerateView, QueueView, GalleryView, ModelsView, SettingsView)
├── components/
│   ├── generation/ (PromptEditor, HistoryPanel, LoraSelector)
│   ├── gallery/ (ImageGrid, ImageCard, FolderTree, TagEditor)
│   └── queue/ (JobCard)
├── stores/ (queue, gallery, models, generation, settings, tags, folders, presets)
├── composables/ (useWebSocket, ...)
└── assets/
```

### Naming Conventions

**Rust:**
- Modules: `snake_case`
- Structs/Enums: `PascalCase`
- Functions: `snake_case`

**TypeScript/Vue:**
- Components: `PascalCase`
- Stores: `useXxxStore()`
- Composables: `useXxx()`
- Types/Interfaces: `PascalCase`
- Functions: `camelCase`

---

## Common Pitfalls

### Async/Tokio
- ❌ Blocking Tokio runtime with sync operations → use `tokio::spawn_blocking`
- ❌ Using `.unwrap()` in commands → handle with `?` operator
- ✅ Return `Result<T, String>` with actionable error messages

### Memory
- ❌ Cloning large tensors unnecessarily → use `Arc`
- ❌ Keeping unused models in memory → drop before loading new
- ✅ Unload old models before loading new variants

### Errors
- ❌ Generic error messages → provide context and solutions
- ❌ Silently ignoring errors → log failures even if not critical
- ✅ Format errors with file paths, expected actions, troubleshooting hints

### Race Conditions
- ❌ Assuming event order → use proper state transitions
- ❌ Mutating shared state without locks → use `Arc<RwLock<T>>`
- ✅ Update state first, then emit single atomic event

---

## Development Workflows

### Adding a Tauri Command

1. Define types in module (e.g., `src-tauri/src/queue/mod.rs`)
2. Implement command function with `#[tauri::command]`
3. Register in `src-tauri/src/lib.rs` → `tauri::generate_handler![]`
4. Add TypeScript types in store (e.g., `src/stores/queue.ts`)
5. Test in UI component

### Adding Pipeline Feature

Example: New scheduler type

1. Add enum variant (`src-tauri/src/inference/samplers.rs`)
2. Implement logic in match statement
3. Update TypeScript types (`src/stores/queue.ts`)
4. Add UI option in component

### Debugging GPU Issues

```bash
# Check device selection
RUST_LOG=debug npm run tauri:dev
# Look for: "Using CUDA GPU device" or "falling back to CPU"

# Monitor GPU memory
watch -n 1 nvidia-smi

# Check feature flags
cd src-tauri && cargo tree -e features | grep candle

# Test CPU fallback
CUDA_VISIBLE_DEVICES="" npm run tauri:dev
```

---

## Summary: Key Principles

1. **State Management**: Initial load + event-based updates. Never poll. Use composables for cleanup.
2. **GPU Handling**: Platform-specific features in Cargo.toml. Gracefully fallback to CPU. Log memory usage.
3. **Model Pipeline**: Lazy load models. Use singletons. Cache embeddings. Clean up LoRAs.
4. **Tauri IPC**: Return `Result<T, String>`. Emit events for async updates. Match types exactly Rust ↔ TypeScript.
5. **Async Safety**: Use `spawn_blocking` for CPU work. Never `unwrap()` in commands. Handle all error cases.
6. **Memory**: Use `Arc` for sharing. Drop unused models. Don't clone large tensors.

---

## Questions or Issues?

When working on this codebase:
- Unclear patterns → Check this document first
- Missing information → Ask the user for clarification
- New patterns → Document them here for future reference

This is a living document. Update it as the codebase evolves.
