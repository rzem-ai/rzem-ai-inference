<template>
  <Popover ref="popoverRef">
    <div class="flex flex-col gap-1 min-w-48 max-h-64 overflow-y-auto p-1">
      <div class="px-2 py-1 text-xs font-semibold text-slate-400 uppercase tracking-wide">Add tags</div>

      <div class="flex flex-wrap gap-1 px-2 py-1">
        <button
          v-for="tag in gallery.tags"
          :key="tag.id"
          class="px-2 py-1 rounded-full font-medium transition-all text-xs"
          :style="tagStyle(tag)"
          @click="onToggleTag(tag)">
          {{ tag.name }}
        </button>
      </div>

      <div v-if="gallery.tags.length === 0" class="px-3 py-2 text-sm text-slate-400">
        No tags yet
      </div>

      <Divider />

      <button
        class="flex items-center gap-2 px-3 py-1 rounded-lg text-left w-full transition-colors hover:bg-blue-50 text-blue-600"
        @click="onNewTag">
        <Plus :size="14" class="shrink-0" />
        <span class="font-medium">New Tag</span>
      </button>
    </div>
  </Popover>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useGalleryStore } from '@/stores/gallery';
import type { Tag } from '@/types/inference';

const props = defineProps<{
  imageIds: string[];
}>();

const emit = defineEmits<{
  tagged: [];
}>();

const gallery = useGalleryStore();
const popoverRef = ref();

function tagStyle(tag: Tag) {
  const color = tag.color || '#64748b';
  return {
    color,
    backgroundColor: color + '15',
  };
}

async function onToggleTag(tag: Tag) {
  for (const imageId of props.imageIds) {
    await gallery.addTagToImage(imageId, tag.id);
  }
  emit('tagged');
}

function onNewTag() {
  const name = prompt('Tag name:');
  if (name?.trim()) {
    gallery.createTag(name.trim());
  }
}

function toggle(event: Event) {
  popoverRef.value?.toggle(event);
}

defineExpose({ toggle });
</script>
