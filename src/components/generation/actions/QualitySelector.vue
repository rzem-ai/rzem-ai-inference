<template>
  <GenerationAction :collapsed="false" :toggleable="false" icon="star" label="Image Quality">
    <!-- Enhanced Model Selector with Bundles -->
    <EnhancedModelSelector />

    <!-- Quality (Steps) -->
    <div class="flex flex-col">
      <div class="flex items-center justify-between pr-2 mb-1">
        <div class="flex items-center">
          <label class="text-sm font-medium tracking-wide text-surface-300">Quality</label>
        </div>
        <fa
          :icon="['fal', 'circle-info']"
          size="sm"
          class="transition-colors text-surface-400 hover:text-blue-500 cursor-help"
          v-tooltip.top="'Number of denoising steps (1-50). Higher values produce more refined images but take longer to generate.'" />
      </div>
      <div class="grid grid-cols-6 gap-1">
        <Slider v-model="steps" :min="1" :max="50" :step="1" class="col-span-5 border shadow border-surface-600" />
        <InputNumber :model-value="steps" size="small" class="col-span-1" />
      </div>
    </div>

    <!-- Prompt Adherence (CFG Scale) -->
    <div class="flex flex-col">
      <div class="flex items-center justify-between pr-2 mb-1">
        <div class="flex items-center gap-1.5">
          <label class="text-sm font-medium tracking-wide text-surface-300">Prompt Adherence</label>
        </div>
        <fa
          :icon="['fal', 'circle-info']"
          size="sm"
          class="transition-colors text-surface-400 hover:text-blue-500 cursor-help"
          v-tooltip.top="
            'Classifier-Free Guidance Scale (0-20). Controls how strictly the model follows your prompt. Higher = more literal, Lower = more creative.'
          " />
      </div>
      <div class="grid grid-cols-6 gap-1">
        <Slider v-model="cfgScale" :min="0" :max="20" :step="0.1" class="col-span-5 border shadow border-surface-600" />
        <InputNumber :model-value="cfgScale" size="small" class="col-span-1" />
      </div>
    </div>
  </GenerationAction>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useGenerationStore } from '@/stores/generation';
import GenerationAction from './GenerationAction.vue';
import EnhancedModelSelector from './EnhancedModelSelector.vue';
import InputNumber from 'primevue/inputnumber';
import Slider from 'primevue/slider';

const generationStore = useGenerationStore();

// Computed properties bound to store
const steps = computed({
  get: () => generationStore.currentParams.steps,
  set: (value: number | null) => {
    generationStore.currentParams.steps = value ?? 4;
  },
});

const cfgScale = computed({
  get: () => generationStore.currentParams.cfgScale,
  set: (value: number | null) => {
    generationStore.currentParams.cfgScale = value ?? 1.0;
  },
});
</script>

<style scoped>
@reference "tailwindcss";

:deep(.p-inputnumber-input) {
  @apply px-1 text-center;
  width: 3rem !important;
}
</style>
