<template>
  <div class="folder-node">
    <div
      class="folder-row"
      :class="{ active: activeId === folder.id, 'drop-target': isDragOver }"
      :style="{ paddingLeft: `${depth * 16 + 8}px` }"
      draggable="false"
      @click="emit('select', folder)"
      @contextmenu.prevent="(e) => emit('contextMenu', e, folder)"
      @dragover.prevent="handleDragOver"
      @dragleave="handleDragLeave"
      @drop="handleDrop">
      <!-- Expand/Collapse Toggle -->
      <button v-if="folder.children.length > 0" class="expand-btn" @click.stop="emit('toggleExpand', folder.id)">
        <ChevronDown v-if="isExpanded" class="w-4 h-4" />
        <ChevronUp v-else class="w-4 h-4" />
      </button>
      <span v-else class="expand-spacer"></span>

      <!-- Folder Icon -->
      <FolderOpen v-if="isExpanded && folder.children.length > 0" class="w-4 h-4" :style="folder.color ? { color: folder.color } : {}" />
      <Folder v-else class="w-4 h-4" :style="folder.color ? { color: folder.color } : {}" />

      <!-- Folder Name -->
      <span class="folder-name">{{ folder.name }}</span>

      <!-- Image Count Badge -->
      <span class="image-count" :title="`${folder.imageCount} direct, ${folder.totalImageCount} total`">
        {{ folder.totalImageCount }}
      </span>
    </div>

    <!-- Children (recursive) -->
    <Transition name="expand">
      <div v-if="isExpanded && folder.children.length > 0" class="folder-children">
        <FolderTreeNode
          v-for="child in folder.children"
          :key="child.id"
          :folder="child"
          :depth="depth + 1"
          :expanded-ids="expandedIds"
          :active-id="activeId"
          @select="emit('select', $event)"
          @toggle-expand="emit('toggleExpand', $event)"
          @context-menu="(e, f) => emit('contextMenu', e, f)"
          @drop="emit('drop', $event)" />
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { FolderNode } from '@/stores/folders';
import { ChevronDown, ChevronUp, Folder, FolderOpen } from 'lucide-vue-next';

interface Props {
  folder: FolderNode;
  depth: number;
  expandedIds: Set<string>;
  activeId: string | null;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  select: [folder: FolderNode];
  toggleExpand: [folderId: string];
  contextMenu: [event: MouseEvent, folder: FolderNode];
  drop: [data: { folderId: string; imageIds: string[] }];
}>();

const isDragOver = ref(false);

const isExpanded = computed(() => props.expandedIds.has(props.folder.id));

const handleDragOver = (e: DragEvent) => {
  // Check if dragging images
  if (e.dataTransfer?.types.includes('application/x-gallery-images')) {
    isDragOver.value = true;
    e.dataTransfer.dropEffect = 'copy';
  }
};

const handleDragLeave = () => {
  isDragOver.value = false;
};

const handleDrop = (e: DragEvent) => {
  isDragOver.value = false;
  const data = e.dataTransfer?.getData('application/x-gallery-images');
  if (data) {
    try {
      const imageIds = JSON.parse(data) as string[];
      emit('drop', { folderId: props.folder.id, imageIds });
    } catch {
      console.error('Failed to parse dropped image data');
    }
  }
};
</script>

<style scoped>
@reference "tailwindcss";

.folder-node {
  @apply select-none;
}

.folder-row {
  @apply flex items-center gap-1 py-1.5 pr-2 rounded-md cursor-pointer transition-colors;
  color: var(--color-slate-300);

  &:hover {
    background-color: var(--color-slate-800);
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
  @apply flex-1 text-sm truncate;
}

.image-count {
  @apply text-xs px-1.5 py-0.5 rounded-full;
  background-color: var(--color-slate-700);
  color: var(--color-slate-400);
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
