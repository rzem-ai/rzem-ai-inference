<template>
  <Dialog
    v-model:visible="visible"
    header="Pick an image"
    modal
    :style="{ width: '640px', maxHeight: '80vh' }">
    <div v-if="loading" class="flex items-center justify-center py-8">
      <div class="h-6 w-6 animate-spin rounded-full border-2 border-blue-500 border-t-transparent" />
    </div>
    <div v-else-if="images.length" class="grid grid-cols-4 gap-2 overflow-y-auto max-h-96">
      <div
        v-for="img in images"
        :key="img.id"
        class="cursor-pointer rounded overflow-hidden border-2 border-transparent hover:border-blue-400 transition-colors"
        @click="emit('pick', img.file_path)">
        <img v-if="img.thumbnailDataUrl" :src="img.thumbnailDataUrl" alt="" class="w-full aspect-square object-cover" />
        <div v-else class="w-full aspect-square bg-slate-200 flex items-center justify-center">
          <ImageIcon :size="20" class="text-slate-400" />
        </div>
      </div>
    </div>
    <div v-else class="text-center text-surface-400 py-8">No images in gallery yet</div>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { getApiAsync } from '@/bridge';

interface PickerImage {
  id: string;
  file_path: string;
  thumbnail_path?: string;
  thumbnailDataUrl?: string;
}

const visible = defineModel<boolean>('visible');
const emit = defineEmits<{ pick: [imagePath: string] }>();

const images = ref<PickerImage[]>([]);
const loading = ref(false);

watch(visible, async (v) => {
  if (!v) return;
  loading.value = true;
  try {
    const api = await getApiAsync();
    const res = await api.get_gallery_images({ limit: 50, offset: 0 });
    if (res.status === 'success' && res.images) {
      const loaded: PickerImage[] = [];
      for (const img of res.images) {
        const item: PickerImage = { id: img.id, file_path: img.file_path };
        if (img.thumbnail_path) {
          const thumbRes = await api.get_image_base64({ image_path: img.thumbnail_path });
          if (thumbRes.status === 'success' && thumbRes.data_url) {
            item.thumbnailDataUrl = thumbRes.data_url;
          }
        }
        loaded.push(item);
      }
      images.value = loaded;
    }
  } finally {
    loading.value = false;
  }
});
</script>
