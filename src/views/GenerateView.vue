<template>
  <div
    class="flex w-full"
    @dragenter.prevent="handleDragEnter"
    @dragover.prevent="handleDragOver"
    @dragleave.prevent="handleDragLeave"
    @drop.prevent="handleDrop">
    <!-- Sidebar -->
    <WorkspaceActions>
      <template #header>Generate Images</template>
      <template #toolbar>
        <div class="w-full grid grid-cols-3 border rounded-md shadow-xs border-surface-600">
          <ToggleButton
            :model-value="sectionVisibility.style"
            severity="secondary"
            size="small"
            class="border-0 rounded-none rounded-l-md"
            @change="handleToggleStyle()">
            <div class="content-center">
              <fa :icon="['fal', 'layer-group']" size="sm" />
              Style
            </div>
          </ToggleButton>
          <ToggleButton
            :model-value="sectionVisibility.quality"
            severity="secondary"
            size="small"
            class="border-0 rounded-none"
            @change="handleToggleQuality()">
            <div class="content-center">
              <fa :icon="['fal', 'star']" size="sm" />
              Quality
            </div>
          </ToggleButton>
          <ToggleButton
            :model-value="sectionVisibility.advanced"
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
        <div class="flex flex-col gap-2 py-2">
          <SplitButton
            :loading="queueStore.hasRunningJobs"
            raised
            fluid
            size="small"
            @click="handleGenerate"
            :model="generationCounts"
            :disabled="!canGenerate">
            {{ queueStore.queueLength > 0 ? `Generate` : 'Generate' }} ( {{ imageCount }} )
          </SplitButton>
          <Button severity="help" size="small" fluid @click="showBatchDialog = true"><fa :icon="['fal', 'list']" size="sm" /> Batch Script</Button>
        </div>
      </template>

      <template #body
        ><GenerateActions
          @generate="handleGenerate"
          :showQuality="sectionVisibility.quality"
          :showStyle="sectionVisibility.style"
          :showAdvanced="sectionVisibility.advanced"
      /></template>
    </WorkspaceActions>

    <!-- Chatbot Panel (expands from sidebar, positioned between sidebar and main content) -->
    <Transition name="expand">
      <div v-if="chatStore.isPanelOpen" class="flex flex-col h-full w-100 min-w-80 shrink-0">
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
          <GeneratedResults :images="generatedImages" :pending-count="pendingCount" :pending-images="pendingImages" @download="handleDownload" />
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

    <!-- Drag Overlay -->
    <div
      v-if="isDragging"
      class="absolute top-0 bottom-0 left-0 right-0 flex items-center justify-center border-none pointer-events-none z-100 bg-black/50 backdrop-blur-3xl">
      <div>
        <div class="flex flex-col items-center gap-3 p-6 bg-white rounded-lg shadow-2xl backdrop-blur-3xl">
          <fa :icon="['fal', 'image']" size="3x" class="text-primary-500" />
          <div class="text-center">
            <div class="text-lg font-semibold text-(--text-heading)">Drop Image to Analyze</div>
            <div class="text-sm text-(--text-secondary)">Generate a prompt to recreate this image</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Analysis Loading Overlay -->
    <div
      v-if="isAnalyzing"
      class="absolute top-0 bottom-0 left-0 right-0 flex items-center justify-center border-none pointer-events-none z-100 bg-black/50 backdrop-blur-3xl">
      <div>
        <div class="flex flex-col items-center gap-3 p-6 bg-white rounded-lg shadow-2xl backdrop-blur-3xl">
          <fa :icon="['fal', 'spinner-third']" spin size="3x" class="text-primary-500" />
          <div class="text-center">
            <div class="text-lg font-semibold text-(--text-heading)">Analyzing Image</div>
            <div class="text-sm text-(--text-secondary)">Claude is creating a prompt...</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Batch Script Dialog -->
    <BatchScriptDialog v-model:visible="showBatchDialog" />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from 'vue';
import { storeToRefs } from 'pinia';
import { useToast } from 'primevue/usetoast';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
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
import SplitButton from 'primevue/splitbutton';
import Button from 'primevue/button';
import BatchScriptDialog from '@/components/generation/batch/BatchScriptDialog.vue';
import { analyzeImageForPrompt, fileToDataUrl, isValidImageFile } from '@/services/imageAnalysis';

const queueStore = useQueueStore();
const generationStore = useGenerationStore();
const modelsStore = useModelsStore();
const chatStore = useChatbotStore();
const toast = useToast();

const { sectionVisibility } = storeToRefs(generationStore);

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

