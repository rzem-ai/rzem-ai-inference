<template>
  <div class="flex w-full h-full">
    <!-- Sidebar -->
    <WorkspaceActions>
      <template #header>Image Gallery</template>
      <template #body>
        <FolderTree @create-folder="openCreateFolderDialog" @edit-folder="openEditFolderDialog" @delete-folder="confirmDeleteFolder" />
      </template>
      <template #footer>
        <TagManager />
      </template>
    </WorkspaceActions>

    <!-- Main Content -->
    <main class="flex flex-col flex-1 h-full overflow-hidden">
      <div class="p-4 bg-gray-800 border-b border-gray-600">
        <!-- Breadcrumb -->
        <div class="breadcrumb">
          <template v-if="foldersStore.currentViewType === 'all'">
            <span class="breadcrumb-item active">All Images</span>
          </template>
          <template v-else-if="foldersStore.currentViewType === 'uncategorized'">
            <span class="breadcrumb-item active">Uncategorized</span>
          </template>
          <template v-else-if="foldersStore.currentFolder">
            <span class="breadcrumb-item clickable" @click="handleSelectAll">All Images</span>
            <i class="pi pi-chevron-right breadcrumb-separator"></i>
            <template v-for="(name, index) in foldersStore.currentBreadcrumb" :key="index">
              <span v-if="index < foldersStore.currentBreadcrumb.length - 1" class="breadcrumb-item clickable" @click="navigateToBreadcrumb(index)">
                {{ name }}
              </span>
              <span v-else class="breadcrumb-item active">{{ name }}</span>
              <i v-if="index < foldersStore.currentBreadcrumb.length - 1" class="pi pi-chevron-right breadcrumb-separator"></i>
            </template>
          </template>
        </div>

        <div class="flex items-center gap-4">
          <div class="flex gap-2 grow">
            <InputText v-model="galleryStore.filters.searchQuery" placeholder="Search prompts..." @keyup.enter="handleSearch" size="small" fluid />
            <Button @click="handleSearch" :loading="galleryStore.isLoading"><Search class="w-4 h-4" /></Button>
          </div>

          <div class="flex items-center gap-2 ml-auto">
            <Button :disabled="galleryStore.selectedImages.size == 0" size="small" severity="secondary" @click="showAddToFolderMenu">
              <div class="flex items-center gap-2"><Folder class="w-4 h-4" /> Add to Folder</div>
            </Button>
            <AutoTagButton @open-settings="openAutoTagSettings" @tagging-complete="handleTaggingComplete" />
            <Button label="Select All" severity="secondary" @click="galleryStore.selectAll"><CheckSquare class="w-4 h-4" /></Button>
            <Button label="Clear" severity="secondary" @click="galleryStore.clearSelection" :disabled="galleryStore.selectedImages.size === 0">
              <X class="w-4 h-4" />
            </Button>
            <span class="selection-count"> {{ galleryStore.selectedImages.size }} selected </span>
          </div>
        </div>
      </div>

      <!-- Loading State:  -->
      <div v-if="galleryStore.isLoading" class="flex flex-col items-center justify-center flex-1 gap-4 text-gray-200">
        <RefreshCw class="w-6 h-6" :class="{ 'animate-spin': galleryStore.isLoading }" />
        <p>Loading images...</p>
      </div>

      <!-- Empty State -->
      <div v-else-if="galleryStore.filteredImages.length === 0" class="empty-state">
        <i class="pi pi-images" style="font-size: 3rem; color: #9ca3af"></i>
        <p>No images found</p>
        <p class="empty-hint">
          <template v-if="foldersStore.currentViewType === 'folder'"> Drag images here to add them to this folder </template>
          <template v-else-if="foldersStore.currentViewType === 'uncategorized'"> All images have been organized into folders </template>
          <template v-else> Generate some images to see them here! </template>
        </p>
      </div>

      <!-- Image Grid -->
      <ImageGrid
        v-else
        :images="galleryStore.filteredImages"
        :selected-ids="galleryStore.selectedImages"
        @select="handleSelectImage"
        @open-detail="handleOpenDetail"
        @toggle-favorite="handleToggleFavorite"
        @add-to-compare="handleAddToCompare" />
    </main>

    <!-- Folder Form Dialog -->
    <FolderForm v-model:visible="folderFormVisible" :folder="editingFolder" :default-parent-id="defaultParentId" />

    <!-- Add to Folder Menu -->
    <Menu ref="addToFolderMenuRef" :model="addToFolderMenuItems" popup />

    <!-- Delete Confirmation -->
    <ConfirmDialog />

    <!-- Auto-Tag Settings Dialog -->
    <AutoTagSettings v-model:visible="autoTagSettingsVisible" />

    <!-- Toast for notifications -->
    <Toast />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useConfirm } from 'primevue/useconfirm';
