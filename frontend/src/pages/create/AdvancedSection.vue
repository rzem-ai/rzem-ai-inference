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
        <span class="text-xs font-medium text-slate-600">Seed</span>
        <div class="flex gap-1">
          <input
            type="number"
            v-model.number="store.params.seed"
            class="flex-1 text-xs bg-slate-100 rounded px-2 py-1.5 [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none" />
          <button class="p-1.5 rounded hover:bg-slate-100 text-slate-500 transition-colors" title="Randomize" @click="store.params.seed = -1">
            <Shuffle :size="14" />
          </button>
          <button
            class="p-1.5 rounded hover:bg-slate-100 transition-colors"
            :class="store.params.seed !== -1 ? 'text-blue-500' : 'text-slate-400'"
            title="Lock seed"
            @click="toggleLock">
            <Lock :size="14" />
          </button>
        </div>
      </div>

      <!-- Sampler -->
      <div class="flex flex-col gap-1">
        <span class="text-xs font-medium text-slate-600">Sampler</span>
        <Select v-model="store.params.sampler" :options="samplerOptions" class="compact-select" size="small" />
      </div>

      <!-- Scheduler -->
      <div class="flex flex-col gap-1">
        <span class="text-xs font-medium text-slate-600">Scheduler</span>
        <Select v-model="store.params.scheduler" :options="schedulerOptions" class="compact-select" size="small" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ChevronDown, Shuffle, Lock } from 'lucide-vue-next';
import Select from 'primevue/select';
import { useInferenceStore } from '@/stores/inference';

const store = useInferenceStore();

const samplerOptions = ['euler', 'euler_a', 'dpm++_2m', 'dpm++_2s', 'dpm++_sde', 'heun', 'lms'];
const schedulerOptions = ['normal', 'karras', 'exponential', 'sgm_uniform', 'simple', 'ddim_uniform'];

function toggleLock() {
  if (store.params.seed === -1) {
    store.params.seed = Math.floor(Math.random() * 2147483647);
  } else {
    store.params.seed = -1;
  }
}
</script>

<style scoped>
.compact-select :deep(.p-select) {
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
}
</style>
