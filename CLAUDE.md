# CLAUDE.md - AI Assistant Guide

## Purpose
This document helps Claude Code work effectively with the rzem-ai-inference codebase. It documents architectural patterns, conventions, critical areas, and common pitfalls specific to this project.

## Document Structure
1. **Architecture Overview** - How the system fits together
2. **Critical Areas** - Deep dives into state management, GPU handling, and model pipeline
3. **Tauri IPC Patterns** - Frontend-backend communication conventions
4. **Code Organization** - File structure and module conventions
5. **Common Pitfalls** - Anti-patterns to avoid with examples
6. **Development Workflows** - Adding features, debugging, testing

## Tech Stack Summary
- **Frontend**: Vue 3 + TypeScript, Pinia stores, PrimeVue UI, TailwindCSS
- **Backend**: Rust + Tauri 2, Candle ML framework, SQLite, Tokio async runtime
- **ML Pipeline**: CLIP text encoder → FLUX.1-schnell transformer → VAE decoder
- **Features**: LoRA support, job queue, gallery with metadata, server/client modes

---

## 1. Architecture Overview

### System Layers

```
┌─────────────────────────────────────────────────────┐
│  Vue 3 Frontend (src/)                              │
│  - Components (UI widgets)                          │
│  - Views (full pages)                               │
│  - Pinia Stores (state management)                  │
│  - Composables (useWebSocket, etc.)                 │
└─────────────────┬───────────────────────────────────┘
                  │ Tauri IPC Commands + Events
┌─────────────────▼───────────────────────────────────┐
│  Rust Backend (src-tauri/src/)                      │
│  ┌───────────────────────────────────────────────┐  │
│  │ Tauri Commands (entry points)                 │  │
│  └───────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────┐  │
│  │ Queue Manager (job scheduling)                │  │
│  │  - QueueManager: job storage & ordering       │  │
│  │  - QueueProcessor: async execution            │  │
│  └───────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────┐  │
│  │ Inference Engine                              │  │
│  │  - FluxPipeline: FLUX.1 generation            │  │
│  │  - ZIndexPipeline: (alternative pipeline)     │  │
│  │  - EmbeddingCache: prompt caching             │  │
│  │  - LoRA support & integration                 │  │
│  └───────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────┐  │
│  │ Model Management                              │  │
│  │  - ModelManager: singleton instances          │  │
│  │  - Downloader: HuggingFace downloads          │  │
│  │  - LoraManager: LoRA weight management        │  │
│  └───────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────┐  │
│  │ Gallery (SQLite database)                     │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### Operation Modes

The application supports three modes (configured at startup):

1. **Local Mode** (default): Desktop app with local GPU inference
2. **Server Mode**: Exposes REST API + WebSocket on specified port for remote clients
3. **Client Mode**: Connects to remote server, offloads all inference to server GPU

**Key files:**
- `src-tauri/src/main.rs` - CLI arg parsing and mode selection
- `src-tauri/src/shared/protocol.rs` - RuntimeConfig type
- `src-tauri/src/server/` - REST/WebSocket server implementation
- `src-tauri/src/client/` - Remote API client

**Environment variables for dev mode:**
```bash
# Server mode
RZEM_SERVER_MODE=1 RZEM_PORT=8080 npm run tauri:dev

# Client mode
RZEM_CLIENT_MODE=1 RZEM_SERVER_URL=http://localhost:8080 npm run tauri:dev
```

### Data Flow: Image Generation

```
User enters prompt in GenerateView.vue
  ↓
Vue calls: await invoke('queue_generation', { params })
  ↓
Tauri command: src-tauri/src/queue/mod.rs::queue_generation()
  ↓
QueueManager.add_job() → creates GenerationJob with UUID
  ↓
QueueProcessor picks up job asynchronously
  ↓
Calls appropriate pipeline (FluxPipeline or ZIndexPipeline)
  ↓
Pipeline stages (with progress events emitted):
  1. ensure_models_loaded() - lazy load T5, CLIP, VAE, FLUX
  2. encode() - text → embeddings (with cache check)
  3. denoise() - run diffusion steps
  4. decode() - latent → RGB image
  5. save_image() - write PNG + insert to gallery DB
  ↓
Emits 'job-completed' event with result_path
  ↓
Frontend receives event via useJobUpdates() composable
  ↓
Pinia store updates jobs array
  ↓
