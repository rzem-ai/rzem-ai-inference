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
              @click="store.params.seed = -1">
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
        <Select v-model="store.params.sampler" :options="samplerOptions" />
      </div>

      <!-- Scheduler -->
      <div class="flex flex-col gap-1">
        <div class="text-sm font-medium text-slate-600">Scheduler</div>
        <Select v-model="store.params.scheduler" :options="schedulerOptions" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ChevronDown, Shuffle, Lock } from 'lucide-vue-next';
import { useInferenceStore } from '@/stores/inference';
import { InputGroup, InputGroupAddon, InputNumber, Select } from 'primevue';

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

<style scoped></style>
