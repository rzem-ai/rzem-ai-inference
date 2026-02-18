<template>
  <Teleport to="body">
    <Transition name="overlay-fade">
      <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center" @keydown.escape="close" tabindex="0" ref="overlayRef">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/90" @click="close" />

        <!-- Close button -->
        <button
          class="absolute top-4 right-4 z-10 p-2 rounded-full bg-white/10 backdrop-blur-sm text-white/70 hover:text-white hover:bg-white/20 transition-colors"
          @click="close">
          <X :size="20" />
        </button>

        <!-- Download button -->
        <button
          class="absolute top-4 right-28 z-10 p-2 rounded-full bg-white/10 backdrop-blur-sm text-white/70 hover:text-white hover:bg-white/20 transition-colors"
          @click="onDownload">
          <Download :size="20" />
        </button>

        <!-- Favorite button -->
        <button
          class="absolute top-4 right-16 z-10 p-2 rounded-full bg-white/10 backdrop-blur-sm text-white/70 hover:text-white hover:bg-white/20 transition-colors"
          @click="emit('favorite', image.id)">
          <Star :size="20" :class="image.favorite ? 'fill-yellow-400 text-yellow-400' : ''" />
        </button>

        <!-- Image container -->
        <div class="relative max-w-[90vw] max-h-[90vh] z-1">
          <!-- Image -->
          <img v-if="dataUrl" :src="dataUrl" :alt="image.prompt" class="max-w-[90vw] max-h-[90vh] object-contain rounded-lg select-none" draggable="false" />

          <!-- Loading state -->
          <div v-else class="w-64 h-64 flex items-center justify-center">
            <div class="w-8 h-8 border-2 border-white/40 border-t-white rounded-full animate-spin" />
          </div>

          <!-- Prompt bar (lower third) -->
          <div
            v-if="dataUrl"
            class="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/90 via-black/50 to-transparent rounded-b-lg cursor-pointer transition-colors hover:from-black/95"
            @click.stop="emit('openDetail')">
            <div class="px-5 pb-4 pt-12">
              <p class="text-white/90 text-sm leading-relaxed line-clamp-2">{{ image.prompt }}</p>
              <div class="flex items-center gap-3 mt-1.5 text-white/50 text-xs">
                <span>{{ image.width }} x {{ image.height }}</span>
                <span>{{ image.steps }} steps</span>
                <span>seed {{ image.seed }}</span>
                <span class="ml-auto text-white/40 flex items-center gap-1">
                  <Info :size="12" />
                  Click for details
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';
import { X, Star, Info, Download } from 'lucide-vue-next';
import { getApi, mockApi } from '@/bridge';
import type { GalleryImage } from '@/types/inference';

const props = defineProps<{
  visible: boolean;
  image: GalleryImage;
}>();

const emit = defineEmits<{
  close: [];
  openDetail: [];
  favorite: [imageId: string];
}>();

const overlayRef = ref<HTMLElement>();
const dataUrl = ref<string | null>(null);

function close() {
  emit('close');
}

async function onDownload() {
  const api = getApi() ?? mockApi;
  await api.save_image_as({ file_path: props.image.file_path });
}

async function loadFullImage() {
  const api = getApi() ?? mockApi;
  const res = await api.get_image_base64({ image_path: props.image.file_path });
  if (res.status === 'success' && res.data_url) {
    dataUrl.value = res.data_url;
  }
}

// Load full-resolution image when overlay opens, focus for keyboard events
watch(
  () => props.visible,
  async (isVisible) => {
    if (isVisible) {
      dataUrl.value = null;
      loadFullImage();
      await nextTick();
      overlayRef.value?.focus();
    }
  },
  { immediate: true },
);
</script>

<style scoped>
.overlay-fade-enter-active,
.overlay-fade-leave-active {
  transition: opacity 0.2s ease;
}

.overlay-fade-enter-from,
.overlay-fade-leave-to {
  opacity: 0;
}
</style>
