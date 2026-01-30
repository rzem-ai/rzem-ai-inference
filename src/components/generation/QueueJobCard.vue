<template>
  <div
    class="relative flex min-w-62.5 items-start gap-3 rounded-lg border p-2"
    :class="{
      'border-surface-700 bg-surface-900 shadow': job.status === 'pending',
      'border-blue-500/50 bg-surface-900 shadow-lg': job.status === 'running',
      'border-green-500/50 bg-surface-900 shadow': job.status === 'completed',
      'border-red-500 bg-red-50 shadow': job.status === 'failed',
    }">
    <!-- Preview overlay (shown when preview available and job running) -->
    <div v-if="job.previewData && job.status === 'running'" class="absolute top-2 right-2 z-10 preview-thumbnail">
      <img
        :src="`data:image/jpeg;base64,${job.previewData}`"
        alt="Generation preview"
        class="w-20 h-20 rounded-lg object-cover shadow-lg border-2 border-blue-400" />
      <div class="preview-badge">Preview</div>
    </div>

    <!-- Status Icon -->
    <div
      class="flex items-center justify-center w-8 h-8 rounded-full"
      :class="{
        'text-surface-400': job.status === 'pending',
        'text-primary-500': job.status === 'running',
        'bg-green-500 text-white': job.status === 'completed',
        'bg-red-500 text-white': job.status === 'failed',
      }">
      <fa :icon="['fal', statusIcon]" :class="{ 'fa-spin': job.status === 'running' }" />
    </div>

    <!-- Job Info -->
    <div class="flex-1 min-w-0">
      <div class="flex-1 min-w-0">
        <div class="text-sm font-medium truncate text-surface-300">{{ prompt }}</div>
        <div class="mt-1 flex gap-2 text-xs text-(--text-secondary)">
          <span>{{ job.params.width }}x{{ job.params.height }}</span>
          <span>{{ job.params.steps }} steps</span>
        </div>
      </div>

      <!-- Progress Bars (only for running jobs) -->
      <div v-if="job.status === 'running'" class="flex flex-col w-full gap-1 pt-2">
        <!-- Pipeline progress (top bar) -->
        <div class="flex items-center gap-2">
          <span class="w-12.5 shrink-0 text-xs text-(--text-primary)">Pipeline</span>
          <ProgressBar :value="progress" :showValue="false" class="flex-1 border border-surface-400" />
          <span class="w-10 font-mono text-xs text-right shrink-0 text-surface-300">{{ progress }}%</span>
        </div>
        <!-- Steps progress (bottom bar) -->
        <div class="flex items-center gap-2">
          <span class="w-12.5 shrink-0 text-xs text-(--text-primary)">Steps</span>
          <ProgressBar :value="getStepProgress(job)" :showValue="false" class="flex-1 border progress-bar-steps border-surface-400" />
          <span class="w-10 font-mono text-xs text-right shrink-0 text-surface-300">{{ getStepDisplay(job) }}</span>
        </div>
        <div class="font-mono text-xs text-center">{{ getStageDisplayName(job.currentStage) }}</div>
      </div>

      <div v-else class="flex flex-col w-full gap-1 pt-2">
        <!-- Pipeline progress (top bar) -->
        <div class="flex items-center gap-2">
          <span class="w-12.5 shrink-0 text-xs text-(--text-primary)">Pipeline</span>
          <ProgressBar :value="progress" :showValue="false" class="flex-1 border border-surface-400" />
          <span class="w-10 font-mono text-xs text-right shrink-0 text-surface-300">{{ progress }}%</span>
        </div>
        <!-- Steps progress (bottom bar) -->
        <div class="flex items-center gap-2">
          <span class="w-12.5 shrink-0 text-xs text-(--text-primary)">Steps</span>
          <ProgressBar :value="0" :showValue="false" class="flex-1 border progress-bar-steps border-surface-400" />
          <span class="w-10 font-mono text-xs text-right shrink-0 text-surface-300">{{ getStepDisplay(job) }}</span>
        </div>
        <div class="font-mono text-xs text-center">Waiting...</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import ProgressBar from 'primevue/progressbar';
import { type GenerationJob } from '@/stores/queue';

interface Props {
  job: GenerationJob;
}

const props = defineProps<Props>();

// Computed map for status icons (FontAwesome icon names)
const statusIconMap = computed(() => ({
  pending: 'clock',
  running: 'arrows-rotate',
  completed: 'check',
  failed: 'timer',
  cancelled: 'ban',
  default: 'file-question',
}));

const statusIcon = computed(() => {
  return statusIconMap.value[props?.job?.status as keyof typeof statusIconMap.value] || statusIconMap.value.default;
});

const progress = computed((): number => {
  return Math.round(props.job.progress * 100);
});

const prompt = computed((): string => {
  const text = props?.job?.params?.prompt ? props.job.params.prompt : '';
  return text.length > 60 ? text.substring(0, 60) + '...' : text;
});

// Computed map for stage display names
const stageDisplayNames = computed(() => ({
  loading_models: 'Loading models...',
  encoding_t5: 'Encoding prompt (T5)...',
  encoding_clip: 'Encoding prompt (CLIP)...',
  denoising: 'Drawing...',
  decoding_vae: 'Drawing image...',
  encoding_png: 'Saving image...',
}));

const getStageDisplayName = computed(() => (stage?: string) => {
  if (!stage) return 'Starting...';
  return stageDisplayNames.value[stage as keyof typeof stageDisplayNames.value] || stage;
});

const getStepProgress = computed(() => (job: GenerationJob) => {
  const totalSteps = job.params.steps;
  if (!totalSteps || totalSteps === 0) return 0;

  const stage = job.currentStage;
  if (!stage || stage === 'loading_models' || stage === 'encoding_t5' || stage === 'encoding_clip') {
    return 0;
  }
  if (stage === 'decoding_vae' || stage === 'encoding_png') {
    return 100;
  }
  if (job.currentStep !== undefined) {
    return (job.currentStep / totalSteps) * 100;
  }
  return 0;
});

const getStepDisplay = computed(() => (job: GenerationJob) => {
  const totalSteps = job.params.steps;
  const stage = job.currentStage;

  if (stage === 'decoding_vae' || stage === 'encoding_png') {
    return `${totalSteps}/${totalSteps}`;
  }
  const current = job.currentStep ?? 0;
  return `${current}/${totalSteps}`;
});
</script>

<style scoped>
@reference "tailwindcss";

.preview-thumbnail {
  position: relative;
  animation: fadeIn 0.3s ease-in;
}

.preview-badge {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: rgba(0, 0, 0, 0.8);
  color: white;
  font-size: 0.625rem;
  text-align: center;
  padding: 2px 4px;
  border-bottom-left-radius: 0.5rem;
  border-bottom-right-radius: 0.5rem;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: scale(0.9);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}
</style>
