<template>
  <Panel :collapsed="props.collapsed" :toggleable="props.toggleable">
    <template #header>
      <div class="flex gap-2 px-0 py-2 text-xs font-semibold tracking-wider uppercase text-surface-300">
        <component :is="props.icon" class="w-4 h-4" />
        {{ props.label }}
      </div>
    </template>

    <div class="flex flex-col gap-4 px-2">
      <div class="flex flex-col gap-2">
        <div class="flex items-center justify-between mb-1">
          <label class="text-xs font-medium tracking-wide text-surface-300">Steps</label>
          <span class="font-mono text-xs text-blue-400">{{ steps }}</span>
        </div>
        <Slider v-model="steps" :min="1" :max="50" />
      </div>

      <div class="flex flex-col gap-2">
        <div class="flex items-center justify-between mb-1">
          <label class="text-xs font-medium tracking-wide text-surface-300">CFG Scale</label>
          <span class="font-mono text-xs text-blue-400">{{ cfgScale.toFixed(1) }}</span>
        </div>
        <Slider v-model="cfgScale" :min="0" :max="20" :step="0.1" />
      </div>

      <div class="flex flex-col gap-1">
        <div class="flex items-center justify-between">
          <label class="text-xs font-medium tracking-wide text-surface-300">Seed</label>
        </div>
        <div class="flex gap-2">
          <InputNumber
            v-model="seed"
            :min="0"
            :max="2147483647"
            :disabled="!seedLocked"
            :placeholder="seedLocked ? '' : 'Random'"
            :useGrouping="false"
            size="small"
            fluid />
          <Button @click="randomizeSeed" :disabled="!seedLocked" v-tooltip.top="'Generate new random seed'">
            <Sprout class="w-4 h-4" />
          </Button>
          <Button @click="toggleSeedLock" v-tooltip.top="seedLocked ? 'Locked' : 'Random'">
            <component :is="seedLocked ? Lock : Unlock" class="w-4 h-4" />
          </Button>
        </div>
      </div>
    </div>
  </Panel>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import Button from 'primevue/button';
import Slider from 'primevue/slider';
import InputNumber from 'primevue/inputnumber';
import { Lock, Unlock, Sprout } from 'lucide-vue-next';
import { useGenerationStore } from '@/stores/generation';
import Panel from 'primevue/panel';

const props = defineProps(['collapsed', 'icon', 'label', 'toggleable']);

const generationStore = useGenerationStore();

// Computed properties bound to store
const steps = computed({
  get: () => generationStore.currentParams.steps,
  set: (value: number | null) => {
    generationStore.currentParams.steps = value ?? 4;
  },
});

const cfgScale = computed({
  get: () => generationStore.currentParams.cfgScale,
  set: (value: number | null) => {
    generationStore.currentParams.cfgScale = value ?? 1.0;
  },
});

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
  },
});

const toggleSeedLock = () => {
  seedLocked.value = !seedLocked.value;
  // When locking, ensure we have a valid seed
  if (seedLocked.value && (seed.value < 0 || seed.value === null)) {
    seed.value = Math.floor(Math.random() * 2147483647);
  }
};

const randomizeSeed = () => {
  seed.value = Math.floor(Math.random() * 2147483647);
};
</script>

<style scoped>
/* PrimeVue component overrides */

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
