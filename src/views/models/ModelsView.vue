<template>
  <div class="flex h-full">
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
import ModelDetails from '@/components/models/ModelDetails.vue';

const modelsStore = useModelsStore();

onMounted(() => {
  modelsStore.loadModels();
});
</script>
