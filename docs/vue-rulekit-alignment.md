# Vue Rulekit Alignment Analysis

## Executive Summary

Your current codebase demonstrates **strong alignment** with Vue 3 best practices from the rulekit. The primary differences are **intentional and appropriate** for a Tauri desktop application with PrimeVue UI components.

**Overall Assessment: ✅ Well-aligned with Vue 3 standards**

---

## Alignment Matrix

| Category | Alignment | Notes |
|----------|-----------|-------|
| **Composition API** | ✅ Perfect | All components use `<script setup>`, no Options API |
| **TypeScript** | ✅ Perfect | Proper type definitions, interface usage |
| **Props/Emits** | ✅ Perfect | Type-safe patterns with `defineProps`/`defineEmits` |
| **Naming Conventions** | ✅ Perfect | PascalCase files, camelCase in JS, kebab-case in templates |
| **State Management** | ✅ Perfect | Pinia stores with setup syntax, proper patterns |
| **Styling Approach** | ⚠️ Hybrid | Uses PrimeVue + TailwindCSS (appropriate for this project) |
| **Project Structure** | 🔶 Different | Traditional Vue Router (not file-based), appropriate for Tauri |
| **Data Fetching** | 🔶 Different | Tauri IPC instead of Pinia Colada (correct for desktop app) |

**Legend:**
- ✅ Perfect: Follows rulekit standards exactly
- ⚠️ Hybrid: Different but appropriate for project context
- 🔶 Different: Intentionally diverges due to project requirements

---

## Detailed Comparison

### 1. ✅ Perfect Alignment

#### Composition API + `<script setup>`
**Rulekit:** ALWAYS use Composition API, NEVER Options API
**Your Codebase:** ✅ Perfect compliance

```vue
<!-- ✅ All your components follow this pattern -->
<script setup lang="ts">
import { ref, computed } from 'vue';
import { useQueueStore } from '@/stores/queue';

const queueStore = useQueueStore();
const jobs = ref<GenerationJob[]>([]);
</script>
```

#### TypeScript Patterns
**Rulekit:** Type-safe props, emits, interfaces
**Your Codebase:** ✅ Excellent TypeScript usage

```typescript
// ✅ Your stores follow proper patterns
export const useQueueStore = defineStore('queue', () => {
  const jobs = ref<GenerationJob[]>([]);
  const pendingJobs = computed(() => jobs.value.filter(j => j.status === 'pending'));
  return { jobs, pendingJobs };
});
```

**Strong Points:**
- ✅ Proper Rust ↔ TypeScript type mapping
- ✅ Comprehensive interfaces for IPC types
- ✅ Type-safe Tauri invoke calls
- ✅ Documentation comments on interfaces

#### Props & Emits
**Rulekit:** `defineProps<{ ... }>()` and `defineEmits<{ ... }>()`
**Your Codebase:** ✅ Correct patterns

```vue
<!-- ✅ BatchScriptDialog.vue example -->
<script setup lang="ts">
const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  'update:visible': [value: boolean];
}>();
</script>
```

#### Naming Conventions
**Rulekit:** PascalCase components, camelCase JS, kebab-case templates
**Your Codebase:** ✅ Consistent naming

```
✅ components/generation/GenerateActions.vue
✅ stores/queue.ts → export const useQueueStore
✅ <ImageCard @image-click="handleImageClick" />
```

---

### 2. ⚠️ Hybrid Approach (Appropriate)

#### Styling: PrimeVue + TailwindCSS

**Rulekit Standard:**
- Pure TailwindCSS for ALL UI
- No component library
- Minimal scoped CSS

**Your Codebase Approach:**
- **PrimeVue** for UI components (Button, Dialog, DataTable, etc.)
- **TailwindCSS** for layout, spacing, responsive design
- **Scoped CSS** for component-specific styles

**Why This Is Correct:**
1. **Desktop UI Complexity**: PrimeVue provides battle-tested components for complex desktop UIs (data grids, dialogs, trees)
2. **Theming**: PrimeVue's theming system integrates with your dark mode
3. **Accessibility**: PrimeVue handles ARIA attributes, keyboard nav, focus management
4. **Development Speed**: Don't rebuild DataTable, Dialog, Toast, etc. from scratch

**Example of Good Hybrid Pattern:**
```vue
<template>
  <!-- TailwindCSS for layout -->
  <div class="flex flex-col gap-4 p-4 bg-surface-800 rounded-xl">
    <!-- PrimeVue for complex UI -->
    <DataTable :value="jobs" scrollable scrollHeight="400px">
      <Column field="id" header="Job ID" />
    </DataTable>

    <!-- PrimeVue + Tailwind together -->
    <Button label="Generate" class="mt-4" @click="handleGenerate" />
  </div>
</template>
```

**Recommendation:**
- ✅ **Keep current approach** (PrimeVue + TailwindCSS)
- 🎯 **Reduce scoped CSS** in new components - prefer Tailwind utilities where possible
- ✅ Use scoped CSS only for: gradients, animations, PrimeVue theme overrides

**Recent Example (Batch Components):**
The batch components I created use too much scoped CSS. They should be refactored to use more Tailwind:

```vue
<!-- ❌ Current (too much scoped CSS) -->
<style scoped>
.preview-table {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
</style>

<!-- ✅ Better (use Tailwind) -->
<div class="flex flex-col gap-3">
  <!-- content -->
</div>
```

---

### 3. 🔶 Intentional Differences

#### Project Structure

**Rulekit Structure:**
```
src/
├── pages/          # File-based routing
├── queries/        # Pinia Colada data fetching
├── api/            # HTTP API functions
└── components/
```

