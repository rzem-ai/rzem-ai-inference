<template>
  <div
    class="group relative rounded-lg overflow-hidden cursor-pointer bg-slate-100 border border-slate-200 hover:border-slate-300 transition-colors"
    :class="{ 'ring-2 ring-blue-500 border-blue-500': selected }"
    @click="emit('click', image.id)"
    @contextmenu.prevent="emit('contextmenu', $event, image.id)">
    <div class="aspect-3/4 w-full relative">
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
      <div @click.stop="emit('openDetail')" class="absolute inset-x-0 bottom-0 bg-linear-to-t from-black/80 via-black/40 to-transparent p-2 pt-8">
        <p class="text-white text-sm leading-snug line-clamp-2">{{ image.raw_prompt || image.prompt }}</p>
      </div>

      <!-- Favorite button (top-right, visible on hover or when favorited) -->
      <Button
        class="absolute top-1 right-1 p-1 rounded-full bg-black/30 backdrop-blur-sm
               opacity-0 group-hover:opacity-100 transition-opacity hover:bg-black/50"
        :class="{ 'opacity-100!': image.favorite }"
        @click.stop="emit('favorite', image.id)">
        <Star
          :size="14"
          class="text-white"
          :class="{ 'fill-yellow-400 text-yellow-400': image.favorite }" />
      </Button>

      <!-- Selection checkbox (top-left, visible on hover or when selected) -->
      <div
        class="absolute top-1 left-1 opacity-0 group-hover:opacity-100 transition-opacity"
        :class="{ 'opacity-100!': selected }">
        <Button
          class="w-5 h-5 rounded border-2 flex items-center justify-center transition-colors"
          :class="selected
            ? 'bg-blue-500 border-blue-500'
            : 'bg-white/80 border-white/60 hover:border-white'"
          @click.stop="emit('select', image.id)">
          <Check v-if="selected" :size="12" class="text-white" />
        </Button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { getApiAsync } from '@/bridge';
import type { GalleryImage } from '@/types/inference';

const props = defineProps<{
  image: GalleryImage;
  selected: boolean;
}>();

const emit = defineEmits<{
  click: [imageId: string];
  favorite: [imageId: string];
  openDetail: [];
  select: [imageId: string];
  contextmenu: [event: MouseEvent, imageId: string];
}>();

const dataUrl = ref<string | null>(null);

async function loadImage() {
  try {
    const api = await getApiAsync();
    const path = props.image.thumbnail_path || props.image.file_path;
    const res = await api.get_image_base64({ image_path: path });
    if (res.status === 'success' && res.data_url) {
      dataUrl.value = res.data_url;
    }
  } catch (err) {
    console.error('[GalleryCard] loadImage failed:', err);
  }
}

// VirtualGrid already handles virtualization — load immediately on mount
onMounted(() => loadImage());
</script>
