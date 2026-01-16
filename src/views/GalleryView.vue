<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'
import ImageGrid from '@/components/gallery/ImageGrid.vue'
import { useGalleryStore } from '@/stores/gallery'

const router = useRouter()
const galleryStore = useGalleryStore()

const searchQuery = ref('')
const selectionMode = ref(false)

onMounted(async () => {
  await galleryStore.loadImages()
})

async function handleSearch() {
  if (searchQuery.value.trim()) {
    await galleryStore.searchImages(searchQuery.value)
  } else {
    await galleryStore.loadImages()
  }
}

function handleToggleSelect(imageId: string) {
  galleryStore.toggleSelectImage(imageId)
}

async function handleToggleFavorite(imageId: string) {
  await galleryStore.toggleFavorite(imageId)
}

function handleViewDetails(imageId: string) {
  // TODO: Navigate to image details view
  console.log('View details:', imageId)
}

function handleSelectAll() {
  galleryStore.selectAll()
}

function handleClearSelection() {
  galleryStore.clearSelection()
  selectionMode.value = false
}

async function handleDeleteSelected() {
  const selected = galleryStore.selectedImagesList
  for (const image of selected) {
    await galleryStore.deleteImages(image.id)
  }
  galleryStore.clearSelection()
  selectionMode.value = false
}

function handleCompareSelected() {
  const selectedIds = Array.from(galleryStore.selectedImages)
  router.push({
    name: 'compare',
    query: { images: selectedIds.join(',') },
  })
}
</script>

<template>
  <div class="gallery-view">
    <div class="gallery-header">
      <h1>Gallery</h1>

      <div class="gallery-controls">
        <!-- Search -->
        <div class="search-container">
          <InputText
            v-model="searchQuery"
            placeholder="Search images..."
            class="search-input"
            @keyup.enter="handleSearch"
          />
          <Button
            icon="pi pi-search"
            @click="handleSearch"
          />
        </div>

        <!-- Action buttons -->
        <div class="action-buttons">
          <Button
            :label="selectionMode ? 'Cancel' : 'Select'"
            :icon="selectionMode ? 'pi pi-times' : 'pi pi-check-square'"
            outlined
            @click="selectionMode = !selectionMode"
          />

          <template v-if="selectionMode && galleryStore.selectedImages.size > 0">
            <Button
              label="Select All"
              icon="pi pi-check-circle"
              outlined
              @click="handleSelectAll"
            />
            <Button
              label="Clear"
              icon="pi pi-times-circle"
              outlined
              @click="handleClearSelection"
            />
            <Button
              label="Compare"
              icon="pi pi-images"
              severity="info"
              :disabled="galleryStore.selectedImages.size < 2 || galleryStore.selectedImages.size > 4"
              @click="handleCompareSelected"
            />
            <Button
              label="Delete"
              icon="pi pi-trash"
              severity="danger"
              @click="handleDeleteSelected"
            />
          </template>
        </div>
      </div>
    </div>

    <!-- Image grid -->
    <ImageGrid
      :images="galleryStore.filteredImages"
      :selected-images="galleryStore.selectedImages"
      :selection-mode="selectionMode"
      @toggle-select="handleToggleSelect"
      @toggle-favorite="handleToggleFavorite"
      @view-details="handleViewDetails"
    />

    <!-- Empty state -->
    <div v-if="galleryStore.filteredImages.length === 0" class="empty-state">
      <i class="pi pi-image" style="font-size: 3rem"></i>
      <p>No images found</p>
    </div>
  </div>
</template>

<style scoped>
.gallery-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 2rem;
}

.gallery-header {
  margin-bottom: 2rem;
}

.gallery-header h1 {
  margin: 0 0 1rem 0;
}

.gallery-controls {
  display: flex;
  gap: 1rem;
  align-items: center;
  flex-wrap: wrap;
}

.search-container {
  display: flex;
  gap: 0.5rem;
  flex: 1;
  max-width: 400px;
}

.search-input {
  flex: 1;
}

.action-buttons {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4rem;
  color: var(--text-color-secondary);
}
</style>
