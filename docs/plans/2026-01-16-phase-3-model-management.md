# Phase 3: Model & LoRA Management Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build model selection, LoRA management, and preset system to prepare for multi-model support and advanced generation features.

**Architecture:** Create database tables for models and LoRAs, build UI components for browsing and managing models, implement preset save/load system, integrate with existing generation flow.

**Tech Stack:** Vue 3 + PrimeVue for UI, SQLite for persistence, Pinia for state management, existing Rust backend integration.

**Dependencies from Phase 2:**
- Working generation UI and backend
- SQLite database initialized
- Pinia stores structure
- File system management working

---

## Task 1: Create Database Schema for Models and Presets

**Files:**
- Modify: `src-tauri/src/gallery/mod.rs`
- Test: `cargo test`

**Step 1: Add models and presets tables**

Update `init_schema()` in `src-tauri/src/gallery/mod.rs`:

```rust
pub fn init_schema(&self) -> Result<()> {
    // ... existing images, tags, image_tags, images_fts tables ...

    // Create models table
    self.conn.execute(
        "CREATE TABLE IF NOT EXISTS models (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            type TEXT NOT NULL,
            path TEXT,
            size_bytes INTEGER,
            is_downloaded INTEGER DEFAULT 0,
            is_active INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL,
            last_used_at INTEGER,
            metadata TEXT
        )",
        [],
    )?;

    // Create loras table
    self.conn.execute(
        "CREATE TABLE IF NOT EXISTS loras (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            trigger_words TEXT,
            base_model TEXT,
            size_bytes INTEGER,
            strength REAL DEFAULT 1.0,
            is_active INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL,
            metadata TEXT
        )",
        [],
    )?;

    // Create presets table
    self.conn.execute(
        "CREATE TABLE IF NOT EXISTS presets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            mode TEXT NOT NULL,
            prompt TEXT,
            negative_prompt TEXT,
            steps INTEGER NOT NULL,
            cfg_scale REAL NOT NULL,
            width INTEGER NOT NULL,
            height INTEGER NOT NULL,
            seed INTEGER,
            model_id TEXT,
            lora_ids TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )?;

    Ok(())
}
```

**Step 2: Add preset methods**

Add to `GalleryDb` impl:

```rust
pub fn save_preset(&self, preset: &GenerationPreset) -> Result<()> {
    self.conn.execute(
        "INSERT OR REPLACE INTO presets
         (id, name, mode, prompt, negative_prompt, steps, cfg_scale, width, height,
          seed, model_id, lora_ids, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            preset.id,
            preset.name,
            preset.mode,
            preset.prompt,
            preset.negative_prompt,
            preset.steps,
            preset.cfg_scale,
            preset.width,
            preset.height,
            preset.seed,
            preset.model_id,
            preset.lora_ids,
            preset.created_at,
            preset.updated_at,
        ],
    )?;
    Ok(())
}

pub fn load_presets(&self) -> Result<Vec<GenerationPreset>> {
    let mut stmt = self.conn.prepare(
        "SELECT id, name, mode, prompt, negative_prompt, steps, cfg_scale,
                width, height, seed, model_id, lora_ids, created_at, updated_at
         FROM presets ORDER BY updated_at DESC"
    )?;

    let presets = stmt.query_map([], |row| {
        Ok(GenerationPreset {
            id: row.get(0)?,
            name: row.get(1)?,
            mode: row.get(2)?,
            prompt: row.get(3)?,
            negative_prompt: row.get(4)?,
            steps: row.get(5)?,
            cfg_scale: row.get(6)?,
            width: row.get(7)?,
            height: row.get(8)?,
            seed: row.get(9)?,
            model_id: row.get(10)?,
            lora_ids: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(presets)
}

pub fn delete_preset(&self, id: &str) -> Result<()> {
    self.conn.execute("DELETE FROM presets WHERE id = ?1", params![id])?;
    Ok(())
}
```

**Step 3: Run tests**

Run: `cd src-tauri && cargo test`
Expected: Tests pass with new schema

**Step 4: Commit database schema**

