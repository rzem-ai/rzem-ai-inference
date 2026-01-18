<template>
  <div class="model-selector">
    <div class="field">
      <label for="model">Model</label>
      <Select
        id="model"
        v-model="modelsStore.selectedModelId"
        :options="modelsStore.downloadedModels"
        option-label="name"
        option-value="id"
        placeholder="Select a model"
        class="w-full" />
    </div>

    <div v-if="modelsStore.activeModel" class="model-info">
      <span class="info-label">Type:</span>
      <span class="info-value">{{ modelsStore.activeModel.type }}</span>
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

.model-info {
  @apply flex gap-2 text-xs text-gray-500;
}

.info-label {
  @apply font-medium;
}
</style>
