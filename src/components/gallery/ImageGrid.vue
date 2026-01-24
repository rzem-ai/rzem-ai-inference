<template>
  <div ref="containerRef" class="virtual-grid-container">
    <VirtualScroller :items="imageRows" :itemSize="ROW_HEIGHT" class="virtual-scroller" :pt="{ content: { class: 'virtual-content' } }">
      <template #item="{ item: row }">
        <div class="image-row" :style="{ height: ROW_HEIGHT + 'px' }">
          <div
            v-for="image in row"
            :key="image.id"
            class="image-card"
            :class="{
              'border-blue-500! bg-surface-700!': selectedIds.has(image.id),
              'opacity-50 scale-95': isDragging && draggedImageIds.has(image.id),
            }"
            draggable="true"
            @dragstart="handleDragStart($event, image)"
            @dragend="handleDragEnd">
            <div class="absolute z-10 p-1 rounded shadow top-2 left-2">
              <Checkbox :model-value="selectedIds.has(image.id)" @change="emit('select', image.id)" binary />
            </div>

            <div class="image-container" >
              <Image :src="getImageSrc(image)" :preview-src="getThumbnailSrc(image)" :alt="image.prompt" preview />

              <!-- Drag overlay showing count -->
              <div
                v-if="isDragging && draggedImageIds.has(image.id) && draggedImageIds.size > 1"
                class="absolute px-2 py-1 text-xs font-bold text-white bg-blue-600 rounded-full top-2 right-2">
                {{ draggedImageIds.size }}
              </div>
            </div>

            <div class="flex flex-col" @click="emit('openDetail', image)">
              <div class="flex items-center justify-between p-2 border-t border-gray-700">
                <Button
                  :severity="image.isFavorite ? 'danger' : 'secondary'"
                  text
                  rounded
                  @click.stop="emit('toggleFavorite', image.id)"
                  :title="image.isFavorite ? 'Remove from favorites' : 'Add to favorites'">
                  <template #icon><Heart :size="14" /></template>
                </Button>
                <Button severity="secondary" text rounded @click.stop="emit('addToCompare', image)" title="Add to compare">
                  <template #icon><Copy :size="14" /></template>
                </Button>
                <span class="text-xs text-gray-400">
                  {{ new Date(image.createdAt * 1000).toLocaleDateString() }}
                </span>
              </div>

              <div class="p-3 bg-gray-800">
                <p class="m-0 mb-2 text-sm leading-5 text-gray-300">{{ image.prompt.substring(0, 60) }}{{ image.prompt.length > 60 ? '...' : '' }}</p>
                <div class="flex gap-3 text-xs text-gray-400 [&_span]:flex [&_span]:items-center">
                  <span>{{ image.width }}×{{ image.height }}</span>
                  <span>{{ image.modelName }}</span>
                </div>
                <!-- Folder badges -->
                <div v-if="image.folderIds && image.folderIds.length > 0" class="flex gap-1 mt-2">
                  <span
                    class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs bg-gray-700 text-amber-400"
                    :title="`In ${image.folderIds.length} folder(s)`">
                    <Folder :size="12" />
                    {{ image.folderIds.length }}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </template>
    </VirtualScroller>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/core';
import type { GalleryImage } from '@/stores/gallery';
import { Heart, Copy, Folder } from 'lucide-vue-next';
import Image from 'primevue/image';
import Checkbox from 'primevue/checkbox';
import Button from 'primevue/button';
import VirtualScroller from 'primevue/virtualscroller';

const CARD_MIN_WIDTH = 280;
const CARD_GAP = 16;
const CONTAINER_PADDING = 16;
const ROW_HEIGHT = 420; // Approximate height of each card

interface Props {
  images: GalleryImage[];
  selectedIds: Set<string>;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  select: [imageId: string];
  openDetail: [image: GalleryImage];
  toggleFavorite: [imageId: string];
  addToCompare: [image: GalleryImage];
}>();

const isDragging = ref(false);
const draggedImageIds = ref<Set<string>>(new Set());
const containerRef = ref<HTMLElement | null>(null);
const columnCount = ref(4);

