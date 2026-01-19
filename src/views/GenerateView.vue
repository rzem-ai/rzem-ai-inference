<template>
  <div class="generate-view">
    <!-- Left Sidebar -->
    <aside class="sidebar">
      <LeftSidebar />
    </aside>

    <!-- Main Content Area -->
    <div class="main-area">
      <!-- Canvas Section with Generate Button -->
      <div class="canvas-section">
        <!-- Generate Button -->
        <div class="generate-bar">
          <Button
            class="generate-button"
            :label="queueStore.queueLength > 0 ? `Generate (${queueStore.queueLength})` : 'Generate'"
            icon="pi pi-sparkles"
            :loading="queueStore.hasRunningJobs"
            @click="handleGenerate" />
        </div>

        <!-- Generated Results -->
        <div class="results-area">
          <div class="results-header">
            <span class="results-label">GENERATED RESULTS</span>
            <span class="results-status">{{ slotsStatus }}</span>
          </div>

          <div class="image-slots">
            <!-- Slot 1 -->
            <div class="image-slot" :class="{ 'has-image': slot1Image }">
              <div v-if="!slot1Image" class="slot-empty">
                <i class="pi pi-image"></i>
                <span class="slot-label">SLOT 1</span>
                <span class="slot-status">Ready</span>
              </div>
              <div v-else class="image-container">
                <Image :src="slot1Image" alt="Generated image" preview image-class="slot-image" />
                <Button
                  icon="pi pi-download"
                  class="download-button"
                  rounded
                  severity="secondary"
                  @click.stop="handleDownload(slot1Image, 1)" />
              </div>
            </div>

            <!-- Slot 2 -->
            <div class="image-slot" :class="{ 'has-image': slot2Image }">
              <div v-if="!slot2Image" class="slot-empty">
                <i class="pi pi-image"></i>
                <span class="slot-label">SLOT 2</span>
                <span class="slot-status">Ready</span>
              </div>
              <div v-else class="image-container">
                <Image :src="slot2Image" alt="Generated image" preview image-class="slot-image" />
                <Button
                  icon="pi pi-download"
                  class="download-button"
                  rounded
                  severity="secondary"
                  @click.stop="handleDownload(slot2Image, 2)" />
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Bottom Panel -->
      <div class="bottom-section">
        <BottomPanel />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useToast } from 'primevue/usetoast';
import { convertFileSrc } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { writeFile, readFile } from '@tauri-apps/plugin-fs';
import Button from 'primevue/button';
import Image from 'primevue/image';
import LeftSidebar from '@/components/generation/LeftSidebar.vue';
import BottomPanel from '@/components/generation/BottomPanel.vue';
import { useQueueStore } from '@/stores/queue';
import { useGenerationStore } from '@/stores/generation';
import type { GenerationParams } from '@/stores/queue';

const queueStore = useQueueStore();
const generationStore = useGenerationStore();
const toast = useToast();

// Image slots
const slot1Image = ref<string | null>(null);
const slot2Image = ref<string | null>(null);
const currentSlot = ref(1);

// Track displayed job IDs
const displayedJobIds = ref<Set<string>>(new Set());

const slotsStatus = computed(() => {
  const ready = [!slot1Image.value, !slot2Image.value].filter(Boolean).length;
  return `${ready} slots ready`;
});

// Watch for completed jobs
watch(
  () => queueStore.jobs,
  (jobs) => {
    const newlyCompleted = jobs.filter((j) => j.status === 'completed' && j.result_path && !displayedJobIds.value.has(j.id));

    if (newlyCompleted.length > 0) {
      const latestJob = newlyCompleted[newlyCompleted.length - 1];
      if (latestJob.result_path) {
        const imageSrc = convertFileSrc(latestJob.result_path);

        // Alternate between slots
        if (currentSlot.value === 1) {
          slot1Image.value = imageSrc;
          currentSlot.value = 2;
        } else {
          slot2Image.value = imageSrc;
          currentSlot.value = 1;
        }

        newlyCompleted.forEach((j) => displayedJobIds.value.add(j.id));
      }
    }
  },
  { deep: true },
);