import { useToast } from 'primevue/usetoast';
import { useGalleryStore } from '@/stores/gallery';
import { useFoldersStore, type FolderNode } from '@/stores/folders';
import { useTagsStore } from '@/stores/tags';
import { useCompareStore } from '@/stores/compare';
import { useAutoTagStore } from '@/stores/autoTag';
import ImageGrid from '@/components/gallery/ImageGrid.vue';
import FolderTree from '@/components/gallery/FolderTree.vue';
import FolderForm from '@/components/gallery/FolderForm.vue';
import TagManager from '@/components/gallery/TagManager.vue';
import AutoTagButton from '@/components/gallery/AutoTagButton.vue';
import AutoTagSettings from '@/components/gallery/AutoTagSettings.vue';
import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import Menu from 'primevue/menu';
import ConfirmDialog from 'primevue/confirmdialog';
import Toast from 'primevue/toast';
import { CheckSquare, Folder, RefreshCw, Search, X } from 'lucide-vue-next';
import WorkspaceActions from '@/components/shared/WorkspaceActions.vue';

const galleryStore = useGalleryStore();
const foldersStore = useFoldersStore();
const tagsStore = useTagsStore();
const compareStore = useCompareStore();
const autoTagStore = useAutoTagStore();
const confirm = useConfirm();
const toast = useToast();

// Folder form state
const folderFormVisible = ref(false);
const editingFolder = ref<FolderNode | null>(null);
const defaultParentId = ref<string | null>(null);

// Auto-tag settings dialog
const autoTagSettingsVisible = ref(false);

// Add to folder menu
const addToFolderMenuRef = ref<InstanceType<typeof Menu> | null>(null);

const addToFolderMenuItems = computed(() => {
  const items = foldersStore.flatFolders.map((folder) => ({
    label: folder.path.length > 0 ? `${folder.path.join(' / ')} / ${folder.name}` : folder.name,
    icon: 'pi pi-folder',
    command: () => addSelectedToFolder(folder.id),
  }));

  if (items.length === 0) {
    return [
      {
        label: 'No folders available',
        disabled: true,
      },
      {
        separator: true,
      },
      {
        label: 'Create New Folder',
        icon: 'pi pi-plus',
        command: () => openCreateFolderDialog(),
      },
    ];
  }

  return [
    ...items,
    { separator: true },
    {
      label: 'Create New Folder',
      icon: 'pi pi-plus',
      command: () => openCreateFolderDialog(),
    },
  ];
});

onMounted(async () => {
  await Promise.all([
    galleryStore.loadImages(),
    foldersStore.loadFolders(),
    tagsStore.loadTags(),
    autoTagStore.loadSettings(),
    // Note: checkModelStatus() is deferred to settings dialog to avoid blocking
  ]);
});

const handleSearch = async () => {
  if (galleryStore.filters.searchQuery.trim()) {
    await galleryStore.searchImages(galleryStore.filters.searchQuery);
  } else {
    // Reload based on current view
    if (foldersStore.currentViewType === 'folder' && foldersStore.currentFolderId) {
      await galleryStore.loadFolderImages(foldersStore.currentFolderId);
    } else if (foldersStore.currentViewType === 'uncategorized') {
      await galleryStore.loadUncategorizedImages();
    } else {
      await galleryStore.loadImages();
    }
  }
};

