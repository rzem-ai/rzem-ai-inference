# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

Desktop AI image generation app built with **Electron** + **Vue 3** + **Vite**. The Electron main process (Node.js) handles database, IPC, and services. A Python sidecar (`rzem-ai-inference-engine` from sibling repo) runs inference via FastAPI with REST + WebSocket.

## Commands

```bash
# Development
npm run dev                    # Vite HMR + Electron (concurrent)
npm run type-check             # vue-tsc type check for frontend

# Build
npm run build                  # Build main process + frontend
npm start                      # Run built Electron app

# Type checking
npx tsc --noEmit -p tsconfig.main.json   # Main process
npm run type-check                        # Vue frontend
```

No test framework is configured. No linter is configured.

## Architecture

```
Electron App
├── src/main/                   — Electron main process (TypeScript → Node.js)
│   ├── index.ts                — Entry: database, sidecar, IPC, window
│   ├── preload.ts              — Preload script: contextBridge + electronAPI
│   ├── ipc.ts                  — IPC handlers (~90 methods via ipcMain.handle)
│   ├── database.ts             — better-sqlite3 schema + migrations + seeding
│   ├── sidecar.ts              — Python engine subprocess (child_process.spawn)
│   └── services/
│       ├── batch.ts            — CSV parsing + template rendering
│       ├── bundles.ts          — Default bundle data (15 bundles, 6 types)
│       ├── chat.ts             — Anthropic SDK streaming + tool use
│       ├── files.ts            — Native file dialogs (Electron dialog)
│       ├── settings.ts         — Engine status, VRAM, cache, paths
│       ├── styles.ts           — Style CRUD, LoRA, tags, AI features
│       └── workflow.ts         — Workflow DAG executor
│
├── src/mainview/               — Vue 3 renderer (Vite-built)
│   └── src/
│       ├── bridge.ts           — Electron IPC adapter (Proxy-based snake↔camel)
│       ├── composables/        — usePywebview (API abstraction)
│       ├── stores/             — Pinia stores (inference, gallery, styles, etc.)
│       ├── pages/              — Route pages (create, gallery, edit, styles, settings)
│       └── types/pywebview.d.ts — API type definitions (snake_case)
│
└── Python Sidecar (subprocess)
    └── rzem-ai-inference-engine (sibling repo)
        ├── FastAPI server (REST + WebSocket)
        ├── InferenceEngine — GPU jobs
        └── HF cache management
```

### IPC Bridge Pattern

The frontend uses a Proxy-based bridge (`bridge.ts`) that transparently converts `api.get_bundles()` → `window.electronAPI.invoke("getBundles", args)`. Response keys are converted camelCase → snake_case to match frontend expectations. This allows existing stores to work with zero changes.

### Event System

Sidecar emits events via WebSocket → main process buffers them → forwarded to renderer via `webContents.send("inferenceEvent")` and polling buffer (`pollEvents`). Image persistence happens in the main process when `job_completed` events arrive.

### API Response Convention

Every RPC handler returns `{"status": "success", ...}` or `{"status": "error", "message": "..."}`.

## Key Constraints

- **Hash router required**: Electron loads via `file://` in production — `vue-router` must use hash mode
- **Vite port 1978**: Hardcoded in `vite.config.ts` and `src/main/index.ts`
- **Main process owns the database**: The Python sidecar is stateless — only the Electron main process writes to SQLite (via better-sqlite3)
- **Database migrations**: Use proper migration strategies for schema changes
- **Preload script**: All renderer ↔ main communication goes through `contextBridge` in `preload.ts`

## Frontend Stack

- Vue 3 + TypeScript + Pinia (composition API, `<script setup>`)
- PrimeVue 4 with custom Glass theme preset (`src/mainview/src/theme/`)
- Tailwind CSS 4 via `@tailwindcss/vite` plugin
- Tiptap rich text editor (for prompt input)
- Lucide icons (`lucide-vue-next`)
- Path alias: `@` → `src/mainview/src/`

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

### Tailwind CSS Rules

- **No arbitrary pixel values**: Never use bracket syntax like `text-[13px]` or `w-[200px]`. Use the closest standard Tailwind class instead.
  - `text-[10px]` → `text-xs`
  - `text-[11px]` → `text-sm`
  - `text-[13px]` → `text-base`
  - For spacing/sizing brackets, convert to Tailwind units: `min-h-[60px]` → `min-h-15`, `max-h-[80px]` → `max-h-20`
- **No point-sized values**: Never use fractional spacing like `gap-2.5` or `p-3.5`. Use whole numbers only.
  - `gap-2.5` → `gap-2`, `p-3.5` → `p-3`, `py-1.5` → `py-1`
  - For `0.5` values, round up to `1` (e.g., `mt-0.5` → `mt-1`)
- **Tailwind v4 important modifier**: The `!` goes at the end (suffix), not the beginning (prefix).
  - `!opacity-100` → `opacity-100!`
  - `!my-1` → `my-1!`
  - `!bg-white` → `bg-white!`
- **Tailwind v4 bare values**: Use bare values instead of arbitrary bracket syntax for utilities that accept them natively.
  - `aspect-[4/3]` → `aspect-4/3`
  - `aspect-[16/9]` → `aspect-16/9`
  - `columns-[2]` → `columns-2`

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