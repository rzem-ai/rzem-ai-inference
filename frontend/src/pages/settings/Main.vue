<template>
  <div class="h-full flex flex-col p-2 pl-0 gap-2">
    <router-view />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { usePywebview } from '@/composables/usePywebview';
import { useSettingsStore } from '@/stores/settings';

const { api, isReady } = usePywebview();
const settingsStore = useSettingsStore();

onMounted(() => {
  const check = setInterval(async () => {
    if (isReady.value) {
      clearInterval(check);
      settingsStore.setApi(api.value);
      await Promise.all([
        settingsStore.loadGpuInfo(),
        settingsStore.loadCudaVersion(),
        settingsStore.loadEngineStatus(),
        settingsStore.loadDataPaths(),
      ]);
    }
  }, 50);
});
</script>
