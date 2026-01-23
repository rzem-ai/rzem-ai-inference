<template>
  <div class="flex flex-col h-full bg-gray-900">
    <div class="bg-gray-900">
      <h1>Compare Images</h1>
      <div class="compare-actions">
        <span class="compare-count"> {{ compareStore.compareCount }} / {{ compareStore.maxCompareImages }} images </span>
        <Button label="Clear All" icon="pi pi-times" severity="secondary" @click="compareStore.clearCompare" :disabled="compareStore.compareCount === 0" />
      </div>
    </div>

    <div v-if="compareStore.compareCount === 0" class="bg-gray-700 empty-state">
      <i class="pi pi-images" style="font-size: 3rem; color: #9ca3af"></i>
      <p>No images to compare</p>
      <p class="empty-hint">Add images from the gallery to compare them side by side</p>
    </div>

    <div v-else class="bg-gray-700 compare-grid" :style="{ gridTemplateColumns: `repeat(${compareStore.compareCount}, 1fr)` }">
      <div v-for="(image, index) in compareStore.compareImages" :key="image.id" class="compare-item">
        <div class="compare-image-header">
          <span class="compare-index">#{{ index + 1 }}</span>
          <Button icon="pi pi-times" severity="danger" text rounded size="small" @click="handleRemove(image.id)" />
        </div>

        <div class="compare-image-container">
          <Image :src="getImageSrc(image.filePath)" :alt="image.prompt" preview />
        </div>

        <div class="compare-metadata">
          <div class="metadata-item">
            <span class="metadata-label">Prompt:</span>
            <span class="metadata-value">{{ image.prompt }}</span>
          </div>

          <div class="metadata-item">
            <span class="metadata-label">Model:</span>
            <span class="metadata-value" :class="getParameterDiff(image, compareStore.compareImages[0], 'modelName')">
              {{ image.modelName }}
            </span>
          </div>

          <div class="metadata-item">
            <span class="metadata-label">Steps:</span>
            <span class="metadata-value" :class="getParameterDiff(image, compareStore.compareImages[0], 'steps')">
              {{ image.steps ?? 'N/A' }}
            </span>
          </div>

          <div class="metadata-item">
            <span class="metadata-label">CFG:</span>
            <span class="metadata-value" :class="getParameterDiff(image, compareStore.compareImages[0], 'cfgScale')">
              {{ image.cfgScale ?? 'N/A' }}
            </span>
          </div>

          <div class="metadata-item">
            <span class="metadata-label">Size:</span>
            <span class="metadata-value" :class="getSizeDiff(image)"> {{ image.width }}×{{ image.height }} </span>
          </div>

          <div class="metadata-item">
            <span class="metadata-label">Seed:</span>
            <span class="metadata-value" :class="getParameterDiff(image, compareStore.compareImages[0], 'seed')">
              {{ image.seed ?? 'N/A' }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useCompareStore } from '@/stores/compare';
import type { GalleryImage } from '@/stores/gallery';
import { convertFileSrc } from '@tauri-apps/api/core';
import Button from 'primevue/button';
import Image from 'primevue/image';

const compareStore = useCompareStore();

const getImageSrc = (filePath: string) => {
  return convertFileSrc(filePath);
};

const handleRemove = (imageId: string) => {
  compareStore.removeFromCompare(imageId);
};

const getParameterDiff = (image: GalleryImage, compareToImage: GalleryImage, param: string) => {
  if (!compareToImage) return null;
  const value = image[param as keyof GalleryImage];
  const compareValue = compareToImage[param as keyof GalleryImage];
  if (value !== compareValue) {
    return 'different';
  }
  return 'same';
};

const getSizeDiff = (image: GalleryImage): string => {
  if (compareStore.compareCount < 2) return '';

  const first = compareStore.compareImages[0];
  return image.width !== first.width || image.height !== first.height ? 'different' : 'same';
};
</script>

<style scoped>
@reference "tailwindcss";

.compare-view {
  @apply flex flex-col h-full overflow-hidden;
}

.compare-header {
  @apply p-6 border-b border-gray-200 bg-white flex justify-between items-center;

  h1 {
    @apply m-0 text-2xl font-semibold;
  }
}

.compare-actions {
  @apply flex gap-4 items-center;
}

.compare-count {
  @apply text-sm text-gray-500;
}

.empty-state {
  @apply flex flex-col items-center justify-center flex-1 gap-4 text-gray-500;
}

.empty-hint {
  @apply text-sm text-gray-400;
}

.compare-grid {
  @apply grid gap-4 p-4 overflow-y-auto flex-1;
}

.compare-item {
  @apply flex flex-col bg-white border border-gray-200 rounded-lg overflow-hidden;
}

.compare-image-header {
  @apply flex justify-between items-center p-2 bg-gray-50 border-b border-gray-200;
}

.compare-index {
  @apply font-semibold text-gray-700;
}

.compare-image-container {
  @apply aspect-square overflow-hidden bg-gray-100;

  :deep(img) {
    @apply w-full h-full object-contain;
  }
}

.compare-metadata {
  @apply p-4 flex flex-col gap-2;
}

.metadata-item {
  @apply flex flex-col gap-1;
}

.metadata-label {
  @apply text-xs font-semibold text-gray-500 uppercase;
}

.metadata-value {
  @apply text-sm text-gray-700 break-words;

  &.different {
    @apply text-red-600 font-semibold;
  }

  &.same {
    @apply text-green-600;
  }
}
</style>
