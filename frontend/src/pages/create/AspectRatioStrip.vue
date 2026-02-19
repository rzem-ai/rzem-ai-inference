<template>
  <div class="flex flex-col gap-2">
    <div class="text-base font-medium text-slate-600">Image Size</div>
    <!-- Preset buttons -->
    <div class="grid grid-cols-7 gap-0 border rounded-md border-surface-300">
      <div v-for="ratio in ASPECT_RATIOS" :key="ratio.label" class="rounded-none bg-surface-100 flex justify-center p-1">
        <button
          class="text-base font-medium py-1 transition-colors w-full rounded"
          :class="store.activeAspectRatio === ratio.label ? 'bg-blue-500 text-white' : 'bg-slate-100 text-slate-600 hover:bg-slate-200'"
          @click="store.setAspectRatio(ratio.label, ratio.width, ratio.height)">
          {{ ratio.label }}
        </button>
      </div>
    </div>

    <!-- Manual W/H inputs -->
    <div class="flex gap-2 items-center">
      <InputGroup>
        <InputGroupAddon class="text-base"> W </InputGroupAddon>
        <InputNumber v-model="store.params.width" placeholder="Width" :useGrouping="false" :min="256" :max="2048" :step="64" @input="onManualWidth" />
      </InputGroup>

      <X :size="12" class="text-slate-400 shrink-0" />

      <InputGroup>
        <InputGroupAddon class="text-base"> H </InputGroupAddon>
        <InputNumber v-model="store.params.height" placeholder="height" :useGrouping="false" :min="256" :max="2048" :step="64" @input="onManualHeight" />
      </InputGroup>
    </div>
  </div>
</template>

<script setup lang="ts">
import { X } from 'lucide-vue-next';
import { useInferenceStore, ASPECT_RATIOS } from '@/stores/inference';
import { InputGroup, InputGroupAddon, InputNumber, InputNumberInputEvent } from 'primevue';

const store = useInferenceStore();

function onManualWidth(e: InputNumberInputEvent) {
  const val = parseInt(e.formattedValue);
  if (!isNaN(val) && val >= 256 && val <= 2048) {
    store.applyParams({ width: val });
    store.setActiveAspectRatio(null);
  }
}

function onManualHeight(e: InputNumberInputEvent) {
  const val = parseInt(e.formattedValue);
  if (!isNaN(val) && val >= 256 && val <= 2048) {
    store.applyParams({ height: val });
    store.setActiveAspectRatio(null);
  }
}
</script>
