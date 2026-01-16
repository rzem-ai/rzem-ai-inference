<script setup lang="ts">
import { computed } from 'vue'
import Image from 'primevue/image'
import Checkbox from 'primevue/checkbox'
import Button from 'primevue/button'
import { convertFileSrc } from '@tauri-apps/api/core'
import type { GalleryImage } from '@/stores/gallery'

interface Props {
  images: GalleryImage[]
  selectedImages: Set<string>
  selectionMode?: boolean
}

interface Emits {
  (e: 'toggle-select', imageId: string): void
  (e: 'toggle-favorite', imageId: string): void
  (e: 'view-details', imageId: string): void
}

const props = withDefaults(defineProps<Props>(), {
  selectionMode: false,
})

const emit = defineEmits<Emits>()

const convertedImages = computed(() =>
  props.images.map((img) => ({
    ...img,
    displayPath: convertFileSrc(img.filePath),
  }))
)

function handleToggleSelect(imageId: string) {
  emit('toggle-select', imageId)
}

function handleToggleFavorite(imageId: string, event: Event) {
  event.stopPropagation()
  emit('toggle-favorite', imageId)
}

function handleViewDetails(imageId: string) {
  emit('view-details', imageId)
}
</script>

<template>
  <div class="image-grid">
    <div
      v-for="image in convertedImages"
      :key="image.id"
      class="image-card"
      :class="{ selected: selectedImages.has(image.id) }"
    >
      <div class="image-container" @click="handleViewDetails(image.id)">
        <Image
          :src="image.displayPath"
          :alt="image.prompt"
          class="gallery-image"
          preview
        />

        <!-- Favorite badge -->
        <Button
          v-if="image.isFavorite"
          icon="pi pi-star-fill"
          class="favorite-badge"
          severity="warning"
          rounded
          text
          @click="handleToggleFavorite(image.id, $event)"
        />

        <!-- Selection checkbox -->
        <Checkbox
          v-if="selectionMode"
          :model-value="selectedImages.has(image.id)"
          class="selection-checkbox"
          binary
          @update:model-value="handleToggleSelect(image.id)"
          @click.stop
        />
      </div>

      <div class="image-info">
        <p class="image-prompt">{{ image.prompt }}</p>
        <div class="image-meta">
          <span>{{ new Date(image.createdAt).toLocaleDateString() }}</span>
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
  gap: 1.5rem;
  padding: 1rem;
}

.image-card {
  border: 1px solid var(--surface-border);
  border-radius: var(--border-radius);
  overflow: hidden;
  transition: all 0.2s;
  cursor: pointer;
}

.image-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  transform: translateY(-2px);
}

.image-card.selected {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 2px var(--primary-color);
}

.image-container {
  position: relative;
  aspect-ratio: 1;
  background: var(--surface-ground);
}

.gallery-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.favorite-badge {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
}

.selection-checkbox {
  position: absolute;
  top: 0.5rem;
  left: 0.5rem;
}

.image-info {
  padding: 1rem;
}

.image-prompt {
  font-size: 0.875rem;
  margin: 0 0 0.5rem 0;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.image-meta {
  display: flex;
  justify-content: space-between;
  font-size: 0.75rem;
  color: var(--text-color-secondary);
}
</style>
