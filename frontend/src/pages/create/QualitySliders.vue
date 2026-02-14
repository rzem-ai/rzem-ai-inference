<template>
  <div class="flex flex-col gap-1 mb-2">
    <div class="text-base font-medium text-slate-600">Quality</div>

    <div class="flex flex-col gap-3">
      <!-- Steps -->
      <div class="flex flex-col gap-2">
        <div class="flex justify-between items-center">
          <span class="text-sm font-medium text-slate-600">Steps</span>
          <span class="text-base font-semibold text-slate-500 tabular-nums">{{ store.params.steps }}</span>
        </div>
        <Slider v-model="store.params.steps" :min="1" :max="50" />
      </div>

      <!-- Prompt Adherence (CFG) -->
      <div class="flex flex-col gap-2">
        <div class="flex justify-between items-center">
          <span class="text-sm font-medium text-slate-600">Prompt Adherence</span>
          <span class="text-base font-semibold text-slate-500 tabular-nums">{{ store.params.cfg_scale.toFixed(1) }}</span>
        </div>
        <Slider :model-value="cfgToInt" @update:model-value="onCfgChange" :min="0" :max="200" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import Slider from 'primevue/slider';
import { useInferenceStore } from '@/stores/inference';

const store = useInferenceStore();

// Map 0.0–30.0 float to 0–300 int for slider precision
const cfgToInt = computed(() => Math.round(store.params.cfg_scale * 10));

function onCfgChange(val: number | number[]) {
  const v = Array.isArray(val) ? val[0] : val;
  store.params.cfg_scale = v / 10;
}
</script>
