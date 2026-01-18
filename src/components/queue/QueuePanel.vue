<template>
  <div class="px-1 pb-2 panel-scroll">
    <Card>
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
            @click="handleClearCompleted" />
        </div>

        <div class="queue-list-scroll">
          <div v-for="job in queueStore.jobs" :key="job.id" class="queue-item" :class="`status-${job.status}`">
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

            <div v-if="job.status === 'running'" class="job-progress">
              <ProgressBar :value="job.progress * 100" :show-value="true" />
              <div v-if="job.statusMessage" class="progress-status">
                {{ job.statusMessage }}
              </div>
            </div>

            <div v-if="job.error" class="job-error">
              {{ job.error }}
            </div>

            <!-- Generation Stats for completed jobs -->
            <div v-if="job.stats && job.status === 'completed'" class="job-stats">
              <div class="stats-header" @click="toggleStats(job.id)">
                <i :class="`pi ${expandedStats[job.id] ? 'pi-chevron-down' : 'pi-chevron-right'}`"></i>
                <span>Stats: {{ formatMs(job.stats.total_ms) }} total</span>
              </div>
              <div v-if="expandedStats[job.id]" class="stats-details">
                <div class="stats-grid">
                  <div v-if="job.stats.model_load_ms" class="stat-row">
                    <span class="stat-label">Model Load</span>
                    <span class="stat-value">{{ formatMs(job.stats.model_load_ms) }}</span>
                  </div>
                  <div class="stat-row">
                    <span class="stat-label">T5 Encode</span>
                    <span class="stat-value">{{ formatMs(job.stats.t5_encode_ms) }}</span>
                  </div>
                  <div class="stat-row">
                    <span class="stat-label">CLIP Encode</span>
                    <span class="stat-value">{{ formatMs(job.stats.clip_encode_ms) }}</span>
                  </div>
                  <div class="stat-row highlight">
                    <span class="stat-label">Denoising ({{ job.stats.steps }} steps)</span>
                    <span class="stat-value">{{ formatMs(job.stats.denoise_ms) }}</span>
                  </div>
                  <div class="stat-row">
                    <span class="stat-label">VAE Decode</span>
                    <span class="stat-value">{{ formatMs(job.stats.vae_decode_ms) }}</span>
                  </div>
                  <div class="stat-row">
                    <span class="stat-label">PNG Encode</span>
                    <span class="stat-value">{{ formatMs(job.stats.png_encode_ms) }}</span>
                  </div>
                </div>
                <div class="stats-meta">
                  <span>{{ job.stats.model_type }}</span>
                  <span>{{ job.stats.image_shape[2] }}×{{ job.stats.image_shape[1] }}</span>
                </div>
              </div>
            </div>

            <div class="job-actions">
              <Button v-if="job.status === 'pending'" label="Cancel" icon="pi pi-times" size="small" severity="danger" text @click="handleCancel(job)" />
            </div>
          </div>

          <div v-if="queueStore.jobs.length === 0" class="empty-queue">
            <i class="pi pi-inbox"></i>
            <p>No jobs in queue</p>
          </div>
        </div>
      </template>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, reactive } from 'vue';
import Card from 'primevue/card';
import Button from 'primevue/button';
import ProgressBar from 'primevue/progressbar';
import { useQueueStore } from '@/stores/queue';
import type { GenerationJob } from '@/stores/queue';

const queueStore = useQueueStore();

// Track which job stats are expanded
const expandedStats = reactive<Record<string, boolean>>({});

function toggleStats(jobId: string) {
  expandedStats[jobId] = !expandedStats[jobId];
}

