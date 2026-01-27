<template>
  <div class="flex w-full">
    <!-- Sidebar -->
    <WorkspaceActions>
      <template #toolbar>
        <div class="grid w-full grid-cols-3 border rounded-md shadow-xs border-surface-600">
          <Button
            :severity="foldersStore.currentViewType === 'all' ? 'primary' : 'secondary'"
            :variant="foldersStore.currentViewType === 'all' ? '' : ''"
            @click="handleSelectAll"
            size="small"
            class="border-0 rounded-none rounded-l-md">
            <div class="content-center">
              <fa :icon="['fal', 'images']" size="sm" />
              All Images
            </div>
          </Button>
          <Button
            :severity="foldersStore.currentViewType === 'uncategorized' ? 'primary' : 'secondary'"
            :variant="foldersStore.currentViewType === 'uncategorized' ? '' : ''"
            @click="handleSelectUncategorized"
            size="small"
            class="border-0 rounded-none">
            <div class="content-center">
              <fa :icon="['fal', 'folder']" size="sm" />
              Uncategorized
            </div>
          </Button>
          <Button @click="openCreateFolderDialog()" severity="secondary" size="small" class="border-0 rounded-none rounded-r-md">
            <div class="content-center">
              <fa :icon="['fal', 'plus']" size="sm" />
              Create Folder
            </div>
          </Button>
        </div>
      </template>
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
      <div class="p-4">
        <!-- Breadcrumb -->
        <div class="flex items-center gap-1 mb-3 text-sm">
          <template v-if="foldersStore.currentViewType === 'all'">
            <span class="text-surface-200 active">All Images</span>
          </template>
          <template v-else-if="foldersStore.currentViewType === 'uncategorized'">
            <span class="text-surface-200 active">Uncategorized</span>
          </template>
          <template v-else-if="foldersStore.currentFolder">
            <span class="text-surface-200 clickable" @click="handleSelectAll">All Images</span>

            <fa :icon="['fal', 'chevron-right']" size="lg" class="mx-1 text-xs text-surface-200" />
            <template v-for="(name, index) in foldersStore.currentBreadcrumb" :key="index">
              <span v-if="index < foldersStore.currentBreadcrumb.length - 1" class="text-surface-200 clickable" @click="navigateToBreadcrumb(index)">
                {{ name }}
              </span>
              <span v-else class="text-surface-200 active">{{ name }}</span>
              <fa v-if="index < foldersStore.currentBreadcrumb.length - 1" :icon="['fal', 'chevron-right']" size="lg" class="mx-1 text-xs text-surface-200" />
            </template>
          </template>
        </div>

        <div class="flex items-center gap-4">
          <div class="flex gap-2 grow">
            <InputText v-model="galleryStore.filters.searchQuery" placeholder="Search prompts..." @keyup.enter="handleSearch" size="small" fluid />
            <Button @click="handleSearch" :loading="galleryStore.isLoading"><fa :icon="['fal', 'magnifying-glass']" size="sm" /></Button>
          </div>

          <div class="flex items-center gap-2 ml-auto">
            <Button :disabled="galleryStore.selectedImages.size == 0" size="small" severity="secondary" @click="showAddToFolderMenu">
              <div class="flex items-center gap-2"><fa :icon="['fal', 'folder']" size="sm" /> Add to Folder</div>
            </Button>
            <Button :disabled="galleryStore.selectedImages.size == 0" size="small" severity="secondary" @click="showBulkTagMenu">
              <div class="flex items-center gap-2"><fa :icon="['fal', 'tag']" size="sm" /> Manage Tags</div>
            </Button>
            <Button :disabled="galleryStore.selectedImages.size == 0" size="small" severity="danger" @click="confirmDeleteSelected">
              <div class="flex items-center gap-2"><fa :icon="['fal', 'trash']" size="sm" /> Delete</div>
            </Button>
            <AutoTagButton @open-settings="openAutoTagSettings" @tagging-complete="handleTaggingComplete" />
            <Button label="Select All" severity="secondary" @click="galleryStore.selectAll"><fa :icon="['fal', 'square-check']" size="sm" /></Button>
            <Button label="Clear" severity="secondary" @click="galleryStore.clearSelection" :disabled="galleryStore.selectedImages.size === 0">
              <fa :icon="['fal', 'xmark']" size="sm" />
            </Button>
            <span class="text-sm text-surface-200"> {{ galleryStore.selectedImages.size }} selected </span>
          </div>
        </div>
      </div>

      <!-- Loading State:  -->
      <div v-if="galleryStore.isLoading" class="flex flex-col items-center justify-center flex-1 gap-4 text-surface-200">
        <fa :icon="['fal', 'arrows-rotate']" size="lg" :class="{ 'animate-spin': galleryStore.isLoading }" />
        <p>Loading images...</p>
      </div>

      <!-- Empty State -->
      <div v-else-if="galleryStore.filteredImages.length === 0" class="flex flex-col items-center justify-center flex-1 gap-4 text-surface-200">
        <fa :icon="['fal', 'image']" size="lg" />
        <p>No images found</p>
        <p class="text-sm text-surface-200">
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
        @add-to-compare="handleAddToCompare"
        @delete="handleDeleteImage" />
    </main>

    <!-- Folder Form Dialog -->
    <FolderForm v-model:visible="folderFormVisible" :folder="editingFolder" :default-parent-id="defaultParentId" />

    <!-- Add to Folder Menu -->
    <Menu ref="addToFolderMenuRef" :model="addToFolderMenuItems" popup />

    <!-- Bulk Tag Menu -->
    <Menu ref="bulkTagMenuRef" :model="bulkTagMenuItems" popup />

    <!-- Image Detail Modal -->
    <ImageDetailModal v-model:visible="imageDetailVisible" v-model:image="selectedImage" />

    <!-- Delete Confirmation -->
    <ConfirmDialog />

    <!-- Auto-Tag Settings Dialog -->
    <AutoTagSettings v-model:visible="autoTagSettingsVisible" />

    <!-- Toast for notifications -->
    <Toast />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
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
import ImageDetailModal from '@/components/gallery/ImageDetailModal.vue';
import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import Menu from 'primevue/menu';
import ConfirmDialog from 'primevue/confirmdialog';
import Toast from 'primevue/toast';
import WorkspaceActions from '@/components/shared/WorkspaceActions.vue';
import type { GalleryImage } from '@/stores/gallery';

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

