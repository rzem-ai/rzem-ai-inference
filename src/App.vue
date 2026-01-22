<template>
  <Toast />
  <WorkspaceNav />
  <main :style="'height: ' + windowsStore.windowsHeight + 'px'" class="overflow-hidden ml-18">
    <RouterView />
  </main>
</template>

<script setup lang="ts">
import { onMounted, watch } from 'vue';
import { useWindowSize } from '@vueuse/core';
import { RouterView } from 'vue-router';
import Toast from 'primevue/toast';
import WorkspaceNav from '@/components/shared/WorkspaceNav.vue';
import { useAppInit } from '@/composables/useAppInit';
import { useWindowsStore } from './stores/windows';

const windowsStore = useWindowsStore();
const { height: windowHeight } = useWindowSize();

// Watch for window height changes
watch(windowHeight, (newHeight) => {
  windowsStore.setWindowsHeight(newHeight);
});

onMounted(() => {
  windowsStore.setWindowsHeight(windowHeight.value);
});

// Initialize app on mount
useAppInit();
</script>

