<template>
  <div class="h-screen flex">
    <div class="w-18 mr-2 h-full shrink-0 overflow-hidden bg-surface-800 border-r border-surface-500">
      <NavBar />
    </div>
    <!-- :class="store.chatbotOpen ? 'w-220' : 'w-120'" -->
    <aside class="h-full shrink-0 transition-[width] duration-300 ease-in-out">
      <RouterView name="menu" />
    </aside>
    <main class="h-full min-w-0 flex-1">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import NavBar from '@/components/NavBar.vue';
import { useDiscoveryStore } from '@/stores/discovery';

const discoveryStore = useDiscoveryStore();

onMounted(() => {
  discoveryStore.loadConnectionMode();
  discoveryStore.startDiscoveryPolling();
});

onUnmounted(() => {
  discoveryStore.stopDiscoveryPolling();
});
</script>