// Chunk images into rows based on column count
const imageRows = computed(() => {
  const rows: GalleryImage[][] = [];
  for (let i = 0; i < props.images.length; i += columnCount.value) {
    rows.push(props.images.slice(i, i + columnCount.value));
  }
  return rows;
});

const getImageSrc = (image: GalleryImage) => {
  const path = image.filePath || image.thumbnailPath || '';
  return convertFileSrc(path);
};

// Get thumbnail source for gallery display (falls back to original if no thumbnail)
const getThumbnailSrc = (image: GalleryImage) => {
  const path = image.thumbnailPath || image.filePath;
  return convertFileSrc(path);
};

// Calculate column count based on container width
const updateColumnCount = () => {
  if (!containerRef.value) return;
  const containerWidth = containerRef.value.clientWidth - CONTAINER_PADDING * 2;
  const cols = Math.max(1, Math.floor((containerWidth + CARD_GAP) / (CARD_MIN_WIDTH + CARD_GAP)));
  columnCount.value = cols;
};

let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  updateColumnCount();
  resizeObserver = new ResizeObserver(updateColumnCount);
  if (containerRef.value) {
    resizeObserver.observe(containerRef.value);
  }
});

onUnmounted(() => {
  resizeObserver?.disconnect();
});

const handleDragStart = (event: DragEvent, image: GalleryImage) => {
  if (!event.dataTransfer) return;

  // If dragging a selected image, drag all selected images
  // Otherwise, just drag this single image
  let imageIds: string[];
  if (props.selectedIds.has(image.id)) {
    imageIds = Array.from(props.selectedIds);
  } else {
    imageIds = [image.id];
  }

  draggedImageIds.value = new Set(imageIds);
  isDragging.value = true;

  // Set drag data
  event.dataTransfer.setData('application/x-gallery-images', JSON.stringify(imageIds));
  event.dataTransfer.effectAllowed = 'copy';

  // Create custom drag image showing count using safe DOM methods
  if (imageIds.length > 1) {
    const dragImage = document.createElement('div');
    dragImage.className = 'custom-drag-image';

    // Create icon element
    const icon = document.createElement('i');
    icon.className = 'pi pi-images';

    // Create text element
    const text = document.createTextNode(` ${imageIds.length} images`);

    dragImage.appendChild(icon);
    dragImage.appendChild(text);

    dragImage.style.cssText = `
      position: absolute;
      top: -1000px;
      left: -1000px;
      padding: 8px 12px;
      background: #2563eb;
      color: white;
      border-radius: 6px;
      font-size: 14px;
      font-weight: 500;
      display: flex;
      align-items: center;
      gap: 6px;
      box-shadow: 0 4px 6px rgba(0,0,0,0.3);
    `;
    document.body.appendChild(dragImage);
    event.dataTransfer.setDragImage(dragImage, 40, 20);

    // Clean up after a short delay
    setTimeout(() => {
      document.body.removeChild(dragImage);
    }, 0);
  }
};

const handleDragEnd = () => {
  isDragging.value = false;
  draggedImageIds.value = new Set();
};
</script>

<style scoped>
@reference "tailwindcss";

.virtual-grid-container {
  height: 100%;
  width: 100%;
  overflow: hidden;
}

.virtual-scroller {
  height: 100%;
  width: 100%;
}

.virtual-scroller :deep(.p-virtualscroller-content) {
  padding: 16px;
}

.image-row {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
  padding-bottom: 16px;
}

.image-card {
  @apply bg-gray-800;
  position: relative;
  border: 2px solid transparent;
  border-radius: 8px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  transition: all 0.2s;
  cursor: grab;
  overflow: hidden;
}

.image-card:hover {
  @apply shadow-2xl border border-blue-400;
}

.image-card:active {
  cursor: grabbing;
}

.image-container {
  position: relative;
  cursor: pointer;
  overflow: hidden;
  background-color: #334155;
  aspect-ratio: 1;
  min-height: 200px;
}

/* Fix PrimeVue Image component wrapper sizing */
.image-container :deep(.p-image) {
  display: block;
  width: 100%;
  height: 100%;
}

.image-container :deep(.p-image-preview-container) {
  display: block;
  width: 100%;
  height: 100%;
}

.image-container :deep(img) {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
</style>
