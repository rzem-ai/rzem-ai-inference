<template>
  <Dialog
    v-model:visible="visibleModel"
    :header="'Image Details'"
    modal
    :style="{ width: '900px', maxWidth: '95vw' }"
    :closable="true"
    :draggable="false"
    class="image-detail-modal">
    <div v-if="image" class="detail-content">
      <!-- Image Preview -->
      <div class="image-section">
        <Image :src="imageSrc" :alt="image.prompt" preview class="detail-image" />
      </div>

      <!-- Info Section -->
      <div class="info-section">
        <!-- Prompt -->
        <div class="info-group">
          <label class="info-label">Prompt</label>
          <p class="info-value prompt-text">{{ image.prompt }}</p>
        </div>

        <!-- Negative Prompt -->
        <div v-if="image.negativePrompt" class="info-group">
          <label class="info-label">Negative Prompt</label>
          <p class="info-value prompt-text">{{ image.negativePrompt }}</p>
        </div>

        <!-- Generation Parameters Grid -->
        <div class="params-grid">
          <div class="param-item">
            <span class="param-label">Model</span>
            <span class="param-value">{{ image.modelName }}</span>
          </div>
          <div class="param-item">
            <span class="param-label">Size</span>
            <span class="param-value">{{ image.width }} × {{ image.height }}</span>
          </div>
          <div v-if="image.steps" class="param-item">
            <span class="param-label">Steps</span>
            <span class="param-value">{{ image.steps }}</span>
          </div>
          <div v-if="image.cfgScale" class="param-item">
            <span class="param-label">CFG Scale</span>
            <span class="param-value">{{ image.cfgScale }}</span>
          </div>
          <div v-if="image.seed" class="param-item">
            <span class="param-label">Seed</span>
            <span class="param-value">{{ image.seed }}</span>
          </div>
          <div v-if="image.sampler" class="param-item">
            <span class="param-label">Sampler</span>
            <span class="param-value">{{ image.sampler }}</span>
          </div>
          <div class="param-item">
            <span class="param-label">Created</span>
            <span class="param-value">{{ formatDate(image.createdAt) }}</span>
          </div>
          <div class="param-item">
            <span class="param-label">File Size</span>
            <span class="param-value">{{ formatFileSize(image.fileSize) }}</span>
          </div>
        </div>

        <!-- Tags Section -->
        <div class="info-group">
          <label class="info-label">Tags</label>
          <div class="tags-container">
            <Chip
              v-for="tag in image.tags"
              :key="tag"
              :label="tag"
              removable
              @remove="removeTag(tag)"
              class="tag-chip" />
            <div class="add-tag-input">
              <AutoComplete
                v-model="newTag"
                :suggestions="tagSuggestions"
                @complete="searchTags"
                @keyup.enter="addTag"
                placeholder="Add tag..."
                size="small" />
              <Button icon="pi pi-plus" size="small" text @click="addTag" :disabled="!newTag" />
            </div>
          </div>
        </div>

        <!-- Folders Section -->
        <div class="info-group">
          <label class="info-label">Folders</label>
          <div class="folders-container">
            <Chip
              v-for="folderId in image.folderIds"
              :key="folderId"
              :label="getFolderName(folderId)"
              removable
              @remove="removeFromFolder(folderId)"
              class="folder-chip" />
            <Select
              v-model="selectedFolder"
              :options="availableFolders"
              optionLabel="label"
              optionValue="value"
              placeholder="Add to folder..."
              size="small"
              class="folder-select"
              @change="addToFolder" />
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="footer-actions">
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
import { ref, computed, watch } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { useGalleryStore, type GalleryImage } from '@/stores/gallery'
import { useFoldersStore } from '@/stores/folders'
import { useTagsStore } from '@/stores/tags'
import Dialog from 'primevue/dialog'
import Image from 'primevue/image'
import Chip from 'primevue/chip'
import Button from 'primevue/button'
import AutoComplete from 'primevue/autocomplete'
import Select from 'primevue/select'

const props = defineProps<{
  visible: boolean
  image: GalleryImage | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'update:image', value: GalleryImage | null): void
}>()

const galleryStore = useGalleryStore()
const foldersStore = useFoldersStore()
const tagsStore = useTagsStore()

const visibleModel = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value),
})

const newTag = ref('')
const tagSuggestions = ref<string[]>([])
const selectedFolder = ref<string | null>(null)

