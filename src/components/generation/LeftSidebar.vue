<template>
  <div
    class="flex flex-col h-full gap-2"
    @dragenter.prevent="handleDragEnter"
    @dragover.prevent="handleDragOver"
    @dragleave.prevent="handleDragLeave"
    @drop.prevent="handleDrop">
    <!-- Header -->
    <div class="px-4 py-4 border-b border-gray-700">
      <div class="flex items-start justify-between">
        <h1 class="m-0 text-lg font-semibold text-gray-100">Image Generator</h1>
        <Cog class="w-5 h-5 text-gray-300 transition-colors cursor-pointer hover:text-gray-100" @click="showPresetModal = true" />
      </div>
    </div>

    <!-- Scrollable Content -->
    <div class="flex flex-col gap-2 overflow-y-auto">
      <!-- Preset Section -->
      <div v-for="section in sections" :key="section.id" class="">
        <div class="flex gap-2 px-2 py-2 text-xs font-semibold tracking-wider text-gray-300 uppercase border-l-3 border-l-gray-500/50">
          <component :is="section.icon" class="w-4 h-4" />
          {{ section.label }} A
        </div>

        <div
          class="flex items-start gap-3 px-4 py-3 transition-colors bg-gray-900 border-transparent cursor-pointer border-l-3 hover:bg-gray-800 border-l-gray-500/50 hover:border-l-blue-500">
          <component :is="section.component" @generate="handleGenerate" />
        </div>

      </div>
    </div>

    <!-- Drag Overlay -->
    <div
      v-if="isDragging"
      class="absolute inset-0 z-50 flex items-center justify-center border-2 border-dashed border-(--accent-primary) bg-[rgba(212,168,83,0.15)] backdrop-blur pointer-events-none">
      <div class="flex flex-col items-center gap-3 rounded-xl bg-(--bg-elevated) p-6 shadow-[0_4px_20px_rgba(0,0,0,0.4)]">
        <i class="pi pi-image text-4xl text-(--accent-primary)"></i>
        <div class="text-center">
          <div class="text-lg font-semibold text-(--text-heading)">Drop Image to Analyze</div>
          <div class="text-sm text-(--text-secondary)">Generate a prompt to recreate this image</div>
        </div>
      </div>
    </div>

    <!-- Analysis Loading Overlay -->
    <div v-if="isAnalyzing" class="absolute inset-0 z-50 flex items-center justify-center bg-[rgba(0,0,0,0.7)] backdrop-blur">
      <div class="flex flex-col items-center gap-4 rounded-xl bg-(--bg-elevated) p-8 shadow-[0_4px_20px_rgba(0,0,0,0.4)]">
        <i class="pi pi-spin pi-spinner text-4xl text-(--accent-primary)"></i>
        <div class="text-center">
          <div class="text-lg font-semibold text-(--text-heading)">Analyzing Image</div>
          <div class="text-sm text-(--text-secondary)">Claude is generating a prompt...</div>
        </div>
      </div>
    </div>

    <!-- Preset Modal -->
    <Dialog v-model:visible="showPresetModal" modal header="Presets" :style="{ width: '450px' }">
      <PresetSelector />
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useToast } from 'primevue/usetoast';
import Dialog from 'primevue/dialog';

import PromptInput from '@/components/generation/PromptInput.vue';
import ImageSettings from '@/components/generation/actions/ImageSettings.vue';
import GenerationSettings from '@/components/generation/GenerationSettings.vue';
import ModelSelector from '@/components/generation/ModelSelector.vue';
import PresetSelector from '@/components/generation/PresetSelector.vue';
import { SquarePen, SlidersHorizontal, Cog, Package } from 'lucide-vue-next';
import { useGenerationStore } from '@/stores/generation';
import { useModelsStore } from '@/stores/models';
import { analyzeImageForPrompt, fileToDataUrl, isValidImageFile } from '@/services/imageAnalysis';
import { readFile } from '@tauri-apps/plugin-fs';

const emit = defineEmits<{
  generate: [];
}>();

const generationStore = useGenerationStore();
const modelsStore = useModelsStore();
const toast = useToast();

const handleGenerate = () => {
  emit('generate');
};

// Drag and drop state
const isDragging = ref(false);
const isAnalyzing = ref(false);
const dragCounter = ref(0);

const showPresetModal = ref(false);

const sections = [
  {
    id: 'PROMPT',
    label: 'PROMPT',
    icon: SquarePen,
    component: PromptInput,
    canCollapse: false,
    collapsed: false,
  },
  {
    id: 'IMAGE SETTINGS',
    label: 'IMAGE SETTINGS',
    icon: SlidersHorizontal,
    component: ImageSettings,
    canCollapse: false,
    collapsed: false,
  },
  {
    id: 'GENERATION SETTINGS',
    label: 'GENERATION SETTINGS',
    icon: Cog,
    component: GenerationSettings,
    canCollapse: false,
    collapsed: false,
  },
  {
    id: 'MODEL',
    label: 'MODEL',
    icon: Package,
    component: ModelSelector,
    canCollapse: false,
    collapsed: false,
  },
];

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
