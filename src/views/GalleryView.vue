<script setup lang="ts">
import { onMounted } from 'vue'
import { useGalleryStore } from '@/stores/gallery'
import { useCompareStore } from '@/stores/compare'
import ImageGrid from '@/components/gallery/ImageGrid.vue'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'

const galleryStore = useGalleryStore()
const compareStore = useCompareStore()

onMounted(async () => {
  await galleryStore.loadImages()
})

const handleSearch = async () => {
  if (galleryStore.filters.searchQuery.trim()) {
    await galleryStore.searchImages(galleryStore.filters.searchQuery)
  } else {
    await galleryStore.loadImages()
  }
}

const handleToggleFavorite = async (imageId: string) => {
  await galleryStore.toggleFavorite(imageId)
}

const handleSelectImage = (imageId: string) => {
  galleryStore.toggleSelectImage(imageId)
}

const handleOpenDetail = (image: any) => {
  console.log('Open detail for:', image)
  // TODO: Open image detail modal (Task 4)
}

const handleAddToCompare = (image: any) => {
  const success = compareStore.addToCompare(image)
  if (!success) {
    console.warn('Cannot add more images to compare')
  }
}
</script>

<template>
  <div class="workspace-content gallery-view">
    <div class="gallery-header">
      <h1>Gallery</h1>

      <div class="search-bar">
        <InputText
          v-model="galleryStore.filters.searchQuery"
          placeholder="Search prompts..."
          class="search-input"
          @keyup.enter="handleSearch"
        />
        <Button
          icon="pi pi-search"
          @click="handleSearch"
          :loading="galleryStore.isLoading"
        />
      </div>

      <div class="gallery-actions">
        <Button
          label="Select All"
          icon="pi pi-check-square"
          severity="secondary"
          @click="galleryStore.selectAll"
        />
        <Button
          label="Clear Selection"
          icon="pi pi-times"
          severity="secondary"
          @click="galleryStore.clearSelection"
          :disabled="galleryStore.selectedImages.size === 0"
        />
        <span class="selection-count">
          {{ galleryStore.selectedImages.size }} selected
        </span>
      </div>
    </div>

    <div v-if="galleryStore.isLoading" class="loading-state">
      <i class="pi pi-spin pi-spinner" style="font-size: 2rem"></i>
      <p>Loading images...</p>
    </div>

    <div v-else-if="galleryStore.filteredImages.length === 0" class="empty-state">
      <i class="pi pi-images" style="font-size: 3rem; color: #9ca3af"></i>
      <p>No images found</p>
      <p class="empty-hint">Generate some images to see them here!</p>
    </div>

    <ImageGrid
      v-else
      :images="galleryStore.filteredImages"
      :selected-ids="galleryStore.selectedImages"
      @select="handleSelectImage"
      @open-detail="handleOpenDetail"
      @toggle-favorite="handleToggleFavorite"
      @add-to-compare="handleAddToCompare"
    />
  </div>
</template>

<style scoped>
.gallery-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.gallery-header {
  padding: 1.5rem;
  border-bottom: 1px solid #e5e7eb;
  background: white;
}

.gallery-header h1 {
  margin: 0 0 1rem 0;
  font-size: 1.5rem;
  font-weight: 600;
}

.search-bar {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.search-input {
  flex: 1;
  max-width: 500px;
}

.gallery-actions {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.selection-count {
  margin-left: auto;
  font-size: 0.875rem;
  color: #6b7280;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 1rem;
  color: #6b7280;
}

.empty-hint {
  font-size: 0.875rem;
  color: #9ca3af;
}
</style>
