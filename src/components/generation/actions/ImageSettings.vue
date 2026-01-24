<template>
  <GenerationAction :collapsed="props.collapsed" :toggleable="props.toggleable" :icon="props.icon" :label="props.label">
    <!-- Aspect Ratio Pills -->
    <div class="flex flex-col">
      <label class="text-xs font-medium tracking-wide text-surface-300">Aspect Ratio</label>
      <SelectButton
        v-model="activeRatio"
        :options="aspectRatios"
        option-label="label"
        option-value="label"
        data-key="label"
        size="small"
        fluid
        class="shaddow-xl">
        <template #option="slotProps">
          {{ slotProps.option.label }}
        </template>
      </SelectButton>
    </div>

    <!-- Width/Height Sliders -->
    <div class="grid grid-cols-2 gap-3">
      <div class="flex flex-col">
        <div class="flex items-center justify-between pr-2 mb-1">
          <label class="text-xs font-medium tracking-wide text-surface-300">Width</label>
          <span class="font-mono text-xs font-semibold text-blue-600">{{ width }}px</span>
        </div>
        <div class="pt-1">
          <Slider v-model="width" :min="256" :max="2048" :step="64" />
        </div>
      </div>
      <div class="flex flex-col">
        <div class="flex items-center justify-between pr-2 mb-1">
          <label class="text-xs font-medium tracking-wide text-surface-300">Height</label>
          <span class="font-mono text-xs font-semibold text-blue-600">{{ height }}px</span>
        </div>
        <div class="pt-1">
          <Slider v-model="height" :min="256" :max="2048" :step="64" />
        </div>
      </div>
    </div>
  </GenerationAction>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import Slider from 'primevue/slider';
import SelectButton from 'primevue/selectbutton';
import { useGenerationStore } from '@/stores/generation';
import GenerationAction from './GenerationAction.vue';

const props = defineProps(['collapsed', 'icon', 'label', 'toggleable']);

const generationStore = useGenerationStore();

// Aspect ratio presets
const aspectRatios = [
  { label: '1:2', width: 512, height: 1024 },
  { label: '9:16', width: 576, height: 1024 },
  { label: '3:4', width: 768, height: 1024 },
  { label: '1:1', width: 1024, height: 1024 },
  { label: '4:3', width: 1024, height: 768 },
  { label: '16:9', width: 1024, height: 576 },
  { label: '2:1', width: 1024, height: 512 },
];

const activeRatio = ref<string>('1:1');

// Update width/height when aspect ratio changes
watch(activeRatio, (label) => {
  const ratio = aspectRatios.find((r) => r.label === label);
  if (ratio) {
    width.value = ratio.width;
    height.value = ratio.height;
  }
});

// Sync active ratio when width/height change externally (e.g., from history reuse)
watch([() => generationStore.currentParams.width, () => generationStore.currentParams.height], ([newWidth, newHeight]) => {
  const matchingRatio = aspectRatios.find((r) => r.width === newWidth && r.height === newHeight);
  if (matchingRatio) {
    activeRatio.value = matchingRatio.label;
  }
});

const width = computed({
  get: () => generationStore.currentParams.width,
  set: (value: number | null) => {
    generationStore.currentParams.width = value ?? 1024;
  },
});

const height = computed({
  get: () => generationStore.currentParams.height,
  set: (value: number | null) => {
    generationStore.currentParams.height = value ?? 1024;
  },
});
</script>

<style scoped>
@reference "tailwindcss";

/* PrimeVue Slider overrides */
</style>
