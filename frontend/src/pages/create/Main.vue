<template>
  <div class="flex flex-col h-full p-2 gap-2">
    <!-- Preview area -->
    <div class="flex-1 min-h-0 relative bg-slate-100 rounded-xl overflow-hidden flex items-center justify-center">
      <!-- Generated image -->
      <img
        v-if="displayedImage?.dataUrl"
        :src="displayedImage.dataUrl"
        alt="Generated image"
        class="max-w-full max-h-full object-contain" />

      <!-- Live preview during generation -->
      <img
        v-else-if="store.previewDataUrl && store.isGenerating"
        :src="store.previewDataUrl"
        alt="Generation preview"
        class="max-w-full max-h-full object-contain opacity-80" />

      <!-- Empty state -->
      <div v-else-if="!store.isGenerating" class="text-center">
        <ImageIcon :size="48" class="text-slate-300 mx-auto mb-3" />
        <p class="text-sm text-slate-400">Generated images will appear here</p>
        <p class="text-xs text-slate-300 mt-1">Enter a prompt and click Generate to begin</p>
      </div>

      <!-- Progress overlay -->
      <div
        v-if="store.isGenerating"
        class="absolute bottom-0 left-0 right-0 bg-white/80 backdrop-blur-sm px-4 py-3">
        <div v-if="store.progress" class="flex flex-col gap-1.5">
          <div class="flex justify-between text-xs text-slate-600">
            <span>Step {{ store.progress.step }} / {{ store.progress.totalSteps }}</span>
            <span class="tabular-nums">{{ progressPercent }}%</span>
          </div>
          <div class="h-1.5 bg-slate-200 rounded-full overflow-hidden">
            <div
              class="h-full bg-blue-500 rounded-full transition-[width] duration-300 ease-out"
              :style="{ width: progressPercent + '%' }" />
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

      <!-- Error overlay -->
      <div
        v-if="store.error && !store.isGenerating"
        class="absolute bottom-0 left-0 right-0 bg-red-50/90 backdrop-blur-sm px-4 py-3 flex items-center gap-2">
        <AlertCircle :size="16" class="text-red-500 shrink-0" />
        <span class="text-xs text-red-600 flex-1">{{ store.error }}</span>
        <button
          class="text-xs text-red-500 hover:text-red-700 underline shrink-0"
          @click="store.error = null">
          Dismiss
        </button>
      </div>
    </div>

    <!-- History strip -->
    <div v-if="store.generatedImages.length" class="shrink-0 bg-white rounded-xl shadow-sm px-3 py-2">
      <div class="flex gap-1.5 overflow-x-auto">
        <div
          v-for="(img, index) in store.generatedImages"
          :key="img.jobId"
          class="shrink-0 w-8 h-8 rounded cursor-pointer overflow-hidden border-2 transition-colors"
          :class="index === selectedIndex ? 'border-blue-500' : 'border-transparent hover:border-slate-300'"
          draggable="true"
          @dragstart="(e) => e.dataTransfer?.setData('text/image-path', img.imagePath)"
          @click="selectedIndex = index">
          <img v-if="img.dataUrl" :src="img.dataUrl" alt="" class="w-full h-full object-cover" />
          <div v-else class="w-full h-full bg-slate-200 flex items-center justify-center">
            <ImageIcon :size="10" class="text-slate-400" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Image as ImageIcon, AlertCircle } from 'lucide-vue-next';
import { useInferenceStore } from '@/stores/inference';

const store = useInferenceStore();
const selectedIndex = ref(0);
const displayedImage = computed(() => store.generatedImages[selectedIndex.value] ?? null);

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
