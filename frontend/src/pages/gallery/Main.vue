<template>
  <div class="h-full flex flex-col px-2 py-4 gap-2">
    <!-- Toolbar -->
    <Toolbar class="shadow-lg">
      <template #start>
        <Button severity="primary" @click="onCreateFolder"><Plus :size="14" />New Folder</Button>

        <Divider layout="vertical" />

        <Button
          :severity="selectionMode ? 'primary' : 'primary'"
          :variant="selectionMode ? 'outlined' : 'outlined'"
          :text="!selectionMode"
          @click="toggleSelectionMode">
          <ListTodo :size="18" />
        </Button>

        <!-- Batch actions -->
        <Button
          title="Download"
          class="transition-all"
          :severity="selectionMode ? 'primary' : 'secondary'"
          :variant="selectionMode && selectedIds.size > 0 ? '' : ''"
          :text="true"
          :disabled="!selectedIds.size">
          <Download :size="18" />
        </Button>
        <Button
          title="Move to folder"
          class="transition-all"
          :severity="selectionMode ? 'primary' : 'secondary'"
          :variant="selectionMode && selectedIds.size > 0 ? '' : ''"
          :text="true"
          :disabled="!selectedIds.size">
          <FolderInput :size="18" />
        </Button>
        <Button
          title="Tag"
          class="transition-all"
          :severity="selectionMode ? 'primary' : 'secondary'"
          :variant="selectionMode && selectedIds.size > 0 ? '' : ''"
          :text="true"
          :disabled="!selectedIds.size">
          <TagIcon :size="18" />
        </Button>

        <Button
          title="Delete"
          class="transition-all"
          :severity="selectionMode ? 'danger' : 'secondary'"
          :variant="selectionMode && selectedIds.size > 0 ? 'outlined' : 'outlined'"
          :text="true"
          :disabled="!selectedIds.size"
          @click="onDeleteSelected">
          <Trash2 :size="18" />
        </Button>
      </template>

      <template #center class="w-[50%]">
        <!-- Search bar -->
        <InputGroup>
          <InputGroupAddon> <Search :size="14" class="text-slate-400" /> </InputGroupAddon>
          <InputText v-model="searchInput" placeholder="Search images..." fluid />
        </InputGroup>
      </template>

      <template #end>
        <div class="flex gap-2">
          <!-- Sort -->
          <Button severity="secondary">
            <ArrowUpDown :size="14" />
            <span>Date</span>
            <ChevronDown :size="12" class="text-slate-400" />
          </Button>
          <!-- View toggle -->
          <SelectButton
            v-model="viewMode"
            :options="viewModes"
            optionLabel="value"
            dataKey="value"
            severity="primary"
            variant="outlined"
            class="transition-colors">
            <template #option="slotProps">
              <component :is="slotProps.option.icon" :size="14" />
            </template>
          </SelectButton>
        </div>
      </template>
    </Toolbar>

    <!-- Image grid area -->
    <div class="flex-1 min-h-0 overflow-y-auto rounded-xl" ref="scrollRef">
      <!-- Empty state -->
      <div v-if="!gallery.loading && gallery.images.length === 0" class="h-full flex items-center justify-center">
        <div class="text-center">
          <ImageIcon :size="48" class="text-slate-300 mx-auto mb-3" />
          <p class="text-sm text-slate-400">No images yet</p>
          <p class="text-xs text-slate-300 mt-1">Generate images on the Create page to see them here</p>
        </div>
      </div>

      <!-- Grid view -->
      <div v-else class="grid gap-3 p-1" :class="isGridMode ? 'grid-cols-6' : 'grid-cols-1'">
        <GalleryCard
          v-for="image in gallery.images"
          :key="image.id"
          :image="image"
          :selected="selectedIds.has(image.id)"
          @click="onCardClick"
          @favorite="onToggleFavorite"
          @open-detail="detailVisible = true"
          @select="onToggleSelect" />
      </div>

      <!-- Infinite scroll trigger -->
      <div ref="loadMoreRef" class="h-10" />

      <!-- Loading spinner -->
      <div v-if="gallery.loading" class="flex justify-center py-4">
        <div class="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
      </div>
    </div>

    <!-- Image overlay (full-screen lightbox) -->
    <ImageOverlay
      v-if="selectedImage"
      :visible="overlayVisible"
      :image="selectedImage"
      @close="overlayVisible = false"
      @open-detail="detailVisible = true"
      @favorite="onToggleFavorite" />

    <!-- Image detail dialog -->
    <ImageDetailDialog
      v-if="selectedImage"
      v-model:visible="detailVisible"
      :image="selectedImage"
      @close-all="overlayVisible = false" />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, reactive, onMounted, onUnmounted, watch } from 'vue';
