<template>
  <GenerationAction :collapsed="props.collapsed" :toggleable="props.toggleable" :icon="props.icon" :label="props.label">
    <!-- Loading State -->
    <div v-if="modelsStore.lorasLoading" class="flex items-center justify-center py-4 text-surface-400">
      <i class="mr-2 pi pi-spin pi-spinner"></i>
      Loading LoRAs...
    </div>

    <!-- Error State -->
    <div v-else-if="modelsStore.lorasError" class="p-3 text-sm text-red-400 rounded bg-red-500/10">
      {{ modelsStore.lorasError }}
    </div>

    <!-- Empty State -->
    <div v-else-if="modelsStore.loras.length === 0" class="flex flex-col items-center gap-2 py-4 text-center text-surface-400">
      <fa :icon="['fal', 'layer-group']" class="opacity-50" size="2x" />
      <p class="text-sm">No LoRAs imported</p>
      <Button label="Import LoRA" icon="pi pi-plus" size="small" outlined @click="showImportDialog = true" />
    </div>

    <!-- LoRA List -->
    <template v-else>
      <ul role="list" class="divide-y divide-surface-600">
        <li v-for="lora in modelsStore.loras" :key="lora.id" class="flex flex-col gap-1 p-2">
          <!-- Header Row -->
          <div class="flex flex-row items-center gap-2">
            <ToggleSwitch v-model="lora.isActive" :input-id="lora.id" binary @change="handleToggle(lora.id)" class="shrink" />
            <div class="text-base font-medium text-surface-200 grow line-clamp-1" :title="lora.name">
              {{ lora.name }}
            </div>
            <Button severity="danger" @click="handleDelete(lora.id)" variant="outlined">
              <fa :icon="['fal', 'trash']" />
            </Button>
          </div>

          <div class="grid grid-cols-8 gap-2">
            <!-- Strength Slider (only when active) -->
            <div class="flex flex-col col-span-8">
              <label class="mb-1 text-xs font-medium tracking-wide text-surface-300">Strength</label>
              <InputGroup>
                <InputGroupAddon>
                  <fa :icon="['fal', 'dial']" size="lg" />
                </InputGroupAddon>
                <InputNumber
                  v-model="lora.strength"
                  :use-grouping="false"
                  :min="0"
                  :max="2"
                  :step="0.1"
                  fluid
                  input-class="flex items-center font-mono text-base leading-none text-center" />
                <InputGroupAddon><fa :icon="['fal', 'rotate-left']" class="cursor-pointer" v-tooltip="'Reset'" @click="lora.strength = 1" /></InputGroupAddon>
              </InputGroup>
            </div>

            <!-- Trigger Words -->
            <div class="flex flex-col col-span-8">
              <label class="mb-1 text-xs font-medium tracking-wide text-surface-300">Trigger</label>
              <InputGroup>
                <InputGroupAddon>
                  <fa :icon="['fal', 'tag']" />
                </InputGroupAddon>
                <InputText v-model="lora.triggerWords" :readonly="true" fluid class="flex items-center font-mono text-base leading-none text-center" />
                <InputGroupAddon>
                  <fa :icon="['fal', 'trash']" class="cursor-pointer" v-tooltip="'Copy trigger words'" @click="copyTriggerWords(lora.triggerWords!)" />
                </InputGroupAddon>
              </InputGroup>
            </div>
          </div>
        </li>
      </ul>
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
import InputGroup from 'primevue/inputgroup';
import InputGroupAddon from 'primevue/inputgroupaddon';
import GenerationAction from './GenerationAction.vue';
import Checkbox from 'primevue/checkbox';
import ToggleSwitch from 'primevue/toggleswitch';
import InputNumber from 'primevue/inputnumber';
import Button from 'primevue/button';
import { useModelsStore } from '@/stores/models';
import LoraImportDialog from './LoraImportDialog.vue';
import InputText from 'primevue/inputtext';

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

<style scoped>
@reference "tailwindcss";

:deep(.p-inputtext) {
  @apply px-2 py-1 h-9;
}
</style>
