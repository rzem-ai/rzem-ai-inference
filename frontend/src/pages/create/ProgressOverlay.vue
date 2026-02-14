<template>
  <!-- Progress overlay -->
  <div v-if="store.isGenerating" class="absolute bottom-0 left-0 right-0 bg-white/80 backdrop-blur-sm px-4 py-3">
    <!-- Batch progress header -->
    <div v-if="store.batchActive" class="flex justify-between text-xs text-slate-600 mb-1.5">
      <span class="font-medium">Batch: Image {{ store.batchCompleted + 1 }} of {{ store.batchTotal }}</span>
      <span class="tabular-nums">{{ Math.round((store.batchCompleted / store.batchTotal) * 100) }}%</span>
    </div>
    <div v-if="store.batchActive" class="h-1 bg-slate-200 rounded-full overflow-hidden mb-2">
      <div
        class="h-full bg-indigo-500 rounded-full transition-[width] duration-300 ease-out"
        :style="{ width: (store.batchCompleted / store.batchTotal) * 100 + '%' }" />
    </div>

    <div v-if="store.progress" class="flex flex-col gap-1.5">
      <div class="flex justify-between text-xs text-slate-600">
        <span>Step {{ store.progress.step }} / {{ store.progress.totalSteps }}</span>
        <span class="tabular-nums">{{ progressPercent }}%</span>
      </div>
      <div class="h-1.5 bg-slate-200 rounded-full overflow-hidden">
        <div class="h-full bg-blue-500 rounded-full transition-[width] duration-300 ease-out" :style="{ width: progressPercent + '%' }" />
      </div>
    </div>
    <div v-else-if="store.modelStatus" class="flex items-center gap-2">
      <div class="w-4 h-4 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
      <span class="text-xs text-slate-600">{{ store.modelStatus }}</span>
    </div>
    <div v-else class="flex items-center gap-2">
      <div class="w-4 h-4 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
      <span class="text-xs text-slate-600">Preparing...</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useInferenceStore } from '@/stores/inference';

const store = useInferenceStore();
const selectedIndex = ref(0);

const progressPercent = computed(() => {
  if (!store.progress) return 0;
  return Math.round((store.progress.step / store.progress.totalSteps) * 100);
});

// Select newest image when a new one is added
watch(
  () => store.generatedImages.length,
  () => {
    selectedIndex.value = 0;
  },
);
</script>