UI reactively updates (QueueView shows completion, GalleryView shows new image)
```

---

## 2. Critical Areas

### 2.1 State Management

**Problem:** Frontend and backend state must stay synchronized across Tauri IPC boundaries, WebSocket events, and async operations. Stale state causes UI bugs.

#### Pinia Store Architecture

All stores are in `src/stores/`:
- `queue.ts` - Generation jobs, progress tracking
- `gallery.ts` - Images, folders, tags, favorites
- `models.ts` - Model download status, available models
- `generation.ts` - Generation form state, current params
- `settings.ts` - User preferences, GPU settings

#### State Sync Patterns

**Pattern 1: Initial Load + Event Updates**

```typescript
// ✅ DO: Initial fetch + subscribe to updates
export const useQueueStore = defineStore('queue', () => {
  const jobs = ref<GenerationJob[]>([]);

  // Initial load
  async function loadJobs() {
    jobs.value = await invoke('get_all_jobs');
  }

  // Subscribe to real-time updates
  const { onJobProgress, onJobCompleted } = useJobUpdates();

  onJobProgress((update) => {
    const job = jobs.value.find(j => j.id === update.job_id);
    if (job) {
      job.progress = update.progress;
      job.currentStage = update.stage;
    }
  });

  return { jobs, loadJobs };
});
```

**Pattern 2: Optimistic Updates**

```typescript
// ✅ DO: Update local state immediately, sync later
async function favoriteImage(imageId: string) {
  // Optimistic update
  const image = images.value.find(i => i.id === imageId);
  if (image) image.is_favorite = true;

  // Background sync (don't await)
  invoke('gallery_toggle_favorite', { imageId })
    .catch(err => {
      // Rollback on error
      if (image) image.is_favorite = false;
      console.error('Failed to favorite:', err);
    });
}
```

**Pattern 3: WebSocket Event Handlers**

Located in `src/composables/useWebSocket.ts`. This composable manages:
- Connection to backend event stream
- Automatic reconnection on disconnect
- Event routing to appropriate stores

```typescript
// Backend emits events with tauri::Manager::emit()
app.emit("job-progress", ProgressUpdate { job_id, progress, stage });

// Frontend receives via useJobUpdates()
const { onJobProgress } = useJobUpdates();
onJobProgress((update) => {
  // Update store
});
```

#### Common State Management Issues

**❌ DON'T: Call invoke() in computed properties**
```typescript
// BAD - causes infinite loops
const jobCount = computed(() => {
  invoke('get_all_jobs').then(jobs => jobs.length); // ⚠️ NO!
});
```

**✅ DO: Load once, use reactive state**
```typescript
const jobs = ref<GenerationJob[]>([]);
onMounted(() => loadJobs());
const jobCount = computed(() => jobs.value.length);
```

**❌ DON'T: Forget to unsubscribe from events**
```typescript
// BAD - memory leak
onMounted(() => {
  listen('job-progress', handleProgress);
  // Missing: onUnmounted(() => unlisten())
});
```

**✅ DO: Use composables with automatic cleanup**
```typescript
// useJobUpdates() handles cleanup automatically
const { onJobProgress } = useJobUpdates();
```

**❌ DON'T: Mutate store state from components**
```typescript
// BAD - bypasses reactivity
queueStore.jobs[0].progress = 0.5;
```

**✅ DO: Use store actions**
```typescript
queueStore.updateJobProgress(jobId, 0.5);
```

#### Type Safety: Rust ↔ TypeScript

Types must match exactly across the IPC boundary:

**Rust side:** `src-tauri/src/queue/mod.rs`
```rust
#[derive(Serialize, Deserialize)]
pub struct GenerationJob {
    pub id: String,
    pub status: JobStatus,  // enum serialized as lowercase strings
    pub progress: f32,      // 0.0-1.0
    // ...
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,    // → "pending"
    Running,    // → "running"
    Completed,  // → "completed"
}
```

**TypeScript side:** `src/stores/queue.ts`
```typescript
export type JobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface GenerationJob {
  id: string;
  status: JobStatus;
  progress: number;  // f32 → number
  // ...
}
```

**Key rules:**
- Use `#[serde(rename_all = "lowercase")]` for enums
- `f32`/`f64` → `number`
- `Option<T>` → `T | undefined` or `T?`
- `Vec<T>` → `T[]`
- Always add comments documenting the Rust type mapping

---

### 2.2 GPU Handling & Platform-Specific Features

**Problem:** CUDA, Metal, and CPU backends require different Candle features. Incorrect feature flags cause compilation errors or runtime crashes.

#### Feature Flag Architecture

Located in `src-tauri/Cargo.toml`:

