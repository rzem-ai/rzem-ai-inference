<template>
  <div ref="containerRef" class="flex-1 min-h-0 overflow-hidden rounded-xl">
    <!-- Empty state -->
    <div v-if="!gallery.loading && gallery.images.length === 0" class="h-full flex items-center justify-center">
      <div class="text-center">
        <ImageIcon :size="48" class="text-slate-300 mx-auto mb-3" />
        <p class="text-sm text-slate-400">No images yet</p>
        <p class="text-xs text-slate-300 mt-1">Generate images on the Create page to see them here</p>
      </div>
    </div>

    <!-- Virtualised grid -->
    <VirtualGrid
      v-else
      :items="rows"
      :item-size="rowHeight"
      @scroll="onScroll">
      <template #default="{ item: row, first }">
        <div :style="gridStyle" :class="{ 'pt-0!': first }">
          <GalleryCard
            v-for="image in row"
            :key="image.id"
            :image="image"
            :selected="selectedIds.has(image.id)"
            @click="emit('card-click', $event)"
            @favorite="emit('favorite', $event)"
            @open-detail="emit('open-detail')"
            @select="emit('select', $event)" />
        </div>
      </template>
    </VirtualGrid>

    <!-- Loading spinner -->
    <div v-if="gallery.loading" class="flex justify-center py-4">
      <div class="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, toRef } from 'vue';
import { Image as ImageIcon } from 'lucide-vue-next';
import { useGalleryStore } from '@/stores/gallery';
import { useVirtualGrid } from '@/composables/useVirtualGrid';
import VirtualGrid from '@/components/VirtualGrid.vue';
import GalleryCard from './GalleryCard.vue';

defineProps<{
  isGridMode: boolean;
  selectedIds: Set<string>;
}>();

const emit = defineEmits<{
  'card-click': [imageId: string];
  favorite: [imageId: string];
  'open-detail': [];
  select: [imageId: string];
}>();

const gallery = useGalleryStore();
const containerRef = ref<HTMLElement>();

const { rows, rowHeight, gridStyle } = useVirtualGrid(
  toRef(() => gallery.images),
  containerRef,
  { aspectRatio: 4 / 3 },
);

function onScroll(event: Event) {
  const el = event.target as HTMLElement;
  const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 300;
  if (nearBottom && gallery.hasMore && !gallery.loading) {
    gallery.loadMore();
  }
}
</script>
