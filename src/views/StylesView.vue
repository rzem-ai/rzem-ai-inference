<template>
  <div class="flex w-full">
    <!-- Sidebar -->
    <WorkspaceActions>
      <template #header>Styles</template>
      <template #toolbar>
        <div class="flex flex-row gap-2">
          <Button @click="handleCreateClick" severity="primary" size="small" class="grow">
            <fa :icon="['fal', 'plus']" size="sm" class="mr-2" />
            New Style
          </Button>
          <Button @click="showLoraImport = true" severity="secondary" variant="outlined" size="small" class="grow">
            <fa :icon="['fal', 'download']" size="sm" class="mr-2" />
            Import LoRA
          </Button>
        </div>
      </template>

      <template #body>
        <!-- Header -->
        <div class="flex items-center justify-between mb-4">
          <InputGroup>
            <InputGroupAddon>
              <fa :icon="['fal', 'magnifying-glass']" size="sm" class="text-surface-400" />
            </InputGroupAddon>
            <InputText v-model="searchQuery" placeholder="Search styles..." size="small" class="w-64" />
          </InputGroup>
        </div>
        <div class="flex flex-col gap-2">
          <StylesList v-for="(styles, category) in stylesStore.stylesByCategory" :key="category" :category="category" :styles="styles" />
        </div>
      </template>
    </WorkspaceActions>

    <!-- Main Content -->
    <main class="flex flex-1 h-full p-2 transition-all duration-300 grow">
      <div class="flex flex-col flex-1 h-full gap-2 p-4 overflow-hidden border rounded-lg border-surface-700 bg-surface-950">
        <RouterView />
      </div>
    </main>

    <!-- Right panel - Always visible -->
    <div class="flex flex-col h-full border-surface-700 bg-surface-900"> </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useStylesStore } from '@/stores/styles';

import type { StyleInfo } from '@/types';
import WorkspaceActions from '@/components/shared/WorkspaceActions.vue';

import StylesList from '@/components/styles/StylesList.vue';

import Button from 'primevue/button';
import InputText from 'primevue/inputtext';

import InputGroup from 'primevue/inputgroup';
import InputGroupAddon from 'primevue/inputgroupaddon';
import { useRouter } from 'vue-router';

const router = useRouter();
const stylesStore = useStylesStore();

const searchQuery = ref('');

const selectedStyleIds = ref<Set<string>>(new Set());
const expandedCategories = ref<Set<string>>(new Set());

const editingStyle = ref<StyleInfo | null>(null);
const showLoraImport = ref(false);

// Computed properties

function clearSelection() {
  selectedStyleIds.value.clear();
  selectedStyleIds.value = new Set(selectedStyleIds.value);
}

function handleCreateClick() {
  editingStyle.value = null;

  clearSelection();
  console.log('goto:', `/styles/create`);
  router.push({ name: `styles-create` });
}

// // Style operations
// async function handleSaveStyle(styleData: any) {
//   try {
//     if (editingStyle.value) {
//       await stylesStore.updateStyle(editingStyle.value.id, styleData);
//     } else {
//       const newStyle = await stylesStore.createStyle(styleData);
//       // Switch to detail view of newly created style
//       selectedStyleId.value = newStyle.id;
//       await stylesStore.loadStyleDetail(newStyle.id);
//       rightPanelMode.value = 'detail';
//     }
//     await stylesStore.loadStyles();
//   } catch (error) {
//     console.error('Failed to save style:', error);
//   }
// }

// function confirmDeleteStyle(style: StyleInfo) {
//   confirm.require({
//     message: `Are you sure you want to delete "${style.name}"?`,
//     header: 'Delete Style',
//     icon: 'fa fa-exclamation-triangle',
//     acceptClass: 'p-button-danger',
//     accept: async () => {
//       try {
//         await stylesStore.deleteStyle(style.id);
//         if (selectedStyleId.value === style.id) {
//           closeRightPanel();
//         }
//       } catch (error) {
//         console.error('Failed to delete style:', error);
//       }
//     },
//   });
// }

// function confirmDeleteStyleById(styleId: string) {
//   const style = stylesStore.styles.find((s) => s.id === styleId);
//   if (style) {
//     confirmDeleteStyle(style);
//   }
// }

// async function toggleStyleFavorite(styleId: string) {
//   const style = stylesStore.styles.find((s) => s.id === styleId);
//   if (style) {
//     try {
//       await stylesStore.updateStyle(styleId, {
//         name: style.name,
//         description: style.description,
//         promptTemplate: style.promptTemplate,
//         defaultStrength: style.defaultStrength,
//         strengthMin: style.strengthMin,
//         strengthMax: style.strengthMax,
//         category: style.category,
//         isFavorite: !style.isFavorite,
//       });
//       await stylesStore.loadStyles();
//       // Reload detail if this style is selected
//       if (selectedStyleId.value === styleId) {
//         await stylesStore.loadStyleDetail(styleId);
//       }
//     } catch (error) {
//       console.error('Failed to toggle favorite:', error);
//     }
//   }
// }

onMounted(async () => {
  // Expand first category by default
  const firstCategory = Object.keys(stylesStore.stylesByCategory)[0];
  if (firstCategory) {
    expandedCategories.value.add(firstCategory);
  }
});
</script>

<style scoped>
@reference "tailwindcss";
</style>