```toml
# Base dependencies (CPU fallback)
[dependencies]
candle-core = "0.8.4"
candle-nn = "0.8.4"
candle-transformers = "0.8.4"

# Linux: CUDA 12.8
[target.'cfg(target_os = "linux")'.dependencies]
candle-core = { version = "0.8.4", features = ["cuda"] }
candle-nn = { version = "0.8.4", features = ["cuda"] }
candle-transformers = { version = "0.8.4", features = ["cuda"] }

# macOS: Metal (Apple Silicon)
[target.'cfg(target_os = "macos")'.dependencies]
candle-core = { version = "0.8.4", features = ["metal"] }
candle-nn = { version = "0.8.4", features = ["metal"] }
candle-transformers = { version = "0.8.4", features = ["metal"] }
```

#### Device Selection Pattern

**Located in:** All pipeline modules (`src-tauri/src/inference/*/loader.rs`)

```rust
use candle_core::Device;

// ✅ DO: Try GPU first, fallback to CPU gracefully
fn select_device() -> Result<Device> {
    #[cfg(feature = "cuda")]
    {
        match Device::cuda_if_available(0) {
            Ok(device) => {
                info!("Using CUDA GPU device");
                return Ok(device);
            }
            Err(e) => {
                warn!("CUDA not available: {}, falling back to CPU", e);
            }
        }
    }

    #[cfg(feature = "metal")]
    {
        match Device::new_metal(0) {
            Ok(device) => {
                info!("Using Metal GPU device");
                return Ok(device);
            }
            Err(e) => {
                warn!("Metal not available: {}, falling back to CPU", e);
            }
        }
    }

    info!("Using CPU device");
    Ok(Device::Cpu)
}
```

#### GPU Memory Management

**Pattern:** Always check GPU memory before loading large models

See `src-tauri/src/inference/flux_pipeline/loader.rs`:

```rust
// ✅ DO: Log GPU memory stats before/after loading
fn load_flux_transformer(&mut self, paths: &ModelPaths) -> Result<()> {
    if let Some((used, total, percent)) = get_gpu_memory_stats() {
        info!("GPU memory before FLUX load: {}/{} ({:.1}%)",
              format_bytes(used), format_bytes(total), percent);
    }

    let timer = Timer::start();
    self.flux = Some(FluxTransformer::load(paths, &self.device)?);

    if let Some((used, total, percent)) = get_gpu_memory_stats() {
        info!("GPU memory after FLUX load: {}/{} ({:.1}%)",
              format_bytes(used), format_bytes(total), percent);
    }

    Ok(())
}
```

**nvidia-smi integration:**
```rust
fn get_gpu_memory_stats() -> Option<(u64, u64, f32)> {
    let output = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=memory.used,memory.total", "--format=csv,noheader,nounits"])
        .output()
        .ok()?;
    // Parse and return (used_bytes, total_bytes, percent)
}
```

#### Common GPU Issues

**❌ DON'T: Use unwrap() on device creation**
```rust
// BAD - panics if GPU unavailable
let device = Device::cuda_if_available(0).unwrap();
```

**✅ DO: Handle device creation failures gracefully**
```rust
let device = Device::cuda_if_available(0)
    .unwrap_or_else(|_| {
        warn!("CUDA unavailable, using CPU");
        Device::Cpu
    });
```

**❌ DON'T: Load all models unconditionally**
```rust
// BAD - wastes VRAM
fn generate(&mut self) -> Result<Tensor> {
    self.load_all_models()?;  // Loads even if already loaded!
    // ...
}
```

**✅ DO: Use lazy loading with state tracking**
```rust
// GOOD - check before loading
fn ensure_models_loaded(&mut self) -> Result<()> {
    if self.flux.is_some() && self.vae.is_some() {
        return Ok(());  // Already loaded
    }
    // Load only missing models
}
```

**❌ DON'T: Mix CPU and GPU tensors**
```rust
// BAD - runtime error
let a = Tensor::zeros((10,), DType::F32, &Device::Cpu)?;
let b = Tensor::ones((10,), DType::F32, &Device::cuda(0)?)?;
let c = a.add(&b)?;  // ⚠️ ERROR: device mismatch
```

**✅ DO: Ensure all tensors on same device**
```rust
// All tensors created on pipeline's device
let a = Tensor::zeros((10,), DType::F32, &self.device)?;
let b = Tensor::ones((10,), DType::F32, &self.device)?;
let c = a.add(&b)?;  // ✓ Works
```

---

### 2.3 Model Pipeline Architecture

**Problem:** The ML pipeline involves multiple heavy models (CLIP, T5, FLUX, VAE) that must be loaded once, reused across generations, and properly cached.

#### Pipeline Structure

Located in `src-tauri/src/inference/flux_pipeline/`:
- `mod.rs` - FluxPipeline struct and public API
- `loader.rs` - Model loading logic with lazy initialization
- `generation.rs` - Core generation logic (encode, denoise, decode)
- `cache_integration.rs` - Embedding cache hooks

