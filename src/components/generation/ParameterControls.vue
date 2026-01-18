<script setup lang="ts">
import { computed } from 'vue'
import { useGenerationStore } from '@/stores/generation'
import type { Sampler, Scheduler } from '@/types'
import InputNumber from 'primevue/inputnumber'
import Slider from 'primevue/slider'
import Select from 'primevue/select'

const store = useGenerationStore()

// Sampler and Scheduler options
const samplerOptions = [
  { label: 'Euler', value: 'euler' },
  { label: 'Euler Ancestral', value: 'euler_a' },
  { label: 'Heun', value: 'heun' },
  { label: 'DPM2', value: 'dpm_2' },
  { label: 'DPM2 Ancestral', value: 'dpm_2_a' },
  { label: 'LMS', value: 'lms' },
  { label: 'DPM++ 2M', value: 'dpmpp_2m' },
  { label: 'DPM++ 2S Ancestral', value: 'dpmpp_2s_a' },
  { label: 'DPM++ SDE', value: 'dpmpp_sde' },
]

const schedulerOptions = [
  { label: 'Normal', value: 'normal' },
  { label: 'Karras', value: 'karras' },
  { label: 'Exponential', value: 'exponential' },
  { label: 'SGM Uniform', value: 'sgm_uniform' },
  { label: 'Simple', value: 'simple' },
  { label: 'DDIM Uniform', value: 'ddim_uniform' },
  { label: 'Beta', value: 'beta' },
]

const sampler = computed({
  get: () => store.currentParams.sampler,
  set: (value: Sampler) => {
    store.currentParams.sampler = value
  }
})

const scheduler = computed({
  get: () => store.currentParams.scheduler,
  set: (value: Scheduler) => {
    store.currentParams.scheduler = value
  }
})

const steps = computed({
  get: () => store.currentParams.steps,
  set: (value: number | null) => {
    store.currentParams.steps = value ?? 4
  }
})

const cfgScale = computed({
  get: () => store.currentParams.cfgScale,
  set: (value: number | null) => {
    store.currentParams.cfgScale = value ?? 1.0
  }
})

const width = computed({
  get: () => store.currentParams.width,
  set: (value: number | null) => {
    store.currentParams.width = value ?? 1024
  }
})

const height = computed({
  get: () => store.currentParams.height,
  set: (value: number | null) => {
    store.currentParams.height = value ?? 1024
  }
})

const seed = computed({
  get: () => store.currentParams.seed,
  set: (value: number | null) => {
    store.currentParams.seed = value ?? -1
  }
})

const commonSizes = [
  { label: 'Square (1024×1024)', width: 1024, height: 1024 },
  { label: 'Landscape (1344×768)', width: 1344, height: 768 },
  { label: 'Portrait (768×1344)', width: 768, height: 1344 },
]

const setSize = (w: number, h: number) => {
  width.value = w
  height.value = h
}

const randomizeSeed = () => {
  seed.value = Math.floor(Math.random() * 2147483647)
}
</script>

<template>
  <div class="parameter-controls">
    <div class="field">
      <label>Steps: {{ steps }}</label>
      <Slider v-model="steps" :min="1" :max="50" />
    </div>

    <div class="field">
      <label>CFG Scale: {{ cfgScale.toFixed(1) }}</label>
      <Slider v-model="cfgScale" :min="0" :max="20" :step="0.1" />
    </div>

    <div class="field-row">
      <div class="field">
        <label>Sampler</label>
        <Select
          v-model="sampler"
          :options="samplerOptions"
          optionLabel="label"
          optionValue="value"
          class="w-full"
        />
      </div>
      <div class="field">
        <label>Scheduler</label>
        <Select
          v-model="scheduler"
          :options="schedulerOptions"
          optionLabel="label"
          optionValue="value"
          class="w-full"
        />
      </div>
    </div>

    <div class="field">
      <label>Size Presets</label>
      <div class="size-buttons">
        <button
          v-for="size in commonSizes"
          :key="size.label"
          @click="setSize(size.width, size.height)"
          class="size-btn"
        >
          {{ size.label }}
        </button>
      </div>
    </div>

    <div class="field-row">
      <div class="field">
        <label>Width</label>
        <InputNumber v-model="width" :min="256" :max="2048" :step="64" />
      </div>
      <div class="field">
        <label>Height</label>
        <InputNumber v-model="height" :min="256" :max="2048" :step="64" />
      </div>
    </div>

    <div class="field">
      <label>Seed</label>
      <div class="seed-control">
        <InputNumber v-model="seed" :min="-1" :max="2147483647" class="flex-1" />
        <button @click="randomizeSeed" class="randomize-btn">🎲</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.parameter-controls {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.field label {
  font-weight: 600;
  font-size: 0.875rem;
  color: #374151;
}

.field-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
}

.size-buttons {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.size-btn {
  padding: 0.5rem 1rem;
  background: #f3f4f6;
  border: 1px solid #d1d5db;
  border-radius: 0.375rem;
  cursor: pointer;
  font-size: 0.875rem;
  transition: all 0.2s;
}

.size-btn:hover {
  background: #e5e7eb;
}

.seed-control {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.randomize-btn {
  padding: 0.5rem 0.75rem;
  background: #f3f4f6;
  border: 1px solid #d1d5db;
  border-radius: 0.375rem;
  cursor: pointer;
  font-size: 1.25rem;
  transition: all 0.2s;
}

.randomize-btn:hover {
  background: #e5e7eb;
}
</style>
