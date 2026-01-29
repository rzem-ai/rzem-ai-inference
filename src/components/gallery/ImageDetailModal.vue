<template>
  <Dialog
    v-model:visible="visibleModel"
    :header="'Image Details'"
    modal
    :style="{ width: '900px', maxWidth: '95vw' }"
    :closable="true"
    :draggable="false"
    class="image-detail-modal">
    <div v-if="image" class="flex gap-6 max-md:flex-col">
      <!-- Image Preview -->
      <div class="shrink-0 w-100 max-md:w-full">
        <Image :src="imageSrc" :alt="image.prompt" preview class="[&_img]:w-full [&_img]:h-auto [&_img]:object-contain [&_img]:max-h-125 w-full rounded-lg overflow-hidden" />
      </div>

      <!-- Info Section -->
      <div class="flex-1 flex flex-col gap-4 overflow-y-auto max-h-125">
        <!-- Prompt -->
        <div class="flex flex-col gap-2">
          <label class="text-xs font-semibold uppercase tracking-wide text-gray-400">Prompt</label>
          <p class="text-sm text-gray-200 leading-relaxed wrap-break-word">{{ image.prompt }}</p>
        </div>

        <!-- Negative Prompt -->
        <div v-if="image.negativePrompt" class="flex flex-col gap-2">
          <label class="text-xs font-semibold uppercase tracking-wide text-gray-400">Negative Prompt</label>
          <p class="text-sm text-gray-200 leading-relaxed wrap-break-word">{{ image.negativePrompt }}</p>
        </div>

        <!-- Generation Parameters Grid -->
        <div class="grid grid-cols-2 gap-3">
          <div class="flex flex-col gap-0.5">
            <span class="text-xs text-gray-500">Model</span>
            <span class="text-sm font-medium text-gray-200">{{ image.modelName }}</span>
          </div>
          <div class="flex flex-col gap-0.5">
            <span class="text-xs text-gray-500">Size</span>
            <span class="text-sm font-medium text-gray-200">{{ image.width }} × {{ image.height }}</span>
          </div>
          <div v-if="image.steps" class="flex flex-col gap-0.5">
            <span class="text-xs text-gray-500">Steps</span>
            <span class="text-sm font-medium text-gray-200">{{ image.steps }}</span>
          </div>
          <div v-if="image.cfgScale" class="flex flex-col gap-0.5">
            <span class="text-xs text-gray-500">CFG Scale</span>
            <span class="text-sm font-medium text-gray-200">{{ image.cfgScale }}</span>
          </div>
          <div v-if="image.seed" class="flex flex-col gap-0.5">
            <span class="text-xs text-gray-500">Seed</span>
            <span class="text-sm font-medium text-gray-200">{{ image.seed }}</span>
          </div>
          <div v-if="image.sampler" class="flex flex-col gap-0.5">
            <span class="text-xs text-gray-500">Sampler</span>
            <span class="text-sm font-medium text-gray-200">{{ image.sampler }}</span>
          </div>
          <div class="flex flex-col gap-0.5">
            <span class="text-xs text-gray-500">Created</span>
            <span class="text-sm font-medium text-gray-200">{{ formatDate(image.createdAt) }}</span>
          </div>
          <div class="flex flex-col gap-0.5">
            <span class="text-xs text-gray-500">File Size</span>
            <span class="text-sm font-medium text-gray-200">{{ formatFileSize(image.fileSize) }}</span>
          </div>
        </div>

        <!-- Tags Section -->
        <div class="flex flex-col gap-2">
          <label class="text-xs font-semibold uppercase tracking-wide text-gray-400">Tags</label>
          <div class="flex flex-wrap gap-2 items-center">
            <Chip v-for="tag in image.tags" :key="tag" :label="tag" removable @remove="removeTag(tag)" class="text-xs" />
            <div class="flex items-center gap-1">
              <AutoComplete v-model="newTag" :suggestions="tagSuggestions" @complete="searchTags" @keyup.enter="addTag" placeholder="Add tag..." size="small" class="[&_.p-autocomplete-input]:w-32" />
              <Button icon="pi pi-plus" size="small" text @click="addTag" :disabled="!newTag" />
            </div>
          </div>
        </div>

        <!-- Folders Section -->
        <div class="flex flex-col gap-2">
          <label class="text-xs font-semibold uppercase tracking-wide text-gray-400">Folders</label>
          <div class="flex flex-wrap gap-2 items-center">
            <Chip
              v-for="folderId in image.folderIds"
              :key="folderId"
              :label="getFolderName(folderId)"
              removable
              @remove="removeFromFolder(folderId)"
              class="text-xs" />
            <Select
              v-model="selectedFolder"
              :options="availableFolders"
              optionLabel="label"
              optionValue="value"
              placeholder="Add to folder..."
              size="small"
              class="w-48"
              @change="addToFolder" />
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-between w-full">
        <Button
          :severity="image?.isFavorite ? 'danger' : 'secondary'"
          @click="toggleFavorite"
          :label="image?.isFavorite ? 'Remove from Favorites' : 'Add to Favorites'"
          :icon="image?.isFavorite ? 'pi pi-heart-fill' : 'pi pi-heart'" />
        <Button label="Close" severity="secondary" @click="visibleModel = false" />
      </div>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/core';
