<template>
  <div class="flex h-full bg-gray-900">
    <!-- Left Panel: Model List -->
    <aside class="flex flex-col h-full bg-gray-900 border-r border-gray-700 w-105 min-w-80">
      <div class="px-4 py-4 border-b border-gray-700">
        <h1 class="m-0 text-lg font-semibold text-gray-100">Model Manager</h1>
      </div>

      <!-- Search & Filter -->
      <div class="flex gap-2 px-4 py-3 border-b border-gray-700">
        <InputText v-model="searchQuery" placeholder="Search models..." class="flex-1" size="small" />
        <Select v-model="categoryFilter" :options="categoryOptions" optionLabel="label" optionValue="value" size="small" class="w-40" />
      </div>

      <!-- Model Count -->
      <div class="flex items-center justify-between px-4 py-2 border-b border-gray-700">
        <span class="text-xs text-gray-400">{{ filteredModels.length }} models</span>
      </div>

      <!-- Grouped Model List -->
      <div class="flex-1 overflow-y-auto">
        <template v-for="group in groupedModels" :key="group.category">
          <div v-if="group.models.length > 0" class="py-2">
            <div class="px-4 py-2 text-xs font-semibold tracking-wider text-gray-500 uppercase">{{ group.label }}</div>
            <div
              v-for="model in group.models"
              :key="model.id"
              class="flex items-start gap-3 px-4 py-3 transition-colors border-transparent cursor-pointer border-l-3 hover:bg-gray-800"
              :class="{ 'bg-gray-800 border-l-blue-500!': selectedModel?.id === model.id }"
              @click="selectModel(model)">
              <div
                class="flex items-center justify-center w-10 h-10 rounded-lg shrink-0"
                :class="getIconClasses(model.iconClass)">
                <component :is="model.icon" class="w-5 h-5" />
              </div>
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 text-sm font-medium text-gray-100">
                  {{ model.name }}
                  <span class="text-xs font-normal text-gray-500">{{ model.size }}</span>
                </div>
                <div class="text-xs mt-0.5 line-clamp-1 text-gray-400">{{ model.description }}</div>
                <div class="flex flex-wrap gap-1 mt-1.5">
                  <span
                    v-for="tag in model.tags"
                    :key="tag"
                    class="px-1.5 py-0.5 text-xs font-medium rounded"
                    :class="getTagClasses(tag)">
                    {{ tag }}
                  </span>
                </div>
              </div>
              <div class="shrink-0">
                <span
                  v-if="model.isDownloaded"
                  class="flex items-center justify-center w-6 h-6 text-green-400 rounded-full bg-green-900/50">
                  <Check class="w-3 h-3" />
                </span>
                <span
                  v-else
                  class="flex items-center justify-center w-6 h-6 text-gray-400 bg-gray-700 rounded-full">
                  <Download class="w-3 h-3" />
                </span>
              </div>
            </div>
          </div>
        </template>
      </div>
    </aside>

    <!-- Right Panel: Model Details -->
    <main class="flex-1 h-full p-6 overflow-y-auto bg-gray-950">
      <template v-if="selectedModel">
        <!-- Detail Header -->
        <div class="flex items-start gap-4 pb-6 mb-6 border-b border-gray-800">
          <div
            class="flex items-center justify-center w-16 h-16 rounded-xl shrink-0"
            :class="getDetailIconClasses(selectedModel.iconClass)">
            <component :is="selectedModel.icon" class="w-8 h-8" />
          </div>
          <div class="flex-1">
            <h2 class="m-0 text-xl font-semibold text-gray-50">{{ selectedModel.name }}</h2>
            <p class="m-0 mt-1 font-mono text-xs text-gray-500">{{ selectedModel.source }}</p>
            <p class="m-0 mt-2 text-sm text-gray-400">{{ selectedModel.description }}</p>
          </div>
          <div class="flex gap-2 shrink-0">
            <Button
              v-if="!selectedModel.isDownloaded"
              label="Download"
              icon="pi pi-download"
              :loading="isDownloading(selectedModel.id)"
              @click="downloadModel(selectedModel)" />
            <Button
              v-if="selectedModel.docsUrl"
              label="Docs"
              icon="pi pi-external-link"
              severity="secondary"
              outlined
              @click="openDocs(selectedModel.docsUrl)" />
          </div>
        </div>

        <!-- Download Progress -->
        <div v-if="isDownloading(selectedModel.id)" class="p-4 mb-6 bg-gray-800 rounded-lg">
          <ProgressBar :value="getDownloadProgress(selectedModel.id)" :showValue="true" />
          <p class="m-0 mt-2 text-sm text-center text-gray-400">Downloading {{ selectedModel.name }}...</p>
        </div>

        <!-- Model Properties -->
        <div class="mb-6">
          <h3 class="m-0 mb-3 text-sm font-semibold tracking-wider text-gray-400 uppercase">Model Information</h3>
          <div class="grid grid-cols-2 gap-4">
            <div class="flex flex-col gap-1">
              <span class="text-xs text-gray-500">Type</span>
              <span class="text-sm font-medium text-gray-200">{{ selectedModel.type }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-xs text-gray-500">Category</span>
              <span class="text-sm font-medium text-gray-200">{{ selectedModel.categoryLabel }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-xs text-gray-500">File Size</span>
              <span class="text-sm font-medium text-gray-200">{{ selectedModel.size }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-xs text-gray-500">Format</span>
              <span class="text-sm font-medium text-gray-200">{{ selectedModel.format }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-xs text-gray-500">License</span>
              <span class="text-sm font-medium text-gray-200">{{ selectedModel.license }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-xs text-gray-500">Status</span>
              <span class="text-sm font-medium" :class="selectedModel.isDownloaded ? 'text-green-400' : 'text-amber-400'">
                {{ selectedModel.isDownloaded ? 'Downloaded' : 'Not Downloaded' }}
              </span>
            </div>
          </div>
        </div>

        <!-- Default Settings (for generation models) -->
        <div v-if="selectedModel.category === 'generation' && selectedModel.defaultSettings" class="mb-6">
          <h3 class="m-0 mb-3 text-sm font-semibold tracking-wider text-gray-400 uppercase">Default Settings</h3>
          <div class="grid grid-cols-2 gap-4 p-4 bg-gray-800 rounded-lg">
            <div class="flex items-center justify-between">
              <span class="text-sm text-gray-400">Steps</span>
              <span class="text-sm font-medium text-gray-200">{{ selectedModel.defaultSettings.steps }}</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-sm text-gray-400">Guidance</span>
              <span class="text-sm font-medium text-gray-200">{{ selectedModel.defaultSettings.guidance }}</span>
            </div>
          </div>
        </div>

        <!-- Features (for vision models) -->
        <div v-if="selectedModel.features && selectedModel.features.length > 0" class="mb-6">
          <h3 class="m-0 mb-3 text-sm font-semibold tracking-wider text-gray-400 uppercase">Features</h3>
          <ul class="pl-5 m-0 text-gray-300">
            <li v-for="feature in selectedModel.features" :key="feature" class="py-1 text-sm">{{ feature }}</li>
          </ul>
        </div>

        <!-- Notes -->
        <div v-if="selectedModel.notes" class="p-4 mb-6 rounded-lg bg-gray-800/50">
          <h3 class="m-0 mb-3 text-sm font-semibold tracking-wider text-gray-400 uppercase">Notes</h3>
          <p class="m-0 text-sm text-gray-400">{{ selectedModel.notes }}</p>
        </div>
      </template>

      <!-- Empty State -->
      <div v-else class="flex flex-col items-center justify-center h-full gap-4 text-gray-500">
        <Box class="w-16 h-16" />
        <p class="m-0 text-sm">Select a model to view details</p>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, markRaw, type Component } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import InputText from 'primevue/inputtext';
import Select from 'primevue/select';
import Button from 'primevue/button';
import ProgressBar from 'primevue/progressbar';
import { Box, Check, Download, Image, Eye } from 'lucide-vue-next';
import { useModelsStore } from '@/stores/models';
import { useAutoTagStore } from '@/stores/autoTag';

interface ModelInfo {
  id: string;
  name: string;
  description: string;
  size: string;
  type: string;
  category: 'generation' | 'vision' | 'utility';
  categoryLabel: string;
  format: string;
  license: string;
  source: string;
  tags: string[];
  icon: Component;
  iconClass: string;
  isDownloaded: boolean;
  docsUrl?: string;
  defaultSettings?: {
    steps: number;
    guidance: number;
  };
  features?: string[];
  notes?: string;
}

const modelsStore = useModelsStore();
const autoTagStore = useAutoTagStore();

const searchQuery = ref('');
const categoryFilter = ref('all');
const selectedModel = ref<ModelInfo | null>(null);
const isDownloadingSchnell = ref(false);
const isDownloadingDev = ref(false);

const categoryOptions = [
  { label: 'All Models', value: 'all' },
  { label: 'Image Generation', value: 'generation' },
  { label: 'Vision & Analysis', value: 'vision' },
];

// Helper functions for dynamic classes
const getIconClasses = (iconClass: string): string => {
  if (iconClass === 'icon-flux') return 'bg-purple-900/50 text-purple-400';
  if (iconClass === 'icon-vision') return 'bg-emerald-900/50 text-emerald-400';
  return 'bg-gray-700 text-gray-300';
};

const getDetailIconClasses = (iconClass: string): string => {
  if (iconClass === 'icon-flux') return 'bg-purple-900/30 text-purple-400';
  if (iconClass === 'icon-vision') return 'bg-emerald-900/30 text-emerald-400';
  return 'bg-gray-800 text-gray-400';
};

const getTagClasses = (tag: string): string => {
  const tagLower = tag.toLowerCase();
  if (tagLower === 'flux') return 'bg-purple-900/50 text-purple-300';
  if (tagLower === 'fast' || tagLower === 'hq') return 'bg-blue-900/50 text-blue-300';
  if (tagLower === 'quantized') return 'bg-amber-900/50 text-amber-300';
  if (tagLower === 'vision' || tagLower === 'vlm') return 'bg-emerald-900/50 text-emerald-300';
  return 'bg-gray-700 text-gray-300';
};

// Build unified model list
const allModels = computed<ModelInfo[]>(() => {
  const schnellDownloaded = modelsStore.models.find(m => m.id === 'schnell')?.isDownloaded ?? false;
  const devDownloaded = modelsStore.models.find(m => m.id === 'dev')?.isDownloaded ?? false;
  const moondreamDownloaded = autoTagStore.isLocalAvailable;

  return [
    {
      id: 'schnell',
      name: 'FLUX.1 [schnell]',
      description: 'Fast generation model optimized for speed (4 steps)',
      size: '~12 GB',
      type: 'Diffusion Transformer',
      category: 'generation',
      categoryLabel: 'Image Generation',
      format: 'Quantized (BF16)',
      license: 'Apache 2.0',
      source: 'black-forest-labs/FLUX.1-schnell',
      tags: ['FLUX', 'FAST', 'QUANTIZED'],
      icon: markRaw(Image),
      iconClass: 'icon-flux',
      isDownloaded: schnellDownloaded,
      docsUrl: 'https://huggingface.co/black-forest-labs/FLUX.1-schnell',
      defaultSettings: { steps: 4, guidance: 1.0 },
      notes: 'Best for quick iterations and drafts. Uses less VRAM than full precision.',
    },
    {
      id: 'dev',
      name: 'FLUX.1 [dev]',
      description: 'High quality generation model for detailed outputs (28+ steps)',
      size: '~12.8 GB',
      type: 'Diffusion Transformer',
      category: 'generation',
      categoryLabel: 'Image Generation',
      format: 'Quantized (Q8_0)',
      license: 'FLUX.1 [dev] Non-Commercial',
      source: 'black-forest-labs/FLUX.1-dev',
      tags: ['FLUX', 'HQ', 'QUANTIZED'],
      icon: markRaw(Image),
      iconClass: 'icon-flux',
      isDownloaded: devDownloaded,
      docsUrl: 'https://huggingface.co/black-forest-labs/FLUX.1-dev',
      defaultSettings: { steps: 28, guidance: 3.5 },
      notes: 'Photorealistic quality, best for final outputs. Non-commercial license.',
    },
    {
      id: 'moondream',
      name: 'Moondream 2',
      description: 'Compact vision-language model for image understanding',
      size: '~1.8 GB',
      type: 'Vision Language Model',
      category: 'vision',
      categoryLabel: 'Vision & Analysis',
      format: 'Safetensors',
      license: 'Apache 2.0',
      source: 'vikhyatk/moondream2',
      tags: ['VISION', 'VLM'],
      icon: markRaw(Eye),
      iconClass: 'icon-vision',
      isDownloaded: moondreamDownloaded,
      docsUrl: 'https://huggingface.co/vikhyatk/moondream2',
      features: [
        'Automatic image tagging',
        'Image description generation',
        'Visual question answering',
        'Runs locally on GPU',
      ],
      notes: 'Used by the Auto-Tag feature to analyze images and generate descriptive tags.',
    },
  ];
});

const filteredModels = computed(() => {
  return allModels.value.filter(model => {
    const matchesSearch = searchQuery.value === '' ||
      model.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      model.description.toLowerCase().includes(searchQuery.value.toLowerCase());
    const matchesCategory = categoryFilter.value === 'all' || model.category === categoryFilter.value;
    return matchesSearch && matchesCategory;
  });
});

const groupedModels = computed(() => {
  const groups = [
    { category: 'generation', label: 'Image Generation', models: [] as ModelInfo[] },
    { category: 'vision', label: 'Vision & Analysis', models: [] as ModelInfo[] },
  ];

  for (const model of filteredModels.value) {
    const group = groups.find(g => g.category === model.category);
    if (group) {
      group.models.push(model);
    }
  }

  return groups;
});

const selectModel = (model: ModelInfo) => {
  selectedModel.value = model;
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

const downloadModel = async (model: ModelInfo) => {
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
    // Refresh selected model state
    if (selectedModel.value?.id === model.id) {
      selectedModel.value = allModels.value.find(m => m.id === model.id) ?? null;
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

onMounted(async () => {
  await Promise.all([
    modelsStore.refreshModelAvailability(),
    autoTagStore.checkModelStatus(),
  ]);
  // Auto-select first model
  if (allModels.value.length > 0) {
    selectedModel.value = allModels.value[0];
  }
});
</script>
