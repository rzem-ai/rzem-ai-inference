<template>
  <Toast />
  <ConfirmDialog />

  <!-- Route navigation progress bar -->
  <div v-if="isNavigating" class="fixed top-0 left-0 right-0 z-[9999] h-1 bg-blue-500 animate-pulse"></div>

  <!-- App Loading Screen -->
  <AppLoadingScreen
    v-if="isInitializing"
    :is-initializing="isInitializing"
    :current="initProgress.current"
    :total="initProgress.total"
    :current-store="initProgress.currentStore"
  />

  <div v-else-if="requireAuth"><!-- STUB --></div>

  <Layout v-else><RouterView /></Layout>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { useWindowSize } from '@vueuse/core';
import { RouterView, useRouter } from 'vue-router';
import ConfirmDialog from 'primevue/confirmdialog';
import Toast from 'primevue/toast';
import Layout from '@/Layout.vue';
import AppLoadingScreen from '@/components/shared/AppLoadingScreen.vue';
import { useAppInit } from '@/composables/useAppInit';
import { useWindowsStore } from '@/stores/windows';

const windowsStore = useWindowsStore();
const router = useRouter();

const { height: windowHeight } = useWindowSize();

const requireAuth = ref(false);
const isNavigating = ref(false);

// Initialize app on mount
const { isInitializing, initProgress } = useAppInit();

// Show navigation loading indicator
router.beforeEach((to, from, next) => {
  isNavigating.value = true;
  next();
});

router.afterEach(() => {
  // Small delay so user can see the indicator
  setTimeout(() => {
    isNavigating.value = false;
  }, 100);
});

// Watch for window height changes
watch(windowHeight, (newHeight) => {
  windowsStore.setWindowsHeight(newHeight);
});

onMounted(() => {
  windowsStore.setWindowsHeight(windowHeight.value);
});
</script>
