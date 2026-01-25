<template>
  <div ref="wrapper" class="flex flex-col flex-1 p-2 overflow-hidden">
    <div class="flex items-center justify-between mb-3">
      <span class=""> </span>
      <span v-if="pendingCount && pendingCount > 0" class="text-xs text-surface-50"> {{ images.length }}/{{ totalSlots }} generating... </span>
      <span v-else class="text-xs text-surface-50"> {{ images.length }} {{ images.length === 1 ? 'image' : 'images' }} </span>
    </div>

    <div v-if="totalSlots === 0" class="flex items-center justify-center flex-1">
      <div class="flex flex-col items-center gap-3 text-surface-50">
        <i class="text-5xl pi pi-image"></i>
        <div class="text-center">
          <p class="text-sm font-medium">No images generated yet</p>
          <p class="text-xs text-surface-500">Generated images will appear here</p>
        </div>
      </div>
    </div>

    <div v-else ref="gridContainer" class="flex flex-1 min-h-0 gap-4" :class="layoutClasses">
      <!-- Rendered images -->
      <div
        v-for="(image, index) in images"
        :key="image.id"
        class="relative overflow-hidden bg-gray-800 border border-gray-600 rounded-xl group"
        :style="getCellStyle(index)">
        <Image
          :src="image.src"
          alt="Generated image"
          class="w-full h-full"
          preview
          :image-class="`w-full h-full object-cover ${getObjectFitClass(index)}`"
          @load="(e: Event) => handleImageLoad(e, index)" />
        <Button
          class="absolute z-10 p-4 transition-opacity duration-200 ease-in-out border-blue-800 opacity-0 right-3 top-3 backdrop-blur-lg group-hover:opacity-100 bg-blue-500/30 hover:bg-blue-500 hover:border-blue-800"
          rounded
          severity="secondary"
          @click.stop="handleDownload(image.src, index + 1)">
          <fa :icon="['fal', 'download']" class="text-white" size="lg" />
        </Button>
      </div>

      <!-- Skeleton placeholders for pending images -->
      <FloatingLines v-for="skeletonIndex in pendingCount" :key="`skeleton-${skeletonIndex}`" class="" :style="getCellStyle(images.length + skeletonIndex - 1)">
        <div class="flex flex-col items-center justify-center w-full h-full text-surface-900">
          <div class="text-lg font-medium">Generating <fa :icon="['fas', 'ellipsis']" fade /></div>
        </div>
      </FloatingLines>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, useTemplateRef } from 'vue';
import { useElementSize } from '@vueuse/core';
import Image from 'primevue/image';
import Button from 'primevue/button';
import FloatingLines from '../shared/FloatingLines.vue';

const wrapper = useTemplateRef('wrapper');
const { width, height } = useElementSize(wrapper);

interface GeneratedImage {
  id: string;
  src: string;
}

interface Props {
  images: GeneratedImage[];
  pendingCount?: number;
}

