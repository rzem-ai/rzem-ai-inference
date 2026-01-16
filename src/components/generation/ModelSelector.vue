<script setup lang="ts">
import { useModelsStore } from '@/stores/models'
import Dropdown from 'primevue/dropdown'

const modelsStore = useModelsStore()
</script>

<template>
  <div class="model-selector">
    <div class="field">
      <label for="model">Model</label>
      <Dropdown
        id="model"
        v-model="modelsStore.selectedModelId"
        :options="modelsStore.downloadedModels"
        option-label="name"
        option-value="id"
        placeholder="Select a model"
        class="w-full"
      />
    </div>

    <div v-if="modelsStore.activeModel" class="model-info">
      <span class="info-label">Type:</span>
      <span class="info-value">{{ modelsStore.activeModel.type }}</span>
    </div>
  </div>
</template>

<style scoped>
.model-selector {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.field label {
  font-weight: 600;
  font-size: 0.875rem;
  color: #374151;
}

.model-info {
  display: flex;
  gap: 0.5rem;
  font-size: 0.75rem;
  color: #6b7280;
}

.info-label {
  font-weight: 500;
}
</style>
