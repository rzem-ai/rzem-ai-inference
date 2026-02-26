<template>
  <MenuPanel title="Gallery" icon="Images">
    <template #title-button>
      <Button class="transition-colors" severity="secondary" title="" disabled>
        <Plus :size="16" />
      </Button>
    </template>
    <template #content>
      <!-- Folders section -->
      <div class="flex flex-col gap-1">
        <div class="flex items-center justify-between px-2 h-8">
          <div class="font-medium text-slate-900 text-lg">Folders</div>
        </div>

        <!-- All Images (special item) -->
        <button
          class="flex items-center gap-2 px-3 h-8 rounded-lg text-left w-full transition-colors"
          :class="isAllActive ? 'bg-blue-50 text-blue-600' : 'hover:bg-slate-50 text-slate-700'"
          @click="onFilterAll">
          <Images :size="14" />
          <div class="flex-1 font-medium truncate">All Images</div>
          <div class="font-medium" :class="isAllActive ? 'text-blue-500' : 'text-slate-400'">
            {{ gallery.total }}
          </div>
        </button>

        <!-- Favorites (special item) -->
        <button
          class="flex items-center gap-2 px-3 h-8 rounded-lg text-left w-full transition-colors"
          :class="gallery.favoritesOnly ? 'bg-blue-50 text-blue-600' : 'hover:bg-slate-50 text-slate-700'"
          @click="onFilterFavorites">
          <Star :size="14" :class="gallery.favoritesOnly ? 'text-blue-500' : 'text-slate-400'" />
          <span class="flex-1 font-medium truncate">Favorites</span>
        </button>

        <!-- Dynamic folders -->
        <button
          v-for="folder in gallery.folders"
          :key="folder.id"
          class="flex items-center gap-2 px-3 h-8 rounded-lg text-left w-full transition-colors"
          :class="gallery.currentFolderId === folder.id ? 'bg-blue-50 text-blue-600' : 'hover:bg-slate-50 text-slate-700'"
          @click="gallery.filterByFolder(folder.id)">
          <FolderIcon :size="14" class="text-slate-400" />
          <span class="flex-1 font-medium truncate">{{ folder.name }}</span>
        </button>
      </div>

      <!-- Tags section -->
      <div class="flex flex-col gap-2">
        <div class="flex items-center justify-between px-2">
          <span class="font-medium text-slate-900">Tags</span>
          <button class="text-slate-400 hover:text-slate-600 transition-colors" @click="onCreateTag">
            <Plus :size="14" />
          </button>
        </div>
        <div class="flex flex-wrap gap-1 px-2">
          <button
            v-for="tag in gallery.tags"
            :key="tag.id"
            class="px-2 py-1 rounded-full font-medium transition-all"
            :class="gallery.currentTagId === tag.id ? 'ring-2 ring-offset-1' : ''"
            :style="tagStyle(tag)"
            @click="onToggleTag(tag.id)">
            {{ tag.name }}
          </button>
        </div>
      </div>
    </template>
  </MenuPanel>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useGalleryStore } from '@/stores/gallery';
import type { Tag } from '@/types/inference';
import MenuPanel from '@/components/MenuPanel.vue';

const gallery = useGalleryStore();

const isAllActive = computed(() => !gallery.currentFolderId && !gallery.favoritesOnly && !gallery.currentTagId);

function tagStyle(tag: Tag) {
  const color = tag.color || '#64748b';
  return {
    color,
    backgroundColor: color + '15',
  };
}

function onFilterAll() {
  gallery.setFavoritesOnly(false);
  gallery.setCurrentTagId(null);
  gallery.filterByFolder(null);
}

function onFilterFavorites() {
  gallery.setCurrentFolderId(null);
  gallery.setCurrentTagId(null);
  gallery.toggleFavoritesFilter();
}

function onToggleTag(tagId: number) {
  gallery.setFavoritesOnly(false);
  gallery.setCurrentFolderId(null);
  gallery.filterByTag(gallery.currentTagId === tagId ? null : tagId);
}

function onCreateTag() {
  const name = prompt('Tag name:');
  if (name?.trim()) {
    gallery.createTag(name.trim());
  }
}

onMounted(() => {
  gallery.init();
});
</script>
