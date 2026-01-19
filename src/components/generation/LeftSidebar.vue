<template>
  <div
    class="left-sidebar" 
    @dragenter.prevent="handleDragEnter"
    @dragover.prevent="handleDragOver"
    @dragleave.prevent="handleDragLeave"
    @drop.prevent="handleDrop">
    <!-- Header -->
    <div class="sidebar-header">
      <div>
        <h2 class="sidebar-title">Generate</h2>
        <p class="sidebar-subtitle">Text-to-image generation</p>
      </div>
      <Button icon="pi pi-cog" text rounded />
    </div>

    <!-- Scrollable Content -->
    <div class="sidebar-content">
      <!-- Prompt Section -->
      <div class="section">
        <div class="section-header">
          <i class="pi pi-file-edit"></i>
          <span>PROMPT</span>
        </div>
        <div class="section-content">
          <PromptInput />
        </div>
      </div>

      <!-- Image Settings Section -->
      <div class="section">
        <div class="section-header">
          <i class="pi pi-sliders-h"></i>
          <span>IMAGE SETTINGS</span>
        </div>
        <div class="section-content">
          <!-- Aspect Ratio Pills -->
          <div class="field">
            <label class="field-label">ASPECT RATIO</label>
            <div class="aspect-pills">
              <button
                v-for="ratio in aspectRatios"
                :key="ratio.label"
                class="aspect-pill"
                :class="{ active: isActiveRatio(ratio) }"
                @click="setAspectRatio(ratio)">
                {{ ratio.label }}
              </button>
            </div>
          </div>

          <!-- Width/Height Sliders -->
          <div class="field-row">
            <div class="field">
              <div class="slider-header">
                <label class="field-label">WIDTH</label>
                <span class="slider-value">{{ width }}px</span>
              </div>
              <Slider v-model="width" :min="256" :max="2048" :step="64" />
            </div>
            <div class="field">
              <div class="slider-header">
                <label class="field-label">HEIGHT</label>
                <span class="slider-value">{{ height }}px</span>
              </div>
              <Slider v-model="height" :min="256" :max="2048" :step="64" />
            </div>
          </div>
        </div>
      </div>

      <!-- Generation Settings Section -->
      <div class="section">
        <div class="section-header">
          <i class="pi pi-cog"></i>
          <span>GENERATION SETTINGS</span>
        </div>
        <div class="section-content">
          <div class="field">
            <div class="slider-header">
              <label class="field-label">Steps</label>
              <span class="slider-value">{{ steps }}</span>
            </div>
            <Slider v-model="steps" :min="1" :max="50" />
          </div>

          <div class="field">
            <div class="slider-header">
              <label class="field-label">CFG Scale</label>
              <span class="slider-value">{{ cfgScale.toFixed(1) }}</span>
            </div>
            <Slider v-model="cfgScale" :min="0" :max="20" :step="0.1" />
          </div>

          <div class="field-row">
            <div class="field">
              <label class="field-label">Sampler</label>
              <Select v-model="sampler" :options="samplerOptions" optionLabel="label" optionValue="value" class="w-full" />
            </div>
            <div class="field">
              <label class="field-label">Scheduler</label>
              <Select v-model="scheduler" :options="schedulerOptions" optionLabel="label" optionValue="value" class="w-full" />
            </div>
          </div>

          <div class="field">
            <div class="seed-header">
              <label class="field-label">Seed</label>
              <div class="seed-mode" @click="toggleSeedLock">
                <i :class="seedLocked ? 'pi pi-lock' : 'pi pi-unlock'"></i>
                <span>{{ seedLocked ? 'Locked' : 'Random' }}</span>
              </div>
            </div>
            <div class="seed-control">
              <InputNumber
                v-model="seed"
                :min="0"
                :max="2147483647"
                :disabled="!seedLocked"
                :placeholder="seedLocked ? '' : 'Random'"
                class="flex-1" />
              <Button
                icon="pi pi-sync"
                @click="randomizeSeed"
                text
                :disabled="!seedLocked"
                v-tooltip.top="'Generate new random seed'" />
            </div>
          </div>
        </div>
      </div>

      <!-- Model Section -->
      <div class="section">
        <div class="section-header">
          <i class="pi pi-box"></i>
          <span>MODEL</span>
        </div>
        <div class="section-content">
          <ModelSelector />
          <PresetSelector />
        </div>
      </div>
    </div>

    <!-- Drag Overlay -->
    <div v-if="isDragging" class="drag-overlay">
      <div class="drag-content">
        <i class="pi pi-image drag-icon"></i>
        <div class="drag-text">
          <div class="drag-title">Drop Image to Analyze</div>
          <div class="drag-subtitle">Generate a prompt to recreate this image</div>
        </div>
      </div>
    </div>

    <!-- Analysis Loading Overlay -->
    <div v-if="isAnalyzing" class="analysis-overlay">
      <div class="analysis-content">
        <i class="pi pi-spin pi-spinner analysis-spinner"></i>
        <div class="analysis-text">
          <div class="analysis-title">Analyzing Image</div>
          <div class="analysis-subtitle">Claude is generating a prompt...</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useToast } from 'primevue/usetoast';
