<template>
  <div class="flex w-full">
    <!-- Sidebar -->
    <WorkspaceActions>
      <template #header>Generate Images</template>
      <template #toolbar>
        <div class="w-full grid grid-cols-3 border rounded-md shadow-xs border-surface-600">
          <ToggleButton :model-value="showQuality" severity="secondary" size="small" class="border-0 rounded-none rounded-l-md" @change="handleToggleQuality()">
            <div class="content-center">
              <fa :icon="['fal', 'star']" size="sm" />
              Quality
            </div>
          </ToggleButton>
          <ToggleButton :model-value="showStyle" severity="secondary" size="small" class="border-0 rounded-none" @change="handleToggleStyle()">
            <div class="content-center">
              <fa :icon="['fal', 'layer-group']" size="sm" />
              Style
            </div>
          </ToggleButton>
          <ToggleButton
            :model-value="showAdvanced"
            severity="secondary"
            size="small"
            class="border-0 rounded-none rounded-r-md"
            @change="handleToggleAdvanced()">
            <div class="content-center">
              <fa :icon="['fal', 'gear']" size="sm" />
              Advanced
            </div>
          </ToggleButton>
        </div>
      </template>

      <template #body><GenerateActions @generate="handleGenerate" :showQuality="showQuality" :showStyle="showStyle" :showAdvanced="showAdvanced" /></template>
    </WorkspaceActions>

    <!-- Chatbot Panel (expands from sidebar, positioned between sidebar and main content) -->
    <Transition name="expand">
      <div v-if="chatStore.isPanelOpen" class="flex flex-col h-full w-80 min-w-80 shrink-0">
        <div class="h-full px-1 py-2 chat-panel-fade-in">
          <ChatPanel />
        </div>
      </div>
    </Transition>

    <!-- Main Content Area -->
    <div class="flex flex-1 p-2 transition-all duration-300 grow">
      <div class="flex flex-col flex-1 gap-2 p-2 overflow-hidden border rounded-lg border-surface-700 bg-surface-950">
        <!-- Canvas Section -->
        <div class="flex flex-1 overflow-hidden">
          <!-- Generated Results -->
          <GeneratedResults :images="generatedImages" :pending-count="pendingCount" @download="handleDownload" />
        </div>

        <!-- Bottom Panel -->
        <div class="shrink-0">
          <QueuePanel />
        </div>
      </div>
    </div>

    <!-- Right Panel - History -->
    <div class="flex flex-col h-full w-60 min-w-60 shrink">
      <HistoryPanel @restore-image="handleRestoreImage" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue';
import { useToast } from 'primevue/usetoast';
import { convertFileSrc } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { writeFile, readFile } from '@tauri-apps/plugin-fs';
import GenerateActions from '@/components/generation/GenerateActions.vue';
import QueuePanel from '@/components/generation/QueuePanel.vue';
import GeneratedResults from '@/components/generation/GeneratedResults.vue';
import { useQueueStore } from '@/stores/queue';
import { useGenerationStore } from '@/stores/generation';
import { useModelsStore } from '@/stores/models';
import { useChatbotStore } from '@/stores/chatbot';
import type { GenerationParams } from '@/stores/queue';
import WorkspaceActions from '@/components/shared/WorkspaceActions.vue';
import HistoryPanel from '@/components/generation/HistoryPanel.vue';
import ChatPanel from '@/components/generation/ChatPanel.vue';
import ToggleButton from 'primevue/togglebutton';

const queueStore = useQueueStore();
const generationStore = useGenerationStore();
const modelsStore = useModelsStore();
const chatStore = useChatbotStore();
const toast = useToast();

// Generated images array
interface GeneratedImage {
  id: string;
  src: string;
}

const generatedImages = ref<GeneratedImage[]>([]);

// Track displayed job IDs
const displayedJobIds = ref<Set<string>>(new Set());

// Track job IDs in the current generation batch
const currentBatchJobIds = ref<Set<string>>(new Set());

// Track pending image count for skeleton placeholders
const pendingCount = ref(0);

// Track pending image count for skeleton placeholders
const showQuality = ref(true);
const showStyle = ref(false);
const showAdvanced = ref(false);

const handleToggleQuality = () => {
  showQuality.value = !showQuality.value;
  console.log('showQuality: ', showQuality.value);
};

const handleToggleStyle = () => {
  showStyle.value = !showStyle.value;
  console.log('showStyle: ', showStyle.value);
};

const handleToggleAdvanced = () => {
  showAdvanced.value = !showAdvanced.value;
  console.log('showAdvanced: ', showAdvanced.value);
};

// Watch for completed jobs - only show jobs from current batch
watch(
  () => queueStore.jobs,
  (jobs) => {
    const newlyCompleted = jobs.filter(
      (j) => j.status === 'completed' && j.result_path && currentBatchJobIds.value.has(j.id) && !displayedJobIds.value.has(j.id),
    );

    if (newlyCompleted.length > 0) {
      newlyCompleted.forEach((job) => {
        if (job.result_path) {
          const imageSrc = convertFileSrc(job.result_path);
          generatedImages.value.push({
            id: job.id,
            src: imageSrc,
          });
          displayedJobIds.value.add(job.id);
          // Decrement pending count as images complete
          if (pendingCount.value > 0) {
            pendingCount.value--;
          }
        }
      });
    }
  },
  { deep: true },
);