```bash
git add src-tauri/src/gallery/mod.rs
git commit -m "feat: add database schema for models, LoRAs, and presets

- Add models table for model metadata
- Add loras table for LoRA management
- Add presets table for saved configurations
- Add preset save/load/delete methods

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Create TypeScript Types for Models and Presets

**Files:**
- Modify: `src/types/index.ts`
- Test: `npx vue-tsc --noEmit`

**Step 1: Add model and LoRA types**

Add to `src/types/index.ts`:

```typescript
export interface Model {
  id: string
  name: string
  type: 'flux-schnell' | 'flux-dev' | 'flux-pro' | 'sdxl' | 'sd15'
  path?: string
  sizeBytes?: number
  isDownloaded: boolean
  isActive: boolean
  createdAt: number
  lastUsedAt?: number
  metadata?: Record<string, any>
}

export interface LoRA {
  id: string
  name: string
  path: string
  triggerWords?: string
  baseModel?: string
  sizeBytes?: number
  strength: number
  isActive: boolean
  createdAt: number
  metadata?: Record<string, any>
}

export interface GenerationPreset {
  id: string
  name: string
  mode: GenerationMode
  prompt?: string
  negativePrompt?: string
  steps: number
  cfgScale: number
  width: number
  height: number
  seed?: number
  modelId?: string
  loraIds?: string // JSON array of LoRA IDs with strengths
  createdAt: number
  updatedAt: number
}
```

**Step 2: Verify TypeScript compilation**

Run: `npx vue-tsc --noEmit`
Expected: No errors

**Step 3: Commit types**

```bash
git add src/types/index.ts
git commit -m "feat: add types for models, LoRAs, and presets

- Add Model interface for model metadata
- Add LoRA interface for LoRA management
- Add GenerationPreset for saved configurations

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Create Models Store

**Files:**
- Create: `src/stores/models.ts`
- Test: Manual store test

**Step 1: Create models store**

Create `src/stores/models.ts`:

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Model, LoRA } from '@/types'

export const useModelsStore = defineStore('models', () => {
  // State
  const models = ref<Model[]>([
    {
      id: 'flux-schnell',
      name: 'Flux Schnell',
      type: 'flux-schnell',
      isDownloaded: true, // Stub model is "downloaded"
      isActive: true,
      createdAt: Date.now(),
    },
  ])

  const loras = ref<LoRA[]>([])

  const selectedModelId = ref<string>('flux-schnell')

  // Getters
  const activeModel = computed(() =>
    models.value.find((m) => m.id === selectedModelId.value)
  )

  const activeLoras = computed(() => loras.value.filter((l) => l.isActive))

  const downloadedModels = computed(() =>
    models.value.filter((m) => m.isDownloaded)
  )

  // Actions
  function addModel(model: Model) {
    models.value.push(model)
  }

  function removeModel(id: string) {
    const index = models.value.findIndex((m) => m.id === id)
    if (index !== -1) {
      models.value.splice(index, 1)
    }
  }

  function selectModel(id: string) {
    selectedModelId.value = id
  }

  function addLora(lora: LoRA) {
    loras.value.push(lora)
  }

  function removeLora(id: string) {
    const index = loras.value.findIndex((l) => l.id === id)
    if (index !== -1) {
      loras.value.splice(index, 1)
    }
  }

  function toggleLora(id: string) {
    const lora = loras.value.find((l) => l.id === id)
    if (lora) {
      lora.isActive = !lora.isActive
    }
  }

  function updateLoraStrength(id: string, strength: number) {
    const lora = loras.value.find((l) => l.id === id)
    if (lora) {
      lora.strength = strength
    }
  }

  return {
    // State
    models,
    loras,
    selectedModelId,
    // Getters
    activeModel,
    activeLoras,
    downloadedModels,
    // Actions
    addModel,
    removeModel,
    selectModel,
    addLora,
    removeLora,
    toggleLora,
    updateLoraStrength,
  }
})
```

**Step 2: Verify TypeScript compilation**

Run: `npx vue-tsc --noEmit`
Expected: No errors

**Step 3: Commit models store**

```bash
git add src/stores/models.ts
git commit -m "feat: add models store for model and LoRA management