interface Emits {
  (e: 'download', src: string, index: number): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// Total slots to display (images + pending skeletons)
const totalSlots = computed(() => props.images.length + (props.pendingCount || 0));

// Track image dimensions as they load
const imageDimensions = ref<Map<number, { width: number; height: number }>>(new Map());

// Header takes ~36px
const HEADER_HEIGHT = 36;
const GAP = 16; // gap-4

// Container dimensions available for images
const containerWidth = computed(() => width.value || 0);
const containerHeight = computed(() => Math.max(0, (height.value || 0) - HEADER_HEIGHT));
const containerAspect = computed(() => {
  if (containerHeight.value === 0) return 1;
  return containerWidth.value / containerHeight.value;
});

// Handle image load to get dimensions
const handleImageLoad = (event: Event, index: number) => {
  const img = event.target as HTMLImageElement;
  if (img.naturalWidth && img.naturalHeight) {
    imageDimensions.value.set(index, {
      width: img.naturalWidth,
      height: img.naturalHeight,
    });
  }
};

// Get average aspect ratio of loaded images (default to 1:1 if not loaded)
const avgImageAspect = computed(() => {
  if (imageDimensions.value.size === 0) return 1;
  let totalAspect = 0;
  imageDimensions.value.forEach((dims) => {
    totalAspect += dims.width / dims.height;
  });
  return totalAspect / imageDimensions.value.size;
});

// Determine best layout direction based on container and image aspects
// Returns 'row' for horizontal layout, 'col' for vertical layout
const layoutDirection = computed(() => {
  const count = totalSlots.value;
  if (count === 0) return 'row';
  if (count === 1) return 'single';

  // For wide containers (aspect > 1.5), prefer row layout
  // For tall containers (aspect < 0.67), prefer column layout
  // For square-ish containers, consider image aspects
  if (containerAspect.value > 1.5) {
    return 'row';
  } else if (containerAspect.value < 0.67) {
    return 'col';
  } else {
    // Container is roughly square - use image aspect to decide
    // If images are portrait, row works better; if landscape, column works better
    return avgImageAspect.value < 1 ? 'row' : 'col';
  }
});

// Compute layout classes based on direction and count
const layoutClasses = computed(() => {
  const count = totalSlots.value;
  const dir = layoutDirection.value;

  if (count === 1 || dir === 'single') {
    return 'justify-center items-center';
  }

  if (dir === 'row') {
    return 'flex-row flex-wrap justify-center items-center';
  } else {
    return 'flex-col flex-wrap justify-center items-center';
  }
});

// Calculate optimal cell size for each slot (image or skeleton)
const getCellStyle = (index: number) => {
  const count = totalSlots.value;
  const dir = layoutDirection.value;
  const cw = containerWidth.value;
  const ch = containerHeight.value;

  if (cw === 0 || ch === 0) {
    return { width: '100%', height: '100%' };
  }

  // Get image dimensions if available (won't exist for skeletons)
  const imgDims = imageDimensions.value.get(index);
  const imgAspect = imgDims ? imgDims.width / imgDims.height : 1;

  let cellWidth: number;
  let cellHeight: number;

  if (count === 1) {
    // Single slot: fit within container while maintaining aspect ratio
    if (imgDims) {
      const fitByWidth = { w: cw, h: cw / imgAspect };
      const fitByHeight = { w: ch * imgAspect, h: ch };

      if (fitByWidth.h <= ch) {
        cellWidth = fitByWidth.w;
        cellHeight = fitByWidth.h;
      } else {
        cellWidth = fitByHeight.w;
        cellHeight = fitByHeight.h;
      }
    } else {
      // No dimensions yet (or skeleton), use square that fits container
      cellWidth = Math.min(cw, ch);
      cellHeight = Math.min(cw, ch);
    }
  } else if (count === 2) {
    if (dir === 'row') {
      // Side by side
      cellWidth = (cw - GAP) / 2;
      cellHeight = ch;
    } else {
      // Stacked
      cellWidth = cw;
      cellHeight = (ch - GAP) / 2;
    }
  } else if (count === 3) {
    if (dir === 'row') {
      // Three in a row
      cellWidth = (cw - GAP * 2) / 3;
      cellHeight = ch;
    } else {
      // Three stacked
      cellWidth = cw;
      cellHeight = (ch - GAP * 2) / 3;
    }
  } else if (count === 4) {
    // 2x2 grid
    cellWidth = (cw - GAP) / 2;
    cellHeight = (ch - GAP) / 2;
  } else {
    // Fallback for more slots
    cellWidth = (cw - GAP) / 2;
    cellHeight = (ch - GAP) / 2;
  }

  return {
    width: `${cellWidth}px`,
    height: `${cellHeight}px`,
    flexShrink: 0,
  };
};

// Determine object-fit class based on how well image fits cell
const getObjectFitClass = (index: number) => {
  const imgDims = imageDimensions.value.get(index);
  if (!imgDims) return 'object-contain'; // Show full image by default

  const count = totalSlots.value;
  const dir = layoutDirection.value;
  const cw = containerWidth.value;
  const ch = containerHeight.value;

  if (cw === 0 || ch === 0) return 'object-contain';

  // Calculate cell dimensions
  let cellWidth: number;
  let cellHeight: number;

  if (count === 1) {
    // Single image uses object-contain to show full image
    return 'object-contain';
  } else if (count === 2) {
    if (dir === 'row') {
      cellWidth = (cw - GAP) / 2;
      cellHeight = ch;
    } else {
      cellWidth = cw;
      cellHeight = (ch - GAP) / 2;
    }
  } else if (count === 3) {
    if (dir === 'row') {
      cellWidth = (cw - GAP * 2) / 3;
      cellHeight = ch;
    } else {
      cellWidth = cw;
      cellHeight = (ch - GAP * 2) / 3;
    }
  } else {
    cellWidth = (cw - GAP) / 2;
    cellHeight = (ch - GAP) / 2;
  }

  const imgAspect = imgDims.width / imgDims.height;
  const cellAspect = cellWidth / cellHeight;

  // Calculate how much of the cell would be filled with object-contain
  let fillRatio: number;
  if (imgAspect > cellAspect) {
    // Image is wider than cell - letterboxed top/bottom
    fillRatio = cellAspect / imgAspect;
  } else {
    // Image is taller than cell - letterboxed left/right
    fillRatio = imgAspect / cellAspect;
  }

  // If image fills less than 50% of cell area, use cover to avoid too much dead space
  // Otherwise use contain to show the full image
  const threshold = 0.5;
  return fillRatio >= threshold ? 'object-contain' : 'object-cover';
};

const handleDownload = (src: string, index: number) => {
  emit('download', src, index);
};

// Reset dimensions when images change
watch(
  () => props.images.length,
  () => {
    imageDimensions.value.clear();
  },
);
</script>

<style scoped>
@reference "tailwindcss";

.skeleton-pulse {
  background: linear-gradient(90deg, rgba(55, 65, 81, 0.4) 0%, rgba(75, 85, 99, 0.6) 50%, rgba(55, 65, 81, 0.4) 100%);
  background-size: 200% 100%;
  animation: skeleton-shimmer 1.5s ease-in-out infinite;
}

@keyframes skeleton-shimmer {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}
</style>
