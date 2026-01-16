# Phase 2: Core Generation Features Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement end-to-end image generation with Flux Schnell model, including prompt interface, parameter controls, queue management, and gallery integration.

**Architecture:** Build Generate workspace UI components (prompt, parameters, canvas) connected to Rust Flux pipeline via Tauri commands. Queue manager handles job lifecycle with progress events emitted via Tauri events. Generated images saved to gallery database and filesystem.

**Tech Stack:** Vue 3 Composition API, PrimeVue components, Candle for ML inference, Flux Schnell model, Tauri events for progress, SQLite for metadata.

**Dependencies from Phase 1:**
- Pinia stores (generation.ts, settings.ts)
- Database schema (images table, FTS)
- InferenceEngine with device detection
- Application initialization
- Workspace navigation

---

## Task 1: Create Generation Parameter Types

**Files:**
- Modify: `src/types/index.ts`
- Test: Manual TypeScript compilation check

**Step 1: Expand GenerationParams type**

Add to `src/types/index.ts` after existing types:

```typescript
export type GenerationMode = 'txt2img' | 'img2img' | 'inpainting'

export interface GenerationParams {
  mode: GenerationMode
  prompt: string
  negativePrompt?: string
  steps: number
  cfgScale: number
  width: number
  height: number
  seed: number
  model: string
  batchSize?: number
  // For img2img/inpainting
  sourceImage?: string
  strength?: number
  maskImage?: string
}

export interface GenerationProgress {
  jobId: string
  step: number
  totalSteps: number
  previewImage?: string
  status: 'queued' | 'preparing' | 'generating' | 'saving' | 'completed' | 'failed'
  error?: string
}

export interface GeneratedImage {
  id: string
  jobId: string
  filePath: string
  thumbnailPath?: string
  params: GenerationParams
  createdAt: number
}
```

**Step 2: Verify TypeScript compilation**

Run: `npx vue-tsc --noEmit`
Expected: No errors related to new types

**Step 3: Commit types**

```bash
git add src/types/index.ts
git commit -m "feat: add generation parameter types for Phase 2

- Add GenerationMode type
- Expand GenerationParams with mode and image inputs
- Add GenerationProgress for tracking
- Add GeneratedImage interface

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Update Generation Store with Mode Support

**Files:**
- Modify: `src/stores/generation.ts`
- Test: Manual store usage check

**Step 1: Update currentParams default**

In `src/stores/generation.ts`, update currentParams initialization:

```typescript
const currentParams = ref<GenerationParams>({
  mode: 'txt2img',
  prompt: '',
  negativePrompt: '',
  steps: 4,  // Flux Schnell default
  cfgScale: 1.0,  // Flux uses CFG=1 typically
  width: 1024,
  height: 1024,
  seed: -1,
  model: 'flux-schnell',
  batchSize: 1
})
```

**Step 2: Add progress tracking state**

Add after currentParams:

```typescript
const activeProgress = ref<Map<string, GenerationProgress>>(new Map())

// Getter for active generation
const isGenerating = computed(() => runningJobs.value.length > 0)

// Getter for progress of specific job
const getProgress = (jobId: string) => activeProgress.value.get(jobId)
```

**Step 3: Add progress update action**

Add to actions section:

```typescript
function updateProgress(jobId: string, progress: GenerationProgress) {
  activeProgress.value.set(jobId, progress)
}

function clearProgress(jobId: string) {
  activeProgress.value.delete(jobId)
}
```

**Step 4: Update return statement**

```typescript
return {
  // State
  jobs,
  currentParams,
  activeProgress,
  // Getters
  queuedJobs,
  runningJobs,
  completedJobs,
  isGenerating,
  getProgress,
  // Actions
  addJob,
  updateJobStatus,
  clearCompleted,
  updateProgress,
  clearProgress
}
```

**Step 5: Verify TypeScript compilation**

Run: `npx vue-tsc --noEmit`
Expected: No errors

**Step 6: Commit store updates**

```bash
git add src/stores/generation.ts
git commit -m "feat: add progress tracking to generation store