- Create models store with Pinia
- Add model selection and management
- Add LoRA toggle and strength control
- Initialize with Flux Schnell stub model

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Create Model Selector Component

**Files:**
- Create: `src/components/generation/ModelSelector.vue`
- Modify: `src/views/GenerateView.vue`
- Test: Manual browser test

**Step 1: Create ModelSelector component**

Create `src/components/generation/ModelSelector.vue`:

```vue
<script setup lang="ts">
import { useModelsStore } from '@/stores/models'
import Dropdown from 'primevue/dropdown'

const modelsStore = useModelsStore()
</script>

<template>
  <div class="model-selector">
    <div class="field">
      <label for="model">Model</label>
      <Dropdown
        id="model"
        v-model="modelsStore.selectedModelId"
        :options="modelsStore.downloadedModels"
        option-label="name"
        option-value="id"
        placeholder="Select a model"
        class="w-full"
      />
    </div>

    <div v-if="modelsStore.activeModel" class="model-info">
      <span class="info-label">Type:</span>
      <span class="info-value">{{ modelsStore.activeModel.type }}</span>
    </div>
  </div>
</template>

<style scoped>
.model-selector {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.field label {
  font-weight: 600;
  font-size: 0.875rem;
  color: #374151;
}

.model-info {
  display: flex;
  gap: 0.5rem;
  font-size: 0.75rem;
  color: #6b7280;
}

.info-label {
  font-weight: 500;
}
</style>
```

**Step 2: Add to GenerateView**

Update `src/views/GenerateView.vue` to add ModelSelector after PromptInput:

```vue
<script setup lang="ts">
import { ref } from 'vue'
import PromptInput from '@/components/generation/PromptInput.vue'
import ModelSelector from '@/components/generation/ModelSelector.vue'
import ParameterControls from '@/components/generation/ParameterControls.vue'
import GenerateButton from '@/components/generation/GenerateButton.vue'
import QueueList from '@/components/generation/QueueList.vue'
import ImageCanvas from '@/components/generation/ImageCanvas.vue'

const canvasRef = ref<InstanceType<typeof ImageCanvas> | null>(null)

defineExpose({
  canvasRef
})
</script>

<template>
  <div class="workspace-content">
    <div class="panel left-panel">
      <h2>Generate</h2>
      <PromptInput />
      <div class="divider"></div>
      <ModelSelector />
      <div class="divider"></div>
      <ParameterControls />
      <div class="divider"></div>
      <GenerateButton :canvas-ref="canvasRef" />
    </div>

    <div class="panel center-panel">
      <h2>Queue</h2>
      <QueueList />
    </div>

    <div class="panel right-panel">
      <h2>Canvas</h2>
      <ImageCanvas ref="canvasRef" />
    </div>
  </div>
</template>
```

**Step 3: Test in browser**

Run: `npm run tauri:dev`
Expected: Model selector appears with "Flux Schnell" option

**Step 4: Commit model selector**

