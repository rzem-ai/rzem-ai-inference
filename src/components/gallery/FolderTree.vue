<template>
  <div class="flex flex-col h-full overflow-hidden">
    <!-- Folders Header -->
    <div class="flex items-center justify-between px-3 py-2">
      <span class="text-xs font-semibold tracking-wide uppercase text-surface-500">Folders</span>
    </div>

    <!-- Loading State -->
    <div v-if="foldersStore.isLoading" class="flex flex-col items-center gap-3 p-4 text-center">
      <fa :icon="['fal', 'arrows-rotate']" size="lg" class="animate-spin text-surface-500" />
      <div class="text-sm text-surface-500">Loading folders...</div>
    </div>

    <!-- Empty State -->
    <div v-else-if="foldersStore.folders.length === 0" class="flex flex-col items-center gap-3 p-4 text-center">
      <div class="text-sm text-surface-500">No folders yet</div>
      <Button label="Create First Folder" size="small" @click="emit('createFolder')" />
    </div>

    <!-- Folder List -->
    <div v-else class="flex-1 px-2 pb-2 overflow-y-auto">
      <FolderTreeNode
        v-for="(folder, index) in foldersStore.folders"
        :key="folder.id"
        :folder="folder"
        :depth="0"
        :expanded-ids="foldersStore.expandedFolderIds"
        :active-id="foldersStore.currentFolderId"
        :sibling-ids="foldersStore.folders.map((f) => f.id)"
        :is-last-sibling="index === foldersStore.folders.length - 1"
        :dragged-folder-id="draggedFolderId"
        @select="handleSelectFolder"
        @toggle-expand="handleToggleExpand"
        @context-menu="handleContextMenu"
        @drop="handleDrop"
        @folder-move="handleFolderMove" />
    </div>

    <!-- Context Menu -->
    <ContextMenu ref="contextMenuRef" :model="contextMenuItems" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, provide, reactive } from 'vue';
import { useFoldersStore, type FolderNode } from '@/stores/folders';
import { useGalleryStore } from '@/stores/gallery';
import Button from 'primevue/button';
import ContextMenu from 'primevue/contextmenu';
import FolderTreeNode from './FolderTreeNode.vue';

const foldersStore = useFoldersStore();
const galleryStore = useGalleryStore();

// Track currently dragged folder for all descendants
const draggedFolderState = reactive({ value: null as string | null });
const draggedFolderId = computed(() => draggedFolderState.value);
provide('draggedFolderId', draggedFolderState);

const emit = defineEmits<{
  createFolder: [parentId?: string];
  editFolder: [folder: FolderNode];
  deleteFolder: [folder: FolderNode];
}>();

const contextMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null);
const contextFolder = ref<FolderNode | null>(null);

const contextMenuItems = computed(() => [
  {
    label: 'New Subfolder',
    icon: 'pi pi-plus',
    command: () => {
      if (contextFolder.value) {
        emit('createFolder', contextFolder.value.id);
      }
    },
  },
  {
    label: 'Rename',
    icon: 'pi pi-pencil',
    command: () => {
      if (contextFolder.value) {
        emit('editFolder', contextFolder.value);
      }
    },
  },
  { separator: true },
  {
    label: 'Delete',
    icon: 'pi pi-trash',
    class: 'p-menuitem-danger',
    command: () => {
      if (contextFolder.value) {
        emit('deleteFolder', contextFolder.value);
      }
    },
  },
]);

const handleSelectFolder = (folder: FolderNode) => {
  foldersStore.setCurrentFolder(folder.id);
  galleryStore.loadFolderImages(folder.id);
};

const handleToggleExpand = (folderId: string) => {
  foldersStore.toggleExpanded(folderId);
};

const handleContextMenu = (event: MouseEvent, folder: FolderNode) => {
  contextFolder.value = folder;
  contextMenuRef.value?.show(event);
};

const handleDrop = async (data: { folderId: string; imageIds: string[] }) => {
  await galleryStore.addToFolder(data.imageIds, data.folderId);
};

const handleFolderMove = async (data: { folderId: string; targetId: string; position: 'before' | 'after' | 'into' }) => {
  const { folderId, targetId, position } = data;

  if (position === 'into') {
    // Move folder to become a child of target
    await foldersStore.moveFolder(folderId, targetId);
  } else {
    // Reorder: find target's parent and siblings, calculate new order
    const targetFolder = foldersStore.flatFolders.find((f) => f.id === targetId);
    if (!targetFolder) return;

    // Get siblings at target's level
    const siblings = targetFolder.parentId ? foldersStore.flatFolders.find((f) => f.id === targetFolder.parentId)?.children || [] : foldersStore.folders;

    const siblingIds = siblings.map((s) => s.id);
    const sourceIndex = siblingIds.indexOf(folderId);

    // Remove source from current position if it's in the same parent
    if (sourceIndex !== -1) {
      siblingIds.splice(sourceIndex, 1);
    } else {
      // Moving from different parent - first move to same parent
      await foldersStore.moveFolder(folderId, targetFolder.parentId);
    }

    // Calculate new index after potential removal
    const targetIndex = siblingIds.indexOf(targetId);
    const insertIndex = position === 'before' ? targetIndex : targetIndex + 1;
    siblingIds.splice(insertIndex, 0, folderId);

    // Apply new order
    await foldersStore.reorderFolders(siblingIds);
  }
};
</script>
