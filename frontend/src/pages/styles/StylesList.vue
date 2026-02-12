<template>
  <div class="h-full flex flex-col gap-2">
    <!-- Toolbar -->
    <Toolbar>
      <template #start>
        <Button severity="primary" size="small"><Plus :size="14" />New Style</Button>
        <Divider layout="vertical" />
        <ToggleButton v-model="selectionMode" onLabel="Select" offLabel="Select" size="small">
          <template #icon><SquareCheck v-if="selectionMode" :size="14" /> <Square v-else="selectionMode" :size="14" /></template>
        </ToggleButton>
        <Button :disabled="!selectedIds.size" severity="secondary" size="small" @click="onDeleteSelected" text><Trash2 :size="14" /> </Button>
      </template>

      <template #center class="w-[50%]">
        <!-- Search bar -->
        <InputGroup>
          <InputGroupAddon> <Search :size="14" class="text-slate-400" /> </InputGroupAddon>
          <InputText v-model="searchInput" placeholder="Search styles..." size="small" fluid />
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

    <!-- Styles grid area -->
    <div class="flex-1 min-h-0 overflow-y-auto rounded-xl">
      <!-- Empty state -->
      <div v-if="!stylesStore.loading && stylesStore.styles.length === 0" class="h-full flex items-center justify-center">
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
      <div v-else class="grid gap-3 p-1" :class="viewMode === 'grid' ? 'grid-cols-[repeat(auto-fill,minmax(220px,1fr))]' : 'grid-cols-1'">
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
import { ref, reactive, watch } from 'vue';
import { useRouter } from 'vue-router';
import { Search, Plus, SquareCheck, Trash2, LayoutGrid, List as ListIcon, Palette, Square } from 'lucide-vue-next';

import { useStylesStore } from '@/stores/styles';
import StyleCard from './StyleCard.vue';
import { Button, Divider, ToggleButton, Toolbar } from 'primevue';
import { InputText, InputGroup, InputGroupAddon } from 'primevue';

const router = useRouter();
const stylesStore = useStylesStore();

const searchInput = ref('');

const viewMode = ref<'grid' | 'list'>('grid');
const selectionMode = ref(false);
const selectedIds = reactive(new Set<string>());

watch(selectionMode, (newValue) => {
  if (!newValue) {
    selectedIds.clear();
  }
});

let searchTimeout: ReturnType<typeof setTimeout>;
watch(searchInput, (value) => {
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    stylesStore.searchStyles(value);
  }, 300);
});

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

<style lang="css" scoped>
:deep(.p-toolbar-center) {
  width: 50%;
}

:deep(.p-toolbar) {
  padding: 0.5rem 0.75rem;
}
</style>
