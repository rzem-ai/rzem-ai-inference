<template>
  <div class="flex flex-col h-full overflow-hidden">
    <!-- Special Views -->
    <div class="flex gap-2 px-2">
      <Button
        class="w-full"
        size="small"
        :severity="foldersStore.currentViewType === 'all' ? 'primary' : 'primary'"
        :variant="foldersStore.currentViewType === 'all' ? '' : 'outlined'"
        @click="handleSelectAll">
        <div class="flex items-center gap-2 text-sm font-medium truncate"><Images :size="14" /> All Images</div>
      </Button>
      <Button
        class="w-full"
        size="small"
        :severity="foldersStore.currentViewType === 'uncategorized' ? 'primary' : 'primary'"
        :variant="foldersStore.currentViewType === 'uncategorized' ? '' : 'outlined'"
        @click="handleSelectUncategorized">
        <div class="flex items-center gap-2 text-sm font-medium truncate"><Inbox :size="14" /> Uncategorized</div>
      </Button>
      <Button size="small" @click="emit('createFolder')" title="Create folder" class="w-20 p-2" variant="outlined" severity="secondary">
        <Plus class="w-4 h-4"/>
      </Button>
      <Button v-if="foldersStore.folders.length > 0" size="small" @click="toggleExpandAll" :title="allExpanded ? 'Collapse all' : 'Expand all'" class="w-20 p-2" variant="outlined" severity="secondary">
        <ChevronUp v-if="allExpanded" class="w-4 h-4" /><ChevronsDown v-if="!allExpanded" class="w-4 h-4" />
      </Button>
    </div>

    <!-- Folders Header -->
    <div class="flex items-center justify-between px-3 py-2">
      <span class="text-xs font-semibold tracking-wide uppercase text-slate-500">Folders</span>
      
    </div>

    <!-- Folder Tree -->
    <div v-if="foldersStore.folders.length === 0" class="flex flex-col items-center gap-3 p-4 text-center">
      <div class="text-sm text-slate-500">No folders yet</div>
      <Button label="Create First Folder" size="small" @click="emit('createFolder')" />
    </div>

    <div v-else class="flex-1 px-2 pb-2 overflow-y-auto">
      <FolderTreeNode
        v-for="folder in foldersStore.folders"
        :key="folder.id"
        :folder="folder"
        :depth="0"
        :expanded-ids="foldersStore.expandedFolderIds"
        :active-id="foldersStore.currentFolderId"
        @select="handleSelectFolder"
        @toggle-expand="handleToggleExpand"
        @context-menu="handleContextMenu"
        @drop="handleDrop" />
    </div>

    <!-- Context Menu -->
    <ContextMenu ref="contextMenuRef" :model="contextMenuItems" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useFoldersStore, type FolderNode } from '@/stores/folders';
import { useGalleryStore } from '@/stores/gallery';
import { Images, Inbox, Plus, ChevronsDown, ChevronUp } from 'lucide-vue-next';
import Button from 'primevue/button';
import ContextMenu from 'primevue/contextmenu';
import FolderTreeNode from './FolderTreeNode.vue';

const foldersStore = useFoldersStore();
const galleryStore = useGalleryStore();

const emit = defineEmits<{
  createFolder: [parentId?: string];
  editFolder: [folder: FolderNode];
  deleteFolder: [folder: FolderNode];
}>();

const contextMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null);
const contextFolder = ref<FolderNode | null>(null);

const allExpanded = computed(() => {
  const expandable = foldersStore.flatFolders.filter((f) => f.children.length > 0);
  return expandable.length > 0 && expandable.every((f) => foldersStore.expandedFolderIds.has(f.id));
});

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

const handleSelectAll = () => {
  foldersStore.setViewType('all');
  galleryStore.loadAllImages();
};

const handleSelectUncategorized = () => {
  foldersStore.setViewType('uncategorized');
  galleryStore.loadUncategorizedImages();
};

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

const toggleExpandAll = () => {
  if (allExpanded.value) {
    foldersStore.collapseAll();
  } else {
    foldersStore.expandAll();
  }
};
</script>
