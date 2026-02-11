<template>
  <div class="flex flex-col gap-2">
    <div class="text-base font-medium text-slate-600">Style / LoRAs</div>

    <!-- LoRA list -->
    <div v-for="(lora, index) in store.params.loras" :key="index" class="flex flex-col gap-1 p-2 bg-slate-50 rounded">
      <div class="flex items-center gap-1">
        <input
          v-model="lora.model_file"
          placeholder="LoRA model path"
          class="flex-1 text-xs bg-white rounded px-2 py-1.5 border border-slate-200" />
        <button
          class="p-1 rounded hover:bg-red-50 text-slate-400 hover:text-red-500 transition-colors"
          @click="store.removeLora(index)">
          <X :size="14" />
        </button>
      </div>
      <div class="flex items-center gap-2">
        <span class="text-[11px] text-slate-500 w-14">Strength</span>
        <Slider v-model="lora.strength" :min="0" :max="2" :step="0.05" class="flex-1" />
        <span class="text-[11px] text-slate-600 w-8 text-right tabular-nums">
          {{ lora.strength.toFixed(2) }}
        </span>
      </div>
    </div>

    <!-- Add button -->
    <button
      class="flex items-center gap-1 text-xs text-blue-500 hover:text-blue-600 transition-colors py-1"
      @click="store.addLora()">
      <Plus :size="14" />
      <span>Add LoRA</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { X, Plus } from 'lucide-vue-next';
import Slider from 'primevue/slider';
import { useInferenceStore } from '@/stores/inference';

const store = useInferenceStore();
</script>
