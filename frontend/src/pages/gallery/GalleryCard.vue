<template>
  <div
    ref="cardRef"
    class="group relative rounded-lg overflow-hidden cursor-pointer bg-slate-100 border border-slate-200 hover:border-slate-300 transition-colors"
    :class="{ 'ring-2 ring-blue-500 border-blue-500': selected }"
    @click="emit('click', image.id)">
    <div class="aspect-[3/4] w-full relative">
      <!-- Loaded image -->
      <img
        v-if="dataUrl"
        :src="dataUrl"
        :alt="image.prompt"
        class="w-full h-full object-cover" />

      <!-- Loading skeleton -->
      <div v-else class="w-full h-full animate-pulse bg-slate-200 flex items-center justify-center">
        <ImageIcon :size="24" class="text-slate-300" />
      </div>

      <!-- Gradient overlay with prompt (always visible, matching Pencil design) -->
      <div class="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/80 via-black/40 to-transparent p-2.5 pt-8">
        <p class="text-white text-[11px] leading-snug line-clamp-2">{{ image.prompt }}</p>
      </div>

      <!-- Favorite button (top-right, visible on hover or when favorited) -->
      <button
        class="absolute top-1.5 right-1.5 p-1 rounded-full bg-black/30 backdrop-blur-sm
               opacity-0 group-hover:opacity-100 transition-opacity hover:bg-black/50"
        :class="{ '!opacity-100': image.favorite }"
        @click.stop="emit('favorite', image.id)">
        <Star
          :size="14"
          class="text-white"
          :class="{ 'fill-yellow-400 text-yellow-400': image.favorite }" />
      </button>

      <!-- Selection checkbox (top-left, visible on hover or when selected) -->
      <div
        class="absolute top-1.5 left-1.5 opacity-0 group-hover:opacity-100 transition-opacity"
        :class="{ '!opacity-100': selected }">
        <button
          class="w-5 h-5 rounded border-2 flex items-center justify-center transition-colors"
          :class="selected
            ? 'bg-blue-500 border-blue-500'
            : 'bg-white/80 border-white/60 hover:border-white'"
          @click.stop="emit('select', image.id)">
          <Check v-if="selected" :size="12" class="text-white" />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { Star, Check, Image as ImageIcon } from 'lucide-vue-next';
import { getApi, mockApi } from '@/bridge';
import type { GalleryImage } from '@/types/inference';

const props = defineProps<{
  image: GalleryImage;
  selected: boolean;
}>();

const emit = defineEmits<{
  click: [imageId: string];
  favorite: [imageId: string];
  select: [imageId: string];
}>();

const cardRef = ref<HTMLElement>();
const dataUrl = ref<string | null>(null);
let observer: IntersectionObserver | null = null;

async function loadImage() {
  const api = getApi() ?? mockApi;
  const path = props.image.thumbnail_path || props.image.file_path;
  const res = await api.get_image_base64({ image_path: path });
  if (res.status === 'success' && res.data_url) {
    dataUrl.value = res.data_url;
  }
}

onMounted(() => {
  observer = new IntersectionObserver(
    ([entry]) => {
      if (entry.isIntersecting) {
        loadImage();
        observer?.disconnect();
      }
    },
    { rootMargin: '200px' },
  );
  if (cardRef.value) observer.observe(cardRef.value);
});

onUnmounted(() => observer?.disconnect());
</script>
