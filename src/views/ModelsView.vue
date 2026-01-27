<template>
  <div class="flex w-full">
    <!-- Sidebar -->
    <WorkspaceActions>
      <template #header>Model Manager</template>
      <template #body>
        <!-- Search & Filter -->
        <div class="flex gap-2 px-4 py-3 border-b border-surface-700">
          <InputText v-model="searchQuery" placeholder="Search models..." class="flex-1" size="small" />
          <Select v-model="categoryFilter" :options="categoryOptions" optionLabel="label" optionValue="value" size="small" class="w-40" />
        </div>

        <!-- Model Count -->
        <div class="flex items-center justify-between px-4 py-2 border-b border-surface-700">
          <span class="text-xs text-surface-400">{{ filteredModels.length }} models</span>
        </div>

        <!-- Grouped Model List -->
        <div class="flex-1 overflow-y-auto">
          <template v-for="group in groupedModels" :key="group.category">
            <div v-if="group.models.length > 0" class="py-2">
              <div class="px-4 py-2 text-xs font-semibold tracking-wider uppercase text-surface-500">{{ group.label }}</div>
              <div
                v-for="model in group.models"
                :key="model.id"
                class="flex items-start gap-3 px-4 py-3 transition-colors border-transparent cursor-pointer border-l-3 hover:bg-surface-800"
                :class="{ 'bg-surface-800 border-l-blue-500!': selectedModel?.id === model.id }"
                @click="selectModel(model)">
                <div class="flex items-center justify-center w-10 h-10 rounded-lg shrink-0" :class="getIconClasses(model.iconClass)">
                  <fa :icon="['fal', model.icon]" size="lg" />
                </div>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 text-sm font-medium text-surface-100">
                    {{ model.name }}
                    <span class="text-xs font-normal text-surface-500">{{ model.size }}</span>
                  </div>
                  <div class="text-xs mt-0.5 line-clamp-1 text-surface-400">{{ model.description }}</div>
                  <div class="flex flex-wrap gap-1 mt-1.5">
                    <span v-for="tag in model.tags" :key="tag" class="px-1.5 py-0.5 text-xs font-medium rounded" :class="getTagClasses(tag)">
                      {{ tag }}
                    </span>
                  </div>
                </div>
                <div class="shrink-0">
                  <span v-if="model.isDownloaded" class="flex items-center justify-center w-6 h-6 text-green-400 rounded-full bg-green-900/50">
                    <fa :icon="['fal', 'check']" size="sm" />
                  </span>
                  <span v-else class="flex items-center justify-center w-6 h-6 rounded-full text-surface-400 bg-surface-700">
                    <fa :icon="['fal', 'download']" size="sm" />
                  </span>
                </div>
              </div>
            </div>
          </template>
        </div>
      </template>
    </WorkspaceActions>

    <!-- Right Panel: Model Details -->
    <div class="p-4">
      <main class="flex-1 h-full p-6 overflow-y-auto rounded-xl bg-surface-950">
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
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import InputText from 'primevue/inputtext';
import Select from 'primevue/select';
import Button from 'primevue/button';
import ProgressBar from 'primevue/progressbar';
import { useModelsStore } from '@/stores/models';
import { useAutoTagStore } from '@/stores/autoTag';
import WorkspaceActions from '@/components/shared/WorkspaceActions.vue';

interface ComponentInfo {
  id: string;
  name: string;
  description: string;
  is_downloaded: boolean;
  size_estimate: string;
  has_quantized: boolean;
  quantized_downloaded: boolean;
}

interface ModelInfo {
  id: string;
  name: string;
  description: string;
  size: string;
  type: string;
  category: 'generation' | 'vision' | 'component';
  categoryLabel: string;
  format: string;
  license: string;
  source: string;
  tags: string[];
  icon: string;
  iconClass: string;
  isDownloaded: boolean;
  docsUrl?: string;
  defaultSettings?: {
    steps: number;
    guidance: number;
  };
  features?: string[];
  notes?: string;
  hasQuantized?: boolean;
  quantizedDownloaded?: boolean;
}

const modelsStore = useModelsStore();
const autoTagStore = useAutoTagStore();

const searchQuery = ref('');
const categoryFilter = ref('all');
const selectedModel = ref<ModelInfo | null>(null);
const isDownloadingSchnell = ref(false);
const isDownloadingDev = ref(false);
const componentAvailability = ref<ComponentInfo[]>([]);

const categoryOptions = [
  { label: 'All Models', value: 'all' },
  { label: 'Image Generation', value: 'generation' },
  { label: 'Vision & Analysis', value: 'vision' },
  { label: 'Encoders & Components', value: 'component' },
];

