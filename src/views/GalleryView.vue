<template>
  <div class="workspace-content gallery-view">
    <div class="gallery-header">
      <h1>Gallery</h1>

      <div class="search-bar">
        <InputText v-model="galleryStore.filters.searchQuery" placeholder="Search prompts..." class="search-input" @keyup.enter="handleSearch" />
        <Button icon="pi pi-search" @click="handleSearch" :loading="galleryStore.isLoading" />
      </div>

      <div class="gallery-actions">
        <Button label="Select All" icon="pi pi-check-square" severity="secondary" @click="galleryStore.selectAll" />
        <Button
          label="Clear Selection"
          icon="pi pi-times"
          severity="secondary"
          @click="galleryStore.clearSelection"
          :disabled="galleryStore.selectedImages.size === 0" />
        <span class="selection-count"> {{ galleryStore.selectedImages.size }} selected </span>
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
      @add-to-compare="handleAddToCompare" />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useGalleryStore } from '@/stores/gallery';
import { useCompareStore } from '@/stores/compare';
import ImageGrid from '@/components/gallery/ImageGrid.vue';
import InputText from 'primevue/inputtext';
import Button from 'primevue/button';

const galleryStore = useGalleryStore();
const compareStore = useCompareStore();

onMounted(async () => {
  await galleryStore.loadImages();
});

const handleSearch = async () => {
  if (galleryStore.filters.searchQuery.trim()) {
    await galleryStore.searchImages(galleryStore.filters.searchQuery);
  } else {
    await galleryStore.loadImages();
  }
};

const handleToggleFavorite = async (imageId: string) => {
  await galleryStore.toggleFavorite(imageId);
};

const handleSelectImage = (imageId: string) => {
  galleryStore.toggleSelectImage(imageId);
};

const handleOpenDetail = (image: any) => {
  console.log('Open detail for:', image);
  // TODO: Open image detail modal (Task 4)
};

const handleAddToCompare = (image: any) => {
  const success = compareStore.addToCompare(image);
  if (!success) {
    console.warn('Cannot add more images to compare');
  }
};
</script>

<style scoped>
@reference "tailwindcss";

.gallery-view {
  @apply flex flex-col h-full overflow-hidden;
  background-color: var(--color-slate-950);
}

.gallery-header {
  @apply p-6 border-b;
  border-color: var(--color-slate-800);
  background-color: var(--color-slate-950);

  h1 {
    @apply m-0 mb-4 text-2xl font-semibold;
    color: var(--color-slate-50);
  }
}

.search-bar {
  @apply flex gap-2 mb-4;
}

.search-input {
  @apply flex-1 max-w-lg;
}

.gallery-actions {
  @apply flex gap-2 items-center;
}

.selection-count {
  @apply ml-auto text-sm;
  color: var(--color-slate-400);
}

.loading-state,
.empty-state {
  @apply flex flex-col items-center justify-center flex-1 gap-4;
  color: var(--color-slate-500);
}

.empty-hint {
  @apply text-sm;
  color: var(--color-slate-600);
}
</style>
