<template>
  <div class="flex h-full">
    <!-- Models List -->
    <div class="w-80 flex flex-col border-r border-surface-700 bg-surface-900">
      <div class="p-3 border-b border-surface-700 flex items-center justify-between">
        <span class="text-sm font-semibold text-surface-300">Models</span>
        <Button severity="secondary" size="small" @click="scanModels" :loading="modelsStore.scanning">
          <template #icon><fa :icon="['fal', 'magnifying-glass']" /></template>
        </Button>
      </div>

      <div v-if="Object.keys(modelsStore.modelsByType).length === 0" class="flex-1 flex items-center justify-center">
        <p class="text-sm text-surface-500 text-center px-4">No models found.<br />Scan a directory to discover models.</p>
      </div>

      <div v-else class="flex-1 overflow-y-auto">
        <template v-for="type in modelsStore.typeOrder" :key="type">
          <div v-if="modelsStore.modelsByType[type]">
            <!-- Type Group Header -->
            <div class="px-3 py-1.5 bg-surface-800 sticky top-0">
              <span class="text-xs font-semibold text-surface-400 uppercase tracking-wider">{{ typeLabel(type) }}</span>
              <span class="text-xs text-surface-600 ml-2">({{ modelsStore.modelsByType[type].length }})</span>
            </div>

            <!-- Models in this group -->
            <div
              v-for="model in modelsStore.modelsByType[type]"
              :key="model.id"
              class="px-3 py-2 border-b border-surface-800 cursor-pointer hover:bg-surface-800 transition-colors"
              :class="{ 'bg-surface-800': modelsStore.selectedModel?.id === model.id }"
              @click="modelsStore.selectModel(model)"
            >
              <div class="flex items-center justify-between">
                <span class="text-sm text-surface-200 truncate">{{ model.displayName }}</span>
                <Tag v-if="model.quantization" :value="model.quantization" severity="info" class="text-xs" />
              </div>
              <div class="flex items-center gap-2 mt-0.5">
                <span class="text-xs text-surface-500">{{ model.family }}</span>
                <span v-if="model.vramMb" class="text-xs text-surface-500">• {{ formatVram(model.vramMb) }}</span>
              </div>
            </div>
          </div>
        </template>
      </div>
    </div>

    <!-- Model Details Panel -->
    <div class="flex-1 p-4 overflow-y-auto">
      <div v-if="modelsStore.selectedModel">
        <ModelDetails :model="modelsStore.selectedModel" />
      </div>
      <div v-else class="flex items-center justify-center h-full">
        <p class="text-sm text-surface-500">Select a model to view details.</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useModelsStore } from '@/stores/models';
import { Button, Tag } from 'primevue';
import ModelDetails from '@/components/models/ModelDetails.vue';

const modelsStore = useModelsStore();

onMounted(() => {
  modelsStore.loadModels();
});

const TYPE_LABELS: Record<string, string> = {
  checkpoint: 'Base Checkpoints',
  text_encoder: 'Text Encoders',
  vae: 'VAE Decoders',
  tokenizer: 'Tokenizers',
  lora: 'LoRA Adapters',
  scheduler: 'Schedulers',
};

function typeLabel(type: string): string {
  return TYPE_LABELS[type] ?? type;
}

function formatVram(vramMb: number): string {
  return vramMb >= 1000 ? `${(vramMb / 1000).toFixed(1)} GB` : `${vramMb} MB`;
}

async function scanModels() {
  // Use native folder picker via Tauri dialog plugin
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const path = await open({ directory: true, title: 'Select directory to scan for models' });
    if (path) {
      await modelsStore.scanDirectory(path as string);
    }
  } catch (e) {
    console.error('Failed to open directory picker:', e);
  }
}
</script>
