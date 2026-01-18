<template>
  <div class="image-grid">
    <div v-for="image in images" :key="image.id" class="image-card" :class="{ selected: selectedIds.has(image.id) }">
      <div class="image-checkbox">
        <Checkbox :model-value="selectedIds.has(image.id)" @change="emit('select', image.id)" binary />
      </div>

      <div class="image-container" @click="emit('openDetail', image)">
        <Image :src="getImageSrc(image.filePath)" :alt="image.prompt" preview />
      </div>

      <div class="image-actions">
        <Button
          icon="pi pi-heart"
          :severity="image.isFavorite ? 'danger' : 'secondary'"
          text
          rounded
          @click.stop="emit('toggleFavorite', image.id)"
          :title="image.isFavorite ? 'Remove from favorites' : 'Add to favorites'" />
        <Button icon="pi pi-clone" severity="secondary" text rounded @click.stop="emit('addToCompare', image)" title="Add to compare" />
        <span class="image-date">
          {{ new Date(image.createdAt).toLocaleDateString() }}
        </span>
      </div>

      <div class="image-info">
        <p class="image-prompt">{{ image.prompt.substring(0, 60) }}{{ image.prompt.length > 60 ? '...' : '' }}</p>
        <div class="image-meta">
          <span>{{ image.width }}×{{ image.height }}</span>
          <span>{{ image.modelName }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core';
import type { GalleryImage } from '@/stores/gallery';
import Image from 'primevue/image';
import Checkbox from 'primevue/checkbox';
import Button from 'primevue/button';

interface Props {
  images: GalleryImage[];
  selectedIds: Set<string>;
}

defineProps<Props>();

const emit = defineEmits<{
  select: [imageId: string];
  openDetail: [image: GalleryImage];
  toggleFavorite: [imageId: string];
  addToCompare: [image: GalleryImage];
}>();

const getImageSrc = (filePath: string) => {
  return convertFileSrc(filePath);
};
</script>

<style scoped>
@reference "tailwindcss";

.image-grid {
  @apply grid gap-4 p-4;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
}

.image-card {
  @apply relative border-2 border-transparent rounded-lg bg-white shadow-sm transition-all duration-200 overflow-hidden;

  &:hover {
    @apply shadow-md -translate-y-0.5;
  }

  &.selected {
    @apply border-blue-500 bg-blue-50;
  }
}

.image-checkbox {
  @apply absolute top-2 left-2 z-10 bg-white rounded p-1 shadow;
}

.image-container {
  @apply cursor-pointer aspect-square overflow-hidden bg-gray-100;

  :deep(img) {
    @apply w-full h-full object-cover;
  }
}

.image-actions {
  @apply flex items-center justify-between p-2 border-t border-gray-200;
}

.image-date {
  @apply text-xs text-gray-500;
}

.image-info {
  @apply p-3 bg-gray-50;
}

.image-prompt {
  @apply m-0 mb-2 text-sm leading-5 text-gray-700;
}

.image-meta {
  @apply flex gap-3 text-xs text-gray-500;

  span {
    @apply flex items-center;
  }
}
</style>
