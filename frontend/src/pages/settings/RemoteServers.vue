<template>
  <div class="flex flex-col gap-4 p-4 overflow-y-auto">
    <!-- Connection Status -->
    <div>
      <div class="text-xl font-semibold text-surface-900 mb-3">Connection Mode</div>
      <Card>
        <template #content>
          <div class="flex items-center gap-3">
            <Tag :severity="discoveryStore.isRemote ? 'primary' : 'secondary'">
              {{ discoveryStore.isRemote ? 'Remote' : 'Local' }}
            </Tag>
            <div v-if="discoveryStore.connectedServer" class=" "> {{ discoveryStore.connectedServer.host }}:{{ discoveryStore.connectedServer.port }} </div>
            <div v-else class="text-sm"> Using local inference engine </div>
          </div>
          <Button v-if="discoveryStore.isRemote" severity="danger" @click="handleDisconnect"> Disconnect </Button>
        </template>
      </Card>
    </div>

    <!-- Error Message -->
    <Message v-if="discoveryStore.error" severity="error"> {{ discoveryStore.error }} </Message>

    <!-- Discovered Servers -->
    <div>
      <div class="flex items-center justify-between mb-3">
        <div class="text-lg font-semibold text-surface-900">Discovered Servers</div>
        <Button severity="help" size="small" @click="discoveryStore.refreshServers()"> Refresh </Button>
      </div>

      <Card v-if="discoveryStore.servers.length === 0">
        <template #content>
          <div class="text-center">
            <Wifi :size="34" class="text-surface-300 mx-auto mb-2" />
            <p class="text-base text-surface-500">No servers found on the network</p>
            <p class="text-sm text-surface-400 mt-1">
              Start an inference engine server with: <code class="bg-surface-100 px-1 rounded">rzem-ai-inference-engine serve --host 0.0.0.0</code>
            </p>
          </div>
        </template>
      </Card>

      <div v-else class="flex flex-col gap-2">
        <div
          v-for="server in discoveryStore.servers"
          :key="`${server.host}:${server.port}`"
          class="bg-white rounded-xl border border-surface-200 p-4 flex items-center justify-between">
          <div class="flex flex-col gap-1">
            <div class="flex items-center gap-2">
              <div class="text-sm font-semibold text-surface-900">{{ server.name }}</div>
              <div class="text-xs bg-surface-100 text-surface-600 px-1 py-1 rounded">
                {{ server.device }}
              </div>
            </div>
            <div class="flex items-center gap-3 text-xs text-surface-500">
              <div>{{ server.host }}:{{ server.port }}</div>
              <div>v{{ server.version }}</div>
            </div>
          </div>
          <Button v-if="!isConnectedTo(server)" :disabled="discoveryStore.connecting" @click="handleConnect(server.host, server.port)">
            {{ discoveryStore.connecting ? 'Connecting...' : 'Connect' }}
          </Button>
          <div v-else class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-emerald-50 text-emerald-700"> Connected </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { useDiscoveryStore } from '@/stores/discovery';
import { useInferenceStore } from '@/stores/inference';
import { useSettingsStore } from '@/stores/settings';

const discoveryStore = useDiscoveryStore();
const inferenceStore = useInferenceStore();
const settingsStore = useSettingsStore();

function isConnectedTo(server: { host: string; port: number }): boolean {
  const conn = discoveryStore.connectedServer;
  return conn !== null && conn.host === server.host && conn.port === server.port;
}

async function handleConnect(host: string, port: number) {
  await discoveryStore.connectToServer(host, port);
  if (discoveryStore.isRemote) {
    settingsStore.setConnectionMode('remote');
    inferenceStore.engineReady = true;
    inferenceStore.startPolling();
    await settingsStore.loadGpuInfo();
    await settingsStore.loadEngineStatus();
  }
}

async function handleDisconnect() {
  await discoveryStore.disconnectFromServer();
  settingsStore.setConnectionMode('local');
  settingsStore.setRemoteEngineInfo(null);
  inferenceStore.engineReady = false;
  inferenceStore.stopPolling();
  await settingsStore.loadGpuInfo();
  await settingsStore.loadEngineStatus();
}

onMounted(() => {
  discoveryStore.loadConnectionMode();
  discoveryStore.startDiscoveryPolling();
});

onUnmounted(() => {
  discoveryStore.stopDiscoveryPolling();
});
</script>
