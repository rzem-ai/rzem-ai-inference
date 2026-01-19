# Phase 5: Advanced Features Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add generation queue system, performance monitoring, and batch generation support to enable concurrent image generation with progress tracking.

**Architecture:** Implement a thread-safe queue manager in Rust with Tokio for concurrent generation, add WebSocket or polling for real-time progress updates, create monitoring UI components to display queue status and system metrics, and integrate batch generation controls in the generation workspace.

**Tech Stack:** Rust (tokio async runtime), Vue 3 Composition API, Pinia stores, Tauri commands/events, real-time progress updates via Tauri events.

**Dependencies from Phase 4:**
- Working gallery and compare system
- Database with images table
- Generation command working with stub
- Pinia store architecture established

**Note:** This phase builds queue infrastructure that works with the current stub implementation. The real Flux model integration will come in Phase 7-8, but this queue system is designed to support it.

---

## Task 1: Create Generation Queue Backend

**Files:**
- Modify: `src-tauri/src/queue/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `cargo test`

**Step 1: Update queue data structures**

Update `src-tauri/src/queue/mod.rs`:

```rust
//! Generation queue management with async execution

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationParams {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub steps: u32,
    pub cfg_scale: f64,
    pub width: u32,
    pub height: u32,
    pub seed: i64,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationJob {
    pub id: String,
    pub params: GenerationParams,
    pub status: JobStatus,
    pub progress: f32,
    pub created_at: i64,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub result_path: Option<String>,
    pub error: Option<String>,
}

impl GenerationJob {
    pub fn new(params: GenerationParams) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            params,
            status: JobStatus::Pending,
            progress: 0.0,
            created_at: chrono::Utc::now().timestamp(),
            started_at: None,
            completed_at: None,
            result_path: None,
            error: None,
        }
    }
}

pub struct QueueManager {
    jobs: Arc<RwLock<Vec<GenerationJob>>>,
    running: Arc<Mutex<usize>>,
    max_concurrent: usize,
}

impl QueueManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            jobs: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(Mutex::new(0)),
            max_concurrent,
        }
    }

    pub async fn add_job(&self, params: GenerationParams) -> String {
        let job = GenerationJob::new(params);
        let job_id = job.id.clone();

        let mut jobs = self.jobs.write().await;
        jobs.push(job);

        job_id
    }

    pub async fn get_jobs(&self) -> Vec<GenerationJob> {
        self.jobs.read().await.clone()
    }

    pub async fn get_job(&self, job_id: &str) -> Option<GenerationJob> {
        self.jobs
            .read()
            .await
            .iter()
            .find(|j| j.id == job_id)
            .cloned()
    }

    pub async fn update_job_status(&self, job_id: &str, status: JobStatus) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            job.status = status.clone();
            match status {
                JobStatus::Running => {
                    job.started_at = Some(chrono::Utc::now().timestamp());
                }
                JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled => {
                    job.completed_at = Some(chrono::Utc::now().timestamp());
                }
                _ => {}
            }
        }
    }

    pub async fn update_job_progress(&self, job_id: &str, progress: f32) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            job.progress = progress;
        }
    }

    pub async fn complete_job(&self, job_id: &str, result_path: String) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            job.status = JobStatus::Completed;
            job.progress = 1.0;
            job.result_path = Some(result_path);
            job.completed_at = Some(chrono::Utc::now().timestamp());
        }
    }

    pub async fn fail_job(&self, job_id: &str, error: String) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            job.status = JobStatus::Failed;
            job.error = Some(error);
            job.completed_at = Some(chrono::Utc::now().timestamp());
        }
    }

    pub async fn cancel_job(&self, job_id: &str) -> bool {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            if job.status == JobStatus::Pending {
                job.status = JobStatus::Cancelled;
                job.completed_at = Some(chrono::Utc::now().timestamp());
                return true;
            }
        }
        false
    }

    pub async fn can_start_job(&self) -> bool {
        let running = *self.running.lock().await;
        running < self.max_concurrent
    }

    pub async fn increment_running(&self) {
        let mut running = self.running.lock().await;
        *running += 1;
    }

    pub async fn decrement_running(&self) {
        let mut running = self.running.lock().await;
        if *running > 0 {
            *running -= 1;
        }
    }

    pub async fn clear_completed(&self) {
        let mut jobs = self.jobs.write().await;
        jobs.retain(|j| {
            !matches!(
                j.status,
                JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
            )
        });
    }
}
```

**Step 2: Add chrono dependency**

Add to `src-tauri/Cargo.toml`:

```toml
chrono = "0.4"
```

**Step 3: Run tests**

Run: `cd src-tauri && cargo build`
Expected: Compiles successfully

**Step 4: Commit queue backend**

```bash
git add src-tauri/src/queue/mod.rs src-tauri/Cargo.toml
git commit -m "feat: implement async generation queue manager

