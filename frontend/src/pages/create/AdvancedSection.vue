<template>
  <div class="flex flex-col gap-0">
    <!-- Toggle header -->
    <button class="flex items-center justify-between w-full py-1" @click="store.toggleAdvanced()">
      <div class="text-base font-medium text-slate-600 hover:text-slate-700 transition-colors">Advanced</div>
      <ChevronDown :size="14" class="transition-transform duration-200" :class="store.advancedOpen ? 'rotate-180' : ''" />
    </button>

    <!-- Collapsible content -->
    <div v-show="store.advancedOpen" class="flex flex-col gap-3 pt-2">
      <!-- Seed -->
      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium text-slate-600">Seed</span>
        <div class="flex gap-1">
          <InputGroup>
            <InputNumber v-model="store.params.seed" placeholder="Search styles..." fluid />
            <InputGroupAddon
              class="bg-surface-200 hover:bg-surface-400 text-surface-700 hover:text-surface-50 cursor-pointer"
              title="Randomize"
              @click="store.applyParams({ seed: -1 })">
              <Shuffle :size="14" />
            </InputGroupAddon>
            <InputGroupAddon
              class="bg-surface-200 hover:bg-surface-400 text-surface-700 hover:text-surface-50 cursor-pointer"
              title="Lock seed"
              @click="toggleLock">
              <Lock :size="14" />
            </InputGroupAddon>
          </InputGroup>
        </div>
      </div>

      <!-- Sampler -->
      <div class="flex flex-col gap-1">
        <div class="text-sm font-medium text-slate-600">Sampler</div>
        <Select v-model="store.params.sampler" :options="samplerOptions" optionLabel="label" optionValue="value">
          <template #option="{ option }">
            <div class="flex flex-col gap-0">
              <div class="font-medium">{{ option.label }}</div>
              <div class="text-xs text-slate-400">{{ option.desc }}</div>
            </div>
          </template>
        </Select>
      </div>

      <!-- Scheduler -->
      <div class="flex flex-col gap-1">
        <div class="text-sm font-medium text-slate-600">Scheduler</div>
        <Select v-model="store.params.scheduler" :options="schedulerOptions" optionLabel="label" optionValue="value">
          <template #option="{ option }">
            <div class="flex flex-col gap-0">
              <div class="font-medium">{{ option.label }}</div>
              <div class="text-xs text-slate-400">{{ option.desc }}</div>
            </div>
          </template>
        </Select>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useInferenceStore } from '@/stores/inference';

const store = useInferenceStore();

const samplerOptions = [
  { value: 'euler', label: 'Euler', desc: 'Fast, stable — good default for most models' },
  { value: 'euler_a', label: 'Euler Ancestral', desc: 'Adds noise each step — more creative, less reproducible' },
  { value: 'dpm++_2m', label: 'DPM++ 2M', desc: 'High quality with fewer steps — great with Karras scheduler' },
  { value: 'dpm++_2s', label: 'DPM++ 2S', desc: 'Single-step variant — sharper details, slightly slower' },
  { value: 'dpm++_sde', label: 'DPM++ SDE', desc: 'Stochastic — good for organic textures and fine detail' },
  { value: 'heun', label: 'Heun', desc: 'More accurate but 2x slower — best for low step counts' },
  { value: 'lms', label: 'LMS', desc: 'Linear multistep — smooth results, can overshoot at high steps' },
];

const schedulerOptions = [
  { value: 'simple', label: 'Simple', desc: 'Minimal schedule — fastest, works well with Euler' },
  { value: 'normal', label: 'Normal', desc: 'Linear noise schedule — simple and predictable' },
  { value: 'beta', label: 'Beta', desc: 'Bell-curve schedule — balanced denoising, good for portraits' },
  { value: 'karras', label: 'Karras', desc: 'Front-loads detail work — recommended for DPM++ samplers' },
  { value: 'exponential', label: 'Exponential', desc: 'Aggressive early denoising — sharp results, fewer steps needed' },
  { value: 'sgm_uniform', label: 'SGM Uniform', desc: 'Evenly spaced steps — stable and consistent' },
  { value: 'ddim_uniform', label: 'DDIM Uniform', desc: 'Uniform denoising steps — deterministic and reproducible' },
];

function toggleLock() {
  if (store.params.seed === -1) {
    store.applyParams({ seed: Math.floor(Math.random() * 2147483647) });
  } else {
    store.applyParams({ seed: -1 });
  }
}
</script>

<style scoped></style>