- Update default params for Flux Schnell
- Add activeProgress Map for tracking
- Add isGenerating computed property
- Add progress update actions

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Create Prompt Input Component

**Files:**
- Create: `src/components/generation/PromptInput.vue`
- Test: Manual rendering in GenerateView

**Step 1: Create PromptInput component**

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { useGenerationStore } from '@/stores/generation'
import Textarea from 'primevue/textarea'

const store = useGenerationStore()

const prompt = computed({
  get: () => store.currentParams.prompt,
  set: (value: string) => {
    store.currentParams.prompt = value
  }
})

const negativePrompt = computed({
  get: () => store.currentParams.negativePrompt || '',
  set: (value: string) => {
    store.currentParams.negativePrompt = value || undefined
  }
})
</script>

<template>
  <div class="prompt-input">
    <div class="field">
      <label for="prompt">Prompt</label>
      <Textarea
        id="prompt"
        v-model="prompt"
        rows="4"
        placeholder="Describe the image you want to generate..."
        class="w-full"
      />
    </div>

    <div class="field">
      <label for="negative-prompt">Negative Prompt</label>
      <Textarea
        id="negative-prompt"
        v-model="negativePrompt"
        rows="2"
        placeholder="What to avoid in the image..."
        class="w-full"
      />
    </div>
  </div>
</template>