```rust
pub struct FluxPipeline {
    // Model components (Option = lazy loaded)
    t5: Option<T5TextEncoder>,
    clip: Option<ClipTextEncoder>,
    vae: Option<VaeDecoder>,
    flux: Option<FluxTransformer>,

    // Pipeline config
    device: Device,
    model_type: ModelType,  // Schnell or Dev

    // Caching
    embedding_cache: Option<Arc<EmbeddingCache>>,
}
```

#### Lazy Loading Pattern

**Why:** Models are ~24GB total. Loading all upfront delays first generation by 2-5 minutes.

**How:** Check `Option<Model>`, load only if `None`

```rust
// ✅ DO: Lazy load in ensure_models_loaded()
impl FluxPipeline {
    pub fn generate(&mut self, params: &GenerationParams) -> Result<GeneratedImage> {
        let mut stats = GenerationStats::default();

        // Only loads missing models
        self.ensure_models_loaded(&mut stats)?;

        // Now all models guaranteed to be Some()
        let embeddings = self.encode(&params.prompt, &params.negative_prompt, &mut stats)?;
        let latent = self.denoise(&embeddings, params, &mut stats)?;
        let image = self.decode(&latent, &mut stats)?;

        Ok(GeneratedImage { image, stats })
    }

    fn ensure_models_loaded(&mut self, stats: &mut GenerationStats) -> Result<()> {
        // Quick return if all loaded
        if self.t5.is_some() && self.clip.is_some()
            && self.vae.is_some() && self.flux.is_some() {
            return Ok(());
        }

        // Load each component individually
        if self.t5.is_none() {
            self.load_t5_encoder(stats)?;
        }
        // ... repeat for clip, vae, flux

        Ok(())
    }
}
```

#### Singleton Pattern for Models

**Why:** Multiple jobs should reuse the same loaded models to save memory.

**How:** Use `Arc<Mutex<Pipeline>>` in QueueProcessor

Located in `src-tauri/src/queue/processor.rs`:

```rust
pub struct QueueProcessor {
    // Shared pipeline instance
    pipeline: Arc<Mutex<FluxPipeline>>,
    queue_manager: Arc<QueueManager>,
}

impl QueueProcessor {
    pub async fn process_job(&self, job_id: String) -> Result<()> {
        // Multiple async tasks can access pipeline
        let mut pipeline = self.pipeline.lock().unwrap();

        // Models stay loaded between jobs
        let result = pipeline.generate(&params)?;

        Ok(())
    }
}
```

**❌ DON'T: Create new pipeline per job**
```rust
// BAD - reloads models every time
async fn process_job(job_id: String) -> Result<()> {
    let pipeline = FluxPipeline::new()?;  // ⚠️ NO! Loads 24GB models
    pipeline.generate(&params)?;
    // Pipeline dropped here, wasting the load
}
```

#### Embedding Cache

**Purpose:** Same prompt always produces same embeddings. Cache to skip T5/CLIP encoding (saves ~2-5 seconds per generation).

Located in `src-tauri/src/inference/embedding_cache.rs`:

```rust
pub struct EmbeddingCache {
    cache: RwLock<LruCache<String, CachedEmbedding>>,
}

impl EmbeddingCache {
    pub fn get(&self, prompt: &str, model: &str) -> Option<(Tensor, Tensor)> {
        let key = Self::cache_key(prompt, model);
        let cache = self.cache.read().unwrap();
        cache.get(&key).map(|ce| (ce.t5_embedding.clone(), ce.clip_embedding.clone()))
    }

    pub fn put(&self, prompt: &str, model: &str, t5: Tensor, clip: Tensor) {
        let key = Self::cache_key(prompt, model);
        let mut cache = self.cache.write().unwrap();
        cache.put(key, CachedEmbedding { t5_embedding: t5, clip_embedding: clip });
    }
}
```

**Integration in pipeline** (`src-tauri/src/inference/flux_pipeline/generation.rs`):

```rust
fn encode(&mut self, prompt: &str, negative: Option<&str>, stats: &mut GenerationStats)
    -> Result<(Tensor, Tensor)> {

    // Check cache first
    if let Some(cache) = &self.embedding_cache {
        if let Some((t5, clip)) = cache.get(prompt, self.model_type.as_str()) {
            debug!("Cache hit for prompt: {}", prompt);
            return Ok((t5, clip));
        }
    }

    // Cache miss - encode and store
    let t5 = self.t5.as_ref().unwrap().encode(prompt, stats)?;
    let clip = self.clip.as_ref().unwrap().encode(prompt, stats)?;

    if let Some(cache) = &self.embedding_cache {
        cache.put(prompt, self.model_type.as_str(), t5.clone(), clip.clone());
    }

    Ok((t5, clip))
}
```

