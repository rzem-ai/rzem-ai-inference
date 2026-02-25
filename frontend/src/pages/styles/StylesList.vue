<template>
  <div class="h-full flex flex-col gap-2">
    <!-- Toolbar -->
    <Toolbar>
      <template #start>
        <Button severity="primary" @click="router.push({ name: 'styles-new' })"><Plus :size="14" />New Style</Button>
        <Button severity="secondary" :loading="stylesStore.importing" @click="onImportMetadata"> <FolderInput :size="14" /> Import </Button>

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
          <Button severity="secondary" @click="sortPopover?.toggle($event)">
            <ArrowUpDown :size="14" />
            <span>{{ currentSortLabel }}</span>
            <ChevronDown :size="12" class="text-slate-400" />
          </Button>
          <Popover ref="sortPopover">
            <div class="flex flex-col gap-1 min-w-40 p-1">
              <div class="px-2 py-1 text-xs font-semibold text-slate-400 uppercase tracking-wide">Sort by</div>
              <button
                v-for="opt in styleSortOptions"
                :key="opt.value"
                class="flex items-center justify-between gap-2 px-3 py-1 rounded-lg text-left w-full transition-colors"
                :class="stylesStore.sortBy === opt.value ? 'bg-blue-50 text-blue-600' : 'hover:bg-slate-50 text-slate-700'"
                @click="onSortBy(opt.value)">
                <span class="font-medium">{{ opt.label }}</span>
                <Check v-if="stylesStore.sortBy === opt.value" :size="14" />
              </button>
              <Divider class="my-1!" />
              <button
                class="flex items-center gap-2 px-3 py-1 rounded-lg text-left w-full transition-colors hover:bg-slate-50 text-slate-700"
                @click="onToggleSortOrder">
                <component :is="stylesStore.sortOrder === 'asc' ? ArrowUpAZ : ArrowDownAZ" :size="14" />
                <span class="font-medium">{{ stylesStore.sortOrder === 'asc' ? 'Ascending' : 'Descending' }}</span>
              </button>
            </div>
          </Popover>
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
    <div ref="containerRef" class="flex-1 min-h-0 overflow-hidden rounded-xl">
      <!-- Empty state -->
      <div v-if="!stylesStore.loading && stylesStore.styles.length === 0" class="h-full flex items-center justify-center">
        <div class="text-center">
          <Palette :size="48" class="text-slate-300 mx-auto mb-3" />
          <p class="text-sm text-slate-400">No styles yet</p>
          <p class="text-xs text-slate-300 mt-1">Create a style to define reusable prompt templates</p>
          <RouterLink
            :to="{ name: 'styles-new' }"
            class="inline-flex items-center gap-1 mt-4 px-4 py-2 rounded-lg bg-blue-500 text-sm font-medium text-white hover:bg-blue-600 transition-colors">
            <Plus :size="16" />
            <span>Create Style</span>
          </RouterLink>
        </div>
      </div>

      <!-- Virtualised grid view -->
      <VirtualGrid
        v-else
        :items="rows"
        :item-size="rowHeight">
        <template #default="{ item: row, first }">
          <div :style="gridStyle" :class="{ 'pt-0!': first }">
            <StyleCard
              v-for="style in row"
              :key="style.id"
              :style-data="style"
              :selected="selectedIds.has(style.id)"
              @click="onCardClick"
              @favorite="onToggleFavorite"
              @select="onToggleSelect"
              @contextmenu="onCardContextMenu" />
          </div>
        </template>
      </VirtualGrid>

      <!-- Loading spinner -->
      <div v-if="stylesStore.loading" class="flex justify-center py-4">
        <div class="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
      </div>
    </div>

    <!-- Card context menu -->
    <ContextMenu ref="cardMenu" :model="cardMenuItems" />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, reactive, watch, toRef } from 'vue';
import { useRouter } from 'vue-router';
import { Search, Plus, Trash2, LayoutGrid, List as ListIcon, Palette, ListTodo } from 'lucide-vue-next';
import { FolderInput, ArrowUpDown, ArrowUpAZ, ArrowDownAZ, ChevronDown, Check } from 'lucide-vue-next';

import { useStylesStore } from '@/stores/styles';
import { useVirtualGrid } from '@/composables/useVirtualGrid';
import StyleCard from './StyleCard.vue';
import VirtualGrid from '@/components/VirtualGrid.vue';
import { Button, ContextMenu, Divider, Toolbar, SelectButton, Popover } from 'primevue';
import { InputText, InputGroup, InputGroupAddon } from 'primevue';

const router = useRouter();
const stylesStore = useStylesStore();
const containerRef = ref<HTMLElement>();

// StyleCard has aspect-4/3 image + ~60px info section below
const { rows, rowHeight, gridStyle } = useVirtualGrid(
  toRef(() => stylesStore.styles),
  containerRef,
  { aspectRatio: 3 / 4, extraHeight: 60 },
);

const GRID_VIEW = { value: 'grid', icon: LayoutGrid };
const LIST_VIEW = { value: 'list', icon: ListIcon };

const viewMode = ref(GRID_VIEW);
const viewModes = ref([GRID_VIEW, LIST_VIEW]);
const selectionMode = ref(false);
const selectedIds = reactive(new Set<string>());
const cardMenu = ref();
const contextStyleId = ref<string | null>(null);

const searchInput = ref('');
const sortPopover = ref();

const styleSortOptions = [
  { value: 'updated_at', label: 'Last Modified' },
  { value: 'created_at', label: 'Date Created' },
  { value: 'name', label: 'Name' },
  { value: 'usage_count', label: 'Usage' },
];

const currentSortLabel = computed(() =>
  styleSortOptions.find(o => o.value === stylesStore.sortBy)?.label ?? 'Last Modified'
);

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

function onSortBy(value: string) {
  stylesStore.setSort(value);
  sortPopover.value?.hide();
}

function onToggleSortOrder() {
  stylesStore.toggleSortOrder();
  sortPopover.value?.hide();
}

// ── Context menu ──

const cardMenuItems = computed(() => [
  {
    label: 'Add to Collection',
    icon: 'pi pi-folder',
    items: stylesStore.categories.map(cat => ({
      label: cat,
      command: () => {
        if (contextStyleId.value) {
          stylesStore.updateStyle(contextStyleId.value, { category: cat });
        }
      },
    })),
  },
  { separator: true },
  {
    label: 'Delete',
    icon: 'pi pi-trash',
    command: () => {
      if (contextStyleId.value) {
        stylesStore.deleteStyle(contextStyleId.value);
      }
    },
  },
]);

function onCardContextMenu(event: MouseEvent, styleId: string) {
  contextStyleId.value = styleId;
  cardMenu.value.show(event);
}

async function onImportMetadata() {
  await stylesStore.importCivitaiMetadata();
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
