# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

Desktop AI image generation app: Python backend (pywebview) + Vue 3 frontend (Vite). The Python process opens a native window that renders the Vue app. Communication happens via pywebview's `js_api` bridge — the frontend calls Python methods directly through `window.pywebview.api`.

Depends on a sibling repo `../rzem-ai-inference-engine` (editable install) which provides `InferenceEngine`, `JobParams`, event types, etc.

## Commands

```bash
# Setup (one-time)
bash scripts/install.sh        # Creates .venv (uv), installs Python + Node deps

# Development (two terminals)
cd frontend && npm run dev     # Terminal 1: Vite HMR on :1978
bash scripts/dev.sh            # Terminal 2: pywebview window → localhost:1978

# Or browser-only dev (mock API, no Python needed)
cd frontend && npm run dev     # Open http://localhost:1978 in browser

# Frontend
cd frontend && npm run build       # Production build → dist/
cd frontend && npm run type-check  # vue-tsc --noEmit

# Run / Build
bash scripts/run.sh            # Build frontend (if needed) + run app
bash scripts/build.sh          # PyInstaller → build/dist/Inference/

# Run any Python command in the project venv
uv run python main.py
```

No test framework is configured. No linter is configured.

## Architecture

```
main.py
  ├── AppConfig (backend/config.py) — dev/prod paths, window dimensions
  ├── CombinedAPI (backend/api/combined.py) — single js_api object for pywebview
  │     ├── SystemAPI (backend/api/system.py)
  │     └── InferenceAPI (backend/api/inference.py)
  └── Services
        ├── AppService (backend/services/app_service.py)
        └── InferenceService (backend/services/inference_service.py)

frontend/src/
  ├── bridge.ts — pywebview readiness detection + mock API fallback
  ├── composables/usePywebview.ts — reactive API composable
  ├── stores/inference.ts — Pinia store: engine state, job lifecycle, event polling
  ├── types/pywebview.d.ts — TypeScript interface for all API methods
  └── pages/create/ — Main generation UI (Main.vue = results, Menu.vue = sidebar)
```

### PyWebView Argument Bridge (Critical Pattern)

pywebview passes JS objects as a single positional dict, not kwargs. The `ApiMeta` metaclass in `backend/api/__init__.py` auto-wraps every public API method to:
1. Detect single-dict positional arg from pywebview
2. Convert camelCase keys to snake_case
3. Unpack as `**kwargs`

**Rule**: Python API param names must match the frontend's camelCase keys after snake_case conversion. Use `**kwargs` to absorb extra keys. Frontend always sends flat dicts.

### Event Polling (Not WebSockets)

The inference engine fires events from a background thread. `InferenceService` serializes them into a thread-safe deque (max 500). The frontend polls `poll_events()` every 200ms to drain them. PIL images are saved to disk as PNGs; only file paths are sent to the frontend, which then calls `get_image_base64()` to load them.

### API Response Convention

Every Python API method returns `{"status": "success", ...}` or `{"status": "error", "message": "..."}`. Never raise exceptions through the bridge.

## Key Constraints

- **Hash router required**: pywebview's built-in HTTP server has no SPA fallback — `vue-router` must use hash mode
- **Vite port 1978**: Hardcoded in both `vite.config.ts` (strictPort) and `backend/config.py`
- **`os._exit(0)` on close**: Tearing down CUDA from non-main thread causes C++ errors; the app force-exits instead
- **`--system-site-packages` venv**: Required on Linux for pywebview's GTK/WebKit bindings (system `gi` module)
- **Database migrations**: Use proper migration strategies for schema changes

## Frontend Stack

- Vue 3 + TypeScript + Pinia (composition API, `<script setup>`)
- PrimeVue 4 with custom Glass theme preset (`frontend/src/theme/`)
- Tailwind CSS 4 via `@tailwindcss/vite` plugin
- Tiptap rich text editor (for prompt input)
- Lucide icons (`lucide-vue-next`)
- Path alias: `@` → `frontend/src/`

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