<template>
  <div class="flex flex-col">
    <div class="">
      <Button size="small" class="w-full" :loading="isScanning" @click="handleScanModels">
        <div class="gap-2"> <fa :icon="['fal', 'barcode-read']" size="sm" /> Scan Models </div>
      </Button>
      <div v-if="scanProgress" class="px-2 py-1 mt-1 text-xs rounded text-surface-300 bg-surface-800">
        {{ scanProgress.message }}
      </div>
    </div>

    <Divider />

    <!-- Search & Filter -->
    <div class="flex gap-2 px-4">
      <InputText v-model="searchQuery" placeholder="Search models..." class="flex-1" size="small" />
      <Select v-model="categoryFilter" :options="categoryOptions" optionLabel="label" optionValue="value" size="small" class="w-40" />
    </div>

    <Divider />

    <!-- Model Count -->
    <div class="flex items-center justify-between px-4 py-2">
      <span class="text-xs text-surface-400">{{ filteredModels.length }} models</span>
    </div>

    <Divider />

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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import Button from 'primevue/button';
import Divider from 'primevue/divider';
import InputText from 'primevue/inputtext';
import Select from 'primevue/select';
import { useModelsStore } from '@/stores/models';
import { useAutoTagStore } from '@/stores/autoTag';
import { useToast } from 'primevue/usetoast';

interface ComponentInfo {
  id: string;
  name: string;
  description: string;
  is_downloaded: boolean;
  size_estimate: string;
  has_quantized: boolean;
  quantized_downloaded: boolean;
}