- Add JobStatus enum with all states
- Create GenerationParams and GenerationJob structs
- Implement QueueManager with async operations
- Support concurrent job execution
- Add progress tracking and cancellation

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Add Queue Tauri Commands

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Test: Manual testing with frontend

**Step 1: Add queue manager to AppState**

Update `src-tauri/src/lib.rs`:

```rust
use std::sync::Arc;
use queue::QueueManager;

struct AppState {
    gallery_db: Mutex<Option<gallery::GalleryDb>>,
    queue_manager: Arc<QueueManager>,
}
```

**Step 2: Initialize queue manager**

Update the `run()` function:

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let queue_manager = Arc::new(QueueManager::new(1)); // Max 1 concurrent for now

    let app_state = AppState {
        gallery_db: Mutex::new(None),
        queue_manager,
    };

    tauri::Builder::default()
        .manage(app_state)
        // ... rest of builder
}
```

**Step 3: Add queue commands**

Add before the `run()` function:

```rust
#[command]
async fn add_to_queue(
    app_state: State<'_, AppState>,
    params: queue::GenerationParams,
) -> Result<String, String> {
    let job_id = app_state.queue_manager.add_job(params).await;
    Ok(job_id)
}

#[command]
async fn get_queue_jobs(
    app_state: State<'_, AppState>,
) -> Result<Vec<queue::GenerationJob>, String> {
    let jobs = app_state.queue_manager.get_jobs().await;
    Ok(jobs)
}

#[command]
async fn get_queue_job(
    app_state: State<'_, AppState>,
    job_id: String,
) -> Result<Option<queue::GenerationJob>, String> {
    let job = app_state.queue_manager.get_job(&job_id).await;
    Ok(job)
}

#[command]
async fn cancel_queue_job(
    app_state: State<'_, AppState>,
    job_id: String,
) -> Result<bool, String> {
    let cancelled = app_state.queue_manager.cancel_job(&job_id).await;
    Ok(cancelled)
}