const imageSrc = computed(() => {
  if (!props.image) return ''
  return convertFileSrc(props.image.filePath)
})

const availableFolders = computed(() => {
  if (!props.image) return []
  return foldersStore.flatFolders
    .filter((f) => !props.image!.folderIds.includes(f.id))
    .map((f) => ({
      label: f.path.length > 0 ? `${f.path.join(' / ')} / ${f.name}` : f.name,
      value: f.id,
    }))
})

const searchTags = (event: { query: string }) => {
  const query = event.query.toLowerCase()
  tagSuggestions.value = tagsStore.tags
    .filter((t) => t.name.toLowerCase().includes(query) && !props.image?.tags.includes(t.name))
    .map((t) => t.name)
    .slice(0, 10)
}

const addTag = async () => {
  if (!newTag.value || !props.image) return
  await tagsStore.bulkAddTag([props.image.id], newTag.value)
  // Update local image
  if (!props.image.tags.includes(newTag.value)) {
    emit('update:image', { ...props.image, tags: [...props.image.tags, newTag.value] })
  }
  newTag.value = ''
}

const removeTag = async (tag: string) => {
  if (!props.image) return
  await tagsStore.bulkRemoveTag([props.image.id], tag)
  emit('update:image', { ...props.image, tags: props.image.tags.filter((t) => t !== tag) })
}

const getFolderName = (folderId: string): string => {
  const folder = foldersStore.flatFolders.find((f) => f.id === folderId)
  return folder?.name || 'Unknown'
}

const addToFolder = async () => {
  if (!selectedFolder.value || !props.image) return
  await galleryStore.addToFolder([props.image.id], selectedFolder.value)
  emit('update:image', { ...props.image, folderIds: [...props.image.folderIds, selectedFolder.value] })
  selectedFolder.value = null
}

const removeFromFolder = async (folderId: string) => {
  if (!props.image) return
  await galleryStore.removeFromFolder([props.image.id], folderId)
  emit('update:image', { ...props.image, folderIds: props.image.folderIds.filter((id) => id !== folderId) })
}

const toggleFavorite = async () => {
  if (!props.image) return
  await galleryStore.toggleFavorite(props.image.id)
  emit('update:image', { ...props.image, isFavorite: !props.image.isFavorite })
}

const formatDate = (timestamp: number): string => {
  return new Date(timestamp * 1000).toLocaleString()
}

const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

// Reset state when dialog opens
watch(
  () => props.visible,
  (isVisible) => {
    if (isVisible) {
      newTag.value = ''
      selectedFolder.value = null
    }
  }
)
</script>

<style scoped>
@reference "tailwindcss";

.detail-content {
  @apply flex gap-6;
}

.image-section {
  @apply flex-shrink-0;
  width: 400px;
}

.detail-image {
  @apply w-full rounded-lg overflow-hidden;
}

.detail-image :deep(img) {
  @apply w-full h-auto object-contain;
  max-height: 500px;
}

.info-section {
  @apply flex-1 flex flex-col gap-4 overflow-y-auto;
  max-height: 500px;
}

.info-group {
  @apply flex flex-col gap-2;
}

.info-label {
  @apply text-xs font-semibold uppercase tracking-wide;
  color: var(--color-gray-400);
}

.info-value {
  @apply text-sm;
  color: var(--color-gray-200);
}

.prompt-text {
  @apply leading-relaxed;
  word-break: break-word;
}

.params-grid {
  @apply grid grid-cols-2 gap-3;
}

.param-item {
  @apply flex flex-col gap-0.5;
}

.param-label {
  @apply text-xs;
  color: var(--color-gray-500);
}

.param-value {
  @apply text-sm font-medium;
  color: var(--color-gray-200);
}

.tags-container,
.folders-container {
  @apply flex flex-wrap gap-2 items-center;
}

.tag-chip,
.folder-chip {
  @apply text-xs;
}

.add-tag-input {
  @apply flex items-center gap-1;
}

.add-tag-input :deep(.p-autocomplete-input) {
  @apply w-32;
}

.folder-select {
  @apply w-48;
}

.footer-actions {
  @apply flex justify-between w-full;
}

@media (max-width: 768px) {
  .detail-content {
    @apply flex-col;
  }

  .image-section {
    @apply w-full;
  }

  .params-grid {
    @apply grid-cols-2;
  }
}
</style>