const handleToggleFavorite = async (imageId: string) => {
  await galleryStore.toggleFavorite(imageId);
};

const handleSelectImage = (imageId: string) => {
  galleryStore.toggleSelectImage(imageId);
};

const handleOpenDetail = (image: any) => {
  console.log('Open detail for:', image);
  // TODO: Open image detail modal
};

const handleAddToCompare = (image: any) => {
  const success = compareStore.addToCompare(image);
  if (!success) {
    console.warn('Cannot add more images to compare');
  }
};

const handleSelectAll = () => {
  foldersStore.setViewType('all');
  galleryStore.loadAllImages();
};

const navigateToBreadcrumb = (index: number) => {
  // Navigate to ancestor folder
  const folder = foldersStore.flatFolders.find((f) => f.name === foldersStore.currentBreadcrumb[index] && f.path.length === index);
  if (folder) {
    foldersStore.setCurrentFolder(folder.id);
    galleryStore.loadFolderImages(folder.id);
  }
};

// Folder management
const openCreateFolderDialog = (parentId?: string) => {
  editingFolder.value = null;
  defaultParentId.value = parentId || null;
  folderFormVisible.value = true;
};

const openEditFolderDialog = (folder: FolderNode) => {
  editingFolder.value = folder;
  defaultParentId.value = null;
  folderFormVisible.value = true;
};

const confirmDeleteFolder = (folder: FolderNode) => {
  confirm.require({
    message: `Are you sure you want to delete "${folder.name}"? ${
      folder.children.length > 0 ? 'All subfolders will also be deleted.' : ''
    } Images will not be deleted.`,
    header: 'Delete Folder',
    icon: 'pi pi-exclamation-triangle',
    rejectClass: 'p-button-secondary',
    acceptClass: 'p-button-danger',
    accept: async () => {
      await foldersStore.deleteFolder(folder.id);
    },
  });
};

const showAddToFolderMenu = (event: Event) => {
  addToFolderMenuRef.value?.toggle(event);
};

const addSelectedToFolder = async (folderId: string) => {
  const imageIds = Array.from(galleryStore.selectedImages);
  await galleryStore.addToFolder(imageIds, folderId);
  await foldersStore.loadFolders(); // Refresh counts
};

// Auto-tagging handlers
const openAutoTagSettings = () => {
  autoTagSettingsVisible.value = true;
};

const handleTaggingComplete = (results: { total: number; success: number }) => {
  if (results.success === results.total) {
    toast.add({
      severity: 'success',
      summary: 'Auto-Tag Complete',
      detail: `Successfully tagged ${results.success} image${results.success > 1 ? 's' : ''}`,
      life: 3000,
    });
  } else if (results.success > 0) {
    toast.add({
      severity: 'warn',
      summary: 'Auto-Tag Partial',
      detail: `Tagged ${results.success} of ${results.total} images`,
      life: 5000,
    });
  } else {
    toast.add({
      severity: 'error',
      summary: 'Auto-Tag Failed',
      detail: 'Failed to tag any images. Check settings.',
      life: 5000,
    });
  }
};
</script>

<style scoped>
@reference "tailwindcss";

.breadcrumb {
  @apply flex items-center gap-1 mb-3 text-sm;
}

.breadcrumb-item {
  @apply text-gray-200;

  &.active {
    @apply text-gray-200 font-medium;
  }

  &.clickable {
    @apply cursor-pointer;

    &:hover {
      @apply text-gray-200;
    }
  }
}

.breadcrumb-separator {
  @apply text-xs mx-1 text-gray-200;
}

.selection-count {
  @apply text-sm text-gray-200;
}

.loading-state,
.empty-state {
  @apply flex flex-col items-center justify-center flex-1 gap-4 text-gray-200;
}

.empty-hint {
  @apply text-sm text-gray-200;
}
</style>
