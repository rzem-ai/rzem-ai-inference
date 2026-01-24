<template>
  <GenerationAction :collapsed="props.collapsed" :toggleable="props.toggleable" :icon="props.icon" :label="props.label">
    <div class="flex flex-col gap-4">
      <!-- -->
      <div class="flex flex-col">
        <div class="flex items-center justify-between pr-2 mb-1">
          <label class="text-xs font-medium tracking-wide text-surface-300">Steps</label>
          <span class="font-mono text-xs font-semibold text-blue-600">{{ steps }}</span>
        </div>
        <Slider v-model="steps" :min="1" :max="50" />
      </div>

      <div class="flex flex-col">
        <div class="flex items-center justify-between pr-2 mb-1">
          <label class="text-xs font-medium tracking-wide text-surface-300">CFG Scale</label>
          <span class="font-mono text-xs font-semibold text-blue-600">{{ cfgScale.toFixed(1) }}</span>
        </div>
        <Slider v-model="cfgScale" :min="0" :max="20" :step="0.1" />
      </div>
    </div>
  </GenerationAction>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import Slider from 'primevue/slider';
import { useGenerationStore } from '@/stores/generation';
import GenerationAction from './GenerationAction.vue';

const props = defineProps(['collapsed', 'icon', 'label', 'toggleable']);

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
