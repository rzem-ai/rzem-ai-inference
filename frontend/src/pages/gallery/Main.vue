<template>
  <div class="h-full flex flex-col px-2 py-4 gap-2">
    <!-- Toolbar -->
    <Toolbar class="shadow-lg">
      <template #start>
        <Button severity="primary" @click="onCreateFolder"><Plus :size="14" />New Folder</Button>

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
          title="Download"
          class="transition-all"
          :severity="selectionMode ? 'primary' : 'secondary'"
          :text="true"
          :disabled="!selectedIds.size"
          @click="onDownloadSelected">
          <Download :size="18" />
        </Button>
        <Button
          title="Move to folder"
          class="transition-all"
          :severity="selectionMode ? 'primary' : 'secondary'"
          :text="true"
          :disabled="!selectedIds.size"
          @click="folderPopover?.toggle($event)">
          <FolderInput :size="18" />
        </Button>
        <Button
          title="Tag"
          class="transition-all"
          :severity="selectionMode ? 'primary' : 'secondary'"
          :text="true"
          :disabled="!selectedIds.size"
          @click="tagPopover?.toggle($event)">
          <TagIcon :size="18" />
        </Button>

        <FolderPopover ref="folderPopover" :image-ids="[...selectedIds]" @moved="onImagesMoved" />
        <TagPopover ref="tagPopover" :image-ids="[...selectedIds]" @tagged="onImagesTagged" />

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
          <InputText v-model="searchInput" placeholder="Search images..." fluid />
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
                v-for="opt in gallerySortOptions"
                :key="opt.value"
                class="flex items-center justify-between gap-2 px-3 py-1 rounded-lg text-left w-full transition-colors"
                :class="gallery.sortBy === opt.value ? 'bg-blue-50 text-blue-600' : 'hover:bg-slate-50 text-slate-700'"
                @click="onSortBy(opt.value)">
                <span class="font-medium">{{ opt.label }}</span>
                <Check v-if="gallery.sortBy === opt.value" :size="14" />
              </button>
              <Divider class="my-1!" />
              <button
                class="flex items-center gap-2 px-3 py-1 rounded-lg text-left w-full transition-colors hover:bg-slate-50 text-slate-700"
                @click="onToggleSortOrder">
                <component :is="gallery.sortOrder === 'asc' ? 'ArrowUpAZ' : 'ArrowDownAZ'" :size="14" />
                <span class="font-medium">{{ gallery.sortOrder === 'asc' ? 'Ascending' : 'Descending' }}</span>
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

    <!-- Image grid area -->
    <GalleryList
      :is-grid-mode="isGridMode"
      :selected-ids="selectedIds"
      @card-click="onCardClick"
      @favorite="onToggleFavorite"
      @open-detail="detailVisible = true"
      @select="onToggleSelect"
      @contextmenu="onCardContextMenu" />

    <!-- Card context menu -->
    <ContextMenu ref="cardMenu" :model="cardMenuItems" />

    <!-- Image overlay (full-screen lightbox) -->
    <ImageOverlay
      v-if="selectedImage"
      :visible="overlayVisible"
      :image="selectedImage"
      @close="overlayVisible = false"
      @open-detail="detailVisible = true"
      @favorite="onToggleFavorite" />

    <!-- Image detail dialog -->
    <ImageDetailDialog
      v-if="selectedImage"
      v-model:visible="detailVisible"
      :image="selectedImage"
      @close-all="overlayVisible = false" />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, reactive, onMounted, watch } from 'vue';
import { useGalleryStore } from '@/stores/gallery';
import type { GalleryImage } from '@/types/inference';
import GalleryList from './GalleryList.vue';
import ImageOverlay from './ImageOverlay.vue';
import ImageDetailDialog from './ImageDetailDialog.vue';
import FolderPopover from './FolderPopover.vue';
import TagPopover from './TagPopover.vue';

const gallery = useGalleryStore();

const GRID_VIEW = { value: 'grid', icon: 'LayoutGrid' };
const LIST_VIEW = { value: 'list', icon: 'ListIcon' };

const viewMode = ref(GRID_VIEW);
const viewModes = ref([GRID_VIEW, LIST_VIEW]);
const selectionMode = ref(false);
const selectedIds = reactive(new Set<string>());

const searchInput = ref('');
const folderPopover = ref<InstanceType<typeof FolderPopover>>();
const tagPopover = ref<InstanceType<typeof TagPopover>>();
const sortPopover = ref();
const cardMenu = ref();
const contextImageId = ref<string | null>(null);

