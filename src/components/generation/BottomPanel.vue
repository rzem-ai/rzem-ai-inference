<template>
  <div class="bottom-panel">
    <Tabs v-model:value="activeTab">
      <TabList>
        <Tab value="queue">
          <i class="pi pi-list mr-2"></i>
          Queue
          <Badge v-if="queueStore.queueLength > 0" :value="queueStore.queueLength" severity="info" class="ml-2" />
        </Tab>
        <Tab value="history">
          <i class="pi pi-history mr-2"></i>
          History
        </Tab>
        <Tab value="presets">
          <i class="pi pi-bookmark mr-2"></i>
          Presets
        </Tab>
      </TabList>

      <TabPanels>
        <TabPanel value="queue">
          <div class="panel-content">
            <div v-if="queueStore.jobs.length === 0" class="empty-state">
              <i class="pi pi-inbox"></i>
              <p>No jobs in queue</p>
            </div>
            <div v-else class="job-grid">
              <div
                v-for="job in queueStore.jobs"
                :key="job.id"
                class="job-card"
                :class="`status-${job.status}`">
                <div class="job-icon">
                  <i :class="`pi ${getStatusIcon(job.status)}`"></i>
                </div>
                <div class="job-info">
                  <div class="job-prompt">{{ truncatePrompt(job.params.prompt) }}</div>
                  <div class="job-meta">
                    <span>{{ job.params.width }}x{{ job.params.height }}</span>
                    <span>{{ job.params.steps }} steps</span>
                  </div>
                </div>
                <div v-if="job.status === 'running'" class="job-progress">
                  <!-- Pipeline progress (top bar) -->
                  <div class="progress-row">
                    <span class="progress-label">Pipeline</span>
                    <ProgressBar :value="job.progress * 100" :showValue="false" class="progress-bar-pipeline" />
                    <span class="progress-value">{{ Math.round(job.progress * 100) }}%</span>
                  </div>
                  <div class="progress-stage">{{ getStageDisplayName(job.currentStage) }}</div>
                  <!-- Steps progress (bottom bar) -->
                  <div class="progress-row">
                    <span class="progress-label">Steps</span>
                    <ProgressBar :value="getStepProgress(job)" :showValue="false" class="progress-bar-steps" />
                    <span class="progress-value">{{ getStepDisplay(job) }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </TabPanel>

        <TabPanel value="history">
          <div class="panel-content">
            <div class="empty-state">
              <i class="pi pi-history"></i>
              <p>Generation history will appear here</p>
            </div>
          </div>
        </TabPanel>

        <TabPanel value="presets">
          <div class="panel-content">
            <div class="empty-state">
              <i class="pi pi-bookmark"></i>
              <p>Saved presets will appear here</p>
            </div>
          </div>
        </TabPanel>
      </TabPanels>
    </Tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import Tabs from 'primevue/tabs';
import TabList from 'primevue/tablist';
import Tab from 'primevue/tab';
import TabPanels from 'primevue/tabpanels';
import TabPanel from 'primevue/tabpanel';
import Badge from 'primevue/badge';
import ProgressBar from 'primevue/progressbar';
import { useQueueStore, type GenerationJob } from '@/stores/queue';

const queueStore = useQueueStore();
const activeTab = ref('queue');

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

function truncatePrompt(prompt: string): string {
  return prompt.length > 60 ? prompt.substring(0, 60) + '...' : prompt;
}

function getStageDisplayName(stage?: string): string {
  if (!stage) return 'Starting...';
  const stageNames: Record<string, string> = {
    loading_models: 'Loading models...',
    encoding_t5: 'Encoding prompt (T5)...',
    encoding_clip: 'Encoding prompt (CLIP)...',
    denoising: 'Denoising...',
    decoding_vae: 'Decoding image...',
    encoding_png: 'Saving image...',
  };
  return stageNames[stage] || stage;
}

function getStepProgress(job: GenerationJob): number {
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
}

function getStepDisplay(job: GenerationJob): string {
  const totalSteps = job.params.steps;
  const stage = job.currentStage;

  if (stage === 'decoding_vae' || stage === 'encoding_png') {
    return `${totalSteps}/${totalSteps}`;
  }
  const current = job.currentStep ?? 0;
  return `${current}/${totalSteps}`;
}
</script>

<style scoped>
@reference "tailwindcss";

.bottom-panel {
  @apply h-full overflow-hidden;
  background: #1e2530;
}

.bottom-panel :deep(.p-tabs) {
  @apply h-full flex flex-col;
  background: transparent;
}

.bottom-panel :deep(.p-tablist) {
  @apply border-b px-4;
  border-color: #2d3748;
  background: #1a1f28;
}

.bottom-panel :deep(.p-tab) {
  color: #64748b;
  background: transparent;

  &:hover {
    color: #94a3b8;
  }

  &[data-p-active="true"] {
    color: #f1f5f9;
    border-color: #3b82f6;
  }
}

.bottom-panel :deep(.p-tabpanels) {
  @apply flex-1 overflow-hidden;
  background: transparent;
}

.bottom-panel :deep(.p-tabpanel) {
  @apply h-full p-0;
  background: transparent;
}

.panel-content {
  @apply h-full overflow-auto p-3;
}

.empty-state {
  @apply flex flex-col items-center justify-center h-full gap-2;
  color: #475569;

  i {
    @apply text-3xl;
  }

  p {
    @apply text-sm;
  }
}

.job-grid {
  @apply flex gap-3 flex-wrap;
}

.job-card {
  @apply flex items-start gap-3 p-3 rounded-lg min-w-[250px] max-w-[300px];
  background: #2d3748;
  border: 1px solid #374151;

  &.status-running {
    border-color: #3b82f6;
    background: rgba(59, 130, 246, 0.1);
  }

  &.status-completed {
    @apply opacity-70;
  }

  &.status-failed {
    border-color: #ef4444;
  }
}

.job-icon {
  @apply flex items-center justify-center w-8 h-8 rounded-full;
  background: #374151;
  color: #64748b;

  .status-running & {
    background: #3b82f6;
    color: white;
  }

  .status-completed & {
    background: #22c55e;
    color: white;
  }

  .status-failed & {
    background: #ef4444;
    color: white;
  }
}

.job-info {
  @apply flex-1 min-w-0;
}

.job-prompt {
  @apply text-sm font-medium truncate;
  color: #e2e8f0;
}

.job-meta {
  @apply flex gap-2 text-xs mt-1;
  color: #64748b;
}

.job-progress {
  @apply w-full mt-2 flex flex-col gap-1;
}

.progress-row {
  @apply flex items-center gap-2;
}

.progress-label {
  @apply text-xs;
  width: 50px;
  flex-shrink: 0;
  color: #94a3b8;
}

.progress-value {
  @apply text-xs font-mono;
  width: 40px;
  text-align: right;
  flex-shrink: 0;
  color: #e2e8f0;
}

.progress-stage {
  @apply text-xs text-center;
  color: #2dd4bf;
  margin: 2px 0;
}

.progress-bar-pipeline,
.progress-bar-steps {
  @apply flex-1;
}

:deep(.progress-bar-pipeline .p-progressbar) {
  background: #334155;
  height: 6px;
  border-radius: 9999px;
}

:deep(.progress-bar-pipeline .p-progressbar-value) {
  background: linear-gradient(90deg, #14b8a6, #2dd4bf);
  border-radius: 9999px;
}

:deep(.progress-bar-steps .p-progressbar) {
  background: #334155;
  height: 5px;
  border-radius: 9999px;
}

:deep(.progress-bar-steps .p-progressbar-value) {
  background: linear-gradient(90deg, #0ea5e9, #38bdf8);
  border-radius: 9999px;
}
</style>
