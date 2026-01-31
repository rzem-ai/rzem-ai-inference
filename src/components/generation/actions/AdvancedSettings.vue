<template>
  <GenerationAction
    :collapsed="false"
    :toggleable="true"
    icon="layer-group"
    label="Advanced Settings"
    panelClass="border border-red-200 rounded-lg px-2 pb-2 bg-surface-950">
    <template #header>
      <div class="flex gap-2 px-0 py-2 text-xs font-semibold tracking-wider uppercase text-surface-300">
        <fa :icon="['fal', 'gears']" size="sm" />
        Advanced
      </div>
    </template>

    <div class="flex flex-col gap-4 px-2">
      <div class="flex flex-col gap-1">
        <div class="flex items-center justify-between">
          <label class="text-xs font-medium tracking-wide text-surface-300">Lock Seed</label>
          <ToggleSwitch v-model="seedLocked" v-tooltip.top="seedLocked ? 'Locked' : 'Random'" />
        </div>
        <div class="flex gap-2">
          <InputNumber
            v-model="seed"
            :min="0"
            :max="9007199254740991"
            :disabled="!seedLocked"
            :placeholder="seedLocked ? '' : 'Random'"
            :useGrouping="false"
            size="small"
            fluid />
          <Button @click="randomizeSeed" :disabled="!seedLocked" v-tooltip.top="'Generate new random seed'">
            <fa :icon="['fal', 'seedling']" size="sm" />
          </Button>
        </div>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <div class="flex flex-col gap-1">
          <label class="text-xs font-medium tracking-wide text-surface-300">Sampler</label>
          <Select v-model="sampler" :options="samplerOptions" optionLabel="label" optionValue="value" size="small" fluid />
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-xs font-medium tracking-wide text-surface-300">Scheduler</label>
          <Select v-model="scheduler" :options="schedulerOptions" optionLabel="label" optionValue="value" size="small" fluid />
        </div>
      </div>
    </div>
  </GenerationAction>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useGenerationStore } from '@/stores/generation';
import Button from 'primevue/button';

import Select from 'primevue/select';
import InputNumber from 'primevue/inputnumber';
import ToggleSwitch from 'primevue/toggleswitch';
import type { Sampler, Scheduler } from '@/types';
import GenerationAction from './GenerationAction.vue';

defineProps(['collapsed', 'icon', 'label', 'toggleable']);

const generationStore = useGenerationStore();

const seed = computed({
  get: () => generationStore.currentParams.seed,
  set: (value: number | null) => {
    generationStore.currentParams.seed = value ?? 0;
  },
});

// seedLocked = true means use the specific seed value (not random)
// seedLocked = false means randomize on each generation
const seedLocked = computed({
  get: () => !generationStore.randomizeSeedOnGenerate,
  set: (value: boolean) => {
    generationStore.randomizeSeedOnGenerate = !value;
    // When locking, ensure we have a valid seed
    if (value && (seed.value < 0 || seed.value === null)) {
      seed.value = Math.floor(Math.random() * Number.MAX_SAFE_INTEGER);
    }
  },
});

const randomizeSeed = () => {
  seed.value = Math.floor(Math.random() * Number.MAX_SAFE_INTEGER);
};

// Sampler and Scheduler options
const samplerOptions = [
  { label: 'Euler', value: 'euler' as Sampler },
  { label: 'Euler Ancestral', value: 'euler_a' as Sampler },
  { label: 'Heun', value: 'heun' as Sampler },
  { label: 'DPM2', value: 'dpm_2' as Sampler },
  { label: 'DPM2 Ancestral', value: 'dpm_2_a' as Sampler },
  { label: 'LMS', value: 'lms' as Sampler },
  { label: 'DPM++ 2M', value: 'dpmpp_2m' as Sampler },
  { label: 'DPM++ 2S Ancestral', value: 'dpmpp_2s_a' as Sampler },
  { label: 'DPM++ SDE', value: 'dpmpp_sde' as Sampler },
];

const schedulerOptions = [
  { label: 'Simple', value: 'simple' as Scheduler },
  { label: 'Normal', value: 'normal' as Scheduler },
  { label: 'Beta', value: 'beta' as Scheduler },
  { label: 'Karras', value: 'karras' as Scheduler },
  { label: 'Exponential', value: 'exponential' as Scheduler },
  { label: 'SGM Uniform', value: 'sgm_uniform' as Scheduler },
  { label: 'DDIM Uniform', value: 'ddim_uniform' as Scheduler },
];

const sampler = computed({
  get: () => generationStore.currentParams.sampler,
  set: (value: Sampler) => {
    generationStore.currentParams.sampler = value;
  },
});

const scheduler = computed({
  get: () => generationStore.currentParams.scheduler,
  set: (value: Scheduler) => {
    generationStore.currentParams.scheduler = value;
  },
});
</script>
