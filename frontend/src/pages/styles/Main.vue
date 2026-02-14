<template>
  <div class="h-full flex flex-col px-2 py-4 gap-2">
    <router-view />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { usePywebview } from '@/composables/usePywebview';
import { useStylesStore } from '@/stores/styles';

const { api, isReady } = usePywebview();
const stylesStore = useStylesStore();

onMounted(() => {
  const check = setInterval(async () => {
    if (isReady.value) {
      clearInterval(check);
      stylesStore.setApi(api.value);
      await Promise.all([stylesStore.loadStyles(), stylesStore.loadCategories(), stylesStore.loadTags(), stylesStore.loadLoras()]);
    }
  }, 50);
});
</script>
