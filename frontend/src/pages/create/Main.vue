<template>
  <div class="flex flex-col h-full px-2 py-4 gap-2">
    <!-- Preview area -->
    <div class="flex-1 min-h-0 relative overflow-hidden flex items-center justify-center">
      <!-- Generated image -->

      <div v-if="store.isGenerating"> 
        <div class="object-cover shadow-xl border-surface-300 border rounded-2xl max-w-[90%] max-h-[90%]">
           
        </div>  
      </div>
      <div v-else>
        <div class="text-center">
          <ImageIcon :size="48" class="text-slate-300 mx-auto mb-3" />
          <p class="text-base text-slate-400">Generated images will appear here</p>
          <p class="text-sm text-slate-300 mt-1">Enter a prompt and click Generate to begin</p>
        </div>
      </div>

      <img
        v-if="displayedImage?.dataUrl"
        :src="displayedImage.dataUrl"
        title="Generated image"
        alt="Generated image"
        class="object-cover shadow-xl border-surface-300 border rounded-2xl max-w-[90%] max-h-[90%] hidden"
        :class="aspectRatio" />

      <!-- Live preview during generation -->
      <img
        v-else-if="store.previewDataUrl && store.isGenerating"
        :src="store.previewDataUrl"
        alt="Generation preview"
        class="max-w-full max-h-full w-fit h-fit object-contain opacity-80 bg-slate-100 hidden" />

      <!-- Empty state -->
      <div v-else-if="!store.isGenerating" class="text-center">
        <ImageIcon :size="48" class="text-slate-300 mx-auto mb-3" />
        <p class="text-sm text-slate-400">Generated images will appear here</p>
        <p class="text-xs text-slate-300 mt-1">Enter a prompt and click Generate to begin</p>
      </div>

      <!-- Progress overlay -->
      <ProgressOverlay v-if="store.isGenerating" />

      <!-- Error overlay -->
      <ErrorOverlay v-if="store.error && !store.isGenerating" />
    </div>

    <!-- History strip -->
    <History />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Image as ImageIcon } from 'lucide-vue-next';
import { useInferenceStore } from '@/stores/inference';

import History from './History.vue';
import ErrorOverlay from './ErrorOverlay.vue';
import ProgressOverlay from './ProgressOverlay.vue';

const store = useInferenceStore();
const selectedIndex = ref(0);
const displayedImage = computed(() => store.generatedImages[selectedIndex.value] ?? null);

const aspectRatio = computed(() => {
  const image = store.generatedImages[selectedIndex.value];
  if (!image?.width || !image?.height) return 'aspect-square';

  const gcd = (a: number, b: number): number => (b === 0 ? a : gcd(b, a % b));
  const d = gcd(image.width, image.height);

  return `aspect-[${image.width / d}/${image.height / d}]`;
});

// Select newest image when a new one is added
watch(
  () => store.generatedImages.length,
  () => {
    selectedIndex.value = 0;
  },
);
</script>