#[command]
async fn clear_completed_jobs(
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    app_state.queue_manager.clear_completed().await;
    Ok(())
}
```

**Step 4: Register commands**

Update `invoke_handler`:

```rust
.invoke_handler(tauri::generate_handler![
    health_check,
    init_database,
    generate_image,
    get_gallery_images,
    search_gallery_images,
    toggle_favorite,
    add_image_tag,
    remove_image_tag,
    delete_gallery_image,
    add_to_queue,
    get_queue_jobs,
    get_queue_job,
    cancel_queue_job,
    clear_completed_jobs,
])
```

**Step 5: Test compilation**

Run: `cd src-tauri && cargo build`
Expected: Builds successfully

**Step 6: Commit queue commands**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add Tauri commands for queue management

- Add queue_manager to AppState
- Add add_to_queue command
- Add get_queue_jobs and get_queue_job commands
- Add cancel_queue_job command
- Add clear_completed_jobs command
- Register all queue commands

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Create Queue Store

**Files:**
- Create: `src/stores/queue.ts`
- Test: TypeScript compilation

**Step 1: Create queue store**

Create `src/stores/queue.ts`:

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface GenerationParams {
  prompt: string
  negative_prompt?: string
  steps: number
  cfg_scale: number
  width: number
  height: number
  seed: number
  model: string
}

export type JobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'

export interface GenerationJob {
  id: string
  params: GenerationParams
  status: JobStatus
  progress: number
  created_at: number
  started_at?: number
  completed_at?: number
  result_path?: string
  error?: string
}

export const useQueueStore = defineStore('queue', () => {
  // State
  const jobs = ref<GenerationJob[]>([])
  const isPolling = ref(false)
  const pollingInterval = ref<number | null>(null)

  // Computed
  const pendingJobs = computed(() =>
    jobs.value.filter((j) => j.status === 'pending')
  )

  const runningJobs = computed(() =>
    jobs.value.filter((j) => j.status === 'running')
  )

  const completedJobs = computed(() =>
    jobs.value.filter((j) => j.status === 'completed')
  )

  const failedJobs = computed(() =>
    jobs.value.filter((j) => j.status === 'failed')
  )

  const queueLength = computed(() => pendingJobs.value.length)
  const hasRunningJobs = computed(() => runningJobs.value.length > 0)

  // Actions
  async function addToQueue(params: GenerationParams): Promise<string> {
    try {
      const jobId = await invoke<string>('add_to_queue', { params })
      await refreshJobs()
      return jobId
    } catch (error) {
      console.error('Failed to add to queue:', error)
      throw error
    }
  }

  async function refreshJobs(): Promise<void> {
    try {
      const result = await invoke<GenerationJob[]>('get_queue_jobs')
      jobs.value = result
    } catch (error) {
      console.error('Failed to refresh jobs:', error)
    }
  }

  async function getJob(jobId: string): Promise<GenerationJob | null> {
    try {
      const result = await invoke<GenerationJob | null>('get_queue_job', {
        jobId,
      })
      return result
    } catch (error) {
      console.error('Failed to get job:', error)
      return null
    }
  }

  async function cancelJob(jobId: string): Promise<boolean> {
    try {
      const cancelled = await invoke<boolean>('cancel_queue_job', { jobId })
      if (cancelled) {
        await refreshJobs()
      }
      return cancelled
    } catch (error) {
      console.error('Failed to cancel job:', error)
      return false
    }
  }

  async function clearCompleted(): Promise<void> {
    try {
      await invoke('clear_completed_jobs')
      await refreshJobs()
    } catch (error) {
      console.error('Failed to clear completed jobs:', error)
    }
  }

  function startPolling(intervalMs: number = 1000): void {
    if (isPolling.value) return

    isPolling.value = true
    pollingInterval.value = window.setInterval(() => {
      refreshJobs()
    }, intervalMs)
  }

  function stopPolling(): void {
    if (!isPolling.value) return

    isPolling.value = false
    if (pollingInterval.value !== null) {
      clearInterval(pollingInterval.value)
      pollingInterval.value = null
    }
  }

  return {
    // State
    jobs,
    isPolling,

    // Computed
    pendingJobs,
    runningJobs,
    completedJobs,
    failedJobs,
    queueLength,
    hasRunningJobs,

    // Actions
    addToQueue,
    refreshJobs,
    getJob,
    cancelJob,
    clearCompleted,
    startPolling,
    stopPolling,
  }
})
```

**Step 2: Test compilation**

Run: `npm run check`
Expected: TypeScript compiles successfully

**Step 3: Commit queue store**

