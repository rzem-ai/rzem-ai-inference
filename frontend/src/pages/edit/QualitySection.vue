<template>
  <div class="flex flex-col gap-2">
    <button class="flex items-center gap-2 text-base font-medium text-slate-600 cursor-pointer" @click="open = !open">
      <ChevronRight :size="14" class="transition-transform" :class="{ 'rotate-90': open }" />
      Quality &amp; Sampling
    </button>

    <div v-if="open" class="flex flex-col gap-3 pl-1">
      <!-- Steps -->
      <div class="flex flex-col gap-1">
        <div class="flex justify-between text-sm text-slate-500">
          <span>Steps</span>
          <span class="tabular-nums">{{ store.params.steps }}</span>
        </div>
        <Slider v-model="store.params.steps" :min="1" :max="50" :step="1" />
      </div>

      <!-- CFG Scale -->
      <div class="flex flex-col gap-1">
        <div class="flex justify-between text-sm text-slate-500">
          <span>Guidance</span>
          <span class="tabular-nums">{{ store.params.cfg_scale.toFixed(1) }}</span>
        </div>
        <Slider v-model="store.params.cfg_scale" :min="0" :max="10" :step="0.5" />
      </div>

      <!-- Seed -->
      <div class="flex flex-col gap-1">
        <div class="text-sm text-slate-500">Seed</div>
        <div class="flex gap-2">
          <InputNumber v-model="store.params.seed" class="flex-1" :min="-1" :max="2147483647" placeholder="-1 = random" fluid />
          <Button size="small" severity="secondary" variant="outlined" title="Randomize" @click="store.params.seed = -1">
            <Shuffle :size="14" />
          </Button>
        </div>
      </div>

      <!-- Sampler -->
      <div class="flex flex-col gap-1">
        <div class="text-sm text-slate-500">Sampler</div>
        <Select v-model="store.params.sampler" :options="samplers" fluid />
      </div>

      <!-- Scheduler -->
      <div class="flex flex-col gap-1">
        <div class="text-sm text-slate-500">Scheduler</div>
        <Select v-model="store.params.scheduler" :options="schedulers" fluid />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useEditStore } from '@/stores/edit';

const store = useEditStore();
const open = ref(false);

const samplers = ['euler', 'euler_ancestral', 'heun', 'dpm_2', 'dpm_2_ancestral', 'lms', 'dpmpp_2m', 'dpmpp_2s_ancestral', 'dpmpp_sde'];
const schedulers = ['normal', 'simple', 'karras', 'exponential', 'sgm_uniform', 'beta'];
</script>
