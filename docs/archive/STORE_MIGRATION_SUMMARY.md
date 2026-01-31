# Pinia Store Migration Summary

## Overview
All Pinia stores have been migrated from **Setup Stores (Composition API)** to **Options Stores (Options API)** while maintaining the same public API.

## Changes Made

### 1. Store Conversions (11 stores)

All stores now use the Options API pattern:
```typescript
export const useXxxStore = defineStore('xxx', {
  state: () => ({ ... }),
  getters: { ... },
  actions: { ... },
})
```

**Converted Stores:**
- ✅ `settings.ts` - Simple state with setters
- ✅ `compare.ts` - State + computed getters
- ✅ `windows.ts` - Layout dimensions
- ✅ `generation.ts` - Generation parameters with localStorage persistence
- ✅ `presets.ts` - Generation presets
- ✅ `tags.ts` - Tag management with backend operations
- ✅ `folders.ts` - Folder tree with recursive operations
- ✅ `models.ts` - Model and LoRA management
- ✅ `queue.ts` - Job queue with event listeners
- ✅ `autoTag.ts` - Auto-tagging with event listeners
- ✅ `gallery.ts` - Image gallery with filters

### 2. Event Listener Management

Stores with event listeners now have explicit initialization/cleanup:

#### **queueStore** (`src/stores/queue.ts`)
- `initializeEventListeners()` - Sets up job update and progress listeners
- `cleanupEventListeners()` - Removes listeners and cleans up

**Usage in components:**
```vue
<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { useQueueStore } from '@/stores/queue'

const queueStore = useQueueStore()

onMounted(() => queueStore.initializeEventListeners())
onUnmounted(() => queueStore.cleanupEventListeners())
</script>
```

**Updated component:** `src/components/generation/QueuePanel.vue`

#### **autoTagStore** (`src/stores/autoTag.ts`)
- `initializeEventListeners()` - Sets up auto-tag event listeners
- `cleanupEventListeners()` - Removes listeners and cleans up

**Updated component:** `src/views/GalleryView.vue`

### 3. localStorage Persistence

The `generationStore` now uses automatic persistence via Pinia's `$subscribe`:

#### **generationStore** (`src/stores/generation.ts`)
- `initializePersistence()` - Sets up automatic localStorage sync
- `cleanupPersistence()` - Removes subscription

**Updated component:** `src/views/GenerateView.vue`

This means components can directly mutate `currentParams` and changes automatically persist:
```typescript
generationStore.currentParams.width = 1024 // Auto-saves to localStorage
```

### 4. Documentation Updates

**CLAUDE.md** updated with:
- ✅ Component order standard: `<template>`, `<script>`, `<style>`
- ✅ Options API pattern for all stores
- ✅ Event listener lifecycle management
- ✅ Updated state management examples
- ✅ Clarified initialization patterns

## Component Requirements

### Components Using queueStore
Must initialize event listeners:
- ✅ `src/components/generation/QueuePanel.vue` - **UPDATED**

### Components Using autoTagStore
Must initialize event listeners:
- ✅ `src/views/GalleryView.vue` - **UPDATED**

### Components Using generationStore
Must initialize persistence:
- ✅ `src/views/GenerateView.vue` - **UPDATED**

## Key Differences: Setup vs Options API

### State Access

**Setup API (OLD):**
```typescript
const jobs = ref<GenerationJob[]>([])
const pendingJobs = computed(() => jobs.value.filter(...))

function loadJobs() {
  jobs.value = await invoke(...)
}

return { jobs, pendingJobs, loadJobs }
```

**Options API (NEW):**
```typescript
state: () => ({
  jobs: [] as GenerationJob[],
}),

getters: {
  pendingJobs(state): GenerationJob[] {
    return state.jobs.filter(...)
  },
},

actions: {
  async loadJobs() {
    this.jobs = await invoke(...)
  },
},
```

### Component Usage (Unchanged)

Both APIs have the same usage in components:
```vue
<script setup lang="ts">
const store = useXxxStore()
const { data } = storeToRefs(store) // Reactive refs
await store.action() // Call actions
</script>
```

## Testing Checklist

Verify these features still work:

### Generation Store
- [ ] Parameters persist to localStorage on change
- [ ] Seed randomization setting persists
- [ ] Settings load from localStorage on app start

### Queue Store
- [ ] Real-time job progress updates
- [ ] Job status changes reflect immediately
- [ ] Gallery refreshes when jobs complete

### Auto-Tag Store
- [ ] Auto-tag event listeners receive backend events
- [ ] Vision model download progress updates
- [ ] Tag results appear in recent results list

### All Stores
- [ ] State mutations trigger UI updates
- [ ] Getters compute correctly
- [ ] Actions complete successfully
- [ ] Cross-store interactions work (e.g., presets accessing generation + models)

## Migration Benefits

1. **Clearer Structure**: Explicit `state`, `getters`, `actions` sections
2. **Consistent Pattern**: All stores follow the same pattern
3. **Better TypeScript**: Getters receive typed `state` parameter
4. **Explicit Lifecycle**: Initialization/cleanup methods are called intentionally
5. **Maintainability**: Easier to understand store boundaries

## Troubleshooting

### Issue: Event listeners not receiving updates
**Solution:** Check that `initializeEventListeners()` is called in `onMounted`

### Issue: localStorage not persisting
**Solution:** Check that `initializePersistence()` is called in GenerateView

### Issue: Store state not reactive
**Solution:** Use `storeToRefs()` for reactive state in components

### Issue: Getters showing stale data
**Solution:** Ensure state is mutated correctly (via `this.` in actions)

## Notes

- All stores maintain backward compatibility with existing component code
- No changes required to most components (except initialization methods)
- The public API (state, getters, actions) remains unchanged
- Event listener and persistence patterns are now explicit and documentable