#### LoRA Integration

**Purpose:** Low-Rank Adaptation layers modify FLUX transformer behavior without full fine-tuning.

**Key files:**
- `src-tauri/src/models/lora.rs` - LoRA weight structures
- `src-tauri/src/models/lora_manager.rs` - LoRA loading and caching
- `src-tauri/src/models/flux.rs` - LoRA application to FLUX layers

**Pattern:**
```rust
// Load LoRA weights
let lora_weights = LoraManager::load_lora(&lora_path)?;

// Apply to FLUX transformer
flux.apply_loras(&[
    (lora_weights, strength),  // strength: 0.0-1.0
]);

// Generate with LoRA active
let image = pipeline.generate(&params)?;

// Remove LoRAs after generation
flux.remove_loras();
```

**❌ DON'T: Forget to remove LoRAs**
```rust
// BAD - LoRAs persist to next generation
flux.apply_loras(&lora_configs);
pipeline.generate(&params)?;
// Oops, next job will have unwanted LoRAs!
```

**✅ DO: Use RAII or explicit cleanup**
```rust
// GOOD - ensures cleanup
flux.apply_loras(&lora_configs);
let result = pipeline.generate(&params);
flux.remove_loras();  // Always runs
result?
```

---

## 3. Tauri IPC Patterns

### Command Registration

All commands registered in `src-tauri/src/lib.rs`:

```rust
pub fn run_with_config(runtime_config: RuntimeConfig, port: Option<u16>) {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // Queue commands
            queue_generation,
            get_all_jobs,
            cancel_job,

            // Gallery commands
            gallery_get_all_images,
            gallery_toggle_favorite,
            gallery_add_tags,

            // Model commands
            download_model,
            get_model_status,

            // System commands
            get_runtime_config,
            get_system_stats,
        ])
        .setup(|app| {
            // Initialize state
            let queue_manager = Arc::new(QueueManager::new(1));
            let pipeline = Arc::new(Mutex::new(FluxPipeline::new()?));
            app.manage(queue_manager);
            app.manage(pipeline);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Command Implementation Pattern

```rust
#[tauri::command]
async fn queue_generation(
    params: GenerationParams,
    queue_manager: State<'_, Arc<QueueManager>>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    // Validate input
    if params.prompt.is_empty() {
        return Err("Prompt cannot be empty".to_string());
    }

    // Business logic
    let job_id = queue_manager.add_job(params).await;

    // Emit event to frontend
    app.emit("job-queued", JobQueuedEvent { job_id: job_id.clone() })
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    // Return serializable result
    Ok(job_id)
}
```

**Key rules:**
1. Always return `Result<T, String>` (String error message for frontend display)
2. Use `State<'_, Arc<T>>` for shared state access
3. Use `tauri::AppHandle` for emitting events
4. Validate inputs early
5. Keep commands thin - delegate to modules

### Event Emission Pattern

```rust
// Backend emits
app.emit("job-progress", ProgressUpdate {
    job_id: job.id.clone(),
    progress: 0.5,
    stage: "denoising".to_string(),
}).ok();  // .ok() = don't fail job if event send fails

// Frontend receives
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<ProgressUpdate>('job-progress', (event) => {
  console.log('Progress:', event.payload);
});

// Cleanup
onUnmounted(() => unlisten());
```

**Common events:**
- `job-queued` - New job added
- `job-progress` - Progress update (0.0-1.0)
- `job-completed` - Job finished successfully
- `job-failed` - Job failed with error
- `model-download-progress` - Model download status

### Frontend Invocation Pattern

```typescript
import { invoke } from '@tauri-apps/api/core';

// ✅ DO: Proper error handling
async function submitGeneration() {
  try {
    const jobId = await invoke<string>('queue_generation', {
      params: generationParams.value
    });
    console.log('Job queued:', jobId);
  } catch (error) {
    console.error('Failed to queue:', error);
    // Show user-friendly error
  }
}

// ❌ DON'T: Ignore errors
async function submitGeneration() {
  const jobId = await invoke('queue_generation', { params });  // Uncaught promise rejection!
}
```

---

## 4. Code Organization

### Rust Module Structure

