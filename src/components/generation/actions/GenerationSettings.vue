<template>
  <GenerationAction :icon="props.icon" :label="props.label">
    <div class="flex flex-col gap-4">
      <!-- Quality (Steps) -->
      <div class="flex flex-col">
        <div class="flex items-center justify-between pr-2 mb-1">
          <div class="flex items-center gap-1.5">
            <label class="text-xs font-medium tracking-wide text-surface-300">Quality</label>
            <fa
              :icon="['fal', 'circle-info']"
              size="sm"
              class="transition-colors text-surface-400 hover:text-blue-500 cursor-help"
              v-tooltip.top="'Number of denoising steps (1-50). Higher values produce more refined images but take longer to generate.'" />
          </div>
          <span class="font-mono text-xs font-semibold text-blue-600">{{ steps }}</span>
        </div>
        <Slider v-model="steps" :min="1" :max="50" />
      </div>

      <!-- Prompt Adherence (CFG Scale) -->
      <div class="flex flex-col">
        <div class="flex items-center justify-between pr-2 mb-1">
          <div class="flex items-center gap-1.5">
            <label class="text-xs font-medium tracking-wide text-surface-300">Prompt Adherence</label>
            <fa
              :icon="['fal', 'circle-info']"
              size="sm"
              class="transition-colors text-surface-400 hover:text-blue-500 cursor-help"
              v-tooltip.top="
                'Classifier-Free Guidance Scale (0-20). Controls how strictly the model follows your prompt. Higher = more literal, Lower = more creative.'
              " />
          </div>
          <span class="font-mono text-xs font-semibold text-blue-600">{{ cfg_scale.toFixed(1) }}</span>
        </div>
        <Slider v-model="cfg_scale" :min="0" :max="20" :step="0.1" />
      </div>
    </div>
  </GenerationAction>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import Slider from 'primevue/slider';
import { useGenerationStore } from '@/stores/generation';
import GenerationAction from './GenerationAction.vue';

const props = defineProps(['icon', 'label']);

const generationStore = useGenerationStore();

// Computed properties bound to store
const steps = computed({
  get: () => generationStore.currentParams.steps,
  set: (value: number | null) => {
    generationStore.currentParams.steps = value ?? 4;
  },
});

const cfg_scale = computed({
  get: () => generationStore.currentParams.cfg_scale,
  set: (value: number | null) => {
    generationStore.currentParams.cfg_scale = value ?? 1.0;
  },
});
</script>
