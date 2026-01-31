<template>
  <div class="flex-1 h-full p-6 overflow-y-auto rounded-xl bg-surface-950">
    <!-- Model Details -->
    <template v-if="selectedModel">
      <!-- Detail Header -->
      <div class="flex items-start gap-4 pb-6 mb-6 border-b border-surface-800">
        <div class="flex items-center justify-center w-16 h-16 rounded-xl shrink-0" :class="getDetailIconClasses(selectedModel.iconClass)">
          <fa :icon="['fal', selectedModel.icon]" size="2x" />
        </div>
        <div class="flex-1">
          <h2 class="m-0 text-xl font-semibold text-surface-50">{{ selectedModel.name }}</h2>
          <p class="m-0 mt-1 font-mono text-xs text-surface-500">{{ selectedModel.source }}</p>
          <p class="m-0 mt-2 text-sm text-surface-400">{{ selectedModel.description }}</p>
        </div>
        <div class="flex gap-2 shrink-0">
          <Button
            v-if="!selectedModel.isDownloaded && selectedModel.category !== 'component'"
            label="Download"
            icon="pi pi-download"
            :loading="isDownloading(selectedModel.id)"
            @click="downloadModel(selectedModel)" />
          <Button v-if="selectedModel.docsUrl" label="Docs" icon="pi pi-external-link" severity="secondary" outlined @click="openDocs(selectedModel.docsUrl)" />
        </div>
      </div>

      <!-- Download Progress -->
      <div v-if="isDownloading(selectedModel.id)" class="p-4 mb-6 rounded-lg bg-surface-800">
        <ProgressBar :value="getDownloadProgress(selectedModel.id)" :showValue="true" />
        <p class="m-0 mt-2 text-sm text-center text-surface-400">Downloading {{ selectedModel.name }}...</p>
      </div>

      <!-- Model Properties -->
      <div class="mb-6">
        <h3 class="m-0 mb-3 text-sm font-semibold tracking-wider uppercase text-surface-400">Model Information</h3>
        <div class="grid grid-cols-2 gap-4">
          <div class="flex flex-col gap-1">
            <span class="text-xs text-surface-500">Type</span>
            <span class="text-sm font-medium text-surface-200">{{ selectedModel.type }}</span>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-xs text-surface-500">Category</span>
            <span class="text-sm font-medium text-surface-200">{{ selectedModel.categoryLabel }}</span>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-xs text-surface-500">File Size</span>
            <span class="text-sm font-medium text-surface-200">{{ selectedModel.size }}</span>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-xs text-surface-500">Format</span>
            <span class="text-sm font-medium text-surface-200">{{ selectedModel.format }}</span>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-xs text-surface-500">License</span>
            <span class="text-sm font-medium text-surface-200">{{ selectedModel.license }}</span>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-xs text-surface-500">Status</span>
            <span class="text-sm font-medium" :class="selectedModel.isDownloaded ? 'text-green-400' : 'text-amber-400'">
              {{ selectedModel.isDownloaded ? 'Downloaded' : 'Not Downloaded' }}
            </span>
          </div>
        </div>
      </div>

      <!-- Default Settings (for generation models) -->
      <div v-if="selectedModel.category === 'generation' && selectedModel.defaultSettings" class="mb-6">
        <h3 class="m-0 mb-3 text-sm font-semibold tracking-wider uppercase text-surface-400">Default Settings</h3>
        <div class="grid grid-cols-2 gap-4 p-4 rounded-lg bg-surface-800">
          <div class="flex items-center justify-between">
            <span class="text-sm text-surface-400">Steps</span>
            <span class="text-sm font-medium text-surface-200">{{ selectedModel.defaultSettings.steps }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-sm text-surface-400">Guidance</span>
            <span class="text-sm font-medium text-surface-200">{{ selectedModel.defaultSettings.guidance }}</span>
          </div>
        </div>
      </div>

      <!-- Features (for vision and component models) -->
      <div v-if="selectedModel.features && selectedModel.features.length > 0" class="mb-6">
        <h3 class="m-0 mb-3 text-sm font-semibold tracking-wider uppercase text-surface-400">Features</h3>
        <ul class="pl-5 m-0 text-surface-300">
          <li v-for="feature in selectedModel.features" :key="feature" class="py-1 text-sm">{{ feature }}</li>
        </ul>
      </div>

      <!-- Quantization Status (for components that support it) -->
      <div v-if="selectedModel.category === 'component' && selectedModel.hasQuantized" class="mb-6">
        <h3 class="m-0 mb-3 text-sm font-semibold tracking-wider uppercase text-surface-400">Quantization</h3>
        <div class="p-4 rounded-lg bg-surface-800">
          <div class="flex items-center justify-between">
            <span class="text-sm text-surface-400">Quantized Version</span>
            <span class="text-sm font-medium" :class="selectedModel.quantizedDownloaded ? 'text-green-400' : 'text-surface-500'">
              {{ selectedModel.quantizedDownloaded ? 'Available' : 'Not Downloaded' }}
            </span>
          </div>
          <p v-if="selectedModel.quantizedDownloaded" class="m-0 mt-2 text-xs text-surface-500"> Using quantized version for reduced VRAM usage. </p>
          <p v-else class="m-0 mt-2 text-xs text-surface-500"> Full precision model is being used. Quantized version saves ~6GB VRAM. </p>
        </div>
      </div>

      <!-- Component Info (for shared components) -->
      <div v-if="selectedModel.category === 'component'" class="p-4 mb-6 border rounded-lg border-cyan-900/50 bg-cyan-900/10">
        <div class="flex items-start gap-2">
          <fa :icon="['fal', 'layer-group']" size="sm" class="mt-0.5 text-cyan-400 shrink-0" />
          <div>
            <p class="m-0 text-sm font-medium text-cyan-300">Shared Component</p>
            <p class="m-0 mt-1 text-xs text-surface-400">
              This component is shared across all FLUX models and is downloaded automatically when you download any FLUX model.
            </p>
          </div>
        </div>
      </div>

      <!-- Notes -->
      <div v-if="selectedModel.notes" class="p-4 mb-6 rounded-lg bg-surface-800/50">
        <h3 class="m-0 mb-3 text-sm font-semibold tracking-wider uppercase text-surface-400">Notes</h3>
        <p class="m-0 text-sm text-surface-400">{{ selectedModel.notes }}</p>
      </div>
    </template>

    <!-- Empty State -->
    <div v-else class="flex flex-col items-center justify-center h-full gap-4 text-surface-500">
      <fa :icon="['fal', 'box']" size="4x" />
      <p class="m-0 text-sm">Select a model to view details</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { storeToRefs } from 'pinia';
import Button from 'primevue/button';
import ProgressBar from 'primevue/progressbar';
import { useModelsStore, type SelectedModelInfo } from '@/stores/models';
import { useAutoTagStore } from '@/stores/autoTag';

const modelsStore = useModelsStore();
const autoTagStore = useAutoTagStore();

// Get selected model from store (shared with sidebar)
const { selectedViewComponent: selectedModel } = storeToRefs(modelsStore);

const isDownloadingSchnell = ref(false);
const isDownloadingDev = ref(false);

const getDetailIconClasses = (iconClass: string): string => {
  if (iconClass === 'icon-flux') return 'bg-purple-900/30 text-purple-400';
  if (iconClass === 'icon-lora') return 'bg-pink-900/30 text-pink-400';
  if (iconClass === 'icon-vision') return 'bg-emerald-900/30 text-emerald-400';
  if (iconClass === 'icon-vae') return 'bg-cyan-900/30 text-cyan-400';
  if (iconClass === 'icon-clip') return 'bg-orange-900/30 text-orange-400';
  if (iconClass === 'icon-t5') return 'bg-indigo-900/30 text-indigo-400';
  return 'bg-surface-800 text-surface-400';
};

const isDownloading = (modelId: string): boolean => {
  if (modelId === 'schnell') return isDownloadingSchnell.value;
  if (modelId === 'dev') return isDownloadingDev.value;
  if (modelId === 'moondream') return autoTagStore.isDownloading;
  return false;
};

const getDownloadProgress = (modelId: string): number => {
  if (modelId === 'moondream') return autoTagStore.downloadProgressPercent;
  return 0; // FLUX downloads don't have progress tracking yet
};

const downloadModel = async (model: SelectedModelInfo) => {
  try {
    if (model.id === 'schnell') {
      isDownloadingSchnell.value = true;
      await invoke<string>('download_flux_schnell');
      await modelsStore.refreshModelAvailability();
      isDownloadingSchnell.value = false;
    } else if (model.id === 'dev') {
      isDownloadingDev.value = true;
      await invoke<string>('download_flux_dev');
      await modelsStore.refreshModelAvailability();
      isDownloadingDev.value = false;
    } else if (model.id === 'moondream') {
      await autoTagStore.downloadModel();
    }
  } catch (error) {
    console.error('Download failed:', error);
    isDownloadingSchnell.value = false;
    isDownloadingDev.value = false;
  }
};

const openDocs = (url: string) => {
  window.open(url, '_blank');
};
</script>
