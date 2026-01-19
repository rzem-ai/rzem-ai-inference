<template>
  <div
    class="flex flex-col gap-2 p-4"
    :style="'height: ' + windowsStore.mainHeight + 'px'"
    @dragenter.prevent="handleDragEnter"
    @dragover.prevent="handleDragOver"
    @dragleave.prevent="handleDragLeave"
    @drop.prevent="handleDrop">
    <div class="shrink">
      <GenerateButton :queue-count="queueStore.queueLength" @generate="handleGenerate" />
    </div>

    <div
      :class="{ 'drag-highlight': isDragging }"
      class="flex-1 bg-red-400"
      :pt="{
        root: { class: 'flex flex-col' },
        body: { class: 'flex-1 flex flex-col overflow-hidden' },
        content: { class: 'overflow-y-auto flex-1' },
      }">
      <div>Generate</div>
      <div>
        <PromptInput />
        <div class="divider"></div>
        <ModelSelector />
        <PresetSelector />
        <div class="divider"></div>
        <ParameterControls />
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
import { ref, watch } from 'vue';
import { useToast } from 'primevue/usetoast';
import PromptInput from '@/components/generation/PromptInput.vue';
import ModelSelector from '@/components/generation/ModelSelector.vue';
import PresetSelector from '@/components/generation/PresetSelector.vue';
import ParameterControls from '@/components/generation/ParameterControls.vue';
import GenerateButton from '@/components/generation/GenerateButton.vue';
import ImageCanvas from '@/components/generation/ImageCanvas.vue';
import { useQueueStore } from '@/stores/queue';
import { useGenerationStore } from '@/stores/generation';
import { useModelsStore } from '@/stores/models';
import type { GenerationParams } from '@/stores/queue';
import { analyzeImageForPrompt, fileToDataUrl, isValidImageFile } from '@/services/imageAnalysis';
import { readFile } from '@tauri-apps/plugin-fs';

import { useWindowsStore } from '../../stores/windows';

const windowsStore = useWindowsStore();

const canvasRef = ref<InstanceType<typeof ImageCanvas> | null>(null);
const queueStore = useQueueStore();
const generationStore = useGenerationStore();
const modelsStore = useModelsStore();
const toast = useToast();

// Drag and drop state
const isDragging = ref(false);
const isAnalyzing = ref(false);
const dragCounter = ref(0);

// Check if drag contains image content (file or URL)
const hasImageContent = (dataTransfer: DataTransfer | null): boolean => {
  if (!dataTransfer) return false;
  const types = dataTransfer.types;
  // Files from file system, or URLs from browser (text/uri-list, text/html)
  return types.includes('Files') || types.includes('text/uri-list') || types.includes('text/html');
};

// Drag and drop handlers
const handleDragEnter = (e: DragEvent) => {
  dragCounter.value++;
  if (hasImageContent(e.dataTransfer)) {
    isDragging.value = true;
  }
};

