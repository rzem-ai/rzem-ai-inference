<template>
  <div class="flex flex-col w-full h-full">
    <!-- div :style="'height: ' + height + 'px'" class="flex flex-row overflow-hidden"></div -->

    <div :style="'height: ' + height + 'px'" class="flex flex-row grow bg-surface-900">
      <WorkspaceNav />
      <RouterView />
    </div>
    <div class="h-10 max-h-10 min-h-10">
      <StatusBar />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, watch } from 'vue';

import StatusBar from './components/shared/StatusBar.vue';
import WorkspaceNav from './components/shared/WorkspaceNav.vue';

import { useWindowSize } from '@vueuse/core';
import { useWindowsStore } from '@/stores/windows';

const windowsStore = useWindowsStore();
const { height: windowHeight } = useWindowSize();

const height = computed(() => {
  return windowsStore.windowsHeight - 100;
});

// Watch for window height changes
watch(windowHeight, (newHeight) => {
  windowsStore.setWindowsHeight(newHeight);
});

onMounted(() => {
  windowsStore.setWindowsHeight(windowHeight.value);
});
</script>