// Bulk tag menu
const bulkTagMenuRef = ref<InstanceType<typeof Menu> | null>(null);

// Image detail modal
const imageDetailVisible = ref(false);
const selectedImage = ref<GalleryImage | null>(null);

// Sync selectedImage changes back to gallery store (for tag/folder edits in modal)
watch(
  selectedImage,
  (newImage) => {
    if (newImage) {
      galleryStore.updateImage(newImage);
    }
  },
  { deep: true },
);

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

const bulkTagMenuItems = computed(() => {
  const popularTags = tagsStore.popularTags.slice(0, 8);

  const addTagItems = popularTags.map((tag) => ({
    label: tag.name,
    icon: 'pi pi-plus',
    command: () => bulkAddTag(tag.name),
  }));

  const removeTagItems = popularTags.map((tag) => ({
    label: tag.name,
    icon: 'pi pi-minus',
    command: () => bulkRemoveTag(tag.name),
  }));

  return [
    {
      label: 'Add Tag',
      icon: 'pi pi-plus-circle',
      items: addTagItems.length > 0 ? addTagItems : [{ label: 'No tags available', disabled: true }],
    },
    {
      label: 'Remove Tag',
      icon: 'pi pi-minus-circle',
      items: removeTagItems.length > 0 ? removeTagItems : [{ label: 'No tags available', disabled: true }],
    },
  ];
});

