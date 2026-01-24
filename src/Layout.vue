<template>
  <div :style="'height: ' + windowsStore.windowsHeight + 'px'" class="flex flex-row overflow-hidden">
    <WorkspaceNav />
    <div class="flex flex-col w-full h-full">
      <div class="flex-1 w-full min-h-0 border-l border-surface-800 bg-surface-900">
        <RouterView />
      </div>
      <StatusBar/>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, watch } from 'vue';

import StatusBar from './components/shared/StatusBar.vue';
import WorkspaceNav from './components/shared/WorkspaceNav.vue';

import { useWindowSize } from '@vueuse/core';
import { useWindowsStore } from '@/stores/windows';

const windowsStore = useWindowsStore();
const { height: windowHeight } = useWindowSize();

// Watch for window height changes
watch(windowHeight, (newHeight) => {
  windowsStore.setWindowsHeight(newHeight);
});

onMounted(() => {
  windowsStore.setWindowsHeight(windowHeight.value);
});
</script>