```bash
git add src/stores/queue.ts
git commit -m "feat: add queue store for generation queue management

- Create GenerationParams and GenerationJob interfaces
- Add queue state management with Pinia
- Add computed properties for job filtering
- Implement queue operations (add, cancel, clear)
- Add polling mechanism for real-time updates

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Create Queue Display Component

> **Note:** This plan was superseded. The queue UI is now integrated into `src/components/generation/BottomPanel.vue` instead of a separate QueuePanel component.

**Files:**
- ~~Create: `src/components/queue/QueuePanel.vue`~~ → Now in `BottomPanel.vue`
- Test: Manual UI testing

**Step 1: Create QueuePanel component** *(Superseded - see BottomPanel.vue)*

Create `src/components/queue/QueuePanel.vue`:

```vue
<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import Card from 'primevue/card'
import Button from 'primevue/button'
import ProgressBar from 'primevue/progressbar'
import { useQueueStore } from '@/stores/queue'
import type { GenerationJob } from '@/stores/queue'

const queueStore = useQueueStore()

onMounted(() => {
  queueStore.refreshJobs()
  queueStore.startPolling(1000)
})

onUnmounted(() => {
  queueStore.stopPolling()
})

function getStatusColor(status: string): string {
  switch (status) {
    case 'pending':
      return 'info'
    case 'running':
      return 'primary'
    case 'completed':
      return 'success'
    case 'failed':
      return 'danger'
    case 'cancelled':
      return 'warning'
    default:
      return 'secondary'
  }
}

function getStatusIcon(status: string): string {
  switch (status) {
    case 'pending':
      return 'pi-clock'
    case 'running':
      return 'pi-spin pi-spinner'
    case 'completed':
      return 'pi-check'
    case 'failed':
      return 'pi-times'
    case 'cancelled':
      return 'pi-ban'
    default:
      return 'pi-question'
  }
}

function formatDuration(startedAt?: number, completedAt?: number): string {
  if (!startedAt) return '-'
  const end = completedAt || Date.now() / 1000
  const duration = Math.floor(end - startedAt)
  if (duration < 60) return `${duration}s`
  return `${Math.floor(duration / 60)}m ${duration % 60}s`
}

async function handleCancel(job: GenerationJob) {
  await queueStore.cancelJob(job.id)
}

async function handleClearCompleted() {
  await queueStore.clearCompleted()
}
</script>

<template>
  <Card class="queue-panel">
    <template #title>
      <div class="queue-header">
        <span>Generation Queue</span>
        <div class="queue-stats">
          <span class="stat">
            <i class="pi pi-clock"></i>
            {{ queueStore.queueLength }}
          </span>
          <span class="stat">
            <i class="pi pi-spin pi-spinner" v-if="queueStore.hasRunningJobs"></i>
            <i class="pi pi-check" v-else></i>
            {{ queueStore.runningJobs.length }}
          </span>
        </div>
      </div>
    </template>

    <template #content>
      <div class="queue-actions">
        <Button
          label="Clear Completed"
          icon="pi pi-trash"
          size="small"
          outlined
          :disabled="queueStore.completedJobs.length === 0"
          @click="handleClearCompleted"
        />
      </div>

      <div class="queue-list">
        <div
          v-for="job in queueStore.jobs"
          :key="job.id"
          class="queue-item"
          :class="`status-${job.status}`"
        >
          <div class="job-header">
            <i :class="`pi ${getStatusIcon(job.status)}`"></i>
            <span class="job-status">{{ job.status }}</span>
            <span class="job-time">{{ formatDuration(job.started_at, job.completed_at) }}</span>
          </div>

          <div class="job-prompt">
            {{ job.params.prompt.substring(0, 80) }}
            {{ job.params.prompt.length > 80 ? '...' : '' }}
          </div>

          <div class="job-params">
            <span>{{ job.params.width }}×{{ job.params.height }}</span>
            <span>{{ job.params.steps }} steps</span>
            <span>CFG {{ job.params.cfg_scale }}</span>
          </div>

          <ProgressBar
            v-if="job.status === 'running'"
            :value="job.progress * 100"
            :show-value="true"
          />

          <div v-if="job.error" class="job-error">
            {{ job.error }}
          </div>

          <div class="job-actions">
            <Button
              v-if="job.status === 'pending'"
              label="Cancel"
              icon="pi pi-times"
              size="small"
              severity="danger"
              text
              @click="handleCancel(job)"
            />
          </div>
        </div>

        <div v-if="queueStore.jobs.length === 0" class="empty-queue">
          <i class="pi pi-inbox"></i>
          <p>No jobs in queue</p>
        </div>
      </div>
    </template>
  </Card>
