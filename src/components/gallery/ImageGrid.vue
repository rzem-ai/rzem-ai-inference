<template>
  <div class="h-full pl-4 overflow-y-auto">
    <CustomScrollbar class="h-full pr-2">
      <div
        class="grid grid-cols-2 gap-4 2xl:grid-cols-3 3xl:grid-cols-4 4xl:grid-cols-4 5xl:grid-cols-5 6xl:grid-cols-6 7xl:grid-cols-7 8xl:grid-cols-8 justify-items-center">
        <div
          v-for="item in images"
          :style="{ height: `320px` }"
          class="border rounded-xl image-card bg-surface-800 shaddow hover:shadow-lg hover:border-blue-400 active:cursor-grabbing;"
          :class="{
            'border-blue-500! ': selectedIds.has(item.id),
            'opacity-50 scale-95': isDragging && draggedImageIds.has(item.id),
          }"
          draggable="true"
          @dragstart="handleDragStart($event, item)"
          @dragend="handleDragEnd"
          @contextmenu.prevent="handleContextMenu($event, item)">
          <div class="absolute z-10 p-1 top-2 left-2">
            <Checkbox :model-value="selectedIds.has(item.id)" @change="emit('select', item.id)" binary />
          </div>

          <div class="absolute top-0 right-0 z-10 flex gap-1 p-0">
            <div
              class="p-2 backdrop-blur-xl"
              @click.stop="emit('toggleFavorite', item.id)"
              @mouseenter="hoveredHeartId = item.id"
              @mouseleave="hoveredHeartId = null"
              :title="item.isFavorite ? 'Remove from favorites' : 'Add to favorites'">
              <div class="p-2 rounded-full bg-white/20 backdrop-blur-xl">
                <fa :icon="[hoveredHeartId === item.id || item.isFavorite ? 'fas' : 'far', 'heart']" size="xl" class="text-red-500" style="" />
              </div>
            </div>
            <!--
            <Button severity="primary" variant="outlined" @click.stop="emit('addToCompare', item)" title="Add to compare">
              <template #icon><fa :icon="['fal', 'copy']" size="sm" /></template>
            </Button>
            -->
          </div>

          <div class="image-container">
            <Image :src="getImageSrc(item)" :preview-src="getThumbnailSrc(item)" :alt="item.prompt" preview />
            <!-- Drag overlay showing count -->
            <div
              v-if="isDragging && draggedImageIds.has(item.id) && draggedImageIds.size > 1"
              class="absolute px-2 py-1 text-xs font-bold text-white bg-blue-600 rounded-full top-2 right-2">
              {{ draggedImageIds.size }}
            </div>
          </div>

          <div class="image-details-container" @click="emit('openDetail', item)">
            <p class="m-0 mb-1 text-sm leading-5 text-white line-clamp-2">{{ item.prompt }}</p>
            <div class="flex gap-3 text-xs text-surface-600 [&_span]:flex [&_span]:items-center">
              <span>{{ item.width }}×{{ item.height }}</span>
              <span>{{ item.modelName }}</span>
              <span>{{ new Date(item.createdAt * 1000).toLocaleDateString() }}</span>
              <span v-if="item.folderIds && item.folderIds.length > 0" class="text-amber-400">
                <fa :icon="['fal', 'folder']" size="xs" class="mr-1" />{{ item.folderIds.length }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </CustomScrollbar>
    <ContextMenu ref="contextMenuRef" :model="contextMenuItems" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/core';
import type { GalleryImage } from '@/stores/gallery';
import Image from 'primevue/image';
import Checkbox from 'primevue/checkbox';

import ContextMenu from 'primevue/contextmenu';
import CustomScrollbar from '../shared/CustomScrollbar.vue';

interface FolderOption {
  id: string;
  name: string;
  path: string[];
}

interface Props {
  images: GalleryImage[];
  selectedIds: Set<string>;
  folders: FolderOption[];
  currentFolderId?: string | null;
  currentFolderName?: string | null;
}

const { images, selectedIds, currentFolderId, currentFolderName, folders } = defineProps<Props>();

const emit = defineEmits<{
  select: [imageId: string];
  openDetail: [image: GalleryImage];
  toggleFavorite: [imageId: string];
  addToCompare: [image: GalleryImage];
  delete: [imageId: string];
  addToFolder: [imageIds: string[], folderId: string];
  addToNewFolder: [imageIds: string[]];
  removeFromFolder: [imageIds: string[], folderId: string];
}>();

const contextMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null);
const contextMenuImageId = ref<string | null>(null);

const getImageIds = () => {
  if (!contextMenuImageId.value) return [];
  // If clicked image is selected, use all selected images; otherwise just this image
  return selectedIds.has(contextMenuImageId.value) ? Array.from(selectedIds) : [contextMenuImageId.value];
};

