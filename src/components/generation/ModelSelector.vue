<template>
  <div class="model-selector">
    <div class="field">
      <label for="model">Model</label>
      <Select
        id="model"
        v-model="modelsStore.selectedModelId"
        :options="modelsStore.models"
        option-label="name"
        option-value="id"
        placeholder="Select a model"
        class="w-full">
        <template #option="slotProps">
          <div class="model-option">
            <span class="model-name">{{ slotProps.option.name }}</span>
            <span v-if="!slotProps.option.isDownloaded" class="model-badge not-downloaded">
              Not Downloaded
            </span>
          </div>
        </template>
      </Select>
    </div>

    <div v-if="modelsStore.activeModel" class="model-info">
      <span v-if="modelsStore.activeModel.description" class="info-desc">
        {{ modelsStore.activeModel.description }}
      </span>
      <div class="info-stats">
        <span>{{ modelsStore.activeModel.defaultSteps }} steps</span>
        <span>CFG {{ modelsStore.activeModel.defaultGuidance }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useModelsStore } from '@/stores/models';
import Select from 'primevue/select';

const modelsStore = useModelsStore();
</script>

<style scoped>
@reference "tailwindcss";

.model-selector {
  @apply flex flex-col gap-2;
}

.field {
  @apply flex flex-col gap-2;
}

.field label {
  @apply text-xs font-medium tracking-wide;
  color: #64748b;
}

.model-option {
  @apply flex items-center justify-between w-full;
}

.model-name {
  @apply font-medium;
  color: #e2e8f0;
}

.model-badge {
  @apply text-xs px-2 py-0.5 rounded;
}

.model-badge.not-downloaded {
  @apply bg-amber-500/20 text-amber-400;
}

.model-info {
  @apply flex flex-col gap-1 text-xs p-2 rounded-lg;
  background: #2d3748;
  color: #94a3b8;
}

.info-desc {
  @apply italic;
}

.info-stats {
  @apply flex gap-3;
  color: #38bdf8;
}

/* PrimeVue Select dark theme */
:deep(.p-select) {
  background: #2d3748;
  border-color: #374151;
  color: #e2e8f0;
}

:deep(.p-select-label) {
  color: #e2e8f0;
}

:deep(.p-select-dropdown) {
  color: #94a3b8;
}
</style>
