<template>
  <GenerationAction :collapsed="props.collapsed" :toggleable="props.toggleable" :icon="props.icon" :label="props.label">
    <!-- Loading State -->
    <div v-if="modelsStore.lorasLoading" class="flex items-center justify-center py-4 text-gray-400">
      <i class="mr-2 pi pi-spin pi-spinner"></i>
      Loading LoRAs...
    </div>

    <!-- Error State -->
    <div v-else-if="modelsStore.lorasError" class="p-3 text-sm text-red-400 rounded bg-red-500/10">
      {{ modelsStore.lorasError }}
    </div>

    <!-- Empty State -->
    <div v-else-if="modelsStore.loras.length === 0" class="flex flex-col items-center gap-2 py-4 text-center text-gray-400">
      <fa :icon="['fal', 'layer-group']" class="opacity-50" size="2x" />
      <p class="text-sm">No LoRAs imported</p>
      <Button label="Import LoRA" icon="pi pi-plus" size="small" outlined @click="showImportDialog = true" />
    </div>

    <!-- LoRA List -->
    <template v-else>
      <div v-for="lora in modelsStore.loras" :key="lora.id" class="flex flex-col gap-2 p-2 rounded bg-surface-800">
        <!-- Header Row -->
        <div class="flex items-center gap-2">
          <Checkbox v-model="lora.isActive" :input-id="lora.id" binary @change="handleToggle(lora.id)" />
          <label :for="lora.id" class="flex-1 text-sm font-medium text-gray-200 cursor-pointer">
            {{ lora.name }}
          </label>
          <Button icon="pi pi-trash" severity="danger" text size="small" @click="handleDelete(lora.id)" />
        </div>

        <!-- Strength Slider (only when active) -->
        <div v-if="lora.isActive" class="flex items-center gap-2">
          <span class="text-xs text-gray-400 w-14">Strength</span>
          <Slider v-model="lora.strength" :min="0" :max="2" :step="0.05" class="flex-1" @change="handleStrengthChange(lora.id, lora.strength)" />
          <InputNumber
            v-model="lora.strength"
            :min="0"
            :max="2"
            :step="0.05"
            size="small"
            input-class="text-xs w-14"
            @update:model-value="(v) => handleStrengthChange(lora.id, v ?? 1)" />
        </div>

        <!-- Trigger Words -->
        <div v-if="lora.triggerWords" class="flex items-center gap-2">
          <span class="text-xs text-gray-500">Trigger:</span>
          <code class="flex-1 px-2 py-1 text-xs truncate rounded bg-surface-700 text-amber-300">
            {{ lora.triggerWords }}
          </code>
          <Button icon="pi pi-copy" text size="small" v-tooltip="'Copy trigger words'" @click="copyTriggerWords(lora.triggerWords!)" />
        </div>

        <!-- File Size -->
        <div class="text-xs text-gray-500">
          {{ formatFileSize(lora.sizeBytes) }}
        </div>
      </div>

      <!-- Import Button -->
      <Button label="Import LoRA" icon="pi pi-plus" size="small" outlined class="mt-2" @click="showImportDialog = true" />
    </template>

    <!-- Import Dialog -->
    <LoraImportDialog v-model:visible="showImportDialog" @imported="handleImported" />
  </GenerationAction>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useToast } from 'primevue/usetoast';
import GenerationAction from './GenerationAction.vue';
import Checkbox from 'primevue/checkbox';
import Slider from 'primevue/slider';
import InputNumber from 'primevue/inputnumber';
import Button from 'primevue/button';
import { useModelsStore } from '@/stores/models';
import LoraImportDialog from './LoraImportDialog.vue';

const props = defineProps(['collapsed', 'icon', 'label', 'toggleable']);

const modelsStore = useModelsStore();
const toast = useToast();

const showImportDialog = ref(false);

const activeLoras = computed(() => modelsStore.loras.filter((l) => l.isActive));

// Load LoRAs on mount
onMounted(() => {
  modelsStore.loadLoras();
});

const handleToggle = (id: string) => {
  modelsStore.toggleLora(id);
};

const handleStrengthChange = (id: string, strength: number) => {
  modelsStore.updateLoraStrength(id, strength);
};

const handleDelete = async (id: string) => {
  try {
    await modelsStore.removeLora(id);
    toast.add({
      severity: 'success',
      summary: 'LoRA Removed',
      detail: 'LoRA has been deleted',
      life: 3000,
    });
  } catch (error) {
    toast.add({
      severity: 'error',
      summary: 'Delete Failed',
      detail: String(error),
      life: 5000,
    });
  }
};

const handleImported = () => {
  toast.add({
    severity: 'success',
    summary: 'LoRA Imported',
    detail: 'LoRA has been added to your collection',
    life: 3000,
  });
};

const copyTriggerWords = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text);
    toast.add({
      severity: 'info',
      summary: 'Copied',
      detail: 'Trigger words copied to clipboard',
      life: 2000,
    });
  } catch (error) {
    console.error('Failed to copy:', error);
  }
};

const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
};
</script>