// Drag and drop state
const isDragging = ref(false);
const isAnalyzing = ref(false);
const dragCounter = ref(0);

// Batch dialog visibility
const showBatchDialog = ref(false);

const canGenerate = computed(() => {
  const hasPrompt = generationStore.currentParams.prompt.trim().length > 0;
  const hasValidConfig = generationStore.isValidConfiguration;
  return hasPrompt && hasValidConfig;
});

const imageCount = computed({
  get: () => generationStore.currentParams.batchSize,
  set: (value: number) => {
    generationStore.currentParams.batchSize = value;
  },
});

const generationCounts = [
  {
    label: 'Generate 1 Image',
    command: () => {
      imageCount.value = 1;
    },
  },
  {
    label: 'Generate 2 Images',
    command: () => {
      imageCount.value = 2;
    },
  },
  {
    label: 'Generate 3 Images',
    command: () => {
      imageCount.value = 3;
    },
  },
  {
    label: 'Generate 4 Images',
    command: () => {
      imageCount.value = 4;
    },
  },
];

// Get pending images with preview data from running jobs
const pendingImages = computed(() => {
  return queueStore.jobs
    .filter(job => job.status === 'running' && currentBatchJobIds.value.has(job.id))
    .map(job => ({
      id: job.id,
      previewData: job.previewData,
    }));
});

function handleToggleQuality() {
  generationStore.toggleSection('quality');
}

function handleToggleStyle() {
  generationStore.toggleSection('style');
}

function handleToggleAdvanced() {
  generationStore.toggleSection('advanced');
}

const handleDragEnter = (e: DragEvent) => {
  dragCounter.value++;
  if (hasImageContent(e.dataTransfer)) {
    isDragging.value = true;
  }
};

const handleDragOver = (_e: DragEvent) => {
  // Required to allow drop
};

const handleDragLeave = () => {
  dragCounter.value--;
  if (dragCounter.value === 0) {
    isDragging.value = false;
  }
};