```
src-tauri/src/
├── main.rs                   # Entry point, CLI args
├── lib.rs                    # Tauri app setup, command registration
│
├── inference/                # ML inference pipelines
│   ├── mod.rs               # Common types (GeneratedImage, etc.)
│   ├── engine.rs            # High-level inference coordinator
│   ├── stats.rs             # Performance timing and metrics
│   ├── progress.rs          # Progress callback types
│   ├── samplers.rs          # Sampling algorithms (Euler, DPM++)
│   ├── cache.rs             # Cache utilities
│   ├── embedding_cache.rs   # Prompt embedding cache
│   │
│   ├── flux_pipeline/       # FLUX.1 pipeline
│   │   ├── mod.rs           # FluxPipeline struct
│   │   ├── loader.rs        # Model loading
│   │   ├── generation.rs    # Generation logic
│   │   └── cache_integration.rs
│   │
│   └── zindex_pipeline/     # Alternative pipeline
│       └── ...
│
├── models/                   # Model structures
│   ├── mod.rs               # Re-exports
│   ├── model_type.rs        # ModelType enum (Schnell, Dev)
│   ├── paths.rs             # Model file paths
│   ├── manager.rs           # Model lifecycle management
│   ├── downloader.rs        # HuggingFace downloads
│   ├── clip.rs              # CLIP text encoder
│   ├── t5.rs                # T5-XXL text encoder
│   ├── vae.rs               # VAE decoder
│   ├── flux.rs              # FLUX transformer
│   ├── lora.rs              # LoRA weight structures
│   └── lora_manager.rs      # LoRA loading
│
├── queue/                    # Job queue system
│   ├── mod.rs               # QueueManager, GenerationJob
│   └── processor.rs         # QueueProcessor (async executor)
│
├── gallery/                  # Image gallery
│   └── mod.rs               # SQLite DB operations
│
├── server/                   # Server mode (REST + WebSocket)
│   ├── mod.rs               # Server setup
│   ├── router.rs            # Axum routes
│   ├── state.rs             # Shared server state
│   ├── websocket.rs         # WebSocket handler
│   ├── ws_state.rs          # WebSocket connection state
│   └── handlers/            # API endpoints
│       ├── generate.rs
│       ├── queue.rs
│       ├── files.rs
│       └── system.rs
│
├── client/                   # Client mode
│   ├── mod.rs               # Client setup
│   └── api.rs               # HTTP client for remote API
│
├── shared/                   # Shared types
│   ├── mod.rs
│   └── protocol.rs          # RuntimeConfig, OperationMode
│
├── settings/                 # User settings
│   └── mod.rs
│
├── vision/                   # Image tagging
│   ├── mod.rs
│   ├── tagger.rs            # Auto-tagging model
│   ├── models.rs
│   ├── taxonomy.rs
│   └── downloader.rs
│
├── utils/                    # Utilities
│   └── mod.rs
│
├── claude/                   # Claude Code integration
│   └── mod.rs
│
└── logging.rs                # Tracing setup
```

### Vue Component Structure

```
src/
├── main.ts                   # App entry point
├── App.vue                   # Root component
├── router/                   # Vue Router
│   └── index.ts
│
├── views/                    # Full-page components
│   ├── GenerateView.vue     # Generation UI
│   ├── QueueView.vue        # Job queue
│   ├── GalleryView.vue      # Image gallery
│   ├── ModelsView.vue       # Model management
│   └── SettingsView.vue     # Settings
│
├── components/               # Reusable components
│   ├── generation/
│   │   ├── PromptEditor.vue
│   │   ├── HistoryPanel.vue
│   │   └── LoraSelector.vue
│   │
│   ├── gallery/
│   │   ├── ImageGrid.vue
│   │   ├── ImageCard.vue
│   │   ├── FolderTree.vue
│   │   ├── FolderTreeNode.vue
│   │   └── TagEditor.vue
│   │
│   └── queue/
│       └── JobCard.vue
│
├── stores/                   # Pinia stores
│   ├── queue.ts
│   ├── gallery.ts
│   ├── models.ts
│   ├── generation.ts
│   ├── settings.ts
│   ├── tags.ts
│   ├── folders.ts
│   ├── presets.ts
│   ├── compare.ts
│   ├── windows.ts
│   └── autoTag.ts
│
├── composables/              # Composition API utilities
│   ├── useWebSocket.ts      # Event listeners
│   └── ...
│
└── assets/                   # Static assets
```

### Naming Conventions

**Rust:**
- Modules: `snake_case` (e.g., `flux_pipeline/`)
- Structs/Enums: `PascalCase` (e.g., `FluxPipeline`, `JobStatus`)
- Functions: `snake_case` (e.g., `ensure_models_loaded()`)
- Tauri commands: `snake_case` (e.g., `queue_generation`)

**TypeScript/Vue:**
- Components: `PascalCase` (e.g., `ImageCard.vue`)
- Stores: `camelCase` file, `useXxxStore()` export (e.g., `queue.ts` → `useQueueStore()`)
- Composables: `use` prefix (e.g., `useWebSocket()`)
- Types/Interfaces: `PascalCase` (e.g., `GenerationJob`)
- Functions: `camelCase` (e.g., `submitGeneration()`)

---

## 5. Common Pitfalls