</template>

<style scoped>
.queue-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.queue-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.queue-stats {
  display: flex;
  gap: 1rem;
}

.stat {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.875rem;
  color: var(--text-color-secondary);
}

.queue-actions {
  margin-bottom: 1rem;
}

.queue-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.queue-item {
  padding: 1rem;
  border: 1px solid var(--surface-border);
  border-radius: var(--border-radius);
  background: var(--surface-card);
}

.queue-item.status-running {
  border-color: var(--primary-color);
  background: var(--primary-50);
}

.queue-item.status-completed {
  opacity: 0.6;
}

.job-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
  font-size: 0.875rem;
}

.job-status {
  font-weight: 600;
  text-transform: capitalize;
}

.job-time {
  margin-left: auto;
  color: var(--text-color-secondary);
}

.job-prompt {
  margin-bottom: 0.5rem;
  font-size: 0.875rem;
}

.job-params {
  display: flex;
  gap: 0.75rem;
  font-size: 0.75rem;
  color: var(--text-color-secondary);
  margin-bottom: 0.5rem;
}

.job-error {
  color: var(--red-500);
  font-size: 0.75rem;
  margin-top: 0.5rem;
  padding: 0.5rem;
  background: var(--red-50);
  border-radius: var(--border-radius);
}

.job-actions {
  margin-top: 0.5rem;
  display: flex;
  justify-content: flex-end;
}

.empty-queue {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 3rem;
  color: var(--text-color-secondary);
}

.empty-queue i {
  font-size: 3rem;
  margin-bottom: 1rem;
}
</style>
```

**Step 2: Test compilation**

Run: `npm run check`
Expected: TypeScript compiles successfully

**Step 3: Commit queue panel**

```bash
git add src/components/queue/QueuePanel.vue
git commit -m "feat: add queue panel component for job display

- Create QueuePanel with real-time job list
- Add status indicators with colors and icons
- Display job progress with ProgressBar
- Show job parameters and duration
- Add cancel button for pending jobs
- Add clear completed jobs action

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Integrate Queue with Generate View

**Files:**
- Modify: `src/views/GenerateView.vue`
- Modify: `src/components/generation/GenerateButton.vue`
- Test: Manual UI testing

**Step 1: Add QueuePanel to GenerateView**

Update `src/views/GenerateView.vue`:

