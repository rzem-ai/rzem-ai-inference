<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import type { GalleryImage } from '@/stores/gallery'
import Image from 'primevue/image'
import Checkbox from 'primevue/checkbox'
import Button from 'primevue/button'

interface Props {
  images: GalleryImage[]
  selectedIds: Set<string>
}

defineProps<Props>()

const emit = defineEmits<{
  select: [imageId: string]
  openDetail: [image: GalleryImage]
  toggleFavorite: [imageId: string]
}>()

const getImageSrc = (filePath: string) => {
  return convertFileSrc(filePath)
}
</script>

<template>
  <div class="image-grid">
    <div
      v-for="image in images"
      :key="image.id"
      class="image-card"
      :class="{ selected: selectedIds.has(image.id) }"
    >
      <div class="image-checkbox">
        <Checkbox
          :model-value="selectedIds.has(image.id)"
          @change="emit('select', image.id)"
          binary
        />
      </div>

      <div class="image-container" @click="emit('openDetail', image)">
        <Image
          :src="getImageSrc(image.filePath)"
          :alt="image.prompt"
          preview
        />
      </div>

      <div class="image-actions">
        <Button
          icon="pi pi-heart"
          :severity="image.isFavorite ? 'danger' : 'secondary'"
          text
          rounded
          @click.stop="emit('toggleFavorite', image.id)"
          :title="image.isFavorite ? 'Remove from favorites' : 'Add to favorites'"
        />
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

<style scoped>
.image-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 1rem;
  padding: 1rem;
}

.image-card {
  position: relative;
  border: 2px solid transparent;
  border-radius: 0.5rem;
  background: white;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  transition: all 0.2s;
  overflow: hidden;
}

.image-card:hover {
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.15);
  transform: translateY(-2px);
}

.image-card.selected {
  border-color: #3b82f6;
  background: #eff6ff;
}

.image-checkbox {
  position: absolute;
  top: 0.5rem;
  left: 0.5rem;
  z-index: 10;
  background: white;
  border-radius: 0.25rem;
  padding: 0.25rem;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.image-container {
  cursor: pointer;
  aspect-ratio: 1;
  overflow: hidden;
  background: #f3f4f6;
}

.image-container :deep(img) {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.image-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem;
  border-top: 1px solid #e5e7eb;
}

.image-date {
  font-size: 0.75rem;
  color: #6b7280;
}

.image-info {
  padding: 0.75rem;
  background: #f9fafb;
}

.image-prompt {
  margin: 0 0 0.5rem 0;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: #374151;
}

.image-meta {
  display: flex;
  gap: 0.75rem;
  font-size: 0.75rem;
  color: #6b7280;
}

.image-meta span {
  display: flex;
  align-items: center;
}
</style>
