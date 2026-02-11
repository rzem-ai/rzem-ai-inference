<template>
  <div class="h-full p-2">
    <div class="h-full w-72 bg-white rounded-xl shadow-sm flex flex-col overflow-hidden border border-slate-200">
      <div class="flex flex-col gap-3 p-3 overflow-y-auto">
        <!-- Search bar -->
        <div class="flex items-center gap-2 px-3 h-8 rounded-lg bg-slate-50 border border-slate-200">
          <Search :size="14" class="text-slate-400 shrink-0" />
          <input
            v-model="searchInput"
            type="text"
            placeholder="Search images..."
            class="flex-1 bg-transparent text-sm text-slate-700 placeholder-slate-400 outline-none" />
        </div>

        <!-- Folders section -->
        <div class="flex flex-col gap-1">
          <div class="flex items-center justify-between px-2">
            <span class="text-xs font-semibold text-slate-900">Folders</span>
            <button class="text-slate-400 hover:text-slate-600 transition-colors" @click="onCreateFolder">
              <Plus :size="14" />
            </button>
          </div>

          <!-- All Images (special item) -->
          <button
            class="flex items-center gap-2 px-3 h-8 rounded-lg text-left w-full transition-colors"
            :class="isAllActive ? 'bg-blue-50 text-blue-600' : 'hover:bg-slate-50 text-slate-700'"
            @click="onFilterAll">
            <Images :size="14" />
            <span class="flex-1 text-sm font-medium truncate">All Images</span>
            <span
              class="text-xs font-medium"
              :class="isAllActive ? 'text-blue-500' : 'text-slate-400'">
              {{ gallery.total }}
            </span>
          </button>

          <!-- Favorites (special item) -->
          <button
            class="flex items-center gap-2 px-3 h-8 rounded-lg text-left w-full transition-colors"
            :class="gallery.favoritesOnly ? 'bg-blue-50 text-blue-600' : 'hover:bg-slate-50 text-slate-700'"
            @click="onFilterFavorites">
            <Star :size="14" :class="gallery.favoritesOnly ? 'text-blue-500' : 'text-slate-400'" />
            <span class="flex-1 text-sm font-medium truncate">Favorites</span>
          </button>

          <!-- Dynamic folders -->
          <button
            v-for="folder in gallery.folders"
            :key="folder.id"
            class="flex items-center gap-2 px-3 h-8 rounded-lg text-left w-full transition-colors"
            :class="gallery.currentFolderId === folder.id
              ? 'bg-blue-50 text-blue-600'
              : 'hover:bg-slate-50 text-slate-700'"
            @click="gallery.filterByFolder(folder.id)">
            <FolderIcon :size="14" class="text-slate-400" />
            <span class="flex-1 text-sm font-medium truncate">{{ folder.name }}</span>
          </button>
        </div>

        <!-- Tags section -->
        <div class="flex flex-col gap-2">
          <div class="flex items-center justify-between px-2">
            <span class="text-xs font-semibold text-slate-900">Tags</span>
            <button class="text-slate-400 hover:text-slate-600 transition-colors" @click="onCreateTag">
              <Plus :size="14" />
            </button>
          </div>
          <div class="flex flex-wrap gap-1.5 px-2">
            <button
              v-for="tag in gallery.tags"
              :key="tag.id"
              class="px-2.5 py-1 rounded-full text-xs font-medium transition-all"
              :class="gallery.currentTagId === tag.id ? 'ring-2 ring-offset-1' : ''"
              :style="tagStyle(tag)"
              @click="onToggleTag(tag.id)">
              {{ tag.name }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { Search, Plus, Images, Star, Folder as FolderIcon } from 'lucide-vue-next';
import { usePywebview } from '@/composables/usePywebview';
import { useGalleryStore } from '@/stores/gallery';
import type { Tag } from '@/types/inference';

const { api, isReady } = usePywebview();
const gallery = useGalleryStore();

const searchInput = ref('');

const isAllActive = computed(
  () => !gallery.currentFolderId && !gallery.favoritesOnly && !gallery.currentTagId,
);

// Debounced search
let searchTimeout: ReturnType<typeof setTimeout>;
watch(searchInput, (value) => {
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    gallery.searchImages(value);
  }, 300);
});

function tagStyle(tag: Tag) {
  const color = tag.color || '#64748b';
  return {
    color,
    backgroundColor: color + '15',
  };
}

function onFilterAll() {
  gallery.favoritesOnly = false;
  gallery.currentTagId = null;
  gallery.filterByFolder(null);
}

function onFilterFavorites() {
  gallery.currentFolderId = null;
  gallery.currentTagId = null;
  gallery.toggleFavoritesFilter();
}

function onToggleTag(tagId: number) {
  gallery.favoritesOnly = false;
  gallery.currentFolderId = null;
  gallery.filterByTag(gallery.currentTagId === tagId ? null : tagId);
}

function onCreateFolder() {
  const name = prompt('Folder name:');
  if (name?.trim()) {
    const id = crypto.randomUUID();
    gallery.createFolder(id, name.trim());
  }
}

function onCreateTag() {
  const name = prompt('Tag name:');
  if (name?.trim()) {
    gallery.createTag(name.trim());
  }
}

// Initialize gallery store once API is ready
onMounted(() => {
  const check = setInterval(async () => {
    if (isReady.value) {
      clearInterval(check);
      gallery.setApi(api.value);
      await Promise.all([
        gallery.loadImages(),
        gallery.loadFolders(),
        gallery.loadTags(),
      ]);
    }
  }, 50);
});
</script>
