<template>
  <div class="w-10 h-10 rounded overflow-hidden bg-slate-200 flex items-center justify-center">
    <img v-if="dataUrl" :src="dataUrl" alt="" class="w-full h-full object-cover" />
    <ImageIcon v-else :size="12" class="text-slate-400" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Image as ImageIcon } from 'lucide-vue-next';
import { getApi, mockApi } from '@/bridge';

const props = defineProps<{ path: string }>();
const dataUrl = ref<string | null>(null);

onMounted(async () => {
  const api = getApi() ?? mockApi;
  const res = await api.get_image_base64({ image_path: props.path });
  if (res.status === 'success' && res.data_url) {
    dataUrl.value = res.data_url;
  }
});
</script>