const contextMenuItems = computed(() => {
  const folderItems = folders.map((folder) => ({
    label: folder.path.length > 0 ? `${folder.path.join(' / ')} / ${folder.name}` : folder.name,
    icon: 'pi pi-folder',
    command: () => {
      const imageIds = getImageIds();
      emit('addToFolder', imageIds, folder.id);
    },
  }));

  const addToFolderSubmenu =
    folderItems.length > 0
      ? folderItems
      : [
          {
            label: 'No folders available',
            disabled: true,
          },
        ];

  const menuItems = [
    {
      label: 'Open',
      icon: 'pi pi-eye',
      command: () => {
        const image = images.find((img) => img.id === contextMenuImageId.value);
        if (image) emit('openDetail', image);
      },
    },
    {
      label: 'Add to Compare',
      icon: 'pi pi-copy',
      command: () => {
        const image = images.find((img) => img.id === contextMenuImageId.value);
        if (image) emit('addToCompare', image);
      },
    },
    {
      label: 'Toggle Favorite',
      icon: 'pi pi-heart',
      command: () => {
        if (contextMenuImageId.value) emit('toggleFavorite', contextMenuImageId.value);
      },
    },
    { separator: true },
    {
      label: 'Add to Folder',
      icon: 'pi pi-folder',
      items: addToFolderSubmenu,
    },
    {
      label: 'Add to New Folder',
      icon: 'pi pi-folder-plus',
      command: () => {
        const imageIds = getImageIds();
        emit('addToNewFolder', imageIds);
      },
    },
  ];

  // Add "Remove from Folder" option when viewing a specific folder
  if (currentFolderId && currentFolderName) {
    menuItems.push({
      label: `Remove from ${currentFolderName}`,
      icon: 'pi pi-folder-open',
      command: () => {
        const imageIds = getImageIds();
        emit('removeFromFolder', imageIds, currentFolderId!);
      },
    });
  }

  menuItems.push({ separator: true }, {
    label: 'Delete',
    icon: 'pi pi-trash',
    style: { color: 'rgb(248 113 113)' },
    command: () => {
      if (contextMenuImageId.value) emit('delete', contextMenuImageId.value);
    },
  } as any);

  return menuItems;
});

function handleContextMenu(event: MouseEvent, image: GalleryImage) {
  contextMenuImageId.value = image.id;
  contextMenuRef.value?.show(event);
}

const isDragging = ref(false);
const draggedImageIds = ref<Set<string>>(new Set());

const hoveredHeartId = ref<string | null>(null);

// Chunk images into rows based on column count

const getImageSrc = (image: GalleryImage) => {
  const path = image.filePath || image.thumbnailPath || '';
  return convertFileSrc(path);
};

// Get thumbnail source for gallery display (falls back to original if no thumbnail)
const getThumbnailSrc = (image: GalleryImage) => {
  const path = image.thumbnailPath || image.filePath;
  if (path) {
    return convertFileSrc(path);
  }

  return '';
};

onMounted(() => {});

onUnmounted(() => {});

const handleDragStart = (event: DragEvent, image: GalleryImage) => {
  if (!event.dataTransfer) return;

  // If dragging a selected image, drag all selected images
  // Otherwise, just drag this single image
  let imageIds: string[];
  if (selectedIds.has(image.id)) {
    imageIds = Array.from(selectedIds);
  } else {
    imageIds = [image.id];
  }

  draggedImageIds.value = new Set(imageIds);
  isDragging.value = true;

  // Set drag data
  event.dataTransfer.setData('application/x-gallery-images', JSON.stringify(imageIds));
  event.dataTransfer.effectAllowed = 'copy';

  // Create custom drag image (thumbnail-sized)
  const dragImage = document.createElement('div');
  dragImage.className = 'custom-drag-image';

  if (imageIds.length > 1) {
    // Multiple images: show icon + count
    const icon = document.createElement('i');
    icon.className = 'pi pi-images';
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
  } else {
    // Single image: show small thumbnail
    const img = document.createElement('img');
    img.src = getThumbnailSrc(image);
    img.style.cssText = `
      width: 100px;
      height: 100px;
      object-fit: cover;
      border-radius: 8px;
      box-shadow: 0 4px 6px rgba(0,0,0,0.3);
    `;
    dragImage.appendChild(img);

    dragImage.style.cssText = `
      position: absolute;
      top: -1000px;
      left: -1000px;
      width: 100px;
      height: 100px;
    `;
  }

  document.body.appendChild(dragImage);
  event.dataTransfer.setDragImage(dragImage, 50, 50);

  // Clean up after a short delay
  setTimeout(() => {
    document.body.removeChild(dragImage);
  }, 0);
};

const handleDragEnd = () => {
  isDragging.value = false;
  draggedImageIds.value = new Set();
};
</script>

<style scoped>
@reference "tailwindcss";

.image-card {
  position: relative;
  transition: all 0.2s;
  cursor: grab;
  overflow: hidden;
  aspect-ratio: 1;
}

.image-container {
  position: relative;
  cursor: pointer;
  overflow: hidden;
  width: 100%;
  height: 100%;
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

.image-details-container {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 2.5rem 0.75rem 0.75rem;
  background: linear-gradient(to top, rgba(0, 0, 0, 0.85) 0%, rgba(0, 0, 0, 0.6) 60%, transparent 100%);
  cursor: pointer;
  z-index: 5;
}
</style>
