<template>
  <div class="flex w-full">
    <!-- Sidebar -->
    <WorkspaceActions>
      <template #header>Styles</template>
      <template #toolbar>
        <div class="flex flex-col gap-2">
          <Button @click="handleCreateClick" severity="primary" size="small">
            <fa :icon="['fal', 'plus']" size="sm" class="mr-2" />
            New Style
          </Button>
          <Button @click="showLoraImport = true" severity="secondary" variant="outlined" size="small">
            <fa :icon="['fal', 'download']" size="sm" class="mr-2" />
            Import LoRA
          </Button>
        </div>
      </template>

      <template #body>
        <!-- Header -->
        <div class="flex items-center justify-between p-4 border-b border-surface-700">
          <h2 class="text-xl font-semibold text-surface-100">
            {{ headerTitle }}
          </h2>

          <InputGroup>
            <InputGroupAddon>
              <fa :icon="['fal', 'magnifying-glass']" size="sm" class="text-surface-400" />
            </InputGroupAddon>
            <InputText v-model="searchQuery" placeholder="Search styles..." size="small" class="w-64" />
          </InputGroup>
        </div>
        <div class="flex flex-col gap-2">
          <!-- Filter buttons -->
          <div class="flex flex-col gap-1">
            <Button
              :severity="selectedFilter === 'all' ? 'primary' : 'secondary'"
              :variant="selectedFilter === 'all' ? 'filled' : 'text'"
              size="small"
              @click="handleFilterChange('all')"
              class="justify-start">
              <fa :icon="['fal', 'layer-group']" size="sm" class="mr-2" />
              All Styles
              <span class="ml-auto text-xs text-surface-400">{{ stylesStore.styles.length }}</span>
            </Button>
            <Button
              :severity="selectedFilter === 'favorites' ? 'primary' : 'secondary'"
              :variant="selectedFilter === 'favorites' ? 'filled' : 'text'"
              size="small"
              @click="handleFilterChange('favorites')"
              class="justify-start">
              <fa :icon="['fas', 'star']" size="sm" class="mr-2" />
              Favorites
              <span class="ml-auto text-xs text-surface-400">{{ stylesStore.favoriteStyles.length }}</span>
            </Button>
          </div>

          <Divider />

          <!-- Category accordions (only show in "all" filter mode) -->
          <div v-if="selectedFilter === 'all' && !searchQuery.trim()" class="flex flex-col gap-2">
            <div class="mb-1 text-xs font-semibold uppercase text-surface-400">Categories</div>
            <StyleCategoryAccordion
              v-for="(styles, category) in stylesStore.stylesByCategory"
              :key="category"
              :category="category"
              :styles="styles"
              :isExpanded="expandedCategories.has(category)"
              :selectedIds="selectedStyleIds"
              @toggle="toggleCategory(category)"
              @select-style="handleStyleSelect"
              @toggle-select="toggleStyleSelection"
              @select-all="selectAllInCategory(category, styles)"
              @deselect-all="deselectAllInCategory(category, styles)"
              @edit="handleEditClick"
              @delete="confirmDeleteStyleById"
              @toggle-favorite="toggleStyleFavorite" />
          </div>
        </div>
      </template>
    </WorkspaceActions>

    <!-- Main Content -->
    <main class="flex flex-1 h-full p-2 transition-all duration-300 grow">
      <div class="flex flex-col flex-1 h-full gap-2 p-4 overflow-hidden border rounded-lg border-surface-700 bg-surface-950">
        <!-- Welcome state -->
        <StyleWelcomePanel v-if="rightPanelMode === 'welcome'" @create="handleCreateClick" @import="showLoraImport = true" />

        <!-- Create/Edit state -->
        <StyleEditorPanel
          v-else-if="rightPanelMode === 'create' || rightPanelMode === 'edit'"
          :style="editingStyle"
          @save="handleSaveStyle"
          @close="closeRightPanel" />

        <!-- Detail state -->
        <div v-else-if="rightPanelMode === 'detail' && stylesStore.selectedStyle" class="flex flex-col h-full">
          <StyleDetailPanel
            :style="stylesStore.selectedStyle"
            @edit="handleEditClick(stylesStore.selectedStyle.id)"
            @delete="confirmDeleteStyle(stylesStore.selectedStyle)"
            @add-lora="openAddLoraDialog"
            @remove-lora="handleRemoveLora"
            @add-example="openAddExampleDialog"
            @remove-example="handleRemoveExample"
            @thumbnail-updated="handleThumbnailUpdated"
            @thumbnail-removed="handleThumbnailRemoved" />
        </div>
      </div>
    </main>

    <!-- Right panel - Always visible -->
    <div class="flex flex-col h-full border-surface-700 bg-surface-900"> </div>
  </div>

  <!-- Import LoRA Dialog -->
  <LoraImportDialog v-model:visible="showLoraImport" @imported="handleLoraImported" />

  <!-- Bulk Categorize Dialog -->
  <BulkCategorizeDialog
    v-model:visible="showBulkCategorize"
    :selectedCount="selectedStyleIds.size"
    :existingCategories="existingCategories"
    @apply="handleBulkCategorize" />

  <!-- Delete Confirmation Dialog -->
  <ConfirmDialog />
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useStylesStore } from '@/stores/styles';
import { useConfirm } from 'primevue/useconfirm';
import type { StyleInfo } from '@/types';
import WorkspaceActions from '@/components/shared/WorkspaceActions.vue';

