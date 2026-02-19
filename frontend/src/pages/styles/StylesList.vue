<template>
  <div class="h-full flex flex-col gap-2">
    <!-- Toolbar -->
    <Toolbar>
      <template #start>
        <Button severity="primary" @click="router.push({ name: 'styles-new' })"><Plus :size="14" />New Style</Button>
        <Button severity="secondary" :loading="importing" @click="onImportMetadata"> <FolderInput :size="14" /> Import </Button>

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
          <InputText v-model="searchInput" placeholder="Search styles..." fluid />
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
      <div
        v-else
        class="grid gap-3 p-1"
        :class="isGridMode ? ' sm:grid-cols-1 lg:grid-cols-3 xl:grid-cols-3 2xl:grid-cols-4 3xl:grid-cols-5 grid-cols-2' : 'grid-cols-1'">
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
import { computed, ref, reactive, watch } from 'vue';
import { useRouter } from 'vue-router';
import { Search, Plus, Trash2, LayoutGrid, List as ListIcon, Palette, ListTodo } from 'lucide-vue-next';
import { FolderInput, ArrowUpDown, ChevronDown } from 'lucide-vue-next';

import { useStylesStore } from '@/stores/styles';
import StyleCard from './StyleCard.vue';
import { Button, Divider, Toolbar, SelectButton } from 'primevue';
import { InputText, InputGroup, InputGroupAddon } from 'primevue';

const router = useRouter();
const stylesStore = useStylesStore();

const GRID_VIEW = { value: 'grid', icon: LayoutGrid };
const LIST_VIEW = { value: 'list', icon: ListIcon };

const viewMode = ref(GRID_VIEW);
const viewModes = ref([GRID_VIEW, LIST_VIEW]);
const selectionMode = ref(false);
const selectedIds = reactive(new Set<string>());

const searchInput = ref('');
const importing = ref(false);

const isGridMode = computed(() => viewMode.value.value === GRID_VIEW.value);

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

async function onImportMetadata() {
  importing.value = true;
  await stylesStore.importCivitaiMetadata();
  importing.value = false;
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
