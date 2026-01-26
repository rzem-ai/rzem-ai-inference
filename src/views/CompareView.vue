<template>
  <div class="flex flex-col h-full ">
    <div class="">
      <h1>Compare Images</h1>
      <div class="compare-actions">
        <span class="compare-count"> {{ compareStore.compareCount }} / {{ compareStore.maxCompareImages }} images </span>
        <Button label="Clear All" icon="pi pi-times" severity="secondary" @click="compareStore.clearCompare" :disabled="compareStore.compareCount === 0" />
      </div>
    </div>

    <div v-if="compareStore.compareCount === 0" class="flex flex-col items-center justify-center flex-1 gap-4 bg-surface-700 text-surface-500">
      <i class="pi pi-images" style="font-size: 3rem; color: #9ca3af"></i>
      <p>No images to compare</p>
      <p class="text-sm text-surface-400">Add images from the gallery to compare them side by side</p>
    </div>

    <div v-else class="bg-surface-700 compare-grid" :style="{ gridTemplateColumns: `repeat(${compareStore.compareCount}, 1fr)` }">
      <div v-for="(image, index) in compareStore.compareImages" :key="image.id" class="flex flex-col overflow-hidden bg-white border rounded-lg border-surface-200">
        <div class="flex items-center justify-between p-2 border-b bg-surface-50 border-surface-200">
          <span class="font-semibold text-surface-700">#{{ index + 1 }}</span>
          <Button icon="pi pi-times" severity="danger" text rounded size="small" @click="handleRemove(image.id)" />
        </div>

        <div class="overflow-hidden aspect-square bg-surface-100">
          <Image :src="getImageSrc(image.filePath)" :alt="image.prompt" preview />
        </div>

        <div class="compare-metadata">
          <div class="metadata-item">
            <span class="text-xs font-semibold uppercase text-surface-500">Prompt:</span>
            <span class="text-sm break-words text-surface-700">{{ image.prompt }}</span>
          </div>

          <div class="metadata-item">
            <span class="text-xs font-semibold uppercase text-surface-500">Model:</span>
            <span class="text-sm break-words text-surface-700" :class="getParameterDiff(image, compareStore.compareImages[0], 'modelName')">
              {{ image.modelName }}
            </span>
          </div>

          <div class="metadata-item">
            <span class="text-xs font-semibold uppercase text-surface-500">Steps:</span>
            <span class="text-sm break-words text-surface-700" :class="getParameterDiff(image, compareStore.compareImages[0], 'steps')">
              {{ image.steps ?? 'N/A' }}
            </span>
          </div>

          <div class="metadata-item">
            <span class="text-xs font-semibold uppercase text-surface-500">CFG:</span>
            <span class="text-sm break-words text-surface-700" :class="getParameterDiff(image, compareStore.compareImages[0], 'cfgScale')">
              {{ image.cfgScale ?? 'N/A' }}
            </span>
          </div>

          <div class="metadata-item">
            <span class="text-xs font-semibold uppercase text-surface-500">Size:</span>
            <span class="text-sm break-words text-surface-700" :class="getSizeDiff(image)"> {{ image.width }}×{{ image.height }} </span>
          </div>

          <div class="metadata-item">
            <span class="text-xs font-semibold uppercase text-surface-500">Seed:</span>
            <span class="text-sm break-words text-surface-700" :class="getParameterDiff(image, compareStore.compareImages[0], 'seed')">
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

 

.compare-actions {
  @apply flex gap-4 items-center;
}

 

 

 

.compare-grid {
  @apply grid gap-4 p-4 overflow-y-auto flex-1;
}

 

 
 

.compare-image-container {
   

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

 
.metadata-value {
 

  &.different {
    @apply text-red-600 font-semibold;
  }

  &.same {
    @apply text-green-600;
  }
}
</style>