// Database component record from backend
interface ComponentRecord {
  id: string;
  componentType: string;
  format: string;
  filePath: string;
  fileSize: number;
  fileHash?: string;
  name: string;
  repoId?: string;
  repoSnapshot?: string;
  architecture?: string;
  quantization?: string;
  supportsLoras: boolean;
  isSharded: boolean;
  shardCount?: number;
  vramMb?: number;
  discoveredAt: number;
  lastVerifiedAt?: number;
  isAvailable: boolean;
  metadata?: Record<string, unknown>;
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
const toast = useToast();

const searchQuery = ref('');
const categoryFilter = ref('all');
const selectedModel = ref<ModelInfo | null>(null);
const componentAvailability = ref<ComponentInfo[]>([]);
const isScanning = ref(false);
const scanProgress = ref<{ stage: string; message: string; progress: number } | null>(null);

// Expose selectedModel for parent to access
defineExpose({ selectedModel });

const categoryOptions = [
  { label: 'All Models', value: 'all' },
  { label: 'Image Generation', value: 'generation' },
  { label: 'Vision & Analysis', value: 'vision' },
  { label: 'Encoders & Components', value: 'component' },
];

// Helper functions for icons and tags
function getIconClasses(iconClass: string): string {
  if (iconClass === 'icon-flux') return 'bg-purple-900/50 text-purple-400';
  if (iconClass === 'icon-lora') return 'bg-pink-900/50 text-pink-400';
  if (iconClass === 'icon-vision') return 'bg-emerald-900/50 text-emerald-400';
  if (iconClass === 'icon-vae') return 'bg-cyan-900/50 text-cyan-400';
  if (iconClass === 'icon-clip') return 'bg-orange-900/50 text-orange-400';
  if (iconClass === 'icon-t5') return 'bg-indigo-900/50 text-indigo-400';
  return 'bg-surface-700 text-surface-300';
}

function getTagClasses(tag: string): string {
  const tagLower = tag.toLowerCase();
  if (tagLower === 'flux') return 'bg-purple-900/50 text-purple-300';
  if (tagLower === 'lora') return 'bg-pink-900/50 text-pink-300';
  if (tagLower === 'fast' || tagLower === 'hq') return 'bg-blue-900/50 text-blue-300';
  if (tagLower === 'quantized') return 'bg-amber-900/50 text-amber-300';
  if (tagLower === 'vision' || tagLower === 'vlm') return 'bg-emerald-900/50 text-emerald-300';
  if (tagLower === 'vae' || tagLower === 'encoder') return 'bg-cyan-900/50 text-cyan-300';
  if (tagLower === 'clip') return 'bg-orange-900/50 text-orange-300';
  if (tagLower === 't5' || tagLower === 'text') return 'bg-indigo-900/50 text-indigo-300';
  if (tagLower === 'shared') return 'bg-surface-600 text-surface-300';
  return 'bg-surface-700 text-surface-300';
}

// Helper to find component by ID
function getComponent(id: string): ComponentInfo | undefined {
  return componentAvailability.value.find((c) => c.id === id);
}

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

function selectModel(model: ModelInfo) {
  selectedModel.value = model;
  // Update the store so the detail panel can access the selected model
  modelsStore.setSelectedViewComponent(model);
}

// Load scanned components from database and transform to ModelInfo
async function loadComponentsFromDb() {
  try {
    const components = await invoke<ComponentRecord[]>('get_available_components');

    // Transform ComponentRecord to ModelInfo
    dbModels.value = components.map((comp): ModelInfo => {
      // Determine category based on component type
      let category: 'generation' | 'vision' | 'component' = 'component';
      let icon = 'cube';
      let iconClass = 'icon-default';

      if (comp.componentType === 'transformer') {
        category = 'generation';
        icon = 'wand-magic-sparkles';
        iconClass = 'icon-flux';
      } else if (comp.componentType === 'lora') {
        category = 'component';
        icon = 'layer-plus';
        iconClass = 'icon-lora';
      } else if (comp.componentType === 't5_encoder') {
        icon = 'text';
        iconClass = 'icon-t5';
      } else if (comp.componentType === 'clip_encoder') {
        icon = 'image';
        iconClass = 'icon-clip';
      } else if (comp.componentType === 'vae') {
        icon = 'compress';
        iconClass = 'icon-vae';
      }

      // Build tags
      const tags: string[] = [];
      if (comp.componentType === 'transformer') tags.push('FLUX');
      if (comp.componentType === 'lora') tags.push('LORA');
      if (comp.quantization) tags.push(comp.quantization);
      if (comp.supportsLoras) tags.push('LoRA-Compatible');
      if (comp.format) tags.push(comp.format.toUpperCase());

      // Format file size
      const sizeMb = comp.fileSize / (1024 * 1024);
      const sizeStr = sizeMb >= 1024 ? `${(sizeMb / 1024).toFixed(1)} GB` : `${sizeMb.toFixed(0)} MB`;

      return {
        id: comp.id,
        name: comp.name,
        description: comp.architecture || comp.componentType,
        size: sizeStr,
        type: comp.componentType,
        category,
        categoryLabel: category === 'generation' ? 'Image Generation' : 'Encoders & Components',
        format: comp.format,
        license: 'Apache 2.0',
        source: comp.repoId || 'local',
        tags,
        icon,
        iconClass,
        isDownloaded: comp.isAvailable,
      };
    });
  } catch (error) {
    console.error('Failed to load components from database:', error);
  }
}

interface ScanResult {
  components_found: number;
  components_added: number;
  bundles_created: number;
}

interface ScanProgressEvent {
  stage: string;
  message: string;
  progress: number;
  filesFound?: number;
  filesProcessed?: number;
}

const unlisteners: UnlistenFn[] = [];

async function handleScanModels() {
  // Open directory picker
  const selectedDir = await open({
    directory: true,
    multiple: false,
    title: 'Select folder to scan for models',
  });

  if (!selectedDir) {
    return; // User cancelled
  }

  isScanning.value = true;
  scanProgress.value = { stage: 'starting', message: 'Starting scan...', progress: 0 };

  // Listen for progress events
  const unlisten = await listen<ScanProgressEvent>('model-scan-progress', (event) => {
    scanProgress.value = {
      stage: event.payload.stage,
      message: event.payload.message,
      progress: event.payload.progress,
    };
  });

  try {
    const result = await invoke<ScanResult>('scan_directory_for_models', {
      directoryPath: selectedDir as string,
    });

    // Show result toast
    if (result.components_found > 0) {
      toast.add({
        severity: 'success',
        summary: 'Scan Complete',
        detail: `Found ${result.components_found} components, added ${result.components_added} new`,
        life: 5000,
      });

      // Refresh model list
      await Promise.all([modelsStore.refreshModelAvailability(), loadComponentsFromDb()]);
    } else {
      toast.add({
        severity: 'info',
        summary: 'No Models Found',
        detail: 'No model files were found in the selected directory',
        life: 5000,
      });
    }
  } catch (error) {
    console.error('Scan failed:', error);
    toast.add({
      severity: 'error',
      summary: 'Scan Failed',
      detail: String(error),
      life: 8000,
    });
  } finally {
    unlisten();
    isScanning.value = false;
    scanProgress.value = null;
  }
}

onMounted(async () => {
  await Promise.all([
    modelsStore.refreshModelAvailability(),
    autoTagStore.checkModelStatus(),
    loadComponentsFromDb(),
  ]);

  // Auto-select first model
  if (allModels.value.length > 0) {
    selectModel(allModels.value[0]);
  }
});

onUnmounted(() => {
  unlisteners.forEach((fn) => fn());
});
</script>
