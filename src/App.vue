<template>
  <Toast />
  <ConfirmDialog />

  <div v-if="isLoading" class="flex items-center justify-center">
    <div class="text-surface-100">Loading...</div>
  </div>

  <div v-else-if="requireAuth"><!-- STUB --></div>

  <Layout v-else><RouterView /></Layout>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { useWindowSize } from '@vueuse/core';
import { RouterView } from 'vue-router';
import ConfirmDialog from 'primevue/confirmdialog';
import Toast from 'primevue/toast';
import Layout from '@/Layout.vue';
import { useAppInit } from '@/composables/useAppInit';
import { useWindowsStore } from '@/stores/windows';
const windowsStore = useWindowsStore();
const { height: windowHeight } = useWindowSize();

const isLoading = ref(false);
const requireAuth = ref(false);

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