// Helper functions for dynamic classes
const getIconClasses = (iconClass: string): string => {
  if (iconClass === 'icon-flux') return 'bg-purple-900/50 text-purple-400';
  if (iconClass === 'icon-vision') return 'bg-emerald-900/50 text-emerald-400';
  if (iconClass === 'icon-vae') return 'bg-cyan-900/50 text-cyan-400';
  if (iconClass === 'icon-clip') return 'bg-orange-900/50 text-orange-400';
  if (iconClass === 'icon-t5') return 'bg-indigo-900/50 text-indigo-400';
  return 'bg-surface-700 text-surface-300';
};

const getDetailIconClasses = (iconClass: string): string => {
  if (iconClass === 'icon-flux') return 'bg-purple-900/30 text-purple-400';
  if (iconClass === 'icon-vision') return 'bg-emerald-900/30 text-emerald-400';
  if (iconClass === 'icon-vae') return 'bg-cyan-900/30 text-cyan-400';
  if (iconClass === 'icon-clip') return 'bg-orange-900/30 text-orange-400';
  if (iconClass === 'icon-t5') return 'bg-indigo-900/30 text-indigo-400';
  return 'bg-surface-800 text-surface-400';
};

const getTagClasses = (tag: string): string => {
  const tagLower = tag.toLowerCase();
  if (tagLower === 'flux') return 'bg-purple-900/50 text-purple-300';
  if (tagLower === 'fast' || tagLower === 'hq') return 'bg-blue-900/50 text-blue-300';
  if (tagLower === 'quantized') return 'bg-amber-900/50 text-amber-300';
  if (tagLower === 'vision' || tagLower === 'vlm') return 'bg-emerald-900/50 text-emerald-300';
  if (tagLower === 'vae' || tagLower === 'encoder') return 'bg-cyan-900/50 text-cyan-300';
  if (tagLower === 'clip') return 'bg-orange-900/50 text-orange-300';
  if (tagLower === 't5' || tagLower === 'text') return 'bg-indigo-900/50 text-indigo-300';
  if (tagLower === 'shared') return 'bg-surface-600 text-surface-300';
  return 'bg-surface-700 text-surface-300';
};

// Helper to find component by ID
const getComponent = (id: string): ComponentInfo | undefined => {
  return componentAvailability.value.find((c) => c.id === id);
};

// Load models from database
const dbModels = ref<ModelInfo[]>([]);

// Build unified model list by merging database and runtime status
const allModels = computed<ModelInfo[]>(() => {
  return dbModels.value.map((model) => {
    // Update download status for generation models from store
    if (model.id === 'schnell') {
      const schnellDownloaded = modelsStore.models.find((m) => m.id === 'schnell')?.isDownloaded ?? false;
      return { ...model, isDownloaded: schnellDownloaded };
    }
    if (model.id === 'dev') {
      const devDownloaded = modelsStore.models.find((m) => m.id === 'dev')?.isDownloaded ?? false;
      return { ...model, isDownloaded: devDownloaded };
    }
    if (model.id === 'moondream') {
      return { ...model, isDownloaded: autoTagStore.isLocalAvailable };
    }

    // Update component availability from runtime check
    if (model.category === 'component') {
      const component = getComponent(model.id);
      if (component) {
        const updatedModel = {
          ...model,
          isDownloaded: component.is_downloaded,
          hasQuantized: component.has_quantized,
          quantizedDownloaded: component.quantized_downloaded,
          size: component.size_estimate,
        };

        // Update T5 format and tags based on quantization status
        if (model.id === 't5' && component.quantized_downloaded) {
          updatedModel.format = 'GGUF (Quantized)';
          if (!updatedModel.tags.includes('QUANTIZED')) {
            updatedModel.tags = [...updatedModel.tags, 'QUANTIZED'];
          }
        }

        return updatedModel;
      }
    }

    return model;
  });
});

const filteredModels = computed(() => {
  return allModels.value.filter((model) => {
    const matchesSearch =
      searchQuery.value === '' ||
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
    { category: 'component', label: 'Encoders & Components', models: [] as ModelInfo[] },
  ];

  for (const model of filteredModels.value) {
    const group = groups.find((g) => g.category === model.category);
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
      selectedModel.value = allModels.value.find((m) => m.id === model.id) ?? null;
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

// Fetch component availability from backend
const fetchComponentAvailability = async () => {
  try {
    const components = await invoke<ComponentInfo[]>('get_component_availability');
    componentAvailability.value = components;
  } catch (error) {
    console.error('Failed to fetch component availability:', error);
  }
};

// Load models from database
const loadModelsFromDatabase = async () => {
  try {
    const models = await invoke<ModelInfo[]>('get_all_models');
    dbModels.value = models;
  } catch (error) {
    console.error('Failed to load models from database:', error);
    // Fallback to legacy models if database fails
    dbModels.value = [];
  }
};

onMounted(async () => {
  await Promise.all([loadModelsFromDatabase(), modelsStore.refreshModelAvailability(), autoTagStore.checkModelStatus(), fetchComponentAvailability()]);
  // Auto-select first model
  if (allModels.value.length > 0) {
    selectedModel.value = allModels.value[0];
  }
});
</script>
