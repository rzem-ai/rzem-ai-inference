<template>
  <Dialog v-model:visible="isVisible" modal header="Import LoRA" :style="{ width: '450px' }">
    <div class="flex flex-col gap-4">
      <!-- File Selection -->
      <div class="flex flex-col gap-2">
        <label class="text-sm font-medium text-gray-300">LoRA File</label>
        <div class="flex gap-2">
          <InputText v-model="selectedPath" placeholder="Select .safetensors file" class="flex-1" readonly />
          <Button icon="pi pi-folder-open" @click="selectFile" />
        </div>
      </div>

      <!-- File Info Preview -->
      <div v-if="fileInfo" class="p-3 text-sm rounded bg-surface-800">
        <div class="grid grid-cols-2 gap-2">
          <span class="text-gray-400">Size:</span>
          <span class="text-gray-200">{{ formatFileSize(fileInfo.sizeBytes) }}</span>
          <span class="text-gray-400">Weights:</span>
          <span class="text-gray-200">{{ fileInfo.weightCount }} layers</span>
          <span class="text-gray-400">Rank:</span>
          <span class="text-gray-200">{{ fileInfo.rank ?? 'Unknown' }}</span>
          <span class="text-gray-400">Parameters:</span>
          <span class="text-gray-200">{{ formatParams(fileInfo.totalParams) }}</span>
        </div>
      </div>

      <!-- Name Input -->
      <div class="flex flex-col gap-2">
        <label class="text-sm font-medium text-gray-300">Display Name</label>
        <InputText v-model="loraName" placeholder="My Custom LoRA" />
      </div>

      <!-- Trigger Words -->
      <div class="flex flex-col gap-2">
        <label class="text-sm font-medium text-gray-300">Trigger Words (optional)</label>
        <InputText v-model="triggerWords" placeholder="e.g., style_name, character_name" />
        <small class="text-gray-500">Words to include in prompt to activate this LoRA's style</small>
      </div>

      <!-- Error Message -->
      <div v-if="error" class="p-3 text-sm text-red-400 rounded bg-red-500/10">
        {{ error }}
      </div>
    </div>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button label="Cancel" severity="secondary" @click="close" />
        <Button label="Import" icon="pi pi-download" :loading="isImporting" :disabled="!canImport" @click="importLora" />
      </div>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import Dialog from 'primevue/dialog';
import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import { useModelsStore } from '@/stores/models';
import type { LoraFileInfo } from '@/types';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  'update:visible': [value: boolean];
  imported: [];
}>();

const modelsStore = useModelsStore();

const isVisible = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value),
});

const selectedPath = ref('');
const loraName = ref('');
const triggerWords = ref('');
const fileInfo = ref<LoraFileInfo | null>(null);
const isImporting = ref(false);
const error = ref<string | null>(null);

const canImport = computed(() => {
  return selectedPath.value && loraName.value.trim() && fileInfo.value && !isImporting.value;
});

// Reset form when dialog opens
watch(isVisible, (visible) => {
  if (visible) {
    selectedPath.value = '';
    loraName.value = '';
    triggerWords.value = '';
    fileInfo.value = null;
    error.value = null;
  }
});

const selectFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'LoRA Files',
          extensions: ['safetensors'],
        },
      ],
    });

    if (selected) {
      selectedPath.value = selected as string;
      error.value = null;

      // Auto-fill name from filename
      const filename = selectedPath.value.split(/[/\\]/).pop() || '';
      const nameWithoutExt = filename.replace(/\.safetensors$/i, '');
      loraName.value = nameWithoutExt.replace(/_/g, ' ').replace(/-/g, ' ');

      // Get file info
      try {
        fileInfo.value = await modelsStore.getLoraFileInfo(selectedPath.value);
      } catch (e) {
        error.value = `Failed to read LoRA file: ${e}`;
        fileInfo.value = null;
      }
    }
  } catch (e) {
    error.value = `Failed to open file dialog: ${e}`;
  }
};

const importLora = async () => {
  if (!canImport.value) return;

  isImporting.value = true;
  error.value = null;

  try {
    await modelsStore.importLora(selectedPath.value, loraName.value.trim(), triggerWords.value.trim() || undefined);
    emit('imported');
    close();
  } catch (e) {
    error.value = `Import failed: ${e}`;
  } finally {
    isImporting.value = false;
  }
};

const close = () => {
  isVisible.value = false;
};

const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
};

const formatParams = (params: number): string => {
  if (params < 1000) return `${params}`;
  if (params < 1000000) return `${(params / 1000).toFixed(1)}K`;
  return `${(params / 1000000).toFixed(1)}M`;
};
</script>