import Button from 'primevue/button';
import Slider from 'primevue/slider';
import Select from 'primevue/select';
import InputNumber from 'primevue/inputnumber';
import PromptInput from '@/components/generation/PromptInput.vue';
import ModelSelector from '@/components/generation/ModelSelector.vue';
import PresetSelector from '@/components/generation/PresetSelector.vue';
import { useGenerationStore } from '@/stores/generation';
import { useModelsStore } from '@/stores/models';
import type { Sampler, Scheduler } from '@/types';
import { analyzeImageForPrompt, fileToDataUrl, isValidImageFile } from '@/services/imageAnalysis';
import { readFile } from '@tauri-apps/plugin-fs';

const generationStore = useGenerationStore();
const modelsStore = useModelsStore();
const toast = useToast();

// Drag and drop state
const isDragging = ref(false);
const isAnalyzing = ref(false);
const dragCounter = ref(0);

// Sampler and Scheduler options
const samplerOptions = [
  { label: 'Euler', value: 'euler' },
  { label: 'Euler Ancestral', value: 'euler_a' },
  { label: 'Heun', value: 'heun' },
  { label: 'DPM2', value: 'dpm_2' },
  { label: 'DPM2 Ancestral', value: 'dpm_2_a' },
  { label: 'LMS', value: 'lms' },
  { label: 'DPM++ 2M', value: 'dpmpp_2m' },
  { label: 'DPM++ 2S Ancestral', value: 'dpmpp_2s_a' },
  { label: 'DPM++ SDE', value: 'dpmpp_sde' },
];

const schedulerOptions = [
  { label: 'Normal', value: 'normal' },
  { label: 'Karras', value: 'karras' },
  { label: 'Exponential', value: 'exponential' },
  { label: 'SGM Uniform', value: 'sgm_uniform' },
  { label: 'Simple', value: 'simple' },
  { label: 'DDIM Uniform', value: 'ddim_uniform' },
  { label: 'Beta', value: 'beta' },
];

// Aspect ratio presets
const aspectRatios = [
  { label: '1:2', width: 512, height: 1024 },
  { label: '9:16', width: 576, height: 1024 },
  { label: '3:4', width: 768, height: 1024 },
  { label: '1:1', width: 1024, height: 1024 },
  { label: '4:3', width: 1024, height: 768 },
  { label: '16:9', width: 1024, height: 576 },
  { label: '2:1', width: 1024, height: 512 },
];

const activeRatio = ref<string>('1:1');

const isActiveRatio = (ratio: { label: string; width: number | null; height: number | null }) => {
  return activeRatio.value === ratio.label;
};

const setAspectRatio = (ratio: { label: string; width: number | null; height: number | null }) => {
  activeRatio.value = ratio.label;
  if (ratio.width !== null && ratio.height !== null) {
    width.value = ratio.width;
    height.value = ratio.height;
  }
};

// Computed properties bound to store
const steps = computed({
  get: () => generationStore.currentParams.steps,
  set: (value: number | null) => {
    generationStore.currentParams.steps = value ?? 4;
  },
});

const cfgScale = computed({
  get: () => generationStore.currentParams.cfgScale,
  set: (value: number | null) => {
    generationStore.currentParams.cfgScale = value ?? 1.0;
  },
});

const sampler = computed({
  get: () => generationStore.currentParams.sampler,
  set: (value: Sampler) => {
    generationStore.currentParams.sampler = value;
  },
});

const scheduler = computed({
  get: () => generationStore.currentParams.scheduler,
  set: (value: Scheduler) => {
    generationStore.currentParams.scheduler = value;
  },
});

const width = computed({
  get: () => generationStore.currentParams.width,
  set: (value: number | null) => {
    generationStore.currentParams.width = value ?? 1024;
  },
});

const height = computed({
  get: () => generationStore.currentParams.height,
  set: (value: number | null) => {
    generationStore.currentParams.height = value ?? 1024;
  },
});

const seed = computed({
  get: () => generationStore.currentParams.seed,
  set: (value: number | null) => {
    generationStore.currentParams.seed = value ?? 0;
  },
});

// seedLocked = true means use the specific seed value (not random)
// seedLocked = false means randomize on each generation
const seedLocked = computed({
  get: () => !generationStore.randomizeSeedOnGenerate,
  set: (value: boolean) => {
    generationStore.randomizeSeedOnGenerate = !value;
  },
});

const toggleSeedLock = () => {
  seedLocked.value = !seedLocked.value;
  // When locking, ensure we have a valid seed
  if (seedLocked.value && (seed.value < 0 || seed.value === null)) {
    seed.value = Math.floor(Math.random() * 2147483647);
  }
};

const randomizeSeed = () => {
  seed.value = Math.floor(Math.random() * 2147483647);
};

