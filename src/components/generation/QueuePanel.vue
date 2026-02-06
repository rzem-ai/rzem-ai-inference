<template>
  <div class="relative h-full overflow-hidden">
    <div class="h-full border rounded-lg bg-surface-800 border-surface-700">
      <!-- Demo mode indicator -->
      <div v-if="demoMode" class="absolute top-1 right-2 z-10 flex items-center gap-1 px-2 py-0.5 bg-purple-600 text-white text-xs rounded-full">
        <span class="animate-pulse">●</span> Demo Mode (Ctrl+Shift+D to exit)
      </div>

      <div class="h-full p-3 overflow-auto">
        <div v-if="allJobs.length === 0" class="flex flex-col items-center justify-center h-full gap-2 text-surface-100">
          <fa :icon="['fal', 'inbox']" size="lg" />
          <p class="text-sm">No jobs in queue</p>
        </div>
        <div v-else class="grid grid-cols-4 gap-4">
          <QueueJobCard v-for="job in allJobs" :key="job.id" :job="job" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useGenerationStore } from '@/stores/generation';
import QueueJobCard from './QueueJobCard.vue';
import { GenerationJob, PipelineStage } from '@/types';

const queueStore = useGenerationStore();

// Demo mode state
const demoMode = ref(false);
const demoProgress = ref(0);
let demoInterval: ReturnType<typeof setInterval> | null = null;

// Demo jobs with various states
const demoAllJobs = computed<GenerationJob[]>(() => {
  const baseParams = {
    prompt: 'A beautiful sunset over mountains with dramatic clouds',
    steps: 28,
    cfg_scale: 3.5,
    width: 1024,
    height: 1024,
    seed: 42,
    model_component_id: 'flux-dev',
    clip_component_id: 'clip',
    t5_component_id: 't5',
    vae_component_id: 'vae',
    sampler: 'euler' as const,
    scheduler: 'normal' as const,
    mode: 'txt2img' as const,
  };

  // Cycle through stages based on demo progress
  const stages: PipelineStage[] = ['loading_models', 'encoding_t5', 'encoding_clip', 'denoising', 'decoding_vae', 'encoding_png'];
  const stageIndex = Math.floor((demoProgress.value / 100) * stages.length);
  const currentStage = stages[Math.min(stageIndex, stages.length - 1)];
  const currentStep = currentStage === 'denoising' ? Math.floor((demoProgress.value / 100) * 28) : undefined;

  return [
    {
      id: 'demo-running',
      params: { ...baseParams, prompt: 'Cyberpunk city at night with neon lights reflecting on wet streets' },
      status: 'running',
      progress: demoProgress.value / 100,
      currentStage,
      currentStep,
      totalSteps: 28,
      created_at: Date.now() - 30000,
      started_at: Date.now() - 15000,
    },
    {
      id: 'demo-pending-2',
      params: { ...baseParams, prompt: 'Futuristic spaceship interior with holographic displays', width: 1280, height: 720 },
      status: 'pending',
      progress: 0,
      created_at: Date.now() - 10000,
    },
    {
      id: 'demo-completed-1',
      params: { ...baseParams, prompt: 'Majestic mountain peak at golden hour' },
      status: 'completed',
      progress: 1,
      created_at: Date.now() - 120000,
      started_at: Date.now() - 110000,
      completed_at: Date.now() - 60000,
      result_path: '/demo/path/image1.png',
    },
    {
      id: 'demo-failed-1',
      params: { ...baseParams, prompt: 'Abstract geometric patterns in vibrant colors' },
      status: 'failed',
      progress: 0.3,
      created_at: Date.now() - 180000,
      started_at: Date.now() - 170000,
      error: 'CUDA out of memory',
    },
  ];
});

// Toggle demo mode with Ctrl+Shift+D
const handleKeydown = (e: KeyboardEvent) => {
  if (e.ctrlKey && e.shiftKey && e.key === 'D') {
    e.preventDefault();
    demoMode.value = !demoMode.value;

    if (demoMode.value) {
      // Start animating progress
      demoProgress.value = 0;
      demoInterval = setInterval(() => {
        demoProgress.value = (demoProgress.value + 1) % 100;
      }, 200);
      console.log('Demo mode enabled - press Ctrl+Shift+D to disable');
    } else {
      // Stop animation
      if (demoInterval) {
        clearInterval(demoInterval);
        demoInterval = null;
      }
      console.log('Demo mode disabled');
    }
  }
};

// All jobs (active queue only - pending and running)
const allJobs = computed(() => {
  if (demoMode.value) {
    return demoAllJobs.value;
  }
  // Only show active jobs (pending/running), not completed/failed ones
  return queueStore.jobs.filter((j) => j.status === 'pending' || j.status === 'running');
});

onMounted(async () => {
  // Initialize event listeners for real-time queue updates
  await queueStore.initializeEventListeners();

  // Initial load of jobs
  await queueStore.refreshJobs();

  // Note: No polling needed! Queue store already listens to real-time events:
  // - Tauri events (local/server mode)
  // - WebSocket messages (client mode)

  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  // Cleanup event listeners
  queueStore.cleanupEventListeners();

  window.removeEventListener('keydown', handleKeydown);
  if (demoInterval) {
    clearInterval(demoInterval);
  }
});
</script>

<style scoped>
@reference "tailwindcss";

/* PrimeVue Tabs overrides */
:deep(.p-tabs) {
  @apply flex h-full flex-col;
}

:deep(.p-tabpanels) {
  @apply h-full p-0;
}

:deep(.p-tabpanel) {
  @apply h-full p-0;
}
</style>