```bash
git add src/components/generation/ModelSelector.vue src/views/GenerateView.vue
git commit -m "feat: add model selector to generation UI

- Create ModelSelector component
- Show available downloaded models
- Display active model info
- Integrate into GenerateView

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Create Presets Store

**Files:**
- Create: `src/stores/presets.ts`
- Test: Manual store test

**Step 1: Create presets store**

Create `src/stores/presets.ts`:

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { GenerationPreset } from '@/types'
import { useGenerationStore } from './generation'
import { useModelsStore } from './models'

export const usePresetsStore = defineStore('presets', () => {
  // State
  const presets = ref<GenerationPreset[]>([
    {
      id: 'default',
      name: 'Default (Flux Schnell)',
      mode: 'txt2img',
      steps: 4,
      cfgScale: 1.0,
      width: 1024,
      height: 1024,
      seed: -1,
      modelId: 'flux-schnell',
      createdAt: Date.now(),
      updatedAt: Date.now(),
    },
    {
      id: 'fast-draft',
      name: 'Fast Draft',
      mode: 'txt2img',
      steps: 2,
      cfgScale: 1.0,
      width: 512,
      height: 512,
      seed: -1,
      modelId: 'flux-schnell',
      createdAt: Date.now(),
      updatedAt: Date.now(),
    },
    {
      id: 'high-quality',
      name: 'High Quality',
      mode: 'txt2img',
      steps: 8,
      cfgScale: 1.5,
      width: 1024,
      height: 1024,
      seed: -1,
      modelId: 'flux-schnell',
      createdAt: Date.now(),
      updatedAt: Date.now(),
    },
  ])

  // Actions
  function savePreset(name: string) {
    const generationStore = useGenerationStore()
    const modelsStore = useModelsStore()

    const preset: GenerationPreset = {
      id: crypto.randomUUID(),
      name,
      mode: generationStore.currentParams.mode,
      prompt: generationStore.currentParams.prompt,
      negativePrompt: generationStore.currentParams.negativePrompt,
      steps: generationStore.currentParams.steps,
      cfgScale: generationStore.currentParams.cfgScale,
      width: generationStore.currentParams.width,
      height: generationStore.currentParams.height,
      seed: generationStore.currentParams.seed,
      modelId: modelsStore.selectedModelId,
      loraIds: JSON.stringify(
        modelsStore.activeLoras.map((l) => ({
          id: l.id,
          strength: l.strength,
        }))
      ),
      createdAt: Date.now(),
      updatedAt: Date.now(),
    }

    presets.value.push(preset)
  }

  function loadPreset(id: string) {
    const preset = presets.value.find((p) => p.id === id)
    if (!preset) return

    const generationStore = useGenerationStore()
    const modelsStore = useModelsStore()

    generationStore.currentParams.mode = preset.mode
    generationStore.currentParams.prompt = preset.prompt || ''
    generationStore.currentParams.negativePrompt = preset.negativePrompt || ''
    generationStore.currentParams.steps = preset.steps
    generationStore.currentParams.cfgScale = preset.cfgScale
    generationStore.currentParams.width = preset.width
    generationStore.currentParams.height = preset.height
    generationStore.currentParams.seed = preset.seed || -1

    if (preset.modelId) {
      modelsStore.selectModel(preset.modelId)
    }
  }

  function deletePreset(id: string) {
    const index = presets.value.findIndex((p) => p.id === id)
    if (index !== -1) {
      presets.value.splice(index, 1)
    }
  }

  return {
    // State
    presets,
    // Actions
    savePreset,
    loadPreset,
    deletePreset,
  }
})
```

**Step 2: Verify TypeScript compilation**

Run: `npx vue-tsc --noEmit`
Expected: No errors

**Step 3: Commit presets store**

```bash
git add src/stores/presets.ts
git commit -m "feat: add presets store for saving generation configurations

- Create presets store with default presets
- Add save/load/delete preset actions
- Link to generation and models stores
- Initialize with common presets

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Create Preset Selector Component

**Files:**
- Create: `src/components/generation/PresetSelector.vue`
- Modify: `src/views/GenerateView.vue`
- Test: Manual browser test

**Step 1: Create PresetSelector component**

Create `src/components/generation/PresetSelector.vue`:

```vue
<script setup lang="ts">
import { ref } from 'vue'
import { usePresetsStore } from '@/stores/presets'
import Dropdown from 'primevue/dropdown'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import Dialog from 'primevue/dialog'

const presetsStore = usePresetsStore()

const selectedPresetId = ref<string | null>(null)
const showSaveDialog = ref(false)
const newPresetName = ref('')

const handleLoadPreset = () => {
  if (selectedPresetId.value) {
    presetsStore.loadPreset(selectedPresetId.value)
  }
}

const handleSavePreset = () => {
  if (newPresetName.value.trim()) {
    presetsStore.savePreset(newPresetName.value.trim())
    newPresetName.value = ''
    showSaveDialog.value = false
  }
}
</script>