import {
  Download,
  FolderInput,
  Tag as TagIcon,
  Trash2,
  ArrowUpDown,
  ChevronDown,
  LayoutGrid,
  Search,
  Plus,
  List as ListIcon,
  Image as ImageIcon,
  ListTodo,
} from 'lucide-vue-next';

import { useGalleryStore } from '@/stores/gallery';
import type { GalleryImage } from '@/types/inference';
import GalleryCard from './GalleryCard.vue';
import ImageOverlay from './ImageOverlay.vue';
import ImageDetailDialog from './ImageDetailDialog.vue';
import { Button, Divider, Toolbar, SelectButton } from 'primevue';
import { InputText, InputGroup, InputGroupAddon } from 'primevue';

const gallery = useGalleryStore();

const GRID_VIEW = { value: 'grid', icon: LayoutGrid };
const LIST_VIEW = { value: 'list', icon: ListIcon };

const viewMode = ref(GRID_VIEW);
const viewModes = ref([GRID_VIEW, LIST_VIEW]);
const selectionMode = ref(false);
const selectedIds = reactive(new Set<string>());

const searchInput = ref('');

// Overlay & detail dialog state
const selectedImage = ref<GalleryImage | null>(null);
const overlayVisible = ref(false);
const detailVisible = ref(false);

const scrollRef = ref<HTMLElement>();
const loadMoreRef = ref<HTMLElement>();
let loadMoreObserver: IntersectionObserver | null = null;

const isGridMode = computed(() => viewMode.value.value === GRID_VIEW.value);

watch(selectionMode, (newValue) => {
  if (!newValue) {
    selectedIds.clear();
  }
});

// Debounced search
let searchTimeout: ReturnType<typeof setTimeout>;
watch(searchInput, (value) => {
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    gallery.searchImages(value);
  }, 300);
});

function toggleSelectionMode() {
  selectionMode.value = !selectionMode.value;
  if (!selectionMode.value) {
    selectedIds.clear();
  }
}

watch(viewMode, (newViewMode) => {
  switch (newViewMode?.value) {
    case GRID_VIEW.value:
      viewMode.value = GRID_VIEW;
      return;
    case LIST_VIEW.value:
      viewMode.value = LIST_VIEW;
      return;
    default:
      viewMode.value = GRID_VIEW;
      return;
  }
});

function onCardClick(imageId: string) {
  if (selectionMode.value) {
    onToggleSelect(imageId);
    return;
  }
  // Open the lightbox overlay
  const image = gallery.images.find(i => i.id === imageId);
  if (image) {
    selectedImage.value = image;
    overlayVisible.value = true;
  }
}

function onToggleSelect(imageId: string) {
  if (selectedIds.has(imageId)) {
    selectedIds.delete(imageId);
  } else {
    selectedIds.add(imageId);
  }
}

function onToggleFavorite(imageId: string) {
  gallery.toggleFavorite(imageId);
}

async function onDeleteSelected() {
  if (!selectedIds.size) return;
  const count = selectedIds.size;
  if (!confirm(`Delete ${count} image${count > 1 ? 's' : ''}?`)) return;

  for (const id of selectedIds) {
    await gallery.deleteImage(id);
  }
  selectedIds.clear();
}

function onCreateFolder() {
  const name = prompt('Folder name:');
  if (name?.trim()) {
    const id = crypto.randomUUID();
    gallery.createFolder(id, name.trim());
  }
}

// Initialize gallery store (idempotent -- safe if Menu.vue already called it)
onMounted(() => {
  gallery.init();

  // Infinite scroll: load more when the sentinel enters viewport
  loadMoreObserver = new IntersectionObserver(
    ([entry]) => {
      if (entry.isIntersecting && gallery.hasMore && !gallery.loading) {
        gallery.loadMore();
      }
    },
    { root: scrollRef.value, rootMargin: '200px' },
  );
  if (loadMoreRef.value) loadMoreObserver.observe(loadMoreRef.value);
});

onUnmounted(() => loadMoreObserver?.disconnect());
</script>

<style scoped>
@reference "tailwindcss";

:deep(.p-toolbar-center) {
  width: 50%;
}

:deep(.p-toolbar) {
  padding: 0.5rem 0.75rem;
}

:deep(.p-toolbar-start) {
  @apply gap-1;
}
</style>