const handleGenerate = async () => {
  const params = generationStore.currentParams;

  // Determine the seed to use
  let seedToUse: number;
  if (generationStore.randomizeSeedOnGenerate) {
    // Generate a new random seed for this generation
    seedToUse = Math.floor(Math.random() * 2147483647);
    // Update the store so user can see/copy the seed that was used
    generationStore.currentParams.seed = seedToUse;
  } else {
    // Use the locked seed value
    seedToUse = params.seed >= 0 ? params.seed : Math.floor(Math.random() * 2147483647);
  }

  const queueParams: GenerationParams = {
    prompt: params.prompt,
    negative_prompt: params.negativePrompt || undefined,
    steps: params.steps,
    cfg_scale: params.cfgScale,
    width: params.width,
    height: params.height,
    seed: seedToUse,
    model: params.model,
  };

  try {
    await queueStore.addToQueue(queueParams);
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

.generate-view {
  @apply flex h-full;
  background: #151a21;
}

.sidebar {
  @apply flex-shrink-0;
  width: 320px;
}

.main-area {
  @apply flex-1 flex flex-col overflow-hidden;
}

.canvas-section {
  @apply flex-1 flex flex-col p-4 overflow-hidden;
}

.generate-bar {
  @apply mb-4;
}

.generate-button {
  @apply w-full py-3 text-lg font-semibold;
  background: #3b82f6 !important;
  border-color: #3b82f6 !important;

  &:hover {
    background: #2563eb !important;
    border-color: #2563eb !important;
  }
}

.results-area {
  @apply flex-1 flex flex-col overflow-hidden;
}

.results-header {
  @apply flex items-center justify-between mb-3;
}

.results-label {
  @apply text-xs font-semibold tracking-wider;
  color: #64748b;
}

.results-status {
  @apply text-xs;
  color: #38bdf8;
}

.image-slots {
  @apply flex-1 grid grid-cols-2 gap-4;
  min-height: 0; /* Allow grid to shrink within flex container */
}

.image-slot {
  @apply flex items-center justify-center rounded-xl overflow-hidden;
  background: #1e2530;
  border: 2px dashed #2d3748;
  min-height: 0; /* Prevent grid items from expanding beyond container */

  &.has-image {
    border-style: solid;
    border-color: #3b82f6;
  }
}

.slot-empty {
  @apply flex flex-col items-center gap-2;
  color: #475569;

  i {
    @apply text-4xl;
  }
}

.slot-label {
  @apply text-sm font-semibold;
}

.slot-status {
  @apply text-xs;
  color: #64748b;
}

/* Image container with download button */
.image-container {
  @apply relative w-full h-full;
}

/* Download button - hidden by default, shown on hover */
.download-button {
  @apply absolute;
  top: 12px;
  right: 12px;
  z-index: 10;
  opacity: 0;
  transition: opacity 0.2s ease-in-out;
  background: rgba(59, 130, 246, 0.9) !important;
  border-color: rgba(59, 130, 246, 0.9) !important;
  backdrop-filter: blur(8px);

  &:hover {
    background: rgba(37, 99, 235, 0.95) !important;
    border-color: rgba(37, 99, 235, 0.95) !important;
  }
}

/* Show download button on image container hover */
.image-container:hover .download-button {
  opacity: 1;
}

/* Image component wrapper - takes full slot size */
:deep(.p-image) {
  @apply w-full h-full flex items-center justify-center cursor-pointer;
}

/* Scaled image inside slot */
.slot-image {
  @apply w-full h-full object-contain;
}

/* Preview toolbar styling */
:deep(.p-image-preview-container) {
  background: rgba(0, 0, 0, 0.95);
}

:deep(.p-image-toolbar) {
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(10px);
}

.bottom-section {
  @apply flex-shrink-0;
  height: 200px;
  background: #1e2530;
  border-top: 1px solid #2d3748;
}
</style>
