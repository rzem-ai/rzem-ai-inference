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

<script setup lang="ts">
import { useGenerationStore } from '@/stores/generation';
import Card from 'primevue/card';
import ProgressBar from 'primevue/progressbar';

const store = useGenerationStore();

const getProgressPercent = (jobId: string) => {
    const progress = store.getProgress(jobId);
    if (!progress) return 0;
    return Math.round((progress.step / progress.totalSteps) * 100);
};

const getStatusText = (jobId: string) => {
    const progress = store.getProgress(jobId);
    if (!progress) return 'Queued';
    return progress.status.charAt(0).toUpperCase() + progress.status.slice(1);
};
</script>

<style scoped>
@reference "tailwindcss";

.empty-state {
  @apply flex items-center justify-center h-full text-sm;
  color: var(--color-slate-500);
}

.jobs-container {
  @apply flex flex-col gap-3;
}

.job-card {
  @apply text-sm;
}

.job-header {
  @apply flex justify-between items-center;
}

.job-status {
  @apply py-1 px-2 rounded text-xs font-semibold uppercase;

  &.queued {
    background: rgba(45, 212, 191, 0.15);
    color: var(--color-teal-400);
  }

  &.running {
    background: rgba(251, 191, 36, 0.15);
    color: #fbbf24;
  }

  &.completed {
    background: rgba(34, 197, 94, 0.15);
    color: #22c55e;
  }

  &.failed {
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
  }
}

.job-content {
  @apply flex flex-col gap-3;
}

.job-prompt {
  @apply m-0 leading-relaxed overflow-hidden text-ellipsis;
  color: var(--color-slate-200);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.progress-section {
  @apply flex flex-col gap-1;
}

.progress-text {
  @apply text-xs;
  color: var(--color-slate-400);
}
</style>