// Watch for model changes and update default parameters
watch(
  () => modelsStore.selectedModelId,
  (newModelId) => {
    const model = modelsStore.models.find((m) => m.id === newModelId);
    if (model) {
      generationStore.currentParams.model = newModelId;
      if (model.defaultSteps) {
        generationStore.currentParams.steps = model.defaultSteps;
      }
      if (model.defaultGuidance !== undefined) {
        generationStore.currentParams.cfgScale = model.defaultGuidance;
      }
    }
  },
);

// Drag and drop handlers
const hasImageContent = (dataTransfer: DataTransfer | null): boolean => {
  if (!dataTransfer) return false;
  const types = dataTransfer.types;
  return types.includes('Files') || types.includes('text/uri-list') || types.includes('text/html');
};

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
</script>

<style scoped>
@reference "tailwindcss";

.left-sidebar {
  @apply relative flex flex-col h-full overflow-hidden;
  background: #1e2530;
  border-right: 1px solid #2d3748;
}

.sidebar-header {
  @apply flex items-center justify-between px-4 py-4 border-b;
  border-color: #2d3748;
}

.sidebar-title {
  @apply text-xl font-bold m-0;
  color: #f1f5f9;
}

.sidebar-subtitle {
  @apply text-xs m-0 mt-0.5;
  color: #64748b;
}

.sidebar-content {
  @apply flex-1 overflow-y-auto;
}

/* Sections */
.section {
  @apply border-b;
  border-color: #2d3748;
}

.section-header {
  @apply flex items-center gap-2 px-4 py-3 text-xs font-semibold tracking-wider;
  color: #94a3b8;

  i {
    @apply text-sm;
  }
}

.section-content {
  @apply px-4 pb-4 flex flex-col gap-4;
}

/* Fields */
.field {
  @apply flex flex-col gap-2;
}

.field-label {
  @apply text-xs font-medium tracking-wide;
  color: #64748b;
}

.field-row {
  @apply grid grid-cols-2 gap-3;
}

.slider-header {
  @apply flex items-center justify-between mb-1;
}

.slider-value {
  @apply text-sm font-mono;
  color: #38bdf8;
}

.seed-header {
  @apply flex items-center justify-between mb-2;
}

.seed-mode {
  @apply flex items-center gap-1.5 px-2 py-1 rounded-md cursor-pointer text-xs transition-all duration-200;
  background: #2d3748;
  color: #94a3b8;
  border: 1px solid #374151;

  &:hover {
    background: #374151;
    color: #e2e8f0;
  }

  i {
    font-size: 0.75rem;
  }
}

.seed-control {
  @apply flex gap-2;
}

/* Disabled input styling */
:deep(.p-inputnumber.p-disabled) {
  opacity: 0.6;
}

:deep(.p-inputnumber.p-disabled input) {
  background: #1e2530;
  color: #64748b;
}

/* Aspect Ratio Pills */
.aspect-pills {
  @apply flex flex-wrap gap-2;
}

.aspect-pill {
  @apply px-3 py-1.5 text-xs font-medium rounded-md cursor-pointer transition-all duration-200;
  background: #2d3748;
  color: #94a3b8;
  border: 1px solid #374151;

  &:hover {
    background: #374151;
    color: #e2e8f0;
  }

  &.active {
    background: #3b82f6;
    color: white;
    border-color: #3b82f6;
  }
}

/* PrimeVue Overrides for dark theme */
:deep(.p-inputnumber),
:deep(.p-select) {
  background: #2d3748;
  border-color: #374151;
  color: #e2e8f0;
}

:deep(.p-slider) {
  background: #374151;
}

:deep(.p-slider .p-slider-range) {
  background: #3b82f6;
}

:deep(.p-slider .p-slider-handle) {
  background: #3b82f6;
  border-color: #3b82f6;
}

/* Drag Overlay */
.drag-overlay {
  @apply absolute inset-0 z-50 flex items-center justify-center pointer-events-none;
  background: rgba(59, 130, 246, 0.15);
  backdrop-filter: blur(4px);
  border: 2px dashed #3b82f6;
}

.drag-content {
  @apply flex flex-col items-center gap-3 p-6 rounded-xl;
  background: #1e2530;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
}

.drag-icon {
  @apply text-4xl;
  color: #3b82f6;
}

.drag-text {
  @apply text-center;
}

.drag-title {
  @apply text-lg font-semibold;
  color: #f1f5f9;
}

.drag-subtitle {
  @apply text-sm;
  color: #64748b;
}

/* Analysis Loading Overlay */
.analysis-overlay {
  @apply absolute inset-0 z-50 flex items-center justify-center;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
}

.analysis-content {
  @apply flex flex-col items-center gap-4 p-8 rounded-xl;
  background: #1e2530;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
}

.analysis-spinner {
  @apply text-4xl;
  color: #3b82f6;
}

.analysis-text {
  @apply text-center;
}

.analysis-title {
  @apply text-lg font-semibold;
  color: #f1f5f9;
}

.analysis-subtitle {
  @apply text-sm;
  color: #64748b;
}
</style>
