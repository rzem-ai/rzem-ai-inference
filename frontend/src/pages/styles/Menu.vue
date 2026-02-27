<template>
  <MenuPanel v-if="showSidebar" title="Styles" icon="Palette">
    <template #title-button>
      <Button class="transition-colors" severity="primary" title="AI Prompt Assistant" @click="router.push({ name: 'styles-new' })">
        <Plus :size="16" />
      </Button>
    </template>
    <template #content>
      <!-- Top-level filters -->
      <div class="flex flex-col gap-1">
        <!-- All Styles -->
        <button
          class="flex h-8 w-full items-center gap-2 rounded-lg px-3 text-left transition-colors"
          :class="isAllActive ? 'bg-blue-50 text-blue-600' : 'text-slate-700 hover:bg-slate-50'"
          @click="onFilterAll">
          <Palette :size="14" />
          <span class="flex-1 truncate font-medium">All Styles</span>
        </button>

        <!-- Favorites -->
        <button
          class="flex h-8 w-full items-center gap-2 rounded-lg px-3 text-left transition-colors"
          :class="stylesStore.favoritesOnly ? 'bg-blue-50 text-blue-600' : 'text-slate-700 hover:bg-slate-50'"
          @click="onFilterFavorites">
          <Star :size="14" :class="stylesStore.favoritesOnly ? 'text-blue-500' : 'text-slate-400'" />
          <span class="flex-1 truncate font-medium">Favorites</span>
        </button>
      </div>

      <!-- Models section -->
      <div v-if="stylesStore.bundleTypeCounts.length" class="flex flex-col gap-1">
        <div class="flex h-8 items-center px-2">
          <span class="font-medium text-slate-900">Models</span>
        </div>
        <button
          v-for="bt in stylesStore.bundleTypeCounts"
          :key="bt.type_id"
          class="flex h-8 w-full items-center gap-2 rounded-lg px-3 text-left transition-colors"
          :class="stylesStore.currentBundleType === bt.type_id ? 'bg-blue-50 text-blue-600' : 'text-slate-700 hover:bg-slate-50'"
          @click="onFilterBundleType(bt.type_id)">
          <Cloud v-if="modelsStore.typeMap[bt.type_id]?.icon === 'cloud'" :size="14" class="text-slate-400" />
          <Gpu v-else :size="14" class="text-slate-400" />
          <span class="flex-1 truncate font-medium">{{ modelsStore.getTypeLabel(bt.type_id) }}</span>
          <span class="text-xs text-slate-400">{{ bt.count }}</span>
        </button>
      </div>

      <!-- Collections section -->
      <div v-if="stylesStore.categories.length" class="flex flex-col gap-1">
        <div class="flex h-8 items-center px-2">
          <span class="font-medium text-slate-900">Collections</span>
        </div>
        <button
          v-for="cat in stylesStore.categories"
          :key="cat"
          class="flex h-8 w-full items-center gap-2 rounded-lg px-3 text-left transition-colors"
          :class="stylesStore.currentCategory === cat ? 'bg-blue-50 text-blue-600' : 'text-slate-700 hover:bg-slate-50'"
          @click="stylesStore.filterByCategory(cat)">
          <FolderIcon :size="14" class="text-slate-400" />
          <span class="flex-1 truncate font-medium">{{ cat }}</span>
        </button>
      </div>

      <!-- Tags section -->
      <div class="flex flex-col gap-2">
        <div class="px-2">
          <span class="font-semibold text-slate-900">Tags</span>
        </div>
        <div class="flex flex-wrap gap-1 px-2">
          <button
            v-for="tag in stylesStore.tags"
            :key="tag.id"
            class="rounded-full px-2 py-1 font-medium transition-all"
            :class="stylesStore.currentTagId === tag.id ? 'ring-2 ring-offset-1' : ''"
            :style="tagStyle(tag)"
            @click="onToggleTag(tag.id)">
            {{ tag.name }}
          </button>
          <span v-if="!stylesStore.tags.length" class="px-1 text-slate-400">No style tags</span>
        </div>
      </div>
    </template>

    <template #footer v-if="stylesStore.importing">
      <div class="flex flex-col gap-1">
        <div class="flex items-center gap-2">
          <div class="w-3 h-3 border-2 border-blue-500 border-t-transparent rounded-full animate-spin shrink-0" />
          <span class="text-xs text-slate-600 truncate">{{ stylesStore.importMessage }}</span>
        </div>
        <div v-if="stylesStore.importTotal > 1" class="h-1 bg-slate-200 rounded-full overflow-hidden">
          <div
            class="h-full bg-blue-500 rounded-full transition-[width] duration-300 ease-out"
            :style="{ width: (stylesStore.importCurrent / stylesStore.importTotal) * 100 + '%' }" />
        </div>
        <ProgressBar v-else mode="indeterminate" class="h-1!" />
      </div>
    </template>
  </MenuPanel>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useStylesStore } from '@/stores/styles';
import { useModelsStore } from '@/stores/models';
import type { Tag } from '@/types/inference';
import MenuPanel from '@/components/MenuPanel.vue';

const route = useRoute();
const router = useRouter();
const stylesStore = useStylesStore();
const modelsStore = useModelsStore();

const showSidebar = computed(() => route.matched.some(r => r.path === '/styles'));
const isAllActive = computed(() =>
  !stylesStore.currentCategory && !stylesStore.currentBundleType && !stylesStore.favoritesOnly && !stylesStore.currentTagId
);

function tagStyle(tag: Tag) {
  const color = tag.color || '#64748b';
  return {
    color,
    backgroundColor: color + '15',
  };
}

function onFilterAll() {
  stylesStore.setFavoritesOnly(false);
  stylesStore.setCurrentTagId(null);
  stylesStore.setCurrentBundleType(null);
  stylesStore.filterByCategory(null);
}

function onFilterFavorites() {
  stylesStore.setCurrentCategory(null);
  stylesStore.setCurrentTagId(null);
  stylesStore.setCurrentBundleType(null);
  stylesStore.toggleFavoritesFilter();
}

function onFilterBundleType(typeId: string) {
  stylesStore.filterByBundleType(stylesStore.currentBundleType === typeId ? null : typeId);
}

function onToggleTag(tagId: number) {
  stylesStore.setFavoritesOnly(false);
  stylesStore.setCurrentCategory(null);
  stylesStore.setCurrentBundleType(null);
  stylesStore.filterByTag(stylesStore.currentTagId === tagId ? null : tagId);
}
</script>
