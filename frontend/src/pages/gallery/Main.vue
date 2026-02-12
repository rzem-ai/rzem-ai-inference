<template>
  <div class="h-full flex flex-col p-2 pl-0 gap-2">
    <!-- Toolbar -->
    <Toolbar>
      <template #start>
        <Button severity="primary" size="small" @click="onCreateFolder"><Plus :size="14" />New Folder</Button>
        <Divider layout="vertical" />
        <Button
          @click="toggleSelectionMode"
          :severity="selectionMode ? 'primary' : 'secondary'"
          :variant="selectionMode ? '' : 'outlined'"
          :text="!selectionMode"
          size="small">
          <ListTodo :size="14" />
        </Button>
        <Button
          :disabled="!selectedIds.size"
          @click="onDeleteSelected"
          :severity="selectionMode ? 'danger' : 'secondary'"
          :variant="selectionMode ? 'outlined' : ''"
          :text="!selectionMode"
          size="small">
          <Trash2 :size="14" />
        </Button>
      </template>

      <template #center class="w-[50%]">
        <!-- Search bar -->
        <InputGroup>
          <InputGroupAddon> <Search :size="14" class="text-slate-400" /> </InputGroupAddon>
          <InputText v-model="searchInput" placeholder="Search images..." size="small" fluid />
        </InputGroup>
      </template>

      <template #end>
        <!-- View toggle -->
        <Button
          size="small"
          text
          class="p-1.5 rounded-md transition-colors"
          :class="viewMode === 'grid' ? 'bg-blue-50 text-blue-600' : 'text-slate-500 hover:bg-slate-50'"
          @click="viewMode = 'grid'">
          <LayoutGrid :size="14" />
        </Button>
        <Button
          size="small"
          text
          class="p-1.5 rounded-md transition-colors"
          :class="viewMode === 'list' ? 'bg-blue-50 text-blue-600' : 'text-slate-500 hover:bg-slate-50'"
          @click="viewMode = 'list'">
          <ListIcon :size="14" />
        </Button>
      </template>
    </Toolbar>

    <!-- Toolbar -->
    <div class="flex items-center gap-1.5 px-3 h-14 bg-white rounded-xl border border-slate-200 shrink-0">
      <!-- Select toggle -->
      <ToggleButton v-model="selectionMode" onLabel="Select" offLabel="Select" size="small">
        <template #icon><SquareCheck v-if="selectionMode" :size="14" /> <Square v-else="selectionMode" :size="14" /></template>
      </ToggleButton>

      <div class="w-px h-4 bg-slate-200" />

      <!-- Batch actions -->
      <button
        class="p-1.5 rounded-md text-slate-500 hover:bg-slate-50 hover:text-slate-700 disabled:opacity-30 disabled:pointer-events-none transition-colors"
        :disabled="!selectedIds.size"
        title="Download">
        <Download :size="14" />
      </button>
      <button
        class="p-1.5 rounded-md text-slate-500 hover:bg-slate-50 hover:text-slate-700 disabled:opacity-30 disabled:pointer-events-none transition-colors"
        :disabled="!selectedIds.size"
        title="Move to folder">
        <FolderInput :size="14" />
      </button>
      <button
        class="p-1.5 rounded-md text-slate-500 hover:bg-slate-50 hover:text-slate-700 disabled:opacity-30 disabled:pointer-events-none transition-colors"
        :disabled="!selectedIds.size"
        title="Tag">
        <TagIcon :size="14" />
      </button>
      <button
        class="p-1.5 rounded-md text-slate-500 hover:bg-slate-50 hover:text-slate-700 disabled:opacity-30 disabled:pointer-events-none transition-colors"
        :disabled="!selectedIds.size"
        title="Delete"
        @click="onDeleteSelected">
        <Trash2 :size="14" />
      </button>

      <!-- Spacer -->
      <div class="flex-1" />

      <div class="w-px h-4 bg-slate-200" />

      <!-- Sort -->
      <button class="flex items-center gap-1.5 px-2.5 h-7 rounded-md bg-slate-50 text-xs font-medium text-slate-600 hover:bg-slate-100 transition-colors">
        <ArrowUpDown :size="14" />
        <span>Date</span>
        <ChevronDown :size="12" class="text-slate-400" />
      </button>

      <div class="w-px h-4 bg-slate-200" />

      <!-- View toggle -->
      <button
        class="p-1.5 rounded-md transition-colors"
        :class="viewMode === 'grid' ? 'bg-blue-50 text-blue-600' : 'text-slate-500 hover:bg-slate-50'"
        @click="viewMode = 'grid'">
        <LayoutGrid :size="14" />
      </button>
      <button
        class="p-1.5 rounded-md transition-colors"
        :class="viewMode === 'list' ? 'bg-blue-50 text-blue-600' : 'text-slate-500 hover:bg-slate-50'"
        @click="viewMode = 'list'">
        <ListIcon :size="14" />
      </button>
    </div>

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
      <div v-else class="grid gap-3 p-1" :class="viewMode === 'grid' ? 'grid-cols-[repeat(auto-fill,minmax(180px,1fr))]' : 'grid-cols-1'">
        <GalleryCard
          v-for="image in gallery.images"
          :key="image.id"
          :image="image"
          :selected="selectedIds.has(image.id)"
          @click="onCardClick"
          @favorite="onToggleFavorite"
          @select="onToggleSelect" />
      </div>

      <!-- Infinite scroll trigger -->
      <div ref="loadMoreRef" class="h-10" />

      <!-- Loading spinner -->
      <div v-if="gallery.loading" class="flex justify-center py-4">
        <div class="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, watch } from 'vue';
import {
  SquareCheck,
  Download,
  FolderInput,
  Tag as TagIcon,
  Trash2,
  ArrowUpDown,
  ChevronDown,
  LayoutGrid,
  List as ListIcon,
  Image as ImageIcon,
  Square,
  ListTodo,
} from 'lucide-vue-next';
import { Search, Plus, Palette } from 'lucide-vue-next';

import { useGalleryStore } from '@/stores/gallery';
import GalleryCard from './GalleryCard.vue';
import { Button, Divider, ToggleButton, Toolbar } from 'primevue';
import { InputText, InputGroup, InputGroupAddon } from 'primevue';

const gallery = useGalleryStore();

const viewMode = ref<'grid' | 'list'>('grid');
const selectionMode = ref(false);
const selectedIds = reactive(new Set<string>());

const searchInput = ref('');

const scrollRef = ref<HTMLElement>();
const loadMoreRef = ref<HTMLElement>();
let loadMoreObserver: IntersectionObserver | null = null;

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

function onCardClick(imageId: string) {
  if (selectionMode.value) {
    onToggleSelect(imageId);
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

// Infinite scroll: load more when the sentinel enters viewport
onMounted(() => {
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
