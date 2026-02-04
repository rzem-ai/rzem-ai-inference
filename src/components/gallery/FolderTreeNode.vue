<template>
  <div class="folder-node" :class="{ 'is-dragging': isDragging }">
    <!-- Drop zone above -->
    <div
      v-if="showDropZones"
      class="drop-zone drop-zone-above"
      :class="{ active: dropPosition === 'before' }"
      @dragover.prevent="handleFolderDragOver($event, 'before')"
      @dragleave="handleFolderDragLeave"
      @drop="handleFolderDrop($event, 'before')"></div>

    <div
      class="flex items-center gap-1 py-1.5 pr-2 rounded-md cursor-pointer transition-colors text-surface-300 hover:bg-surface-700"
      :class="{
        active: activeId === folder.id,
        'drop-target': isImageDragOver,
        'drop-target-folder': dropPosition === 'into',
      }"
      :style="{ paddingLeft: `${depth * 16 + 8}px` }"
      draggable="true"
      @dragstart="handleDragStart"
      @dragend="handleDragEnd"
      @click="emit('select', folder)"
      @contextmenu.prevent="(e) => emit('contextMenu', e, folder)"
      @dragover.prevent="handleDragOver"
      @dragleave="handleDragLeave"
      @drop="handleDrop">


      <!-- Folder Icon -->
      <fa v-if="isExpanded && folder.children.length > 0 && activeId === folder.id" :icon="['fas', 'folder-open']" size="lg" :style="folder.color ? { color: folder.color } : {}" @click.stop="emit('toggleExpand', folder.id)"/>
      <fa v-else-if="isExpanded && folder.children.length > 0" :icon="['fas', 'folder-arrow-down']" size="lg" :style="folder.color ? { color: folder.color } : {}" @click.stop="emit('toggleExpand', folder.id)"/>
      <fa v-else-if="folder.children.length > 0" :icon="['fas', 'folder-plus']" size="lg" :style="folder.color ? { color: folder.color } : {}" @click.stop="emit('toggleExpand', folder.id)"/>
      <fa v-else-if="activeId === folder.id" :icon="['fas', 'folder-open']" size="lg" :style="folder.color ? { color: folder.color } : {}"/>
      <fa v-else :icon="['fas', 'folder']" size="lg" :style="folder.color ? { color: folder.color } : {}" />

      <!-- Folder Name -->
      <span class="flex-1 text-sm truncate">{{ folder.name }}</span>

      <!-- Image Count Badge -->
      <span class="text-xs px-1.5 py-0.5 rounded-full bg-surface-600 text-surface-900" :title="`${folder.imageCount} direct, ${folder.totalImageCount} total`">
        {{ folder.totalImageCount }}
      </span>
    </div>

    <!-- Drop zone below (only for last sibling or when not expanded) -->
    <div
      v-if="showDropZones && isLastSibling"
      class="drop-zone drop-zone-below"
      :class="{ active: dropPosition === 'after' }"
      @dragover.prevent="handleFolderDragOver($event, 'after')"
      @dragleave="handleFolderDragLeave"
      @drop="handleFolderDrop($event, 'after')"></div>

    <!-- Children (recursive) -->
    <Transition name="expand">
      <div v-if="isExpanded && folder.children.length > 0" class="folder-children">
        <FolderTreeNode
          v-for="(child, index) in folder.children"
          :key="child.id"
          :folder="child"
          :depth="depth + 1"
          :expanded-ids="expandedIds"
          :active-id="activeId"
          :sibling-ids="folder.children.map((c) => c.id)"
          :is-last-sibling="index === folder.children.length - 1"
          :dragged-folder-id="draggedFolderId"
          @select="emit('select', $event)"
          @toggle-expand="emit('toggleExpand', $event)"
          @context-menu="(e, f) => emit('contextMenu', e, f)"
          @drop="emit('drop', $event)"
          @folder-move="emit('folderMove', $event)" />
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject } from 'vue';
import type { FolderNode } from '@/stores/folders';
import Button from 'primevue/button';

interface Props {
  folder: FolderNode;
  depth: number;
  expandedIds: Set<string>;
  activeId: string | null;
  siblingIds?: string[];
  isLastSibling?: boolean;
  draggedFolderId?: string | null;
}

const props = withDefaults(defineProps<Props>(), {
  siblingIds: () => [],
  isLastSibling: false,
  draggedFolderId: null,
});

const emit = defineEmits<{
  select: [folder: FolderNode];
  toggleExpand: [folderId: string];
  contextMenu: [event: MouseEvent, folder: FolderNode];
  drop: [data: { folderId: string; imageIds: string[] }];
  folderMove: [data: { folderId: string; targetId: string; position: 'before' | 'after' | 'into' }];
}>();

// Local drag state
const isDragging = ref(false);
const isImageDragOver = ref(false);
const dropPosition = ref<'before' | 'after' | 'into' | null>(null);

// Track globally dragged folder via provide/inject from parent
const globalDraggedFolderId = inject<{ value: string | null }>('draggedFolderId', { value: null });

const isExpanded = computed(() => props.expandedIds.has(props.folder.id));

// Show drop zones when a folder is being dragged (not this one)
const showDropZones = computed(() => {
  const draggedId = globalDraggedFolderId.value || props.draggedFolderId;
  return draggedId && draggedId !== props.folder.id;
});

