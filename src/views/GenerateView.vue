<template>
  <div class="flex w-full h-full">
    <!-- Left Sidebar -->
    <WorkspaceActions>
      <template #header>Generate Images</template>
      <template #body><GenerateActions @generate="handleGenerate" /></template>
    </WorkspaceActions>

    <!-- Main Content Area -->
    <div class="flex flex-col flex-1 overflow-hidden">
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

    <div class="flex flex-col h-full border-l bg-surface-800 border-surface-700 w-60 min-w-60">
      <HistoryPanel/>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
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
import type { GenerationParams } from '@/stores/queue';
import WorkspaceActions from '@/components/shared/WorkspaceActions.vue';
import HistoryPanel from '@/components/generation/HistoryPanel.vue';

const queueStore = useQueueStore();
const generationStore = useGenerationStore();
const modelsStore = useModelsStore();
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
      const activeLoraConfigs = modelsStore.getActiveLoraConfigs();

      const queueParams: GenerationParams = {
        prompt: params.prompt,
        steps: params.steps,
        cfg_scale: params.cfgScale,
        width: params.width,
        height: params.height,
        seed: seedToUse,
        model: params.model,
        sampler: params.sampler,
        scheduler: params.scheduler,
        // Include active LoRAs if any
        ...(activeLoraConfigs.length > 0 && { loras: activeLoraConfigs }),
      };

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
</style>