function formatMs(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

onMounted(() => {
  queueStore.refreshJobs();
  queueStore.startPolling(1000);
});

onUnmounted(() => {
  queueStore.stopPolling();
});

function getStatusIcon(status: string): string {
  switch (status) {
    case 'pending':
      return 'pi-clock';
    case 'running':
      return 'pi-spin pi-spinner';
    case 'completed':
      return 'pi-check';
    case 'failed':
      return 'pi-times';
    case 'cancelled':
      return 'pi-ban';
    default:
      return 'pi-question';
  }
}

// @ts-expect-error - Function reserved for future use with Badge/Tag components
function getStatusColor(status: string): string {
  switch (status) {
    case 'pending':
      return 'info';
    case 'running':
      return 'primary';
    case 'completed':
      return 'success';
    case 'failed':
      return 'danger';
    case 'cancelled':
      return 'warning';
    default:
      return 'secondary';
  }
}

function formatDuration(startedAt?: number, completedAt?: number): string {
  if (!startedAt) return '-';
  const end = completedAt || Date.now() / 1000;
  const duration = Math.floor(end - startedAt);
  if (duration < 60) return `${duration}s`;
  return `${Math.floor(duration / 60)}m ${duration % 60}s`;
}

async function handleCancel(job: GenerationJob) {
  await queueStore.cancelJob(job.id);
}

async function handleClearCompleted() {
  await queueStore.clearCompleted();
}
</script>

<style scoped>
@reference "tailwindcss";

.queue-header {
  @apply flex justify-between items-center;
  color: var(--color-slate-50);
}

.queue-stats {
  @apply flex gap-4;
}

.stat {
  @apply flex items-center gap-1 text-sm;
  color: var(--color-slate-400);
}

.queue-actions {
  @apply mb-4;
}

.queue-list-scroll {
  @apply flex flex-col gap-3;
}

.queue-item {
  @apply p-4 border rounded-lg;
  border-color: var(--color-slate-700);
  background: var(--color-slate-900);
  color: var(--color-slate-50);

  &.status-running {
    border-color: var(--color-teal-400);
    background: rgba(45, 212, 191, 0.1);
  }

  &.status-completed {
    @apply opacity-70;
  }

  &.status-failed {
    border-color: #ef4444;
  }
}

.job-header {
  @apply flex items-center gap-2 mb-2 text-sm;
}

.job-status {
  @apply font-semibold capitalize;
}

.job-time {
  @apply ml-auto;
  color: var(--color-slate-400);
}

.job-prompt {
  @apply mb-2 text-sm;
  color: var(--color-slate-200);
}

.job-params {
  @apply flex gap-3 text-xs mb-2;
  color: var(--color-slate-400);
}

.job-progress {
  @apply flex flex-col gap-1;
}

.progress-status {
  @apply text-xs text-center;
  color: var(--color-teal-400);
}

.job-error {
  @apply text-xs mt-2 p-2 rounded;
  color: #fca5a5;
  background: rgba(239, 68, 68, 0.15);
}

.job-stats {
  @apply mt-2 text-xs border rounded-lg;
  border-color: var(--color-slate-700);
  background: var(--color-slate-800);
}

.stats-header {
  @apply flex items-center gap-2 p-2 cursor-pointer rounded-t-lg;
  color: var(--color-slate-400);

  &:hover {
    background: var(--color-slate-700);
  }

  i {
    @apply text-xs;
  }
}

.stats-details {
  @apply p-2 pt-0;
}

.stats-grid {
  @apply flex flex-col gap-1;
}

.stat-row {
  @apply flex justify-between;
  color: var(--color-slate-300);

  &.highlight {
    @apply font-semibold;
    color: var(--color-teal-400);
  }
}

.stat-label {
  color: var(--color-slate-400);
}

.stat-value {
  @apply font-mono;
  color: var(--color-slate-200);
}

.stats-meta {
  @apply flex gap-3 mt-2 pt-2 border-t;
  border-color: var(--color-slate-700);
  color: var(--color-slate-500);
}

.job-actions {
  @apply mt-2 flex justify-end;
}

.empty-queue {
  @apply flex flex-col items-center justify-center p-12;
  color: var(--color-slate-500);

  i {
    @apply text-5xl mb-4;
    color: var(--color-slate-600);
  }
}
</style>
