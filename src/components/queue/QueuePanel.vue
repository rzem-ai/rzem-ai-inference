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