const handleGenerate = async () => {
  const params = generationStore.currentParams;
  const batchSize = params.batchSize || 1;

  // Move completed jobs to history before starting new generation
  queueStore.moveCompletedToHistory();

  // Clear state for new generation batch
  generatedImages.value = [];
  displayedJobIds.value.clear();
  currentBatchJobIds.value.clear();
  pendingCount.value = batchSize;

  // Determine the base seed to use
  let baseSeed: number;
  if (generationStore.randomizeSeedOnGenerate) {
    // Generate a new random seed for this generation
    baseSeed = Math.floor(Math.random() * Number.MAX_SAFE_INTEGER);
    // Update the store so user can see/copy the seed that was used
    generationStore.currentParams.seed = baseSeed;
  } else {
    // Use the locked seed value
    baseSeed = params.seed >= 0 ? params.seed : Math.floor(Math.random() * Number.MAX_SAFE_INTEGER);
  }

  try {
    // Generate multiple images based on batchSize
    for (let i = 0; i < batchSize; i++) {
      // For each image in the batch, use a different seed
      let seedToUse: number;
      if (generationStore.randomizeSeedOnGenerate) {
        // Each image gets a unique random seed
        seedToUse = i === 0 ? baseSeed : Math.floor(Math.random() * Number.MAX_SAFE_INTEGER);
      } else {
        // Increment seed for reproducibility (baseSeed, baseSeed+1, baseSeed+2, etc.)
        seedToUse = baseSeed + i;
      }

      // Get active LoRA configs for this generation
      const activeLoraConfigs = modelsStore.loras.map((l) => ({
        id: l.id,
        strength: l.strength,
      }));
      console.log('activeLoraConfigs:', activeLoraConfigs);

      const queueParams: GenerationParams = {
        prompt: params.prompt,
        steps: params.steps,
        cfg_scale: params.cfgScale,
        width: params.width,
        height: params.height,
        seed: seedToUse,
        bundle_id: params.bundleId,
        model_component_id: params.modelComponentId,
        clip_component_id: params.clipComponentId,
        t5_component_id: params.t5ComponentId,
        vae_component_id: params.vaeComponentId,
        sampler: params.sampler,
        scheduler: params.scheduler,
        // Include active LoRAs if any
        ...(activeLoraConfigs.length > 0 && { loras: activeLoraConfigs }),
        //loras: [{ id: '098df4c8-384b-426d-a257-20bae1dc9327', strength: 1 }],
      };

      console.log(queueParams);

      const jobId = await queueStore.addToQueue(queueParams);
      currentBatchJobIds.value.add(jobId);
    }
  } catch (error) {
    console.error('Failed to add to queue:', error);
    toast.add({
      severity: 'error',
      summary: 'Generation Failed',
      detail: queueStore.error || 'Failed to add generation to queue',
      life: 5000,
    });
  }
};

const handleDownload = async (imageSrc: string, slotNumber: number) => {
  try {
    // Extract the original file path from the convertFileSrc URL
    // convertFileSrc transforms paths like: file:///path/to/image.png -> http://asset.localhost/path/to/image.png
    const originalPath = imageSrc.replace('http://asset.localhost/', '').replace('https://asset.localhost/', '');

    // Show save dialog
    const savePath = await save({
      defaultPath: `generated-image-${slotNumber}-${Date.now()}.png`,
      filters: [
        {
          name: 'PNG Image',
          extensions: ['png'],
        },
        {
          name: 'All Files',
          extensions: ['*'],
        },
      ],
    });

    // User cancelled the dialog
    if (!savePath) {
      return;
    }

    // Read the original file and write to the new location
    const imageData = await readFile(originalPath);
    await writeFile(savePath, imageData);

    toast.add({
      severity: 'success',
      summary: 'Image Saved',
      detail: 'Image has been saved successfully',
      life: 3000,
    });
  } catch (error) {
    console.error('Failed to save image:', error);
    toast.add({
      severity: 'error',
      summary: 'Save Failed',
      detail: 'Failed to save the image',
      life: 5000,
    });
  }
};

const handleRestoreImage = (imagePath: string) => {
  // Clear existing images and display the history item's image
  generatedImages.value = [
    {
      id: `history-${Date.now()}`,
      src: convertFileSrc(imagePath),
    },
  ];

  // Reset pending count
  pendingCount.value = 0;

  // Clear current batch tracking
  currentBatchJobIds.value.clear();
  displayedJobIds.value.clear();
};

// Initialize automatic localStorage persistence
onMounted(() => {
  generationStore.initializePersistence();
  modelsStore.loadModels();
});

onUnmounted(() => {
  generationStore.cleanupPersistence();
});
</script>

<style scoped>
@reference "tailwindcss";

/* PrimeVue deep selectors for Image component */
:deep(.p-image) {
  @apply flex h-full w-full cursor-pointer items-center justify-center;
}

.slot-image {
  @apply object-fill;
}

:deep(.p-image-preview-container) {
  background: rgba(0, 0, 0, 0.95);
}

:deep(.p-image-toolbar) {
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(10px);
}

:deep(.p-togglebutton-content:hover) {
  @apply bg-blue-500 text-white;
}

/* Expand animation for chatbot panel (width-based) */
.expand-enter-active {
  transition: all 0.3s ease-out;
  overflow: hidden;
}

.expand-leave-active {
  transition: all 0.25s ease-in;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  width: 0;
  min-width: 0;
  opacity: 0;
}

/* Delayed fade-in for ChatPanel content */
.expand-enter-active .chat-panel-fade-in {
  animation: fade-in 0.2s ease-out 0.2s both;
}

/* Immediate fade-out for ChatPanel content when closing */
.expand-leave-active .chat-panel-fade-in {
  animation: fade-out 0.15s ease-in both;
}

@keyframes fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes fade-out {
  from {
    opacity: 1;
  }
  to {
    opacity: 0;
  }
}
</style>
