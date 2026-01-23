<template>
  <Panel :collapsed="props.collapsed" :toggleable="props.toggleable">
    <template #header>
      <div class="flex gap-2 px-0 py-2 text-xs font-semibold tracking-wider text-gray-300 uppercase">
        <component :is="props.icon" class="w-4 h-4" />
        {{ props.label }}
      </div>
    </template>

    <div class="flex flex-col gap-2 px-2 py-1">
      <!-- Image Count -->
      <div class="flex flex-col gap-1">
        <label class="text-xs font-medium tracking-wide text-surface-300">Number of Images</label>
        <SelectButton v-model="imageCount" :options="imageCountOptions" option-label="label" option-value="value" size="small" fluid />
      </div>

      <!-- Aspect Ratio Pills -->
      <div class="flex flex-col gap-1">
        <label class="text-xs font-medium tracking-wide text-surface-300">Aspect Ratio</label>
        <SelectButton v-model="activeRatio" :options="aspectRatios" option-label="label" option-value="label" data-key="label" size="small" fluid>
          <template #option="slotProps">
            {{ slotProps.option.label }}
          </template>
        </SelectButton>
      </div>

      <!-- Width/Height Sliders -->
      <div class="grid grid-cols-2 gap-3">
        <div class="flex flex-col gap-2">
          <div class="flex items-center justify-between mb-1">
            <label class="text-xs font-medium tracking-wide text-surface-300">Width</label>
            <span class="font-mono text-xs text-blue-400">{{ width }}px</span>
          </div>
          <div class="pt-1">
            <Slider v-model="width" :min="256" :max="2048" :step="64" />
          </div>
        </div>
        <div class="flex flex-col gap-2">
          <div class="flex items-center justify-between mb-1">
            <label class="text-xs font-medium tracking-wide text-surface-300">Height</label>
            <span class="font-mono text-xs text-blue-400">{{ height }}px</span>
          </div>
          <div class="pt-1">
            <Slider v-model="height" :min="256" :max="2048" :step="64" />
          </div>
        </div>
      </div>
    </div>
  </Panel>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import Panel from 'primevue/panel';
import Slider from 'primevue/slider';
import SelectButton from 'primevue/selectbutton';
import { useGenerationStore } from '@/stores/generation';

const props = defineProps(['collapsed', 'icon', 'label', 'toggleable']);

const generationStore = useGenerationStore();

// Image count options
const imageCountOptions = [
  { label: '1', value: 1 },
  { label: '2', value: 2 },
  { label: '3', value: 3 },
  { label: '4', value: 4 },
];

const imageCount = computed({
  get: () => generationStore.currentParams.batchSize,
  set: (value: number) => {
    generationStore.currentParams.batchSize = value;
  },
});

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
:deep(.p-slider) {
  background: #374151;
}

:deep(.p-slider .p-slider-range) {
  background: #3b82f6;
}

:deep(.p-slider .p-slider-handle) {
  background: #3b82f6;
  border-color: #3b82f6;
}
</style>
