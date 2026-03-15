<template>
  <div class="flex flex-col h-full px-2 py-4 gap-2">
    <!-- Grid view when active -->
    <GridView v-if="store.gridConfig" class="flex-1" />

    <!-- Preview area — ref on the stable outer container whose size is determined by flex layout -->
    <div v-else ref="imageWrapper" class="relative place-items-center h-full w-full overflow-hidden">
      <div v-if="store.isGenerating && store.previewDataUrl" class="rounded-2xl p-4 h-full flex justify-center w-full">
        <div class="w-full h-full flex justify-center items-center">
          <div>
            <img :src="store.previewDataUrl" alt="Generation preview" class="rounded-2xl" style="filter: blur(1px)" :style="placeholderStyle" />
          </div>
        </div>
      </div>

      <div v-else-if="displayedImage?.dataUrl" class="rounded-2xl p-4 h-full flex justify-center w-full">
        <div class="w-full h-full flex justify-center items-center">
          <div>
            <img :src="displayedImage.dataUrl" alt="Generated image" class="rounded-2xl" :style="placeholderStyle" />
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-else class="rounded-2xl p-4 h-full flex justify-center w-full">
        <div class="w-full h-full flex justify-center items-center">
          <div class="border-surface-200 border rounded-2xl flex flex-col justify-center text-center bg-surface-100" :style="placeholderStyle">
            <ImageIcon :size="48" class="w-full text-slate-500" />
            <div class="text-lg text-slate-500">Generated images will appear here</div>
            <div class="text-base text-slate-400">Enter a prompt and click Generate to begin</div>
          </div>
        </div>
      </div>

      <!-- Error overlay -->
      <ErrorOverlay v-if="store.error && !store.isGenerating" />
    </div>

    <!-- History strip -->
    <History />
  </div>
</template>

<script setup lang="ts">
import { computed, useTemplateRef } from 'vue';
import { useInferenceStore } from '@/stores/inference';
import { useElementSize } from '@vueuse/core';

import History from './History.vue';
import ErrorOverlay from './ErrorOverlay.vue';
import GridView from './GridView.vue';

const imageWrapper = useTemplateRef('imageWrapper');
const imageWrapperSize = useElementSize(imageWrapper);

const store = useInferenceStore();
const displayedImage = computed(() => store.selectedImage);

const calculateMaxImageDimensions = function (imageWidth: number, imageHeight: number, viewportWidth: number, viewportHeight: number) {
  const imageAspectRatio = imageWidth / imageHeight;
  const viewportAspectRatio = viewportWidth / viewportHeight;

  let displayWidth, displayHeight;

  if (imageAspectRatio > viewportAspectRatio) {
    // Image is wider relative to viewport — constrain by width
    displayWidth = viewportWidth;
    displayHeight = viewportWidth / imageAspectRatio;
  } else {
    // Image is taller relative to viewport — constrain by height
    displayHeight = viewportHeight;
    displayWidth = viewportHeight * imageAspectRatio;
  }

  return {
    width: Math.round(displayWidth),
    height: Math.round(displayHeight),
  };
};

// Calculate placeholder dimensions based on aspect ratio
// Subtract padding (p-4 = 16px each side) from the observed wrapper size
const PADDING = 32;
const placeholderStyle = computed(() => {
  const availableWidth = Math.max(0, imageWrapperSize.width.value - PADDING);
  const availableHeight = Math.max(0, imageWrapperSize.height.value - PADDING);

  let imageDimensions = { width: 0, height: 0 };
  if (store.isGenerating && store.previewDataUrl) {
    imageDimensions = calculateMaxImageDimensions(store.params.width, store.params.height, availableWidth, availableHeight);
  } else if (displayedImage?.value?.dataUrl) {
    imageDimensions = calculateMaxImageDimensions(
      store.selectedImage?.width || store.params.width,
      store.selectedImage?.height || store.params.height,
      availableWidth,
      availableHeight,
    );
  } else {
    imageDimensions = calculateMaxImageDimensions(store.params.width, store.params.height, availableWidth, availableHeight);
  }

  const targetWidth = imageDimensions.width;
  const targetHeight = imageDimensions.height;

  // For landscape (wide) images: constrain HEIGHT so width can expand
  // For portrait (tall) images: constrain WIDTH so height can expand
  return {
    height: `${targetHeight}px`,
    width: `${targetWidth}px`,
    aspectRatio: `${targetWidth} / ${targetHeight}`,
  };
});
</script>