import { useGalleryStore, type GalleryImage } from '@/stores/gallery';
import { useFoldersStore } from '@/stores/folders';
import { useTagsStore } from '@/stores/tags';
import Dialog from 'primevue/dialog';
import Image from 'primevue/image';
import Chip from 'primevue/chip';
import Button from 'primevue/button';
import AutoComplete from 'primevue/autocomplete';
import Select from 'primevue/select';

const props = defineProps<{
  visible: boolean;
  image: GalleryImage | null;
}>();

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'update:image', value: GalleryImage | null): void;
}>();

const galleryStore = useGalleryStore();
const foldersStore = useFoldersStore();
const tagsStore = useTagsStore();

const visibleModel = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value),
});

const newTag = ref('');
const tagSuggestions = ref<string[]>([]);
const selectedFolder = ref<string | null>(null);

const imageSrc = computed(() => {
  if (!props.image) return '';
  return convertFileSrc(props.image.filePath);
});

const availableFolders = computed(() => {
  if (!props.image) return [];
  return foldersStore.flatFolders
    .filter((f) => !props.image!.folderIds.includes(f.id))
    .map((f) => ({
      label: f.path.length > 0 ? `${f.path.join(' / ')} / ${f.name}` : f.name,
      value: f.id,
    }));
});

const searchTags = (event: { query: string }) => {
  const query = event.query.toLowerCase();
  tagSuggestions.value = tagsStore.tags
    .filter((t) => t.name.toLowerCase().includes(query) && !props.image?.tags.includes(t.name))
    .map((t) => t.name)
    .slice(0, 10);
};

const addTag = async () => {
  if (!newTag.value || !props.image) return;
  await tagsStore.bulkAddTag([props.image.id], newTag.value);
  // Update local image
  if (!props.image.tags.includes(newTag.value)) {
    emit('update:image', { ...props.image, tags: [...props.image.tags, newTag.value] });
  }
  newTag.value = '';
};

const removeTag = async (tag: string) => {
  if (!props.image) return;
  await tagsStore.bulkRemoveTag([props.image.id], tag);
  emit('update:image', { ...props.image, tags: props.image.tags.filter((t) => t !== tag) });
};

const getFolderName = (folderId: string): string => {
  const folder = foldersStore.flatFolders.find((f) => f.id === folderId);
  return folder?.name || 'Unknown';
};

const addToFolder = async () => {
  if (!selectedFolder.value || !props.image) return;
  await galleryStore.addToFolder([props.image.id], selectedFolder.value);
  emit('update:image', { ...props.image, folderIds: [...props.image.folderIds, selectedFolder.value] });
  selectedFolder.value = null;
};

const removeFromFolder = async (folderId: string) => {
  if (!props.image) return;
  await galleryStore.removeFromFolder([props.image.id], folderId);
  emit('update:image', { ...props.image, folderIds: props.image.folderIds.filter((id) => id !== folderId) });
};

const toggleFavorite = async () => {
  if (!props.image) return;
  await galleryStore.toggleFavorite(props.image.id);
  emit('update:image', { ...props.image, isFavorite: !props.image.isFavorite });
};

const formatDate = (timestamp: number): string => {
  return new Date(timestamp * 1000).toLocaleString();
};

const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
};

// Reset state when dialog opens
watch(
  () => props.visible,
  (isVisible) => {
    if (isVisible) {
      newTag.value = '';
      selectedFolder.value = null;
    }
  },
);
</script>