<template>
  <div class="preset-selector">
    <div class="field">
      <label for="preset">Preset</label>
      <div class="preset-controls">
        <Dropdown
          id="preset"
          v-model="selectedPresetId"
          :options="presetsStore.presets"
          option-label="name"
          option-value="id"
          placeholder="Select a preset"
          class="flex-1"
          @change="handleLoadPreset"
        />
        <Button
          icon="pi pi-save"
          severity="secondary"
          @click="showSaveDialog = true"
          title="Save current settings as preset"
        />
      </div>
    </div>

    <Dialog
      v-model:visible="showSaveDialog"
      modal
      header="Save Preset"
      :style="{ width: '350px' }"
    >
      <div class="save-dialog-content">
        <label for="preset-name">Preset Name</label>
        <InputText
          id="preset-name"
          v-model="newPresetName"
          placeholder="My Preset"
          class="w-full"
          @keyup.enter="handleSavePreset"
        />
      </div>
      <template #footer>
        <Button label="Cancel" severity="secondary" @click="showSaveDialog = false" />
        <Button
          label="Save"
          @click="handleSavePreset"
          :disabled="!newPresetName.trim()"
        />
      </template>
    </Dialog>
  </div>
</template>

<style scoped>
.preset-selector {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.field label {
  font-weight: 600;
  font-size: 0.875rem;
  color: #374151;
}

.preset-controls {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.save-dialog-content {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 1rem 0;
}
</style>
```

**Step 2: Add to GenerateView**

Update `src/views/GenerateView.vue` to add PresetSelector after ModelSelector:

```vue
<script setup lang="ts">
import { ref } from 'vue'
import PromptInput from '@/components/generation/PromptInput.vue'
import ModelSelector from '@/components/generation/ModelSelector.vue'
import PresetSelector from '@/components/generation/PresetSelector.vue'
import ParameterControls from '@/components/generation/ParameterControls.vue'
import GenerateButton from '@/components/generation/GenerateButton.vue'
import QueueList from '@/components/generation/QueueList.vue'
import ImageCanvas from '@/components/generation/ImageCanvas.vue'

const canvasRef = ref<InstanceType<typeof ImageCanvas> | null>(null)

defineExpose({
  canvasRef
})
</script>

<template>
  <div class="workspace-content">
    <div class="panel left-panel">
      <h2>Generate</h2>
      <PromptInput />
      <div class="divider"></div>
      <ModelSelector />
      <PresetSelector />
      <div class="divider"></div>
      <ParameterControls />
      <div class="divider"></div>
      <GenerateButton :canvas-ref="canvasRef" />
    </div>

    <div class="panel center-panel">
      <h2>Queue</h2>
      <QueueList />
    </div>

    <div class="panel right-panel">
      <h2>Canvas</h2>
      <ImageCanvas ref="canvasRef" />
    </div>
  </div>
</template>
```

**Step 3: Test in browser**

Run: `npm run tauri:dev`
Expected:
- Preset selector shows default presets
- Selecting a preset loads parameters
- Save button opens dialog
- Saving creates new preset

**Step 4: Commit preset selector**

```bash
git add src/components/generation/PresetSelector.vue src/views/GenerateView.vue
git commit -m "feat: add preset selector to generation UI

- Create PresetSelector component
- Load preset on selection
- Save current settings as new preset
- Add save dialog with PrimeVue Dialog
- Integrate into GenerateView

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Phase 3 Complete!

**What We Built:**

✅ Database schema for models, LoRAs, and presets
✅ TypeScript types for model management
✅ Models store with model and LoRA management
✅ Presets store with save/load functionality
✅ Model selector component in UI
✅ Preset selector with save dialog
✅ Integration with existing generation flow

**What Works Now:**

1. Select between models (currently just Flux Schnell stub)
2. Use presets to quickly change generation settings
3. Save custom presets for reuse
4. Foundation ready for:
   - Multi-model support
   - LoRA mixing
   - Model downloading
   - LoRA library UI

**Ready for Phase 4:**
- Gallery & Compare workspace
- Smart search and filtering
- Metadata management
- Export functionality

The application now has a solid model management foundation that can scale to multiple models and LoRAs when real model integration is added!
