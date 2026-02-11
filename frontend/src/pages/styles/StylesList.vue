<template>
  <div class="h-full flex flex-col gap-2">
    <!-- Toolbar -->
    <div class="flex items-center gap-1.5 px-3 h-10 bg-white rounded-xl border border-slate-200 shrink-0">
      <!-- New Style button -->
      <RouterLink
        :to="{ name: 'styles-new' }"
        class="flex items-center gap-1.5 px-2.5 h-7 rounded-md bg-blue-500 text-xs font-medium text-white hover:bg-blue-600 transition-colors">
        <Plus :size="14" />
        <span>New Style</span>
      </RouterLink>

      <div class="w-px h-4 bg-slate-200" />

      <!-- Select toggle -->
      <button
        class="flex items-center gap-1.5 px-2.5 h-7 rounded-md text-xs font-medium transition-colors"
        :class="selectionMode
          ? 'bg-blue-50 text-blue-600'
          : 'bg-slate-50 text-slate-600 hover:bg-slate-100'"
        @click="toggleSelectionMode">
        <SquareCheck :size="14" />
        <span>Select</span>
      </button>

      <!-- Batch delete -->
      <button
        class="p-1.5 rounded-md text-slate-500 hover:bg-slate-50 hover:text-slate-700
               disabled:opacity-30 disabled:pointer-events-none transition-colors"
        :disabled="!selectedIds.size"
        title="Delete"
        @click="onDeleteSelected">
        <Trash2 :size="14" />
      </button>

      <!-- Spacer -->
      <div class="flex-1" />

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

    <!-- Styles grid area -->
    <div class="flex-1 min-h-0 overflow-y-auto rounded-xl">
      <!-- Empty state -->
      <div
        v-if="!stylesStore.loading && stylesStore.styles.length === 0"
        class="h-full flex items-center justify-center">
        <div class="text-center">
          <Palette :size="48" class="text-slate-300 mx-auto mb-3" />
          <p class="text-sm text-slate-400">No styles yet</p>
          <p class="text-xs text-slate-300 mt-1">Create a style to define reusable prompt templates</p>
          <RouterLink
            :to="{ name: 'styles-new' }"
            class="inline-flex items-center gap-1.5 mt-4 px-4 py-2 rounded-lg bg-blue-500 text-sm font-medium text-white hover:bg-blue-600 transition-colors">
            <Plus :size="16" />
            <span>Create Style</span>
          </RouterLink>
        </div>
      </div>

      <!-- Grid view -->
      <div
        v-else
        class="grid gap-3 p-1"
        :class="viewMode === 'grid'
          ? 'grid-cols-[repeat(auto-fill,minmax(220px,1fr))]'
          : 'grid-cols-1'">
        <StyleCard
          v-for="style in stylesStore.styles"
          :key="style.id"
          :style-data="style"
          :selected="selectedIds.has(style.id)"
          @click="onCardClick"
          @favorite="onToggleFavorite"
          @select="onToggleSelect" />
      </div>

      <!-- Loading spinner -->
      <div v-if="stylesStore.loading" class="flex justify-center py-4">
        <div class="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue';
import { useRouter } from 'vue-router';
import {
  Plus, SquareCheck, Trash2, LayoutGrid, List as ListIcon, Palette,
} from 'lucide-vue-next';
import { useStylesStore } from '@/stores/styles';
import StyleCard from './StyleCard.vue';

const router = useRouter();
const stylesStore = useStylesStore();

const viewMode = ref<'grid' | 'list'>('grid');
const selectionMode = ref(false);
const selectedIds = reactive(new Set<string>());

function toggleSelectionMode() {
  selectionMode.value = !selectionMode.value;
  if (!selectionMode.value) {
    selectedIds.clear();
  }
}

function onCardClick(styleId: string) {
  if (selectionMode.value) {
    onToggleSelect(styleId);
  } else {
    router.push({ name: 'styles-edit', params: { id: styleId } });
  }
}

function onToggleSelect(styleId: string) {
  if (selectedIds.has(styleId)) {
    selectedIds.delete(styleId);
  } else {
    selectedIds.add(styleId);
  }
}

function onToggleFavorite(styleId: string) {
  stylesStore.toggleFavorite(styleId);
}

async function onDeleteSelected() {
  if (!selectedIds.size) return;
  const count = selectedIds.size;
  if (!confirm(`Delete ${count} style${count > 1 ? 's' : ''}?`)) return;

  for (const id of selectedIds) {
    await stylesStore.deleteStyle(id);
  }
  selectedIds.clear();
}
</script>
