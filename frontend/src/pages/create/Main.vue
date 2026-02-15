<template>
  <div class="flex flex-col h-full px-2 py-4 gap-2">
    <!-- Preview area -->
    <div class="place-items-center h-full w-full">
      <div v-if="store.isGenerating" class="rounded-2xl p-4 h-full flex justify-center w-full">
        <!-- Live preview during generation -->
        <div ref="imageWrapper" class="w-full h-full flex justify-center">
          <div class="border-red-500 border rounded-2xl shadow-md shadow-surface-800/90">
            <img
              v-if="store.previewDataUrl"
              :src="store.previewDataUrl"
              alt="Generation preview"
              style="filter: blur(1px)"
              class="rounded-2xl"
              :style="placeholderStyle" />
          </div>
        </div>
      </div>

      <div v-else-if="displayedImage?.dataUrl" class="rounded-2xl p-4 h-full flex justify-center w-full">
        <!-- Live preview during generation -->
        <div ref="imageWrapper" class="w-full h-full flex justify-center">
          <div class="border-red-500 border rounded-2xl shadow-md shadow-surface-800/90">
            <img
              :src="displayedImage.dataUrl"
              title="Generated image"
              alt="Generated image"
              class="object-cover shadow-xl border-surface-300 border rounded-2xl max-w-[90%] max-h-[90%] hidden"
              :class="placeholderStyle" />
          </div>
        </div>
      </div>

      <img
        v-if="displayedImage?.dataUrl"
        :src="displayedImage.dataUrl"
        title="Generated image"
        alt="Generated image"
        class="object-cover shadow-xl border-surface-300 border rounded-2xl max-w-[90%] max-h-[90%] hidden"
        :class="aspectRatio" />

      <div v-if="false">
        <!-- Generated image -->
        <div v-if="store.isGenerating" class="w-full h-full text-center place-content-center justify-center">
          <div class="text-center border place-content-center rounded-xl border-surface-300 bg-surface-100 w-full h-full">
            <!-- Live preview during generation -->

            <img
              v-if="store.previewDataUrl"
              :src="store.previewDataUrl"
              alt="Generation preview"
              class="object-contain opacity-80 bg-slate-100"
              :style="placeholderStyle" />

            <img
              v-if="displayedImage?.dataUrl"
              :src="displayedImage.dataUrl"
              title="Generated image"
              alt="Generated image"
              class="object-cover shadow-xl border-surface-300 border rounded-2xl max-w-[90%] max-h-[90%] hidden"
              :class="aspectRatio" />
          </div>
        </div>
        <!-- Empty state -->
        <div v-else class="text-center border place-content-center rounded-xl border-surface-300 bg-surface-100" :style="placeholderStyle">
          <div class="justify-center text-surface-500 gap-2 flex flex-col p-8">
            <ImageIcon :size="48" class="w-full" />
            <div class="text-lg text-slate-500">Generated images will appear here</div>
            <div class="text-base text-slate-400">Enter a prompt and click Generate to begin</div>
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

        <!-- Progress overlay -->
        <ProgressOverlay v-if="store.isGenerating" />

        <!-- Error overlay -->
        <ErrorOverlay v-if="store.error && !store.isGenerating" />
      </div>
    </div>

    <!-- History strip -->
    <History />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, useTemplateRef } from 'vue';
import { Image as ImageIcon } from 'lucide-vue-next';
import { useInferenceStore } from '@/stores/inference';
import { useWindowSize } from '@vueuse/core';
import { useElementSize } from '@vueuse/core';

import History from './History.vue';
import ErrorOverlay from './ErrorOverlay.vue';
import ProgressOverlay from './ProgressOverlay.vue';

const imageWrapper = useTemplateRef('imageWrapper');
const imageWrapperSize = useElementSize(imageWrapper);

const store = useInferenceStore();
const selectedIndex = ref(0);
const displayedImage = computed(() => store.generatedImages[selectedIndex.value] ?? null);

const height = computed(() => {
  const image = store.generatedImages[selectedIndex.value];
  const height = image?.height || store.params.height;

  return height;
});

const width = computed(() => {
  const image = store.generatedImages[selectedIndex.value];
  const width = image?.width || store.params.width;

  return width;
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