onMounted(async () => {
  await Promise.all([galleryStore.loadImages(), foldersStore.loadFolders(), tagsStore.loadTags(), autoTagStore.loadSettings()]);
  // Check model status in background (don't block initial load)
  autoTagStore.checkModelStatus();
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

const handleOpenDetail = (image: GalleryImage) => {
  selectedImage.value = image;
  imageDetailVisible.value = true;
};

const handleAddToCompare = (image: any) => {
  const success = compareStore.addToCompare(image);
  if (!success) {
    console.warn('Cannot add more images to compare');
  }
};

const handleDeleteImage = (imageId: string) => {
  confirm.require({
    message: 'Are you sure you want to delete this image? This action cannot be undone.',
    header: 'Delete Image',
    icon: 'pi pi-exclamation-triangle',
    rejectClass: 'p-button-secondary',
    acceptClass: 'p-button-danger',
    accept: () => {
      galleryStore.deleteImage(imageId).then((success) => {
        if (success) {
          toast.add({
            severity: 'success',
            summary: 'Image Deleted',
            detail: 'Image has been deleted',
            life: 3000,
          });
        } else {
          toast.add({
            severity: 'error',
            summary: 'Deletion Failed',
            detail: 'Failed to delete image',
            life: 5000,
          });
        }
      });
    },
  });
};

const handleSelectAll = () => {
  foldersStore.setViewType('all');
  galleryStore.loadAllImages();
};

const handleSelectUncategorized = () => {
  foldersStore.setViewType('uncategorized');
  galleryStore.loadUncategorizedImages();
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

const confirmDeleteSelected = () => {
  const count = galleryStore.selectedImages.size;
  confirm.require({
    message: `Are you sure you want to delete ${count} image${count > 1 ? 's' : ''}? This action cannot be undone.`,
    header: 'Delete Images',
    icon: 'pi pi-exclamation-triangle',
    rejectClass: 'p-button-secondary',
    acceptClass: 'p-button-danger',
    accept: () => {
      galleryStore.deleteSelectedImages().then((result) => {
        if (result.failed === 0) {
          toast.add({
            severity: 'success',
            summary: 'Images Deleted',
            detail: `Successfully deleted ${result.success} image${result.success > 1 ? 's' : ''}`,
            life: 3000,
          });
        } else if (result.success > 0) {
          toast.add({
            severity: 'warn',
            summary: 'Partial Deletion',
            detail: `Deleted ${result.success} image${result.success > 1 ? 's' : ''}, ${result.failed} failed`,
            life: 5000,
          });
        } else {
          toast.add({
            severity: 'error',
            summary: 'Deletion Failed',
            detail: 'Failed to delete images',
            life: 5000,
          });
        }
      });
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

// Bulk tag management
const showBulkTagMenu = (event: Event) => {
  bulkTagMenuRef.value?.toggle(event);
};

const bulkAddTag = async (tagName: string) => {
  const imageIds = Array.from(galleryStore.selectedImages);
  const success = await tagsStore.bulkAddTag(imageIds, tagName);
  if (success) {
    toast.add({
      severity: 'success',
      summary: 'Tags Added',
      detail: `Added "${tagName}" to ${imageIds.length} image${imageIds.length > 1 ? 's' : ''}`,
      life: 3000,
    });
    await galleryStore.loadImages(); // Refresh to show new tags
  }
};

const bulkRemoveTag = async (tagName: string) => {
  const imageIds = Array.from(galleryStore.selectedImages);
  const success = await tagsStore.bulkRemoveTag(imageIds, tagName);
  if (success) {
    toast.add({
      severity: 'success',
      summary: 'Tags Removed',
      detail: `Removed "${tagName}" from ${imageIds.length} image${imageIds.length > 1 ? 's' : ''}`,
      life: 3000,
    });
    await galleryStore.loadImages(); // Refresh
  }
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
</style>
