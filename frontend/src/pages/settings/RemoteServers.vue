<template>
  <div class="flex flex-col gap-4 p-4 overflow-y-auto">
    <!-- Connection Status -->
    <div>
      <h2 class="text-sm font-semibold text-slate-900 mb-3">Connection Mode</h2>
      <div class="bg-white rounded-xl border border-slate-200 p-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <span
              class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium"
              :class="discoveryStore.isRemote
                ? 'bg-blue-50 text-blue-700'
                : 'bg-slate-100 text-slate-700'">
              {{ discoveryStore.isRemote ? 'Remote' : 'Local' }}
            </span>
            <span v-if="discoveryStore.connectedServer" class="text-sm text-slate-600">
              {{ discoveryStore.connectedServer.host }}:{{ discoveryStore.connectedServer.port }}
            </span>
            <span v-else class="text-sm text-slate-500">
              Using local inference engine
            </span>
          </div>
          <button
            v-if="discoveryStore.isRemote"
            class="px-3 py-1.5 text-xs font-medium rounded-lg border border-red-200 text-red-600 hover:bg-red-50 transition-colors"
            @click="handleDisconnect">
            Disconnect
          </button>
        </div>
      </div>
    </div>

    <!-- Error Message -->
    <div v-if="discoveryStore.error" class="bg-red-50 border border-red-200 rounded-xl p-3">
      <p class="text-sm text-red-700">{{ discoveryStore.error }}</p>
    </div>

    <!-- Discovered Servers -->
    <div>
      <div class="flex items-center justify-between mb-3">
        <h2 class="text-sm font-semibold text-slate-900">Discovered Servers</h2>
        <button
          class="text-xs text-blue-600 hover:text-blue-700 font-medium"
          @click="discoveryStore.refreshServers()">
          Refresh
        </button>
      </div>

      <div v-if="discoveryStore.servers.length === 0" class="bg-white rounded-xl border border-slate-200 p-6 text-center">
        <Wifi :size="24" class="text-slate-300 mx-auto mb-2" />
        <p class="text-sm text-slate-500">No servers found on the network</p>
        <p class="text-xs text-slate-400 mt-1">
          Start an inference engine server with: <code class="bg-slate-100 px-1 rounded">rzem-ai-inference-engine serve --host 0.0.0.0</code>
        </p>
      </div>

      <div v-else class="flex flex-col gap-2">
        <div
          v-for="server in discoveryStore.servers"
          :key="`${server.host}:${server.port}`"
          class="bg-white rounded-xl border border-slate-200 p-4 flex items-center justify-between">
          <div class="flex flex-col gap-1">
            <div class="flex items-center gap-2">
              <span class="text-sm font-semibold text-slate-900">{{ server.name }}</span>
              <span class="text-xs bg-slate-100 text-slate-600 px-1.5 py-0.5 rounded">
                {{ server.device }}
              </span>
            </div>
            <div class="flex items-center gap-3 text-xs text-slate-500">
              <span>{{ server.host }}:{{ server.port }}</span>
              <span>v{{ server.version }}</span>
            </div>
          </div>
          <button
            v-if="!isConnectedTo(server)"
            class="px-3 py-1.5 text-xs font-medium rounded-lg bg-blue-600 text-white hover:bg-blue-700 transition-colors disabled:opacity-50"
            :disabled="discoveryStore.connecting"
            @click="handleConnect(server.host, server.port)">
            {{ discoveryStore.connecting ? 'Connecting...' : 'Connect' }}
          </button>
          <span
            v-else
            class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-emerald-50 text-emerald-700">
            Connected
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { Wifi } from 'lucide-vue-next';
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
    settingsStore.connectionMode = 'remote';
    inferenceStore.engineReady = true;
    inferenceStore.startPolling();
    await settingsStore.loadGpuInfo();
    await settingsStore.loadEngineStatus();
  }
}

async function handleDisconnect() {
  await discoveryStore.disconnectFromServer();
  settingsStore.connectionMode = 'local';
  settingsStore.remoteEngineInfo = null;
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
