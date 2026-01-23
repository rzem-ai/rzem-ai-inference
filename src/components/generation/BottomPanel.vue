<template>
  <div class="relative h-full overflow-hidden bg-gray-800">
    <!-- Demo mode indicator -->
    <div v-if="demoMode" class="absolute top-1 right-2 z-10 flex items-center gap-1 px-2 py-0.5 bg-purple-600 text-white text-xs rounded-full">
      <span class="animate-pulse">●</span> Demo Mode (Ctrl+Shift+D to exit)
    </div>

    <Tabs v-model:value="activeTab">
      <TabList>
        <Tab value="queue">
          <div class="flex flex-row items-center gap-2">
            <List class="w-5 h-5" /> Queue <Badge v-if="pendingJobCount > 0" :value="pendingJobCount" severity="info" class="ml-2" />
          </div>
        </Tab>
        <Tab value="presets">
          <div class="flex flex-row items-center gap-2"> <BookMarked class="w-5 h-5" /> Presets </div>
        </Tab>
      </TabList>

      <TabPanels>
        <TabPanel value="queue">
          <div class="h-full p-3 overflow-auto">
            <div v-if="allJobs.length === 0" class="flex h-full flex-col items-center justify-center gap-2 text-(--text-secondary)">
              <Inbox class="w-4 h-4" />
              <p class="text-sm">No jobs in queue</p>
            </div>
            <div v-else class="flex flex-wrap gap-2">
              <QueueJobCard v-for="job in allJobs" :key="job.id" :job="job" />
            </div>
          </div>
        </TabPanel>

        <TabPanel value="presets">
          <div class="h-full p-3 overflow-auto">
            <div class="flex h-full flex-col items-center justify-center gap-2 text-(--text-secondary)">
              <i class="text-3xl pi pi-bookmark"></i>
              <p class="text-sm">Saved presets will appear here</p>
            </div>
          </div>
        </TabPanel>
      </TabPanels>
    </Tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import Tabs from 'primevue/tabs';
import TabList from 'primevue/tablist';
import Tab from 'primevue/tab';
import TabPanels from 'primevue/tabpanels';
import TabPanel from 'primevue/tabpanel';
import Badge from 'primevue/badge';
import { useQueueStore, type GenerationJob, type PipelineStage } from '@/stores/queue';
import { BookMarked, Inbox, List } from 'lucide-vue-next';
import QueueJobCard from './QueueJobCard.vue';

const queueStore = useQueueStore();
const activeTab = ref('queue');

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
    model: 'flux-dev',
    sampler: 'euler' as const,
    scheduler: 'normal' as const,
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
      id: 'demo-pending-1',
      params: { ...baseParams, prompt: 'Ancient forest with mystical fog and rays of sunlight' },
      status: 'pending',
      progress: 0,
      created_at: Date.now() - 20000,
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

// All jobs (active queue + history)
const allJobs = computed(() => {
  if (demoMode.value) {
    return demoAllJobs.value;
  }
  // Combine current jobs and history, with active jobs first
  return [...queueStore.jobs, ...queueStore.historyJobs];
});

// Count of pending/running jobs for badge
const pendingJobCount = computed(() => {
  if (demoMode.value) {
    return demoAllJobs.value.filter((j) => j.status === 'pending' || j.status === 'running').length;
  }
  return queueStore.jobs.filter((j) => j.status === 'pending' || j.status === 'running').length;
});

onMounted(() => {
  // Initial load of jobs
  queueStore.refreshJobs();

  // Note: No polling needed! Queue store already listens to real-time events:
  // - Tauri events (local/server mode)
  // - WebSocket messages (client mode)

  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
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
