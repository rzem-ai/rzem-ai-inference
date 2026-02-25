<template>
  <div v-if="showQuality" class="flex flex-col gap-0">
    <!-- Toggle header -->
    <button class="flex items-center justify-between w-full py-1" @click="store.toggleQuality()">
      <div class="text-base font-medium text-slate-600 hover:text-slate-700 transition-colors">Quality</div>
      <ChevronDown :size="14" class="transition-transform duration-200" :class="store.qualityOpen ? 'rotate-180' : ''" />
    </button>

    <!-- Collapsible content -->
    <div v-show="store.qualityOpen" class="flex flex-col gap-3 pt-2">
      <!-- Steps (hidden when bundle declares steps=0 — not supported by endpoint) -->
      <div v-if="showSteps" class="flex flex-col gap-2">
        <div class="flex justify-between items-center">
          <span class="text-sm font-medium text-slate-600">Steps</span>
          <span class="text-base font-semibold text-slate-500 tabular-nums">{{ store.params.steps }}</span>
        </div>
        <Slider v-model="store.params.steps" :min="1" :max="50" />
      </div>

      <!-- Prompt Adherence (CFG) (hidden when bundle declares cfg_scale=0 — not supported by endpoint) -->
      <div v-if="showCfgScale" class="flex flex-col gap-2">
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
import { useInferenceStore } from '@/stores/inference';

const store = useInferenceStore();

const showSteps = computed(() => store.selectedBundle?.steps !== 0);
const showCfgScale = computed(() => store.selectedBundle?.cfg_scale !== 0);
const showQuality = computed(() => showSteps.value || showCfgScale.value);

// Map 0.0–30.0 float to 0–300 int for slider precision
const cfgToInt = computed(() => Math.round(store.params.cfg_scale * 10));

function onCfgChange(val: number | number[]) {
  const v = Array.isArray(val) ? val[0] : val;
  store.params.cfg_scale = v / 10;
}
</script>
