<template>
  <div class="flex-1 min-h-0 overflow-y-auto rounded-xl" ref="scrollRef">
    <!-- Empty state -->
    <div v-if="!gallery.loading && gallery.images.length === 0" class="h-full flex items-center justify-center">
      <div class="text-center">
        <ImageIcon :size="48" class="text-slate-300 mx-auto mb-3" />
        <p class="text-sm text-slate-400">No images yet</p>
        <p class="text-xs text-slate-300 mt-1">Generate images on the Create page to see them here</p>
      </div>
    </div>

    <!-- Grid view -->
    <div v-else class="grid gap-3 p-1" :class="isGridMode ? 'grid-cols-6' : 'grid-cols-1'">
      <GalleryCard
        v-for="image in gallery.images"
        :key="image.id"
        :image="image"
        :selected="selectedIds.has(image.id)"
        @click="emit('card-click', $event)"
        @favorite="emit('favorite', $event)"
        @open-detail="emit('open-detail')"
        @select="emit('select', $event)" />
    </div>

    <!-- Infinite scroll trigger -->
    <div ref="loadMoreRef" class="h-10" />

    <!-- Loading spinner -->
    <div v-if="gallery.loading" class="flex justify-center py-4">
      <div class="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { Image as ImageIcon } from 'lucide-vue-next';
import { useGalleryStore } from '@/stores/gallery';
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

const scrollRef = ref<HTMLElement>();
const loadMoreRef = ref<HTMLElement>();
let loadMoreObserver: IntersectionObserver | null = null;

onMounted(() => {
  loadMoreObserver = new IntersectionObserver(
    ([entry]) => {
      if (entry.isIntersecting && gallery.hasMore && !gallery.loading) {
        gallery.loadMore();
      }
    },
    { root: scrollRef.value, rootMargin: '200px' },
  );
  if (loadMoreRef.value) loadMoreObserver.observe(loadMoreRef.value);
});

onUnmounted(() => loadMoreObserver?.disconnect());
</script>