### 5.1 Async/Tokio Issues

**❌ DON'T: Block Tokio runtime with sync operations**
```rust
#[tauri::command]
async fn process_job(job_id: String) -> Result<String, String> {
    // BAD - blocks entire async runtime
    std::thread::sleep(Duration::from_secs(10));

    // BAD - sync file I/O blocks
    std::fs::read_to_string("large_file.txt").unwrap();

    Ok(job_id)
}
```

**✅ DO: Use tokio::spawn_blocking for CPU-intensive work**
```rust
#[tauri::command]
async fn process_job(job_id: String) -> Result<String, String> {
    // Offload to thread pool
    let result = tokio::task::spawn_blocking(move || {
        // CPU-intensive work here
        expensive_computation()
    }).await.map_err(|e| e.to_string())?;

    Ok(result)
}
```

**❌ DON'T: Use `.unwrap()` in Tauri commands**
```rust
#[tauri::command]
async fn get_job(job_id: String) -> Result<GenerationJob, String> {
    let jobs = JOBS.read().unwrap();  // ⚠️ Panics on poison!
    let job = jobs.get(&job_id).unwrap();  // ⚠️ Panics if missing!
    Ok(job.clone())
}
```

**✅ DO: Handle errors with `?` operator and proper error messages**
```rust
#[tauri::command]
async fn get_job(job_id: String) -> Result<GenerationJob, String> {
    let jobs = JOBS.read()
        .map_err(|e| format!("Lock poisoned: {}", e))?;

    let job = jobs.get(&job_id)
        .ok_or_else(|| format!("Job not found: {}", job_id))?;

    Ok(job.clone())
}
```

### 5.2 Memory Leaks

**❌ DON'T: Clone large tensors unnecessarily**
```rust
// BAD - duplicates 12GB of VRAM!
let latent = generate_latent()?;
cache.insert(prompt.to_string(), latent.clone());  // Unnecessary clone
decode_latent(latent)?;
```

**✅ DO: Use Arc for shared ownership**
```rust
use std::sync::Arc;

let latent = Arc::new(generate_latent()?);
cache.insert(prompt.to_string(), Arc::clone(&latent));
decode_latent(&latent)?;
```

**❌ DON'T: Keep unused models in memory**
```rust
// BAD - keeps old model loaded
fn switch_model(&mut self, new_model: ModelType) {
    self.model_type = new_model;
    self.load_all_models()?;  // Now two FLUX models in VRAM!
}
```

**✅ DO: Unload before loading new model**
```rust
fn switch_model(&mut self, new_model: ModelType) {
    // Drop old model
    self.flux = None;

    self.model_type = new_model;
    self.ensure_models_loaded()?;
}
```

### 5.3 Error Handling

**❌ DON'T: Use generic error messages**
```rust
// BAD - user has no idea what failed
fn load_model() -> Result<Model, String> {
    let model = Model::load().map_err(|_| "Failed to load".to_string())?;
    Ok(model)
}
```

**✅ DO: Provide actionable error messages**
```rust
fn load_model() -> Result<Model, String> {
    Model::load().map_err(|e|
        format!("Failed to load FLUX model. Ensure model is downloaded and you have ~12GB free VRAM. Error: {}", e)
    )
}
```

**❌ DON'T: Silently ignore errors**
```rust
// BAD - user never knows download failed
app.emit("model-download-progress", progress).ok();  // Swallowed error
```

**✅ DO: Log failures even if not critical**
```rust
if let Err(e) = app.emit("model-download-progress", progress) {
    warn!("Failed to emit progress event: {}", e);
}
```

### 5.4 Race Conditions

**❌ DON'T: Assume event order**
```rust
// BAD - "job-completed" might arrive before "job-progress" 100%
app.emit("job-progress", 100.0).ok();
tokio::time::sleep(Duration::from_millis(10)).await;
app.emit("job-completed", result).ok();
```

**✅ DO: Use proper state transitions**
```rust
// Update state first, then emit
job.status = JobStatus::Completed;
job.progress = 1.0;
job.result_path = Some(path.clone());

// Single atomic event
app.emit("job-completed", JobCompletedEvent {
    job_id: job.id.clone(),
    result_path: path,
}).ok();
```

**❌ DON'T: Mutate shared state without locks**
```rust
// BAD - data race!
async fn update_job(job_id: String) {
    let mut jobs = JOBS.clone();  // ⚠️ Clone doesn't prevent race!
    jobs.get_mut(&job_id).unwrap().progress = 0.5;
}
```

**✅ DO: Use proper synchronization**
```rust
async fn update_job(job_id: String) {
    let mut jobs = JOBS.write().await;  // Exclusive lock
    if let Some(job) = jobs.get_mut(&job_id) {
        job.progress = 0.5;
    }
}
```