import StyleCategoryAccordion from '@/components/styles/StyleCategoryAccordion.vue';

import BulkCategorizeDialog from '@/components/styles/BulkCategorizeDialog.vue';
import StyleWelcomePanel from '@/components/styles/StyleWelcomePanel.vue';
import StyleEditorPanel from '@/components/styles/StyleEditorPanel.vue';
import StyleDetailPanel from '@/components/styles/StyleDetailPanel.vue';
import LoraImportDialog from '@/components/styles/LoraImportDialog.vue';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import Divider from 'primevue/divider';

import ConfirmDialog from 'primevue/confirmdialog';
import InputGroup from 'primevue/inputgroup';
import InputGroupAddon from 'primevue/inputgroupaddon';

const stylesStore = useStylesStore();
const confirm = useConfirm();

type RightPanelMode = 'welcome' | 'create' | 'edit' | 'detail';

const selectedFilter = ref<'all' | 'favorites'>('all');
const searchQuery = ref('');
const selectedStyleId = ref<string | null>(null);
const selectedStyleIds = ref<Set<string>>(new Set());
const expandedCategories = ref<Set<string>>(new Set());
const rightPanelMode = ref<RightPanelMode>('welcome');
const editingStyle = ref<StyleInfo | null>(null);
const showLoraImport = ref(false);
const showBulkCategorize = ref(false);

// Computed properties
const headerTitle = computed(() => {
  if (searchQuery.value.trim()) {
    return `Search Results (${displayedStyles.value.length})`;
  }
  return selectedFilter.value === 'favorites' ? 'Favorite Styles' : 'All Styles';
});

const displayedStyles = computed(() => {
  let styles = stylesStore.styles;

  // Apply filter
  if (selectedFilter.value === 'favorites') {
    styles = stylesStore.favoriteStyles;
  }

  // Apply search
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase();
    styles = styles.filter(
      (s) => s.name.toLowerCase().includes(query) || s.description?.toLowerCase().includes(query) || s.promptTemplate.toLowerCase().includes(query),
    );
  }

  return styles;
});

const existingCategories = computed(() => {
  return Object.keys(stylesStore.stylesByCategory).filter((cat) => cat !== 'Uncategorized');
});

// Category accordion management
function toggleCategory(category: string) {
  if (expandedCategories.value.has(category)) {
    expandedCategories.value.delete(category);
  } else {
    expandedCategories.value.add(category);
  }
}

function handleFilterChange(filter: 'all' | 'favorites') {
  selectedFilter.value = filter;
  clearSelection();
}

// Selection management
function toggleStyleSelection(styleId: string) {
  if (selectedStyleIds.value.has(styleId)) {
    selectedStyleIds.value.delete(styleId);
  } else {
    selectedStyleIds.value.add(styleId);
  }
  // Trigger reactivity
  selectedStyleIds.value = new Set(selectedStyleIds.value);
}

function selectAllInCategory(_category: string, styles: StyleInfo[]) {
  styles.forEach((style) => selectedStyleIds.value.add(style.id));
  selectedStyleIds.value = new Set(selectedStyleIds.value);
}

function deselectAllInCategory(_category: string, styles: StyleInfo[]) {
  styles.forEach((style) => selectedStyleIds.value.delete(style.id));
  selectedStyleIds.value = new Set(selectedStyleIds.value);
}

function clearSelection() {
  selectedStyleIds.value.clear();
  selectedStyleIds.value = new Set(selectedStyleIds.value);
}

// Right panel management
function closeRightPanel() {
  rightPanelMode.value = 'welcome';
  editingStyle.value = null;
  selectedStyleId.value = null;
}

function handleCreateClick() {
  editingStyle.value = null;
  rightPanelMode.value = 'create';
  clearSelection();
}

