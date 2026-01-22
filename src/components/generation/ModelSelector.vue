<template>
  <div class="flex flex-col gap-4">
    <div class="flex flex-col gap-2">
      <label for="model" class="text-xs font-medium tracking-wide text-gray-400">Model</label>
      <Select
        id="model"
        v-model="modelsStore.selectedModelId"
        :options="modelsStore.models"
        option-label="name"
        option-value="id"
        placeholder="Select a model"
        size="small"
        class="w-full">
        <template #option="slotProps">
          <div class="flex items-center justify-between w-full">
            <span class="font-medium">{{ slotProps.option?.name }}</span>
            <span v-if="!slotProps.option.isDownloaded" class="rounded bg-amber-500/20 px-2 py-0.5 text-xs text-amber-400"> Not Downloaded </span>
          </div>
        </template>
      </Select>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div class="flex flex-col gap-1">
        <label class="text-xs font-medium tracking-wide text-gray-400">Sampler</label>
        <Select v-model="sampler" :options="samplerOptions" optionLabel="label" optionValue="value" size="small" fluid />
      </div>
      <div class="flex flex-col gap-1">
        <label class="text-xs font-medium tracking-wide text-gray-400">Scheduler</label>
        <Select v-model="scheduler" :options="schedulerOptions" optionLabel="label" optionValue="value" size="small" fluid />
      </div>
    </div>

    <div v-if="modelsStore.activeModel" class="flex flex-col gap-1 p-2 text-xs bg-gray-800 rounded text-slate-400">
      <span v-if="modelsStore.activeModel.description" class="italic">
        {{ modelsStore.activeModel.description }}
      </span>
      <div class="flex gap-3 text-sky-400">
        <span>{{ modelsStore.activeModel.defaultSteps }} steps</span>
        <span>CFG {{ modelsStore.activeModel.defaultGuidance }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useModelsStore } from '@/stores/models';
import { useGenerationStore } from '@/stores/generation';
import Select from 'primevue/select';
import type { Sampler, Scheduler } from '@/types';

const modelsStore = useModelsStore();
const generationStore = useGenerationStore();

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
