<template>
  <div class="flex flex-col w-full h-full gap-4">
    <div class="flex items-center justify-center pt-4 h-16 shrink">
      <PixelCard variant="blue" :gap="4" :speed="35" :no-focus="false"></PixelCard>
    </div>
    <div class="flex grow place-content-center">
      <div class="place-content-start flex flex-col gap-4">
        <RouterLink :to="{ name: 'create' }" v-slot="{ isActive }">
          <div
            class="p-3 transition-all duration-300 ease-in-out"
            :class="isActive ? 'text-white' : 'text-surface-400 hover:text-white'">
            <ImagePlus :size="20" />
          </div>
        </RouterLink>
        <RouterLink :to="{ name: 'edit' }" v-slot="{ isActive }">
          <div
            class="p-3 transition-all duration-300 ease-in-out"
            :class="isActive ? 'text-white' : 'text-surface-400 hover:text-white'">
            <PenTool :size="20" />
          </div>
        </RouterLink>
        <RouterLink :to="{ name: 'gallery' }" v-slot="{ isActive }">
          <div
            class="p-3 transition-all duration-300 ease-in-out"
            :class="isActive ? 'text-white' : 'text-surface-400 hover:text-white'">
            <Images :size="20" />
          </div>
        </RouterLink>
        <RouterLink :to="{ name: 'styles' }" v-slot="{ isActive }">
          <div
            class="p-3 transition-all duration-300 ease-in-out"
            :class="isActive ? 'text-white' : 'text-surface-400 hover:text-white'">
            <Palette :size="20" />
          </div>
        </RouterLink>
        <RouterLink :to="{ name: 'models' }" v-slot="{ isActive }">
          <div
            class="p-3 transition-all duration-300 ease-in-out"
            :class="isActive ? 'text-white' : 'text-surface-400 hover:text-white'">
            <Box :size="20" />
          </div>
        </RouterLink>
      </div>
    </div>
    <div class="flex flex-col items-center justify-center h-20 shrink gap-1">
      <button
        v-if="discoveryStore.servers.length > 0 || discoveryStore.isRemote"
        class="p-3 transition-all duration-300 ease-in-out disabled:opacity-50"
        :class="discoveryStore.isRemote ? 'text-emerald-400 hover:text-emerald-300' : 'text-amber-400 hover:text-amber-300'"
        :title="serverButtonTitle"
        :disabled="discoveryStore.connecting"
        @click="handleServerClick">
        <Server v-if="discoveryStore.isRemote" :size="20" />
        <ServerOff v-else :size="20" />
      </button>
      <RouterLink :to="{ name: 'settings' }" v-slot="{ isActive }">
        <div
          class="p-3 transition-all duration-300 ease-in-out"
          :class="isActive ? 'text-white' : 'text-surface-400 hover:text-white'">
          <Cog :size="20" />
        </div>
      </RouterLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useDiscoveryStore } from '@/stores/discovery';
import { useServerConnection } from '@/composables/useServerConnection';
import PixelCard from '../components/PixelCard.vue';

const discoveryStore = useDiscoveryStore();
const { connect, disconnect } = useServerConnection();

const serverButtonTitle = computed(() => {
  if (discoveryStore.isRemote && discoveryStore.connectedServer) {
    return `Connected to ${discoveryStore.connectedServer.host}:${discoveryStore.connectedServer.port} — click to disconnect`;
  }
  const count = discoveryStore.servers.length;
  return `${count} server${count !== 1 ? 's' : ''} available — click to connect`;
});

async function handleServerClick() {
  if (discoveryStore.isRemote) {
    await disconnect();
  } else if (discoveryStore.servers.length > 0) {
    const first = discoveryStore.servers[0];
    await connect(first.host, first.port);
  }
}
</script>
