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
