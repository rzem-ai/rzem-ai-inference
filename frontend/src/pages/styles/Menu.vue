<template>
  <MenuPanel v-if="showSidebar" title="Styles" :icon="Palette">
    <template #title-button>
      <Button class="transition-colors" severity="primary" title="AI Prompt Assistant" @click="router.push({ name: 'styles-new' })">
        <Plus :size="16" />
      </Button>
    </template>
    <template #content>
      <!-- Collections section -->
      <div class="flex flex-col gap-1">
        <div class="flex items-center justify-between px-2 h-8">
          <div class="font-medium text-slate-900">Collections</div>
        </div>

        <!-- Favorites -->
        <button
          class="flex items-center gap-2 px-3 h-8 rounded-lg text-left w-full transition-colors"
          :class="stylesStore.favoritesOnly ? 'bg-blue-50 text-blue-600' : 'hover:bg-slate-50 text-slate-700'"
          @click="onFilterFavorites">
          <Star :size="14" :class="stylesStore.favoritesOnly ? 'text-blue-500' : 'text-slate-400'" />
          <span class="flex-1 font-medium truncate">Favorites</span>
        </button>

        <!-- Dynamic categories -->
        <button
          v-for="cat in stylesStore.categories"
          :key="cat"
          class="flex items-center gap-2 px-3 h-8 rounded-lg text-left w-full transition-colors"
          :class="stylesStore.currentCategory === cat ? 'bg-blue-50 text-blue-600' : 'hover:bg-slate-50 text-slate-700'"
          @click="stylesStore.filterByCategory(cat)">
          <FolderIcon :size="14" class="text-slate-400" />
          <span class="flex-1 font-medium truncate">{{ cat }}</span>
        </button>
      </div>

      <!-- Tags section -->
      <div class="flex flex-col gap-2">
        <div class="px-2">
          <span class="font-semibold text-slate-900">Tags</span>
        </div>
        <div class="flex flex-wrap gap-1.5 px-2">
          <button
            v-for="tag in stylesStore.tags"
            :key="tag.id"
            class="px-2.5 py-1 rounded-full font-medium transition-all"
            :class="stylesStore.currentTagId === tag.id ? 'ring-2 ring-offset-1' : ''"
            :style="tagStyle(tag)"
            @click="onToggleTag(tag.id)">
            {{ tag.name }}
          </button>
          <span v-if="!stylesStore.tags.length" class="text-slate-400 px-1">No style tags</span>
        </div>
      </div>
    </template>
  </MenuPanel>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { Folder as FolderIcon, Palette, Plus, Star } from 'lucide-vue-next';
import { useStylesStore } from '@/stores/styles';
import type { Tag } from '@/types/inference';
import { InputText, InputGroup, Button } from 'primevue';
import MenuPanel from '@/components/MenuPanel.vue';

const route = useRoute();
const router = useRouter();
const stylesStore = useStylesStore();

const searchInput = ref('');

const showSidebar = computed(() => route.name === 'styles');

const isAllActive = computed(() => !stylesStore.currentCategory && !stylesStore.favoritesOnly && !stylesStore.currentTagId);

let searchTimeout: ReturnType<typeof setTimeout>;
watch(searchInput, (value) => {
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    stylesStore.searchStyles(value);
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
  stylesStore.favoritesOnly = false;
  stylesStore.currentTagId = null;
  stylesStore.filterByCategory(null);
}

function onFilterFavorites() {
  stylesStore.currentCategory = null;
  stylesStore.currentTagId = null;
  stylesStore.toggleFavoritesFilter();
}

function onToggleTag(tagId: number) {
  stylesStore.favoritesOnly = false;
  stylesStore.currentCategory = null;
  stylesStore.filterByTag(stylesStore.currentTagId === tagId ? null : tagId);
}
</script>
