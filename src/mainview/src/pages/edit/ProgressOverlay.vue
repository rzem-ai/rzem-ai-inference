<template>
  <div v-if="store.isGenerating" class="flex flex-col gap-1">
    <!-- Step progress -->
    <template v-if="store.progress">
      <div class="flex justify-between text-xs text-slate-600">
        <span>Step {{ store.progress.step }} / {{ store.progress.totalSteps }}</span>
        <span class="tabular-nums">{{ progressPercent }}%</span>
      </div>
      <div class="h-1 overflow-hidden rounded-full bg-slate-200">
        <div class="h-full rounded-full bg-blue-500 transition-[width] duration-300 ease-out" :style="{ width: progressPercent + '%' }" />
      </div>
    </template>

    <!-- Loading spinner (no step info yet) -->
    <div v-else-if="store.modelStatus" class="flex items-center gap-2">
      <div class="h-3 w-3 animate-spin rounded-full border-2 border-blue-500 border-t-transparent" />
      <span class="text-xs text-slate-600">{{ store.modelStatus }}</span>
    </div>
    <div v-else class="flex items-center gap-2">
      <div class="h-3 w-3 animate-spin rounded-full border-2 border-blue-500 border-t-transparent" />
      <span class="text-xs text-slate-600">Preparing...</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useEditStore } from '@/stores/edit';

const store = useEditStore();

const progressPercent = computed(() => {
  if (!store.progress) return 0;
  return Math.round((store.progress.step / store.progress.totalSteps) * 100);
});
</script>