const gallerySortOptions = [
  { value: 'created_at', label: 'Date' },
  { value: 'raw_prompt', label: 'Prompt' },
  { value: 'steps', label: 'Steps' },
  { value: 'seed', label: 'Seed' },
  { value: 'file_size', label: 'File Size' },
];

const currentSortLabel = computed(() =>
  gallerySortOptions.find(o => o.value === gallery.sortBy)?.label ?? 'Date'
);

// Overlay & detail dialog state
const selectedImage = ref<GalleryImage | null>(null);
const overlayVisible = ref(false);
const detailVisible = ref(false);

const isGridMode = computed(() => viewMode.value.value === GRID_VIEW.value);

watch(selectionMode, (newValue) => {
  if (!newValue) {
    selectedIds.clear();
  }
});

// Debounced search
let searchTimeout: ReturnType<typeof setTimeout>;
watch(searchInput, (value) => {
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    gallery.searchImages(value);
  }, 300);
});

function toggleSelectionMode() {
  selectionMode.value = !selectionMode.value;
  if (!selectionMode.value) {
    selectedIds.clear();
  }
}

function onCardClick(imageId: string) {
  if (selectionMode.value) {
    onToggleSelect(imageId);
    return;
  }
  // Open the lightbox overlay
  const image = gallery.images.find(i => i.id === imageId);
  if (image) {
    selectedImage.value = image;
    overlayVisible.value = true;
  }
}

function onToggleSelect(imageId: string) {
  if (selectedIds.has(imageId)) {
    selectedIds.delete(imageId);
  } else {
    selectedIds.add(imageId);
  }
}

function onToggleFavorite(imageId: string) {
  gallery.toggleFavorite(imageId);
}

async function onDeleteSelected() {
  if (!selectedIds.size) return;
  const count = selectedIds.size;
  if (!confirm(`Delete ${count} image${count > 1 ? 's' : ''}?`)) return;

  for (const id of selectedIds) {
    await gallery.deleteImage(id);
  }
  selectedIds.clear();
}

async function onDownloadSelected() {
  if (!selectedIds.size) return;
  await gallery.batchSaveImages([...selectedIds]);
}

function onImagesMoved() {
  if (gallery.currentFolderId) {
    gallery.loadImages();
  }
}

function onImagesTagged() {
  if (gallery.currentTagId) {
    gallery.loadImages();
  }
}

function onSortBy(value: string) {
  gallery.setSort(value);
  sortPopover.value?.hide();
}

function onToggleSortOrder() {
  gallery.toggleSortOrder();
  sortPopover.value?.hide();
}

// ── Context menu ──

const cardMenuItems = computed(() => [
  {
    label: 'Add to Folder',
    icon: 'pi pi-folder',
    items: [
      ...gallery.folders.map(f => ({
        label: f.name,
        command: () => {
          if (contextImageId.value) {
            gallery.addImageToFolder(contextImageId.value, f.id);
          }
        },
      })),
      { separator: true },
      {
        label: 'New Folder',
        icon: 'pi pi-plus',
        command: () => {
          const name = prompt('Folder name:');
          if (name?.trim() && contextImageId.value) {
            const id = crypto.randomUUID();
            gallery.createFolder(id, name.trim()).then(() => {
              gallery.addImageToFolder(contextImageId.value!, id);
            });
          }
        },
      },
    ],
  },
  { separator: true },
  {
    label: 'Delete',
    icon: 'pi pi-trash',
    command: () => {
      if (contextImageId.value) {
        gallery.deleteImage(contextImageId.value);
      }
    },
  },
]);

function onCardContextMenu(event: MouseEvent, imageId: string) {
  contextImageId.value = imageId;
  cardMenu.value.show(event);
}

function onCreateFolder() {
  const name = prompt('Folder name:');
  if (name?.trim()) {
    const id = crypto.randomUUID();
    gallery.createFolder(id, name.trim());
  }
}

// Initialize gallery store (idempotent -- safe if Menu.vue already called it)
onMounted(() => {
  gallery.init();
});
</script>

<style scoped>
@reference "tailwindcss";

:deep(.p-toolbar-center) {
  width: 50%;
}

:deep(.p-toolbar) {
  padding: 0.5rem 0.75rem;
}

:deep(.p-toolbar-start) {
  @apply gap-1;
}
</style>
