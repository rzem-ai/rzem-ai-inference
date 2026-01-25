<template>
  <GenerationAction :collapsed="props.collapsed" :toggleable="props.toggleable" :icon="props.icon" :label="props.label">
    <Select
      id="model"
      v-model="selectedModel"
      :options="modelsStore.models"
      option-label="name"
      option-value="id"
      placeholder="Select a model"
      size="small"
      class="w-full">
      <template #option="slotProps">
        <div class="flex items-center justify-between w-full">
          <span class="font-medium">{{ slotProps.option?.name }}</span>
          <span v-if="!slotProps.option.isDownloaded" class="rounded bg-amber-500/20 px-2 py-0.5 text-xs text-amber-400"> Not Downloaded </span>
        </div>
      </template>
    </Select>
    <div v-if="modelsStore.activeModel" class="flex flex-col gap-1 p-2 text-xs text-gray-400 rounded bg-surface-800">
      <span v-if="modelsStore.activeModel.description" class="italic">
        {{ modelsStore.activeModel.description }}
      </span>
      <div class="flex gap-3 text-sky-400">
        <span>{{ modelsStore.activeModel.defaultSteps }} steps</span>
        <span>CFG {{ modelsStore.activeModel.defaultGuidance }}</span>
      </div>
    </div>
  </GenerationAction>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useModelsStore } from '@/stores/models';
import { useGenerationStore } from '@/stores/generation';
import GenerationAction from './GenerationAction.vue';
import Select from 'primevue/select';

const props = defineProps(['collapsed', 'icon', 'label', 'toggleable']);

const modelsStore = useModelsStore();
const generationStore = useGenerationStore();

// Model selector - syncs both stores
const selectedModel = computed({
  get: () => {
    // Use the value from generation store as the source of truth
    return generationStore.currentParams.model || modelsStore.selectedModelId;
  },
  set: (value: string) => {
    // Update both stores
    modelsStore.selectedModelId = value;
    generationStore.currentParams.model = value;
  },
});
</script>
