<template>
  <Popover ref="popoverRef">
    <div class="flex flex-col gap-1 min-w-48 max-h-64 overflow-y-auto p-1">
      <div class="px-2 py-1 text-xs font-semibold text-slate-400 uppercase tracking-wide">Move to folder</div>

      <button
        v-for="folder in gallery.folders"
        :key="folder.id"
        class="flex items-center gap-2 px-3 py-1 rounded-lg text-left w-full transition-colors hover:bg-slate-50 text-slate-700"
        @click="onSelectFolder(folder.id)">
        <FolderIcon :size="14" class="text-slate-400 shrink-0" />
        <span class="font-medium truncate">{{ folder.name }}</span>
      </button>

      <div v-if="gallery.folders.length === 0" class="px-3 py-2 text-sm text-slate-400">
        No folders yet
      </div>

      <Divider />

      <button
        class="flex items-center gap-2 px-3 py-1 rounded-lg text-left w-full transition-colors hover:bg-blue-50 text-blue-600"
        @click="onNewFolder">
        <Plus :size="14" class="shrink-0" />
        <span class="font-medium">New Folder</span>
      </button>
    </div>
  </Popover>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useGalleryStore } from '@/stores/gallery';

const props = defineProps<{
  imageIds: string[];
}>();

const emit = defineEmits<{
  moved: [];
}>();

const gallery = useGalleryStore();
const popoverRef = ref();

async function onSelectFolder(folderId: string) {
  for (const imageId of props.imageIds) {
    await gallery.addImageToFolder(imageId, folderId);
  }
  popoverRef.value?.hide();
  emit('moved');
}

function onNewFolder() {
  const name = prompt('Folder name:');
  if (name?.trim()) {
    const id = crypto.randomUUID();
    gallery.createFolder(id, name.trim());
  }
}

function toggle(event: Event) {
  popoverRef.value?.toggle(event);
}

defineExpose({ toggle });
</script>