async function handleEditClick(styleIdOrInfo: string | StyleInfo) {
  const styleId = typeof styleIdOrInfo === 'string' ? styleIdOrInfo : styleIdOrInfo.id;
  const style = stylesStore.styles.find((s) => s.id === styleId);
  if (style) {
    editingStyle.value = style;
    rightPanelMode.value = 'edit';
    clearSelection();
  }
}

async function handleStyleSelect(styleId: string) {
  selectedStyleId.value = styleId;
  await stylesStore.loadStyleDetail(styleId);
  rightPanelMode.value = 'detail';
  clearSelection();
}

async function handleBulkCategorize(category: string) {
  try {
    await stylesStore.bulkUpdateStyles(Array.from(selectedStyleIds.value), { category });
    await stylesStore.loadStyles();
    clearSelection();
  } catch (error) {
    console.error('Failed to categorize styles:', error);
  }
}

// Style operations
async function handleSaveStyle(styleData: any) {
  try {
    if (editingStyle.value) {
      await stylesStore.updateStyle(editingStyle.value.id, styleData);
    } else {
      const newStyle = await stylesStore.createStyle(styleData);
      // Switch to detail view of newly created style
      selectedStyleId.value = newStyle.id;
      await stylesStore.loadStyleDetail(newStyle.id);
      rightPanelMode.value = 'detail';
    }
    await stylesStore.loadStyles();
  } catch (error) {
    console.error('Failed to save style:', error);
  }
}

function confirmDeleteStyle(style: StyleInfo) {
  confirm.require({
    message: `Are you sure you want to delete "${style.name}"?`,
    header: 'Delete Style',
    icon: 'fa fa-exclamation-triangle',
    acceptClass: 'p-button-danger',
    accept: async () => {
      try {
        await stylesStore.deleteStyle(style.id);
        if (selectedStyleId.value === style.id) {
          closeRightPanel();
        }
      } catch (error) {
        console.error('Failed to delete style:', error);
      }
    },
  });
}

function confirmDeleteStyleById(styleId: string) {
  const style = stylesStore.styles.find((s) => s.id === styleId);
  if (style) {
    confirmDeleteStyle(style);
  }
}

async function toggleStyleFavorite(styleId: string) {
  const style = stylesStore.styles.find((s) => s.id === styleId);
  if (style) {
    try {
      await stylesStore.updateStyle(styleId, {
        name: style.name,
        description: style.description,
        promptTemplate: style.promptTemplate,
        defaultStrength: style.defaultStrength,
        strengthMin: style.strengthMin,
        strengthMax: style.strengthMax,
        category: style.category,
        isFavorite: !style.isFavorite,
      });
      await stylesStore.loadStyles();
      // Reload detail if this style is selected
      if (selectedStyleId.value === styleId) {
        await stylesStore.loadStyleDetail(styleId);
      }
    } catch (error) {
      console.error('Failed to toggle favorite:', error);
    }
  }
}

// Thumbnail operations
async function handleThumbnailUpdated(file: File) {
  if (selectedStyleId.value) {
    try {
      await stylesStore.uploadThumbnail(selectedStyleId.value, file);
      await stylesStore.loadStyles();
      await stylesStore.loadStyleDetail(selectedStyleId.value);
    } catch (error) {
      console.error('Failed to upload thumbnail:', error);
    }
  }
}

async function handleThumbnailRemoved() {
  if (selectedStyleId.value) {
    try {
      await stylesStore.deleteThumbnail(selectedStyleId.value);
      await stylesStore.loadStyles();
      await stylesStore.loadStyleDetail(selectedStyleId.value);
    } catch (error) {
      console.error('Failed to remove thumbnail:', error);
    }
  }
}

// LoRA and example operations (stub implementations)
function openAddLoraDialog() {
  console.log('Add LoRA to style');
}

function handleRemoveLora(loraId: string) {
  console.log('Remove LoRA:', loraId);
}

function openAddExampleDialog() {
  console.log('Add example');
}

function handleRemoveExample(exampleId: string) {
  console.log('Remove example:', exampleId);
}

async function handleLoraImported() {
  await stylesStore.loadStyles();
}

onMounted(async () => {
  await stylesStore.loadStyles();
  // Expand first category by default
  const firstCategory = Object.keys(stylesStore.stylesByCategory)[0];
  if (firstCategory) {
    expandedCategories.value.add(firstCategory);
  }
});
</script>

<style scoped>
@reference "tailwindcss";

.slide-down-enter-active,
.slide-down-leave-active {
  transition: all 0.3s ease;
}

.slide-down-enter-from,
.slide-down-leave-to {
  transform: translateY(-100%);
  opacity: 0;
}
</style>