<style scoped>
.prompt-input {
  display: flex;
  flex-direction: column;
  gap: 1rem;
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
</style>
```

**Step 2: Integrate into GenerateView**

Update `src/views/GenerateView.vue` left panel:

```vue
<script setup lang="ts">
import PromptInput from '@/components/generation/PromptInput.vue'
</script>

<template>
  <div class="workspace-content">
    <div class="panel left-panel">
      <h2>Generate</h2>
      <PromptInput />
    </div>
    <div class="panel center-panel">
      <h2>Queue</h2>
      <p>Generation queue and history will go here</p>
    </div>
    <div class="panel right-panel">
      <h2>Canvas</h2>
      <p>Image preview and editing will go here</p>
    </div>
  </div>
</template>
```

**Step 3: Test in browser**

Run: `npm run tauri:dev`
Expected: Generate workspace shows prompt textareas, typing updates store

**Step 4: Commit prompt component**

```bash
git add src/components/generation/PromptInput.vue src/views/GenerateView.vue
git commit -m "feat: add prompt input component

- Create PromptInput with prompt and negative prompt
- Use PrimeVue Textarea components
- Bind directly to generation store
- Integrate into GenerateView left panel

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Create Parameter Controls Component

**Files:**
- Create: `src/components/generation/ParameterControls.vue`
- Modify: `src/views/GenerateView.vue`
- Test: Manual rendering and value changes

**Step 1: Create ParameterControls component**

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { useGenerationStore } from '@/stores/generation'
import InputNumber from 'primevue/inputnumber'
import Slider from 'primevue/slider'

const store = useGenerationStore()

const steps = computed({
  get: () => store.currentParams.steps,
  set: (value: number | null) => {
    store.currentParams.steps = value ?? 4
  }
})

const cfgScale = computed({
  get: () => store.currentParams.cfgScale,
  set: (value: number | null) => {
    store.currentParams.cfgScale = value ?? 1.0
  }
})

const width = computed({
  get: () => store.currentParams.width,
  set: (value: number | null) => {
    store.currentParams.width = value ?? 1024
  }
})

const height = computed({
  get: () => store.currentParams.height,
  set: (value: number | null) => {
    store.currentParams.height = value ?? 1024
  }
})

const seed = computed({
  get: () => store.currentParams.seed,
  set: (value: number | null) => {
    store.currentParams.seed = value ?? -1
  }
})

const commonSizes = [
  { label: 'Square (1024×1024)', width: 1024, height: 1024 },
  { label: 'Landscape (1344×768)', width: 1344, height: 768 },
  { label: 'Portrait (768×1344)', width: 768, height: 1344 },
]

const setSize = (w: number, h: number) => {
  width.value = w
  height.value = h
}

const randomizeSeed = () => {
  seed.value = Math.floor(Math.random() * 2147483647)
}
</script>

<template>
  <div class="parameter-controls">
    <div class="field">
      <label>Steps: {{ steps }}</label>
      <Slider v-model="steps" :min="1" :max="50" />
    </div>

    <div class="field">
      <label>CFG Scale: {{ cfgScale.toFixed(1) }}</label>
      <Slider v-model="cfgScale" :min="0" :max="20" :step="0.1" />
    </div>

    <div class="field">
      <label>Size Presets</label>
      <div class="size-buttons">
        <button
          v-for="size in commonSizes"
          :key="size.label"
          @click="setSize(size.width, size.height)"
          class="size-btn"
        >
          {{ size.label }}
        </button>
      </div>
    </div>

    <div class="field-row">
      <div class="field">
        <label>Width</label>
        <InputNumber v-model="width" :min="256" :max="2048" :step="64" />
      </div>
      <div class="field">
        <label>Height</label>
        <InputNumber v-model="height" :min="256" :max="2048" :step="64" />
      </div>
    </div>

    <div class="field">
      <label>Seed</label>
      <div class="seed-control">
        <InputNumber v-model="seed" :min="-1" :max="2147483647" class="flex-1" />
        <button @click="randomizeSeed" class="randomize-btn">🎲</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.parameter-controls {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
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

.field-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
}

.size-buttons {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.size-btn {
  padding: 0.5rem 1rem;
  background: #f3f4f6;
  border: 1px solid #d1d5db;
  border-radius: 0.375rem;
  cursor: pointer;
  font-size: 0.875rem;
  transition: all 0.2s;
}

.size-btn:hover {
  background: #e5e7eb;
}

.seed-control {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.randomize-btn {
  padding: 0.5rem 0.75rem;
  background: #f3f4f6;
  border: 1px solid #d1d5db;
  border-radius: 0.375rem;
  cursor: pointer;
  font-size: 1.25rem;
  transition: all 0.2s;
}

.randomize-btn:hover {
  background: #e5e7eb;
}
</style>
```

**Step 2: Add to GenerateView**

Update left panel in `src/views/GenerateView.vue`:

```vue
<script setup lang="ts">
import PromptInput from '@/components/generation/PromptInput.vue'
import ParameterControls from '@/components/generation/ParameterControls.vue'
</script>

<template>
  <div class="workspace-content">
    <div class="panel left-panel">
      <h2>Generate</h2>
      <PromptInput />
      <div class="divider"></div>
      <ParameterControls />
    </div>
    <!-- ... rest unchanged ... -->
  </div>
</template>

<style scoped>
/* ... existing styles ... */

.divider {
  height: 1px;
  background: #e5e7eb;
  margin: 1.5rem 0;
}
</style>
```

**Step 3: Test parameter controls**

Run: `npm run tauri:dev`
Expected:
- Sliders for steps and CFG
- Size preset buttons work
- Width/height inputs work
- Randomize seed button works

**Step 4: Commit parameter controls**

```bash
git add src/components/generation/ParameterControls.vue src/views/GenerateView.vue
git commit -m "feat: add parameter controls component

- Create ParameterControls with sliders and inputs
- Add size presets (square, landscape, portrait)
- Add seed randomization
- Use PrimeVue InputNumber and Slider

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Create Generate Button Component

**Files:**
- Create: `src/components/generation/GenerateButton.vue`
- Modify: `src/views/GenerateView.vue`
- Test: Click handling (no backend yet)

**Step 1: Create GenerateButton component**

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { useGenerationStore } from '@/stores/generation'
import Button from 'primevue/button'

const store = useGenerationStore()

const canGenerate = computed(() => {
  return store.currentParams.prompt.trim().length > 0 && !store.isGenerating
})

const queueCount = computed(() => {
  return store.queuedJobs.length + store.runningJobs.length
})

const buttonLabel = computed(() => {
  if (store.isGenerating) {
    return 'Generating...'
  }
  if (queueCount.value > 0) {
    return `Generate (${queueCount.value} in queue)`
  }
  return 'Generate'
})

const handleGenerate = () => {
  if (!canGenerate.value) return

  // Create job
  const job: GenerationJob = {
    id: crypto.randomUUID(),
    prompt: store.currentParams.prompt,
    status: 'Queued'
  }

  store.addJob(job)

  // TODO: Dispatch to backend in next task
  console.log('Job added to queue:', job)
}
</script>

<template>
  <div class="generate-button-container">
    <Button
      :label="buttonLabel"
      @click="handleGenerate"
      :disabled="!canGenerate"
      severity="success"
      size="large"
      class="w-full"
    />

    <div v-if="queueCount > 0" class="queue-info">
      {{ queueCount }} job{{ queueCount !== 1 ? 's' : '' }} in queue
    </div>
  </div>
</template>

<style scoped>
.generate-button-container {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.queue-info {
  text-align: center;
  font-size: 0.875rem;
  color: #6b7280;
}
</style>
```

**Step 2: Add import to GenerateView**

Add after ParameterControls in left panel:

```vue
<script setup lang="ts">
import PromptInput from '@/components/generation/PromptInput.vue'
import ParameterControls from '@/components/generation/ParameterControls.vue'
import GenerateButton from '@/components/generation/GenerateButton.vue'
</script>

<template>
  <div class="workspace-content">
    <div class="panel left-panel">
      <h2>Generate</h2>
      <PromptInput />
      <div class="divider"></div>
      <ParameterControls />
      <div class="divider"></div>
      <GenerateButton />
    </div>
    <!-- ... rest unchanged ... -->
  </div>
</template>
```

**Step 3: Test generate button**

Run: `npm run tauri:dev`
Expected:
- Button disabled when prompt is empty
- Button shows "Generate"
- Click adds job to queue (check console log)
- Button updates to show queue count

**Step 4: Commit generate button**

```bash
git add src/components/generation/GenerateButton.vue src/views/GenerateView.vue
git commit -m "feat: add generate button component

- Create GenerateButton with queue integration
- Disable when prompt empty or generating
- Show queue count in label
- Add job to store on click

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Create Queue Display Component

**Files:**
- Create: `src/components/generation/QueueList.vue`
- Modify: `src/views/GenerateView.vue`
- Test: Visual display of queued jobs

**Step 1: Create QueueList component**

```vue
<script setup lang="ts">
import { useGenerationStore } from '@/stores/generation'
import Card from 'primevue/card'
import ProgressBar from 'primevue/progressbar'

const store = useGenerationStore()

const getProgressPercent = (jobId: string) => {
  const progress = store.getProgress(jobId)
  if (!progress) return 0
  return Math.round((progress.step / progress.totalSteps) * 100)
}

const getStatusText = (jobId: string) => {
  const progress = store.getProgress(jobId)
  if (!progress) return 'Queued'
  return progress.status.charAt(0).toUpperCase() + progress.status.slice(1)
}
</script>

<template>
  <div class="queue-list">
    <div v-if="store.jobs.length === 0" class="empty-state">
      <p>No jobs yet. Click Generate to start!</p>
    </div>

    <div v-else class="jobs-container">
      <Card v-for="job in store.jobs" :key="job.id" class="job-card">
        <template #title>
          <div class="job-header">
            <span class="job-status" :class="job.status.toLowerCase()">
              {{ job.status }}
            </span>
          </div>
        </template>
        <template #content>
          <div class="job-content">
            <p class="job-prompt">{{ job.prompt }}</p>

            <div v-if="job.status === 'Running'" class="progress-section">
              <ProgressBar :value="getProgressPercent(job.id)" />
              <span class="progress-text">{{ getStatusText(job.id) }}</span>
            </div>
          </div>
        </template>
      </Card>
    </div>
  </div>
</template>

<style scoped>
.queue-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow-y: auto;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #9ca3af;
  font-size: 0.875rem;
}

.jobs-container {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.job-card {
  font-size: 0.875rem;
}

.job-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.job-status {
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
}

.job-status.queued {
  background: #dbeafe;
  color: #1e40af;
}

.job-status.running {
  background: #fef3c7;
  color: #92400e;
}

.job-status.completed {
  background: #d1fae5;
  color: #065f46;
}

.job-status.failed {
  background: #fee2e2;
  color: #991b1b;
}

.job-content {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.job-prompt {
  margin: 0;
  color: #374151;
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.progress-section {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.progress-text {
  font-size: 0.75rem;
  color: #6b7280;
}
</style>
```

**Step 2: Add to GenerateView center panel**

```vue
<script setup lang="ts">
import PromptInput from '@/components/generation/PromptInput.vue'
import ParameterControls from '@/components/generation/ParameterControls.vue'
import GenerateButton from '@/components/generation/GenerateButton.vue'
import QueueList from '@/components/generation/QueueList.vue'
</script>

<template>
  <div class="workspace-content">
    <div class="panel left-panel">
      <h2>Generate</h2>
      <PromptInput />
      <div class="divider"></div>
      <ParameterControls />
      <div class="divider"></div>
      <GenerateButton />
    </div>

    <div class="panel center-panel">
      <h2>Queue</h2>
      <QueueList />
    </div>

    <div class="panel right-panel">
      <h2>Canvas</h2>
      <p>Image preview and editing will go here</p>
    </div>
  </div>
</template>
```

**Step 3: Test queue display**

Run: `npm run tauri:dev`
Expected:
- Empty state shows when no jobs
- Jobs appear as cards when added
- Status badges color-coded

**Step 4: Commit queue component**

```bash
git add src/components/generation/QueueList.vue src/views/GenerateView.vue
git commit -m "feat: add queue list component

- Create QueueList displaying jobs as cards
- Show empty state when no jobs
- Display job status with color-coded badges
- Show progress bar for running jobs

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Create Basic Flux Pipeline (Rust)

**Files:**
- Create: `src-tauri/src/inference/pipeline.rs`
- Modify: `src-tauri/src/inference/mod.rs`
- Create: `src-tauri/src/inference/pipeline.rs` tests
- Test: `cargo test`

**Step 1: Create pipeline stub with test**

Create `src-tauri/src/inference/pipeline.rs`:

```rust
//! Flux model inference pipeline

use anyhow::Result;
use candle_core::{Device, Tensor};

pub struct FluxPipeline {
    device: Device,
}

impl FluxPipeline {
    pub fn new(device: Device) -> Result<Self> {
        Ok(Self { device })
    }

    /// Generate image from text prompt (stub for now)
    pub fn generate_stub(&self, prompt: &str, steps: usize) -> Result<Vec<u8>> {
        // For now, return a simple test pattern
        // This will be replaced with actual Flux model inference
        let size = 1024 * 1024 * 3; // 1024x1024 RGB
        let mut data = vec![0u8; size];

        // Create a simple gradient pattern based on prompt length
        let intensity = (prompt.len() % 256) as u8;
        for i in 0..size {
            data[i] = ((i % 256) as u8).wrapping_add(intensity);
        }

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::InferenceEngine;

    #[test]
    fn test_pipeline_creation() {
        let engine = InferenceEngine::new().unwrap();
        let device = engine.get_device().clone();
        let pipeline = FluxPipeline::new(device).unwrap();
        // Just verify it can be created
    }

    #[test]
    fn test_generate_stub() {
        let engine = InferenceEngine::new().unwrap();
        let device = engine.get_device().clone();
        let pipeline = FluxPipeline::new(device).unwrap();

        let result = pipeline.generate_stub("test prompt", 4).unwrap();

        // Should return 1024x1024x3 bytes
        assert_eq!(result.len(), 1024 * 1024 * 3);
        // Should not be all zeros
        assert!(result.iter().any(|&x| x != 0));
    }
}
```

**Step 2: Export from mod.rs**

Update `src-tauri/src/inference/mod.rs`:

```rust
//! Inference engine for running Flux models with Candle

mod engine;
mod pipeline;

pub use engine::InferenceEngine;
pub use pipeline::FluxPipeline;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_module() {
        let _engine = InferenceEngine::new().unwrap();
    }
}
```

**Step 3: Run tests**

Run: `cd src-tauri && cargo test`
Expected: All tests pass (now 9 tests including 2 new pipeline tests)

**Step 4: Commit pipeline stub**

```bash
git add src-tauri/src/inference/
git commit -m "feat: add Flux pipeline stub with tests

- Create FluxPipeline struct
- Add generate_stub for testing without model
- Returns test pattern image data
- Full Flux model integration in next task

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Add Generate Command to Backend

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Test: `cargo test` and manual Tauri command test

**Step 1: Add generate command**

Add to `src-tauri/src/lib.rs` after init_database command:

```rust
#[command]
fn generate_image(
    app_state: State<AppState>,
    prompt: String,
    steps: u32,
    width: u32,
    height: u32,
    seed: i64,
) -> Result<String, String> {
    use crate::inference::{InferenceEngine, FluxPipeline};

    // Get or create inference engine
    let engine = InferenceEngine::new()
        .map_err(|e| format!("Failed to initialize inference engine: {}", e))?;

    let device = engine.get_device().clone();
    let pipeline = FluxPipeline::new(device)
        .map_err(|e| format!("Failed to create pipeline: {}", e))?;

    // Generate stub image (will be real model later)
    let image_data = pipeline.generate_stub(&prompt, steps as usize)
        .map_err(|e| format!("Generation failed: {}", e))?;

    // For now, just return success with data size
    Ok(format!("Generated {} bytes", image_data.len()))
}
```

**Step 2: Register command**

Update invoke_handler:

```rust
.invoke_handler(tauri::generate_handler![
    health_check,
    init_database,
    generate_image,
])
```

**Step 3: Test compilation**

Run: `cd src-tauri && cargo build`
Expected: Builds successfully

**Step 4: Commit generate command**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add generate_image Tauri command

- Accept prompt, steps, dimensions, seed
- Create InferenceEngine and FluxPipeline
- Call generate_stub (placeholder)
- Return success with data size

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 9: Connect Frontend Generation to Backend

**Files:**
- Modify: `src/components/generation/GenerateButton.vue`
- Test: Click generate button and check console

**Step 1: Update handleGenerate to call backend**

Replace handleGenerate function in `src/components/generation/GenerateButton.vue`:

```typescript
const handleGenerate = async () => {
  if (!canGenerate.value) return

  const params = store.currentParams

  // Create job
  const job: GenerationJob = {
    id: crypto.randomUUID(),
    prompt: params.prompt,
    status: 'Queued'
  }

  store.addJob(job)
  store.updateJobStatus(job.id, 'Running')

  try {
    // Call backend
    const result = await invoke<string>('generate_image', {
      prompt: params.prompt,
      steps: params.steps,
      width: params.width,
      height: params.height,
      seed: params.seed === -1 ? Math.floor(Math.random() * 2147483647) : params.seed
    })

    console.log('Generation result:', result)
    store.updateJobStatus(job.id, 'Completed')
  } catch (error) {
    console.error('Generation failed:', error)
    store.updateJobStatus(job.id, 'Failed')
  }
}
```

**Step 2: Add invoke import**

Add to imports section:

```typescript
import { invoke } from '@tauri-apps/api/core'
```

**Step 3: Test generation**

Run: `npm run tauri:dev`
Expected:
- Click generate creates job
- Job status changes to Running
- Console shows "Generated X bytes"
- Job status changes to Completed

**Step 4: Commit frontend-backend connection**

```bash
git add src/components/generation/GenerateButton.vue
git commit -m "feat: connect generate button to backend

- Invoke generate_image command on click
- Update job status to Running/Completed/Failed
- Handle errors and log results
- Pass all parameters from store

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 10: Add Image Saving to Filesystem

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Test: Generate and check ~/.flux-generator/outputs/

**Step 1: Update generate_image to save file**

Replace generate_image command in `src-tauri/src/lib.rs`:

```rust
#[command]
fn generate_image(
    app_state: State<AppState>,
    prompt: String,
    steps: u32,
    width: u32,
    height: u32,
    seed: i64,
) -> Result<String, String> {
    use crate::inference::{InferenceEngine, FluxPipeline};
    use std::fs;
    use std::path::PathBuf;

    // Get or create inference engine
    let engine = InferenceEngine::new()
        .map_err(|e| format!("Failed to initialize inference engine: {}", e))?;

    let device = engine.get_device().clone();
    let pipeline = FluxPipeline::new(device)
        .map_err(|e| format!("Failed to create pipeline: {}", e))?;

    // Generate stub image
    let image_data = pipeline.generate_stub(&prompt, steps as usize)
        .map_err(|e| format!("Generation failed: {}", e))?;

    // Determine output path
    let home = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?;
    let output_dir = home.join(".flux-generator").join("outputs");

    // Create output directory
    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    // Generate filename with timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let filename = format!("flux_{}_{}.raw", timestamp, seed);
    let output_path = output_dir.join(&filename);

    // Save raw image data (will be PNG later)
    fs::write(&output_path, &image_data)
        .map_err(|e| format!("Failed to write image: {}", e))?;

    Ok(output_path.to_string_lossy().to_string())
}
```

**Step 2: Add dirs dependency**

Update `src-tauri/Cargo.toml`, add to dependencies:

```toml
dirs = "5.0"
```

**Step 3: Test image saving**

Run: `cargo build` then `npm run tauri:dev`
Expected:
- Generate creates file in ~/.flux-generator/outputs/
- Returns full file path
- File contains image data (can check with `ls -lh`)

**Step 4: Commit image saving**

```bash
git add src-tauri/Cargo.toml src-tauri/src/lib.rs
git commit -m "feat: save generated images to filesystem

- Add dirs dependency for home directory
- Save to ~/.flux-generator/outputs/
- Generate timestamped filenames
- Return full file path to frontend

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 11: Display Generated Image in Canvas

**Files:**
- Create: `src/components/generation/ImageCanvas.vue`
- Modify: `src/views/GenerateView.vue`
- Modify: `src/components/generation/GenerateButton.vue`
- Test: Generate and see result in canvas

**Step 1: Create ImageCanvas component**

```vue
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useGenerationStore } from '@/stores/generation'
import { convertFileSrc } from '@tauri-apps/api/core'

const store = useGenerationStore()

const latestCompleted = computed(() => {
  return store.completedJobs[0] // Most recent completed
})

// For now, we'll store the image path in the job
// In a future task, this will come from gallery
const imageSrc = ref<string | null>(null)

defineExpose({
  setImage: (path: string) => {
    // Convert filesystem path to asset URL
    imageSrc.value = convertFileSrc(path)
  }
})
</script>

<template>
  <div class="image-canvas">
    <div v-if="!imageSrc" class="canvas-empty">
      <p>Generated images will appear here</p>
    </div>

    <div v-else class="canvas-content">
      <img :src="imageSrc" alt="Generated image" class="generated-image" />
    </div>
  </div>
</template>

<style scoped>
.image-canvas {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f9fafb;
  border-radius: 0.5rem;
  overflow: hidden;
}

.canvas-empty {
  color: #9ca3af;
  font-size: 0.875rem;
}

.canvas-content {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
}

.generated-image {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  border-radius: 0.375rem;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
}
</style>
```

**Step 2: Add to GenerateView and expose ref**

Update `src/views/GenerateView.vue`:

```vue
<script setup lang="ts">
import { ref } from 'vue'
import PromptInput from '@/components/generation/PromptInput.vue'
import ParameterControls from '@/components/generation/ParameterControls.vue'
import GenerateButton from '@/components/generation/GenerateButton.vue'
import QueueList from '@/components/generation/QueueList.vue'
import ImageCanvas from '@/components/generation/ImageCanvas.vue'

const canvasRef = ref<InstanceType<typeof ImageCanvas> | null>(null)

// Expose canvas ref for GenerateButton to access
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

**Step 3: Update GenerateButton to display image**

Update `src/components/generation/GenerateButton.vue`:

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { useGenerationStore } from '@/stores/generation'
import { invoke } from '@tauri-apps/api/core'
import Button from 'primevue/button'
import type ImageCanvas from './ImageCanvas.vue'

// Accept canvas ref from parent
const props = defineProps<{
  canvasRef: InstanceType<typeof ImageCanvas> | null
}>()

const store = useGenerationStore()

// ... existing computed properties ...

const handleGenerate = async () => {
  if (!canGenerate.value) return

  const params = store.currentParams

  // Create job
  const job: GenerationJob = {
    id: crypto.randomUUID(),
    prompt: params.prompt,
    status: 'Queued'
  }

  store.addJob(job)
  store.updateJobStatus(job.id, 'Running')

  try {
    // Call backend - now returns file path
    const filePath = await invoke<string>('generate_image', {
      prompt: params.prompt,
      steps: params.steps,
      width: params.width,
      height: params.height,
      seed: params.seed === -1 ? Math.floor(Math.random() * 2147483647) : params.seed
    })

    console.log('Generated image saved to:', filePath)

    // Display image in canvas
    if (props.canvasRef) {
      props.canvasRef.setImage(filePath)
    }

    store.updateJobStatus(job.id, 'Completed')
  } catch (error) {
    console.error('Generation failed:', error)
    store.updateJobStatus(job.id, 'Failed')
  }
}
</script>
```

**Step 4: Test image display**

Run: `npm run tauri:dev`
Expected:
- Generate button creates image
- Image appears in right canvas panel
- Image is a test pattern (gradient based on prompt)

**Step 5: Commit canvas display**

```bash
git add src/components/generation/ImageCanvas.vue src/views/GenerateView.vue src/components/generation/GenerateButton.vue
git commit -m "feat: display generated image in canvas

- Create ImageCanvas component
- Use convertFileSrc for Tauri asset protocol
- Pass canvas ref from GenerateView to GenerateButton
- Display image after generation completes

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Phase 2 Complete!

**What We Built:**
- ✅ Complete Generate workspace UI (prompt, parameters, queue, canvas)
- ✅ Frontend-backend generation flow
- ✅ Flux pipeline stub (ready for real model)
- ✅ Image saving to filesystem
- ✅ Canvas display of generated images

**What Works:**
- Enter prompt and parameters
- Click generate
- Job queues and runs
- Image data generated and saved
- Result displays in canvas

**Ready for Phase 3:**
- Replace pipeline stub with real Flux model
- Add progress events during generation
- Save to gallery database with metadata
- Add image-to-image and inpainting modes

**Testing the Full Flow:**
1. Run `npm run tauri:dev`
2. Go to Generate workspace
3. Enter a prompt (e.g., "a beautiful sunset")
4. Adjust parameters (steps, size, seed)
5. Click Generate
6. Watch queue update
7. See generated image in canvas
8. Check ~/.flux-generator/outputs/ for saved file

**File Structure Summary:**

Frontend Components:
- `src/components/generation/PromptInput.vue`
- `src/components/generation/ParameterControls.vue`
- `src/components/generation/GenerateButton.vue`
- `src/components/generation/QueueList.vue`
- `src/components/generation/ImageCanvas.vue`

Backend:
- `src-tauri/src/inference/pipeline.rs` (Flux stub)
- `src-tauri/src/lib.rs` (generate_image command)

State:
- `src/stores/generation.ts` (updated with progress tracking)
- `src/types/index.ts` (generation types)
