<template>
  <div
    class="relative rounded-xl border-2 border-dashed transition-colors"
    :class="isDragging ? 'border-blue-400 bg-blue-950/20' : store.inputImageDataUrl ? 'border-transparent' : 'border-surface-300'"
    @dragover.prevent="isDragging = true"
    @dragleave="isDragging = false"
    @drop.prevent="onDrop">
    <!-- Loaded state -->
    <div v-if="store.inputImageDataUrl" class="relative">
      <img :src="store.inputImageDataUrl" alt="Input image" class="w-full rounded-xl object-contain max-h-48" />
      <Button
        class="absolute top-1 right-1"
        severity="danger"
        size="small"
        rounded
        text
        @click="store.clearInputImage()">
        <X :size="14" />
      </Button>
    </div>

    <!-- Empty state -->
    <div v-else class="flex flex-col items-center gap-3 py-6 px-4">
      <ImagePlus :size="32" class="text-surface-400" />
      <span class="text-sm text-surface-400">Drop image here or</span>
      <div class="flex gap-2">
        <Button size="small" severity="secondary" variant="outlined" @click="browseFile">
          <Upload :size="14" class="mr-1" />
          Browse
        </Button>
        <Button size="small" severity="secondary" variant="outlined" @click="showGalleryPicker = true">
          <Images :size="14" class="mr-1" />
          Gallery
        </Button>
      </div>
    </div>
  </div>

  <GalleryPickerDialog v-model:visible="showGalleryPicker" @pick="onGalleryPick" />
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useEditStore } from '@/stores/edit';
import { getApiAsync } from '@/bridge';
import GalleryPickerDialog from './GalleryPickerDialog.vue';

const store = useEditStore();
const isDragging = ref(false);
const showGalleryPicker = ref(false);

async function browseFile() {
  const api = await getApiAsync();
  const res = await api.browse_input_image();
  if (res.status === 'success' && res.path) {
    store.setInputImage(res.path);
  }
}

function onGalleryPick(imagePath: string) {
  store.setInputImage(imagePath);
  showGalleryPicker.value = false;
}

function onDrop(e: DragEvent) {
  isDragging.value = false;

  // Check for image path from drag data (e.g. dragged from gallery history strip)
  const imagePath = e.dataTransfer?.getData('text/image-path');
  if (imagePath) {
    store.setInputImage(imagePath);
    return;
  }

  // Check for dropped files (pywebview exposes file.path)
  const file = e.dataTransfer?.files?.[0];
  if (file && /\.(png|jpe?g|webp|bmp|tiff?)$/i.test(file.name)) {
    if ((file as any).path) {
      store.setInputImage((file as any).path);
    }
  }
}
</script>
