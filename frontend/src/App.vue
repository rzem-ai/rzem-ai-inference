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

  <SetupDialog v-model:visible="showSetup" @completed="onSetupCompleted" />
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import NavBar from '@/components/NavBar.vue';
import SetupDialog from '@/components/SetupDialog.vue';
import { useDiscoveryStore } from '@/stores/discovery';
import { getApiAsync } from '@/bridge';

const discoveryStore = useDiscoveryStore();
const showSetup = ref(false);

async function checkFirstRun() {
  const api = await getApiAsync();
  const res = await api.get_setting({ key: 'SETUP_COMPLETED' });
  if (res.status === 'success' && res.value !== '1') {
    showSetup.value = true;
  }
}

function onSetupCompleted() {
  showSetup.value = false;
}

onMounted(() => {
  discoveryStore.loadConnectionMode();
  discoveryStore.startDiscoveryPolling();
  checkFirstRun();
});

onUnmounted(() => {
  discoveryStore.stopDiscoveryPolling();
});
</script>