const handleDrop = async (e: DragEvent) => {
  dragCounter.value = 0;
  isDragging.value = false;

  const dataTransfer = e.dataTransfer;
  if (!dataTransfer) return;

  const file = dataTransfer.files?.[0];
  if (file && isValidImageFile(file)) {
    await analyzeDroppedImage(file);
    return;
  }

  const uriList = dataTransfer.getData('text/uri-list');
  if (uriList) {
    const uri = uriList
      .split('\n')
      .find((line) => line.trim() && !line.startsWith('#'))
      ?.trim();

    if (uri?.startsWith('file://')) {
      await analyzeLocalFile(uri);
      return;
    } else if (uri && isImageUrl(uri)) {
      await analyzeImageFromUrl(uri);
      return;
    }
  }

  const html = dataTransfer.getData('text/html');
  if (html) {
    const imgUrl = extractImageUrlFromHtml(html);
    if (imgUrl) {
      if (imgUrl.startsWith('file://')) {
        await analyzeLocalFile(imgUrl);
      } else {
        await analyzeImageFromUrl(imgUrl);
      }
      return;
    }
  }

  toast.add({
    severity: 'warn',
    summary: 'Invalid Content',
    detail: 'Please drop an image file or image from a webpage',
    life: 3000,
  });
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

      // Apply style template if a style is selected
      const finalPrompt = generationStore.getFinalPrompt(params.prompt);

      const queueParams: GenerationParams = {
        prompt: finalPrompt,
        steps: params.steps,
        cfg_scale: params.cfgScale,
        width: params.width,
        height: params.height,
        seed: seedToUse,
        bundle_id: params.bundleId,
        model_component_id: params.modelComponentId ?? '',
        clip_component_id: params.clipComponentId ?? '',
        t5_component_id: params.t5ComponentId ?? '',
        vae_component_id: params.vaeComponentId ?? '',
        sampler: params.sampler,
        scheduler: params.scheduler,
        // Include active LoRAs if any
        ...(activeLoraConfigs.length > 0 && { loras: activeLoraConfigs }),
        //loras: [{ id: '098df4c8-384b-426d-a257-20bae1dc9327', strength: 1 }],
      };

      console.log('queueParams:', queueParams);

      const jobId = await queueStore.addToQueue(queueParams);
      currentBatchJobIds.value.add(jobId);
    }

    // Increment style usage counter if a style was applied
    if (generationStore.selectedStyleId) {
      try {
        await invoke('increment_style_usage', { styleId: generationStore.selectedStyleId });
      } catch (error) {
        console.warn('Failed to increment style usage:', error);
      }
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

const analyzeLocalFile = async (fileUri: string) => {
  isAnalyzing.value = true;

  try {
    let filePath = fileUri.replace(/^file:\/\/(localhost)?/, '');
    filePath = decodeURIComponent(filePath);

    if (!isImageUrl(filePath)) {
      throw new Error('File is not a supported image format');
    }

    const fileData = await readFile(filePath);
    const base64 = btoa(fileData.reduce((data: string, byte: number) => data + String.fromCharCode(byte), ''));
    const mimeType = getMimeType(filePath);
    const dataUrl = `data:${mimeType};base64,${base64}`;

    const prompt = await analyzeImageForPrompt(dataUrl);
    generationStore.currentParams.prompt = prompt;

    toast.add({
      severity: 'success',
      summary: 'Image Analyzed',
      detail: 'Prompt generated from image',
      life: 3000,
    });
  } catch (error) {
    console.error('Failed to analyze local file:', error);
    toast.add({
      severity: 'error',
      summary: 'Analysis Failed',
      detail: error instanceof Error ? error.message : 'Failed to analyze image',
      life: 5000,
    });
  } finally {
    isAnalyzing.value = false;
  }
};

const extractImageUrlFromHtml = (html: string): string | null => {
  const imgMatch = html.match(/<img[^>]+src=["']([^"']+)["']/i);
  if (imgMatch) return imgMatch[1];

  const anchorMatch = html.match(/<a[^>]*>(file:\/\/[^<]+)<\/a>/i);
  if (anchorMatch && isImageUrl(anchorMatch[1])) return anchorMatch[1];

  const hrefMatch = html.match(/<a[^>]+href=["'](file:\/\/[^"']+)["']/i);
  if (hrefMatch && isImageUrl(hrefMatch[1])) return hrefMatch[1];

  return null;
};

const analyzeImageFromUrl = async (url: string) => {
  isAnalyzing.value = true;

  try {
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`Failed to fetch image: ${response.status}`);
    }

    const blob = await response.blob();

    if (!blob.type.startsWith('image/')) {
      throw new Error('URL did not return an image');
    }

    const dataUrl = await new Promise<string>((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(reader.result as string);
      reader.onerror = () => reject(new Error('Failed to read image'));
      reader.readAsDataURL(blob);
    });

    const prompt = await analyzeImageForPrompt(dataUrl);
    generationStore.currentParams.prompt = prompt;

    toast.add({
      severity: 'success',
      summary: 'Image Analyzed',
      detail: 'Prompt generated from image',
      life: 3000,
    });
  } catch (error) {
    console.error('Failed to analyze image from URL:', error);
    toast.add({
      severity: 'error',
      summary: 'Analysis Failed',
      detail: error instanceof Error ? error.message : 'Failed to analyze image',
      life: 5000,
    });
  } finally {
    isAnalyzing.value = false;
  }
};

const analyzeDroppedImage = async (file: File) => {
  isAnalyzing.value = true;

  try {
    const dataUrl = await fileToDataUrl(file);
    const prompt = await analyzeImageForPrompt(dataUrl);
    generationStore.currentParams.prompt = prompt;

    toast.add({
      severity: 'success',
      summary: 'Image Analyzed',
      detail: 'Prompt generated from image',
      life: 3000,
    });
  } catch (error) {
    console.error('Failed to analyze image:', error);
    toast.add({
      severity: 'error',
      summary: 'Analysis Failed',
      detail: error instanceof Error ? error.message : 'Failed to analyze image',
      life: 5000,
    });
  } finally {
    isAnalyzing.value = false;
  }
};

// Drag and drop handlers
const hasImageContent = (dataTransfer: DataTransfer | null): boolean => {
  if (!dataTransfer) return false;
  const types = dataTransfer.types;
  return types.includes('Files') || types.includes('text/uri-list') || types.includes('text/html');
};

const isImageUrl = (url: string): boolean => {
  const imageExtensions = ['.png', '.jpg', '.jpeg', '.gif', '.webp', '.bmp'];
  const lowerUrl = url.toLowerCase();
  return imageExtensions.some((ext) => lowerUrl.includes(ext)) || lowerUrl.includes('image');
};

const getMimeType = (filePath: string): string => {
  const ext = filePath.toLowerCase().split('.').pop();
  const mimeTypes: Record<string, string> = {
    png: 'image/png',
    jpg: 'image/jpeg',
    jpeg: 'image/jpeg',
    gif: 'image/gif',
    webp: 'image/webp',
    bmp: 'image/bmp',
  };
  return mimeTypes[ext || ''] || 'image/png';
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