const handleDragOver = (_e: DragEvent) => {
  // Required to allow drop - just prevent default
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

  // Try to get file first (from file system drop - Windows/macOS)
  const file = dataTransfer.files?.[0];
  if (file && isValidImageFile(file)) {
    await analyzeDroppedImage(file);
    return;
  }

  // Try to get URI (Linux file managers use file:// URIs)
  const uriList = dataTransfer.getData('text/uri-list');
  console.log('handleDrop:uriList:', uriList);
  if (uriList) {
    // Parse URI list (can contain multiple lines, take first non-comment)
    const uri = uriList
      .split('\n')
      .find((line) => line.trim() && !line.startsWith('#'))
      ?.trim();
    console.log('handleDrop:', uri);

    if (uri?.startsWith('file://')) {
      // Local file from Linux file manager
      await analyzeLocalFile(uri);
      return;
    } else if (uri && isImageUrl(uri)) {
      // Remote URL
      await analyzeImageFromUrl(uri);
      return;
    }
  }

  // Try to extract image URL from HTML (some browsers provide this)
  const html = dataTransfer.getData('text/html');
  console.log('handleDrop:html:', html);
  if (html) {
    const imgUrl = extractImageUrlFromHtml(html);
    if (imgUrl) {
      console.log('handleDrop:', imgUrl);
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

// Check if URL looks like an image
const isImageUrl = (url: string): boolean => {
  const imageExtensions = ['.png', '.jpg', '.jpeg', '.gif', '.webp', '.bmp'];
  const lowerUrl = url.toLowerCase();
  return imageExtensions.some((ext) => lowerUrl.includes(ext)) || lowerUrl.includes('image');
};

// Get MIME type from file extension
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

// Read and analyze a local file (from file:// URI)
const analyzeLocalFile = async (fileUri: string) => {
  isAnalyzing.value = true;

  try {
    // Convert file:// URI to path
    // URI format: file:///path/to/file or file://localhost/path/to/file
    let filePath = fileUri.replace(/^file:\/\/(localhost)?/, '');
    // Decode URI components (handles spaces and special chars)
    filePath = decodeURIComponent(filePath);

    // Check if it's an image file
    if (!isImageUrl(filePath)) {
      throw new Error('File is not a supported image format');
    }

    // Read file using Tauri fs plugin
    const fileData = await readFile(filePath);

    // Convert to base64 data URL
    const base64 = btoa(fileData.reduce((data: string, byte: number) => data + String.fromCharCode(byte), ''));
    const mimeType = getMimeType(filePath);
    const dataUrl = `data:${mimeType};base64,${base64}`;

    // Call Claude API to analyze the image
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

// Extract image URL from HTML string
const extractImageUrlFromHtml = (html: string): string | null => {
  // Try <img src="...">
  const imgMatch = html.match(/<img[^>]+src=["']([^"']+)["']/i);
  if (imgMatch) return imgMatch[1];

  // Try <a>file://...</a> (Linux file managers like Nautilus)
  const anchorMatch = html.match(/<a[^>]*>(file:\/\/[^<]+)<\/a>/i);
  if (anchorMatch && isImageUrl(anchorMatch[1])) return anchorMatch[1];

  // Try <a href="file://...">
  const hrefMatch = html.match(/<a[^>]+href=["'](file:\/\/[^"']+)["']/i);
  if (hrefMatch && isImageUrl(hrefMatch[1])) return hrefMatch[1];

  return null;
};

// Fetch image from URL and analyze it
const analyzeImageFromUrl = async (url: string) => {
  isAnalyzing.value = true;

  try {
    // Fetch the image
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`Failed to fetch image: ${response.status}`);
    }

    const blob = await response.blob();

    // Validate it's an image
    if (!blob.type.startsWith('image/')) {
      throw new Error('URL did not return an image');
    }

    // Convert to data URL
    const dataUrl = await new Promise<string>((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(reader.result as string);
      reader.onerror = () => reject(new Error('Failed to read image'));
      reader.readAsDataURL(blob);
    });

    // Call Claude API to analyze the image
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
    // Convert file to data URL
    const dataUrl = await fileToDataUrl(file);

    // Call Claude API to analyze the image
    const prompt = await analyzeImageForPrompt(dataUrl);

    // Update the prompt in the store
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

// Watch for model changes and update default parameters
watch(
  () => modelsStore.selectedModelId,
  (newModelId) => {
    const model = modelsStore.models.find((m) => m.id === newModelId);
    if (model) {
      // Update generation params with model defaults
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

// Track displayed job IDs to avoid re-displaying
const displayedJobIds = ref<Set<string>>(new Set());

// Watch all jobs for NEW completions with result_path
watch(
  () => queueStore.jobs,
  (jobs) => {
    // Find completed jobs with result_path that we haven't displayed yet
    const newlyCompleted = jobs.filter((j) => j.status === 'completed' && j.result_path && !displayedJobIds.value.has(j.id));

    if (newlyCompleted.length > 0) {
      // Display the most recent newly completed one
      const latestJob = newlyCompleted[newlyCompleted.length - 1];
      if (canvasRef.value && latestJob.result_path) {
        canvasRef.value.setImage(latestJob.result_path);
        // Mark ALL newly completed jobs as displayed
        newlyCompleted.forEach((j) => displayedJobIds.value.add(j.id));
      }
    }
  },
  { deep: true },
);

const handleGenerate = async () => {
  const params = generationStore.currentParams;

  // Build queue params from current generation params
  const queueParams: GenerationParams = {
    prompt: params.prompt,
    negative_prompt: params.negativePrompt || undefined,
    steps: params.steps,
    cfg_scale: params.cfgScale,
    width: params.width,
    height: params.height,
    seed: params.seed === -1 ? Math.floor(Math.random() * 2147483647) : params.seed,
    model: params.model,
  };

  try {
    // Add to queue - backend will handle processing
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

defineExpose({
  canvasRef,
});
</script>

<style scoped>
@reference "tailwindcss";

.generation-input-container {
  @apply relative;
}

.generation-input-container.drag-highlight {
  @apply border-2 border-dashed;
  border-color: var(--p-primary-color);
}

/* Drag Overlay */
.drag-overlay {
  @apply absolute inset-0 z-50 flex items-center justify-center rounded-lg pointer-events-none;
  background: rgba(59, 130, 246, 0.15);
  backdrop-filter: blur(4px);
  border: 2px dashed var(--p-primary-color);
}

.drag-content {
  @apply flex flex-col items-center gap-3 p-6 rounded-xl;
  background: var(--p-surface-0);
  box-shadow: var(--p-card-shadow);
}

.drag-icon {
  @apply text-4xl;
  color: var(--p-primary-color);
}

.drag-text {
  @apply text-center;
}

.drag-title {
  @apply text-lg font-semibold;
  color: var(--p-text-color);
}

.drag-subtitle {
  @apply text-sm;
  color: var(--p-text-muted-color);
}

/* Analysis Loading Overlay */
.analysis-overlay {
  @apply absolute inset-0 z-50 flex items-center justify-center rounded-lg;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
}

.analysis-content {
  @apply flex flex-col items-center gap-4 p-8 rounded-xl;
  background: var(--p-surface-0);
  box-shadow: var(--p-card-shadow);
}

.analysis-spinner {
  @apply text-4xl;
  color: var(--p-primary-color);
}

.analysis-text {
  @apply text-center;
}

.analysis-title {
  @apply text-lg font-semibold;
  color: var(--p-text-color);
}

.analysis-subtitle {
  @apply text-sm;
  color: var(--p-text-muted-color);
}
</style>