// Drag start - set data and visual feedback
const handleDragStart = (e: DragEvent) => {
  if (!e.dataTransfer) return;

  isDragging.value = true;
  globalDraggedFolderId.value = props.folder.id;

  e.dataTransfer.effectAllowed = 'move';
  e.dataTransfer.setData(
    'application/x-folder',
    JSON.stringify({
      id: props.folder.id,
      parentId: props.folder.parentId,
    }),
  );

  // Set drag image
  const target = e.target as HTMLElement;
  e.dataTransfer.setDragImage(target, 20, 20);
};

const handleDragEnd = () => {
  isDragging.value = false;
  globalDraggedFolderId.value = null;
  dropPosition.value = null;
};

// Handle both image and folder drag over on the main row
const handleDragOver = (e: DragEvent) => {
  if (!e.dataTransfer) return;

  // Image drag
  if (e.dataTransfer.types.includes('application/x-gallery-images')) {
    isImageDragOver.value = true;
    e.dataTransfer.dropEffect = 'copy';
    return;
  }

  // Folder drag - drop "into" this folder (make it a child)
  if (e.dataTransfer.types.includes('application/x-folder')) {
    const draggedId = globalDraggedFolderId.value;
    // Don't allow dropping onto self or descendant
    if (draggedId && draggedId !== props.folder.id && !isDescendant(draggedId)) {
      dropPosition.value = 'into';
      e.dataTransfer.dropEffect = 'move';
    }
  }
};

const handleDragLeave = () => {
  isImageDragOver.value = false;
  dropPosition.value = null;
};

// Handle drop on main row (images or folder-into)
const handleDrop = (e: DragEvent) => {
  isImageDragOver.value = false;
  dropPosition.value = null;

  if (!e.dataTransfer) return;

  // Image drop
  const imageData = e.dataTransfer.getData('application/x-gallery-images');
  if (imageData) {
    try {
      const imageIds = JSON.parse(imageData) as string[];
      emit('drop', { folderId: props.folder.id, imageIds });
    } catch {
      console.error('Failed to parse dropped image data');
    }
    return;
  }

  // Folder drop (into)
  const folderData = e.dataTransfer.getData('application/x-folder');
  if (folderData) {
    try {
      const { id: folderId } = JSON.parse(folderData);
      if (folderId !== props.folder.id) {
        emit('folderMove', {
          folderId,
          targetId: props.folder.id,
          position: 'into',
        });
      }
    } catch {
      console.error('Failed to parse dropped folder data');
    }
  }
};

// Handle folder drag over drop zones (before/after)
const handleFolderDragOver = (e: DragEvent, position: 'before' | 'after') => {
  if (!e.dataTransfer?.types.includes('application/x-folder')) return;

  const draggedId = globalDraggedFolderId.value;
  if (draggedId && draggedId !== props.folder.id) {
    dropPosition.value = position;
    e.dataTransfer.dropEffect = 'move';
  }
};

const handleFolderDragLeave = () => {
  dropPosition.value = null;
};

// Handle folder drop on drop zones (reorder)
const handleFolderDrop = (e: DragEvent, position: 'before' | 'after') => {
  dropPosition.value = null;

  const folderData = e.dataTransfer?.getData('application/x-folder');
  if (!folderData) return;

  try {
    const { id: folderId } = JSON.parse(folderData);
    if (folderId !== props.folder.id) {
      emit('folderMove', {
        folderId,
        targetId: props.folder.id,
        position,
      });
    }
  } catch {
    console.error('Failed to parse dropped folder data');
  }
};

// Check if a folder is a descendant of the current folder
const isDescendant = (folderId: string): boolean => {
  const checkChildren = (children: FolderNode[]): boolean => {
    for (const child of children) {
      if (child.id === folderId) return true;
      if (child.children.length > 0 && checkChildren(child.children)) return true;
    }
    return false;
  };
  return checkChildren(props.folder.children);
};
</script>

<style scoped>
@reference "tailwindcss";

.folder-node {
  @apply select-none relative;

  &.is-dragging {
    opacity: 0.5;
  }
}

.folder-row {
  

  &:hover {
    background-color: var(--color-gray-800);
  }

  &.active {
    background-color: var(--color-blue-900);
    color: var(--color-blue-200);
  }

  &.drop-target {
    background-color: var(--color-green-900);
    outline: 2px dashed var(--color-green-500);
    outline-offset: -2px;
  }

  &.drop-target-folder {
    background-color: var(--color-purple-900);
    outline: 2px dashed var(--color-purple-500);
    outline-offset: -2px;
  }
}

/* Drop zones for folder reordering */
.drop-zone {
  @apply h-1 mx-2 rounded-full transition-all;
  background-color: transparent;

  &.active {
    @apply h-2;
    background-color: var(--color-purple-500);
  }
}

.drop-zone-above {
  @apply -mt-0.5 mb-0.5;
}

.drop-zone-below {
  @apply mt-0.5 -mb-0.5;
}

.expand-btn {
  @apply flex items-center justify-center w-5 h-5 rounded hover:bg-white/10;
  background: transparent;
  border: none;
  color: inherit;
  cursor: pointer;

  i {
    @apply text-xs;
  }
}

.expand-spacer {
  @apply w-5;
}

.folder-icon {
  @apply text-sm;
  color: var(--color-amber-400);
}

.folder-name {
 
}

.image-count {
 
}

.folder-children {
  @apply overflow-hidden;
}

/* Expand/Collapse Animation */
.expand-enter-active,
.expand-leave-active {
  transition: all 0.15s ease-out;
}

.expand-enter-from,
.expand-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