**Your Structure:**
```
src/
├── views/          # Traditional Vue Router
├── stores/         # Pinia stores (includes data fetching via Tauri IPC)
└── components/
```

**Why This Is Correct:**
- **Tauri Desktop App**: Not a web SPA, so file-based routing adds complexity without benefits
- **IPC Communication**: Data comes from Rust backend via Tauri commands, not HTTP APIs
- **Traditional Router**: `views/` is clearer for desktop app "screens" than `pages/`

**Recommendation:** ✅ **Keep current structure** - it's appropriate for Tauri

#### Data Fetching

**Rulekit:** Pinia Colada queries for HTTP data fetching
**Your Codebase:** Tauri IPC + Pinia stores

```typescript
// ✅ Your pattern (correct for Tauri)
export const useQueueStore = defineStore('queue', () => {
  const jobs = ref<GenerationJob[]>([]);

  async function loadJobs() {
    jobs.value = await invoke('get_all_jobs');  // IPC, not HTTP
  }

  return { jobs, loadJobs };
});
```

**Recommendation:** ✅ **Keep current pattern** - Pinia Colada is for web APIs, not Tauri IPC

---

## Key Recommendations

### 1. Continue Current Patterns ✅

Your codebase already follows Vue 3 best practices well:
- ✅ Composition API everywhere
- ✅ TypeScript-first approach
- ✅ Proper state management
- ✅ Type-safe IPC communication
- ✅ Good naming conventions

### 2. Opportunities for Improvement 🎯

#### A. Reduce Scoped CSS in New Components

**Current (Batch Components):**
```vue
<style scoped>
.preview-table { display: flex; flex-direction: column; gap: 0.75rem; }
.section-title { margin: 0; font-size: 1.1rem; font-weight: 600; }
.file-picker-row { display: flex; align-items: center; gap: 1rem; }
</style>
```

**Better Approach:**
```vue
<template>
  <div class="flex flex-col gap-3">
    <h3 class="text-lg font-semibold text-gray-200">Section Title</h3>
    <div class="flex items-center gap-4">
      <!-- content -->
    </div>
  </div>
</template>

<style scoped>
/* Only for things Tailwind can't handle */
.custom-gradient {
  background: linear-gradient(to right, var(--primary-color), var(--primary-400));
}
</style>
```

#### B. Use `defineModel()` for Two-Way Bindings

**Current Pattern (works but verbose):**
```vue
<script setup lang="ts">
const props = defineProps<{ value: string }>();
const emit = defineEmits<{ 'update:value': [value: string] }>();

function handleChange(newValue: string) {
  emit('update:value', newValue);
}
</script>
```

**Better with `defineModel()`:**
```vue
<script setup lang="ts">
const value = defineModel<string>();
// Automatically handles v-model prop and update event
</script>

<template>
  <input v-model="value" />
</template>
```

**When to use:**
- ✅ Simple two-way bindings (inputs, toggles)
- ❌ Complex validation or transformation (use manual props/emits)

#### C. Prefer `interface` over `type` for Objects

**Current (mixed usage):**
```typescript
export type JobStatus = 'pending' | 'running';  // ✅ Good for unions
export type GenerationJob = { id: string };     // ⚠️ Use interface instead
```

**Better:**
```typescript
export type JobStatus = 'pending' | 'running';  // ✅ Correct for unions

export interface GenerationJob {  // ✅ Better for objects
  id: string;
  status: JobStatus;
}
```

**Why?**
- Interfaces have better error messages
- Interfaces can be extended: `interface ExtendedJob extends GenerationJob { ... }`
- Types are better for unions, primitives, utility types

### 3. What NOT to Change 🚫

**Don't adopt these rulekit patterns:**
- ❌ File-based routing (unnecessary for Tauri)
- ❌ Pinia Colada (designed for HTTP APIs, not IPC)
- ❌ Remove PrimeVue (it's appropriate for desktop apps)
- ❌ Pure Tailwind UI (component library saves development time)

---

## CLAUDE.md Updates Summary

I've added a comprehensive **"Vue 3 Coding Standards"** section to your CLAUDE.md file covering:

1. **Core Principles**: Composition API, TypeScript-first, naming conventions
2. **Component Standards**: Props, emits, v-model, file naming
3. **Styling Guidelines**: PrimeVue + TailwindCSS hybrid approach
4. **State Management**: Pinia store patterns specific to your codebase
5. **TypeScript Patterns**: Interface definitions, Tauri IPC type safety
6. **Template Best Practices**: Slots, shorthands, directives
7. **Event Handling**: Named functions vs arrow functions
8. **Composables**: Reusable logic with cleanup patterns
9. **Common Anti-Patterns**: What to avoid with examples
10. **Testing Patterns**: Component testing with Vitest
11. **Project-Specific Patterns**: Tauri event flow, state sync

The documentation balances rulekit best practices with your project's specific needs (Tauri desktop app with PrimeVue).

---

## Conclusion

Your codebase demonstrates **mature Vue 3 practices**. The differences from the rulekit are not deficiencies - they're **appropriate architectural choices** for a Tauri desktop application.

**Final Verdict:**
- ✅ **Strong foundation** in Vue 3 patterns
- ✅ **Appropriate technology choices** (PrimeVue + Tauri)
- 🎯 **Minor improvements** possible (more Tailwind, less scoped CSS)
- ✅ **Continue current approach** - don't force web SPA patterns onto desktop app

The CLAUDE.md documentation now provides clear guidance for maintaining consistency going forward.
