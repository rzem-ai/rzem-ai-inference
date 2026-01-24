<template>
  <div class="flex flex-col h-full">
    <!-- Header -->
    <div class="p-3 border-b bg-surface-800 border-surface-700">
      <div class="flex items-center justify-between">
        <span class="text-xs font-semibold tracking-wider uppercase text-surface-50">History</span>
        <span class="text-xs text-surface-400">{{ reversedHistoryJobs.length }}</span>
      </div>
    </div>

    <!-- Empty State -->
    <div v-if="reversedHistoryJobs.length === 0" class="flex flex-col items-center justify-center flex-1 gap-3 p-4 text-surface-500">
      <fa :icon="['fal', 'clock-rotate-left']" size="2x" />
      <p class="text-xs text-center">No generation history yet</p>
    </div>

    <!-- History List -->
    <div v-else class="flex-1 overflow-y-auto">
      <div class="flex flex-col gap-2 p-2">
        <div
          v-for="job in reversedHistoryJobs"
          :key="job.id"
          class="relative overflow-hidden transition-all duration-200 border rounded-lg cursor-pointer bg-surface-900 border-surface-700 hover:border-blue-500 hover:shadow-lg group"
          @click="handleJobClick(job)">
          <!-- Thumbnail -->
          <div v-if="job.result_path" class="relative w-full overflow-hidden aspect-square bg-surface-950">
            <img :src="convertFileSrc(job.result_path)" :alt="job.params.prompt" class="object-cover w-full h-full" />

            <!-- Status Overlay -->
            <div v-if="job.status === 'failed'" class="absolute inset-0 flex items-center justify-center bg-red-900/50">
              <fa :icon="['fal', 'circle-xmark']" class="text-red-400" size="2x" />
            </div>
          </div>

          <!-- Placeholder for failed jobs without image -->
          <div v-else class="flex items-center justify-center w-full bg-surface-950 aspect-square">
            <XCircle class="w-8 h-8 text-red-400" />
          </div>

          <!-- Info -->
          <div class="p-2">
            <p class="mb-1 text-xs leading-tight line-clamp-2 text-surface-200">
              {{ job.params.prompt }}
            </p>
            <div class="flex items-center justify-between text-xs text-surface-500">
              <span>{{ formatTime(job.completed_at) }}</span>
              <div class="flex items-center gap-1">
                <span>{{ job.params.width }}×{{ job.params.height }}</span>
              </div>
            </div>
          </div>

          <!-- Hover Actions -->
          <div class="absolute top-1 right-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <Button
              size="small"
              rounded
              severity="secondary"
              class="backdrop-blur-lg bg-surface-800/80"
              @click.stop="handleReuse(job)"
              v-tooltip.left="'Reuse settings'">
              <fa :icon="['fal', 'rotate-left']" size="sm" />
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/core';
import { useQueueStore, type GenerationJob } from '@/stores/queue';
import { useGenerationStore } from '@/stores/generation';
import Button from 'primevue/button';

const queueStore = useQueueStore();
const generationStore = useGenerationStore();

// Show all completed/failed jobs (both from active queue and history)
// Reverse so newest appears at the top
const reversedHistoryJobs = computed(() => {
  // Get completed/failed jobs from active queue
  const completedActive = queueStore.jobs.filter(
    (j) => j.status === 'completed' || j.status === 'failed'
  );

  // Combine with history jobs and reverse
  return [...completedActive, ...queueStore.historyJobs].reverse();
});

const formatTime = (timestamp?: number) => {
  if (!timestamp) return '';
  const date = new Date(timestamp * 1000);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toLocaleDateString();
};

const handleJobClick = (job: GenerationJob) => {
  // TODO: Open detail modal or navigate to image in gallery
  console.log('Clicked job:', job);
};

const handleReuse = (job: GenerationJob) => {
  // Copy the job's parameters to current generation settings
  generationStore.currentParams = {
    ...generationStore.currentParams,
    prompt: job.params.prompt,
    steps: job.params.steps,
    cfgScale: job.params.cfg_scale,
    width: job.params.width,
    height: job.params.height,
    seed: job.params.seed,
    model: job.params.model,
    sampler: job.params.sampler || 'euler',
    scheduler: job.params.scheduler || 'simple',
  };
};
</script>

<style scoped>
@reference "tailwindcss";

/* Custom scrollbar */
.overflow-y-auto::-webkit-scrollbar {
  width: 6px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: #1e293b;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background: #475569;
  border-radius: 3px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background: #64748b;
}

/* Line clamp utility */
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>