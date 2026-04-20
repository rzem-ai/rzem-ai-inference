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

  <!-- Update banner -->
  <Transition name="slide-up">
    <div
      v-if="updateVersion"
      class="fixed bottom-4 right-4 z-50 flex items-center gap-3 bg-blue-600 text-white px-4 py-3 rounded-xl shadow-lg">
      <span class="text-sm font-medium">v{{ updateVersion }} is ready to install</span>
      <Button size="small" severity="contrast" @click="installUpdate">Restart</Button>
      <Button size="small" severity="contrast" text @click="updateVersion = null">Later</Button>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { Button } from 'primevue';
import NavBar from '@/components/NavBar.vue';
import SetupDialog from '@/components/SetupDialog.vue';
import { useDiscoveryStore } from '@/stores/discovery';
import { useChatStore } from '@/stores/chat';
import { getApiAsync, onUpdateDownloaded } from '@/bridge';

const discoveryStore = useDiscoveryStore();
const chatStore = useChatStore();
const showSetup = ref(false);
const updateVersion = ref<string | null>(null);

let unsubUpdate: (() => void) | undefined;

async function checkFirstRun() {
  const api = await getApiAsync();
  const res = await api.get_setting({ key: 'SETUP_COMPLETED' });
  if (res.status === 'success' && res.value !== '1') {
    showSetup.value = true;
  }
}

async function onSetupCompleted() {
  showSetup.value = false;
  await chatStore.checkConfigured();
}

async function installUpdate() {
  const api = await getApiAsync();
  await api.install_update();
}

onMounted(async () => {
  discoveryStore.loadConnectionMode();
  discoveryStore.startDiscoveryPolling();
  checkFirstRun();
  unsubUpdate = onUpdateDownloaded(({ version }) => {
    updateVersion.value = version;
  });
});

onUnmounted(() => {
  discoveryStore.stopDiscoveryPolling();
  unsubUpdate?.();
});
</script>

<style scoped>
.slide-up-enter-active,
.slide-up-leave-active {
  transition: transform 0.3s ease, opacity 0.3s ease;
}
.slide-up-enter-from,
.slide-up-leave-to {
  transform: translateY(20px);
  opacity: 0;
}
</style>