---

## 6. Development Workflows

### Adding a New Tauri Command

1. **Define types in appropriate module** (e.g., `src-tauri/src/queue/mod.rs`)
   ```rust
   #[derive(Serialize, Deserialize)]
   pub struct CancelJobRequest {
       pub job_id: String,
   }
   ```

2. **Implement command function**
   ```rust
   #[tauri::command]
   async fn cancel_job(
       job_id: String,
       queue: State<'_, Arc<QueueManager>>,
   ) -> Result<(), String> {
       queue.cancel_job(&job_id).await
           .map_err(|e| format!("Failed to cancel job: {}", e))
   }
   ```

3. **Register in `src-tauri/src/lib.rs`**
   ```rust
   .invoke_handler(tauri::generate_handler![
       // ... existing commands
       cancel_job,  // Add here
   ])
   ```

4. **Add TypeScript types in appropriate store** (e.g., `src/stores/queue.ts`)
   ```typescript
   async function cancelJob(jobId: string): Promise<void> {
     await invoke('cancel_job', { jobId });
   }
   ```

5. **Test in UI component**
   ```vue
   <script setup lang="ts">
   import { useQueueStore } from '@/stores/queue';

   const queueStore = useQueueStore();

   async function handleCancel(jobId: string) {
     await queueStore.cancelJob(jobId);
   }
   </script>
   ```

### Adding a New Pipeline Feature

Example: Adding a new scheduler type

1. **Add enum variant** (`src-tauri/src/inference/samplers.rs`)
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(rename_all = "snake_case")]
   pub enum SchedulerType {
       Normal,
       Karras,
       Exponential,
       NewScheduler,  // Add here
   }
   ```

2. **Implement logic** (`src-tauri/src/inference/samplers.rs`)
   ```rust
   impl SchedulerType {
       pub fn get_sigmas(&self, steps: usize) -> Vec<f64> {
           match self {
               // ... existing
               Self::NewScheduler => self.compute_new_scheduler(steps),
           }
       }
   }
   ```

3. **Update TypeScript types** (`src/stores/queue.ts`)
   ```typescript
   export type SchedulerType = 'normal' | 'simple' | 'karras' | 'exponential' | 'new_scheduler';
   ```

4. **Add UI option** (`src/components/generation/SchedulerSelector.vue`)
   ```vue
   <Select v-model="selectedScheduler">
     <option value="new_scheduler">New Scheduler</option>
   </Select>
   ```

### Debugging GPU Issues

1. **Check device selection:**
   ```bash
   # Run with CUDA logs
   RUST_LOG=debug npm run tauri:dev

   # Look for:
   # "Using CUDA GPU device" (success)
   # "CUDA not available, falling back to CPU" (fallback)
   ```

2. **Monitor GPU memory:**
   ```bash
   # In separate terminal
   watch -n 1 nvidia-smi
   ```

3. **Check feature flags:**
   ```bash
   cd src-tauri
   cargo tree -e features | grep candle
   # Should show: candle-core (features: cuda)
   ```

4. **Test CPU fallback:**
   ```bash
   # Force CPU mode
   CUDA_VISIBLE_DEVICES="" npm run tauri:dev
   ```

### Testing State Sync

1. **Open Vue DevTools** (in Tauri dev mode: right-click → Inspect Element)
2. **Open Pinia tab** → Watch store state changes
3. **Trigger backend event:**
   ```rust
   app.emit("test-event", TestPayload { value: 42 }).ok();
   ```
4. **Check composable receives it:**
   ```typescript
   const { onTestEvent } = useWebSocket();
   onTestEvent((payload) => {
     console.log('Received:', payload);  // Should log immediately
   });
   ```

---

## Summary: Key Principles

1. **State Management**: Initial load + event-based updates. Never poll. Use composables for event cleanup.

2. **GPU Handling**: Platform-specific features in Cargo.toml. Always gracefully fallback to CPU. Log memory usage.

3. **Model Pipeline**: Lazy load models. Use singletons. Cache embeddings. Clean up LoRAs.

4. **Tauri IPC**: Return `Result<T, String>`. Emit events for async updates. Match types exactly Rust ↔ TypeScript.

5. **Async Safety**: Use `spawn_blocking` for CPU work. Never `unwrap()` in commands. Handle all error cases.

6. **Memory**: Use `Arc` for sharing. Drop unused models. Don't clone large tensors.

---

## Questions or Issues?

When working on this codebase, if you encounter:
- Unclear patterns → Check this document first
- Missing information → Ask the user for clarification
- New patterns → Document them here for future reference

This is a living document. Update it as the codebase evolves.
