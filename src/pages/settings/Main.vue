<template>
  <div class="h-full flex flex-col p-2 pl-0 gap-2">
    <router-view />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useSettingsStore } from '@/stores/settings';

const settingsStore = useSettingsStore();

onMounted(() => {
  Promise.all([
    settingsStore.loadConnectionMode(),
    settingsStore.loadGpuInfo(),
    settingsStore.loadCudaVersion(),
    settingsStore.loadEngineStatus(),
    settingsStore.loadDataPaths(),
  ]);
});
</script>