```vue
<script setup lang="ts">
import { ref } from 'vue'
import Card from 'primevue/card'
import GenerateButton from '@/components/generation/GenerateButton.vue'
import PromptInput from '@/components/generation/PromptInput.vue'
import ParameterControls from '@/components/generation/ParameterControls.vue'
import QueuePanel from '@/components/queue/QueuePanel.vue'
import { useGenerationStore } from '@/stores/generation'
import { useQueueStore } from '@/stores/queue'

const generationStore = useGenerationStore()
const queueStore = useQueueStore()

const prompt = ref('')
const negativePrompt = ref('')

async function handleGenerate() {
  const params = {
    prompt: prompt.value,
    negative_prompt: negativePrompt.value || undefined,
    steps: generationStore.steps,
    cfg_scale: generationStore.cfgScale,
    width: generationStore.width,
    height: generationStore.height,
    seed: generationStore.seed === -1 ? Math.floor(Math.random() * 2147483647) : generationStore.seed,
    model: 'flux-schnell',
  }

  await queueStore.addToQueue(params)
}
</script>

<template>
  <div class="workspace-content generate-view">
    <div class="generate-layout">
      <!-- Left: Controls -->
      <div class="generate-controls">
        <Card>
          <template #title>Generation Settings</template>
          <template #content>
            <PromptInput v-model="prompt" label="Prompt" />
            <PromptInput v-model="negativePrompt" label="Negative Prompt" />
            <ParameterControls />
            <GenerateButton
              :disabled="!prompt"
              :queue-count="queueStore.queueLength"
              @generate="handleGenerate"
            />
          </template>
        </Card>
      </div>

      <!-- Center: Queue -->
      <div class="generate-queue">
        <QueuePanel />
      </div>

      <!-- Right: Preview (placeholder for now) -->
      <div class="generate-preview">
        <Card>
          <template #title>Preview</template>
          <template #content>
            <div class="preview-placeholder">
              <i class="pi pi-image" style="font-size: 3rem"></i>
              <p>Generated images will appear here</p>
            </div>
          </template>
        </Card>
      </div>
    </div>
  </div>
</template>

<style scoped>
.generate-view {
  padding: 1rem;
}

.generate-layout {
  display: grid;
  grid-template-columns: 350px 1fr 400px;
  gap: 1rem;
  height: 100%;
}

.generate-controls,
.generate-queue,
.generate-preview {
  height: 100%;
  overflow: auto;
}

.preview-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 3rem;
  color: var(--text-color-secondary);
}
</style>
```

**Step 2: Update GenerateButton**

Update `src/components/generation/GenerateButton.vue` to show queue count:

```vue
<script setup lang="ts">
interface Props {
  disabled?: boolean
  queueCount?: number
}

interface Emits {
  (e: 'generate'): void
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  queueCount: 0,
})

const emit = defineEmits<Emits>()

function handleClick() {
  emit('generate')
}
</script>

<template>
  <Button
    label="Generate"
    icon="pi pi-play"
    size="large"
    :disabled="disabled"
    class="generate-button"
    @click="handleClick"
  >
    <template v-if="queueCount > 0" #badge>
      <span class="queue-badge">{{ queueCount }}</span>
    </template>
  </Button>
</template>

<style scoped>
.generate-button {
  width: 100%;
  margin-top: 1rem;
  position: relative;
}

.queue-badge {
  position: absolute;
  top: -0.5rem;
  right: -0.5rem;
  background: var(--primary-color);
  color: white;
  border-radius: 50%;
  width: 1.5rem;
  height: 1.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.75rem;
  font-weight: 600;
}
</style>
```

**Step 3: Test compilation**

Run: `npm run check`
Expected: Compiles successfully

**Step 4: Commit queue integration**

```bash
git add src/views/GenerateView.vue src/components/generation/GenerateButton.vue
git commit -m "feat: integrate queue system with generation view

- Add QueuePanel to GenerateView layout
- Update handleGenerate to add jobs to queue
- Add queue count badge to GenerateButton
- Create 3-column layout: controls, queue, preview

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Summary

Phase 5 Task List:
1. ✅ Create Generation Queue Backend (async queue manager)
2. ✅ Add Queue Tauri Commands (queue operations)
3. ✅ Create Queue Store (frontend state management)
4. ✅ Create Queue Display Component (QueuePanel UI)
5. ✅ Integrate Queue with Generate View (full integration)

**What's Working:**
- Jobs can be added to queue
- Queue state is managed in Rust with async operations
- Frontend polls for updates every second
- Jobs show status, progress, and duration
- Pending jobs can be cancelled
- Completed jobs can be cleared

**What's Still Needed (Future Phases):**
- Actual queue processing (jobs don't execute yet)
- Real-time progress via Tauri events (instead of polling)
- Batch generation UI
- Performance monitoring dashboard
- Auto-recovery on failures

**Testing:**
Run the app and click Generate multiple times. Jobs should appear in the queue panel with "pending" status. The queue count badge should update on the button.
