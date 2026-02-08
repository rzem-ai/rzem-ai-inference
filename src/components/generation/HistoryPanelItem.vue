<template>
  <div
    class="relative overflow-hidden transition-all duration-200 border rounded-lg cursor-pointer bg-surface-950 border-surface-700 hover:border-blue-500 hover:shadow-lg group"
    @click="handleJobClick(historyJob)">
    <!-- Thumbnail -->
    <div v-if="historyJob.result_path" class="relative w-full overflow-hidden aspect-square bg-surface-950">
      <img :src="convertFileSrc(historyJob.result_path)" :alt="historyJob.params.prompt" class="object-cover w-full h-full" />

      <!-- Status Overlay -->
      <div v-if="historyJob.status === 'failed'" class="absolute inset-0 flex items-center justify-center bg-red-900/50">
        <fa :icon="['fal', 'circle-xmark']" class="text-red-400" size="2x" />
      </div>
    </div>

    <!-- Placeholder for failed jobs without image -->
    <div v-else class="flex items-center justify-center w-full bg-surface-950 aspect-square">
      <fa :icon="['fal', 'circle-xmark']" class="text-red-400" size="2x" />
    </div>

    <!-- Info -->
    <div class="p-2">
      <p class="mb-1 text-xs leading-tight line-clamp-2 text-surface-200">
        {{ historyJob.params.prompt }}
      </p>
      <div class="flex items-center justify-between text-xs text-surface-500">
        <span>{{ formatTime(historyJob.completed_at) }}</span>
        <div class="flex items-center gap-1">
          <span>{{ historyJob.params.width }}×{{ historyJob.params.height }}</span>
        </div>
      </div>
    </div>

    <!-- Hover Actions -->
    <div class="absolute transition-opacity opacity-0 top-1 right-1 group-hover:opacity-100">
      <Button
        size="small"
        rounded
        severity="secondary"
        class="backdrop-blur-lg bg-surface-800/80"
        @click.stop="handleReuse(historyJob)"
        v-tooltip.left="'Reuse settings'">
        <fa :icon="['fal', 'rotate-left']" size="sm" />
      </Button>
    </div>
  </div>
</template>
<script setup lang="ts">
import { convertFileSrc } from '@/utils/backend-bridge';
import { useToast } from 'primevue/usetoast';
import {  useGenerationStore } from '@/stores/generation';
import Button from 'primevue/button';
import { GenerationJob } from '@/types';

const { historyJob } = defineProps(['historyJob']);

const generationStore = useGenerationStore();
const toast = useToast();

const emit = defineEmits<{
  restoreImage: [imagePath: string];
}>();

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

const restoreParameters = (job: GenerationJob) => {
  // Copy the job's parameters to current generation settings
  generationStore.currentParams = {
    ...generationStore.currentParams,
    prompt: job.params.prompt,
    steps: job.params.steps,
    cfg_scale: job.params.cfg_scale,
    width: job.params.width,
    height: job.params.height,
    seed: job.params.seed,
    model_component_id: job.params.model_component_id,
    sampler: job.params.sampler || 'euler',
    scheduler: job.params.scheduler || 'simple',
  };

  // Show confirmation toast
  toast.add({
    severity: 'success',
    summary: 'Settings Restored',
    detail: `Loaded parameters from previous generation (seed: ${job.params.seed})`,
    life: 3000,
  });
};

const handleJobClick = (job: GenerationJob) => {
  // Restore generation parameters when clicking on history item
  restoreParameters(job);

  // Also restore the image to the center panel if it exists
  if (job.result_path) {
    emit('restoreImage', job.result_path);
  }
};

const handleReuse = (job: GenerationJob) => {
  // Also restore parameters when clicking the reuse button
  restoreParameters(job);

  // Also restore the image to the center panel if it exists
  if (job.result_path) {
    emit('restoreImage', job.result_path);
  }
};
</script>
