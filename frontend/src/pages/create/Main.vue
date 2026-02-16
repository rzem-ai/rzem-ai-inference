<template>
  <div class="flex flex-col h-full px-2 py-4 gap-2">
    <!-- Preview area -->
    <div class="relative place-items-center h-full w-full">
      <div v-if="store.isGenerating && store.previewDataUrl" class="rounded-2xl p-4 h-full flex justify-center w-full">
        <!-- Live preview during generation -->
        <div ref="imageWrapper" class="w-full h-full flex justify-center">
          <div class=" ">
            <img :src="store.previewDataUrl" alt="Generation preview" class="rounded-2xl" style="filter: blur(1px)" :style="placeholderStyle" />
          </div>
        </div>
      </div>

      <div v-else-if="displayedImage?.dataUrl" class="rounded-2xl p-4 h-full flex justify-center w-full">
        <!-- Live preview during generation -->
        <div ref="imageWrapper" class="w-full h-full flex justify-center">
          <div class=" ">
            <img :src="displayedImage.dataUrl" alt="Generated image" class="rounded-2xl" :style="placeholderStyle" />
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-else class="rounded-2xl p-4 h-full flex justify-center w-full">
        <div ref="imageWrapper" class="w-full h-full flex justify-center">
          <div class="border-surface-200 border rounded-2xl flex flex-col justify-center text-center bg-surface-100" :style="placeholderStyle">
            <ImageIcon :size="48" class="w-full text-slate-500" />
            <div class="text-lg text-slate-500">Generated images will appear here</div>
            <div class="text-base text-slate-400">Enter a prompt and click Generate to begin</div>
          </div>
        </div>
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
import { ref, computed, watch, useTemplateRef } from 'vue';
import { Image as ImageIcon } from 'lucide-vue-next';
import { useInferenceStore } from '@/stores/inference';
import { useElementSize } from '@vueuse/core';

import History from './History.vue';
import ErrorOverlay from './ErrorOverlay.vue';
import ProgressOverlay from './ProgressOverlay.vue';

const imageWrapper = useTemplateRef('imageWrapper');
const imageWrapperSize = useElementSize(imageWrapper);

const store = useInferenceStore();
const selectedIndex = ref(0);
const displayedImage = computed(() => store.latestImage);

const height = computed(() => {
  return store.latestImage?.height || store.params.height;
});

const width = computed(() => {
  return store.latestImage?.width || store.params.width;
});

const aspectRatio = computed(() => {
  const gcd = (a: number, b: number): number => (b === 0 ? a : gcd(b, a % b));
  const d = gcd(width.value, height.value);

  return `aspect-${width.value / d}/${height.value / d}`;
});

// Calculate placeholder dimensions based on aspect ratio
const placeholderStyle = computed(() => {
  const targetWidth = width.value;
  const targetHeight = height.value;
  const ratio = targetWidth / targetHeight;

  // For landscape (wide) images: constrain HEIGHT so width can expand
  // For portrait (tall) images: constrain WIDTH so height can expand
  if (ratio >= 2) {
    // Very wide landscape (2:1): constrain height to allow maximum width
    return {
      width: `${imageWrapperSize.height.value - 10}px`,
      aspectRatio: `${targetWidth} / ${targetHeight}`,
    };
  } else if (ratio >= 1.5) {
    // Wide landscape (16:9, etc): constrain height
    return {
      height: `${imageWrapperSize.height.value - 10}px`,
      aspectRatio: `${targetWidth} / ${targetHeight}`,
    };
  } else if (ratio > 1) {
    // Moderate landscape (4:3): constrain height
    return {
      height: `${imageWrapperSize.height.value - 10}px`,
      aspectRatio: `${targetWidth} / ${targetHeight}`,
    };
  } else if (ratio === 1) {
    // Square: constrain either dimension
    return {
      height: `${imageWrapperSize.height.value - 2}px`,
      aspectRatio: `${targetWidth} / ${targetHeight}`,
    };
  } else {
    // Very tall portrait (1:2): constrain width significantly
    return {
      height: `${imageWrapperSize.height.value - 10}px`,
      aspectRatio: `${targetWidth} / ${targetHeight}`,
    };
  }
});

// Select newest image when a new one is added
watch(
  () => store.generatedImages.length,
  () => {
    selectedIndex.value = 0;
  },
);
</script>
