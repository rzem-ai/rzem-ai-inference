<template>
  <div class="flex flex-col gap-4 p-4 overflow-y-auto">
    <!-- Connection Mode Banner (remote only) -->
    <div v-if="settingsStore.isRemote" class="bg-blue-50 border border-blue-200 rounded-xl p-3 flex items-center gap-2">
      <Wifi :size="14" class="text-blue-600" />
      <span class="text-sm text-blue-700 font-medium">Connected to remote server</span>
    </div>

    <!-- System Info -->
    <div>
      <div class="text-xl font-semibold text-slate-900 mb-3">System Info</div>
      <div class="grid grid-cols-3 gap-3">
        <div class="bg-white rounded-xl border border-slate-200 p-4">
          <div class="text-sm font-medium text-slate-500 mb-1">GPU</div>
          <div class="text-base font-semibold text-slate-900">
            {{ settingsStore.gpuInfo?.device_name ?? 'Not detected' }}
          </div>
        </div>
        <div class="bg-white rounded-xl border border-slate-200 p-4">
          <div class="text-sm font-medium text-slate-500 mb-1">VRAM</div>
          <div class="text-base font-semibold text-slate-900"> {{ settingsStore.gpuInfo?.total_vram_gb ?? 0 }} GB </div>
        </div>
        <div class="bg-white rounded-xl border border-slate-200 p-4">
          <div class="text-sm font-medium text-slate-500 mb-1">
            {{ settingsStore.isRemote ? 'Device' : 'CUDA' }}
          </div>
          <div class="text-base font-semibold text-slate-900">
            {{ settingsStore.isRemote ? (settingsStore.remoteEngineInfo?.device ?? 'unknown') : (settingsStore.cudaVersion ?? 'N/A') }}
          </div>
        </div>
      </div>
    </div>

    <!-- VRAM Usage (local only) -->
    <div v-if="!settingsStore.isRemote">
      <div class="flex items-center justify-between mb-3">
        <div class="text-lg font-semibold text-slate-900">VRAM Usage</div>
        <Button severity="help" size="small" @click="refreshVram"> Refresh </Button>
      </div>
      <div class="bg-white rounded-xl border border-slate-200 p-4">
        <template v-if="settingsStore.vramUsage?.available">
          <!-- Segmented progress bar -->
          <div class="h-3 rounded-full bg-slate-100 overflow-hidden flex mb-3">
            <div class="bg-blue-500 transition-all" :style="{ width: allocatedPct + '%' }" />
            <div class="bg-amber-400 transition-all" :style="{ width: cachedPct + '%' }" />
          </div>
          <div class="flex gap-4 text-sm">
            <div class="flex items-center gap-1.5">
              <div class="w-2.5 h-2.5 rounded-full bg-blue-500" />
              <span class="text-slate-600">Used {{ formatBytes(settingsStore.vramUsage.allocated) }} ({{ allocatedPct.toFixed(1) }}%)</span>
            </div>
            <div class="flex items-center gap-1.5">
              <div class="w-2.5 h-2.5 rounded-full bg-amber-400" />
              <span class="text-slate-600">Cached {{ formatBytes(cachedBytes) }} ({{ cachedPct.toFixed(1) }}%)</span>
            </div>
            <div class="flex items-center gap-1.5">
              <div class="w-2.5 h-2.5 rounded-full bg-emerald-400" />
              <span class="text-slate-600">Free {{ formatBytes(settingsStore.vramUsage.free) }} ({{ freePct.toFixed(1) }}%)</span>
            </div>
          </div>
        </template>
        <div v-else class="text-base text-slate-500"> VRAM information not available (no CUDA device) </div>
      </div>
    </div>

    <!-- Remote Queue Info (remote only) -->
    <div v-if="settingsStore.isRemote && settingsStore.remoteEngineInfo">
      <div class="text-lg font-semibold text-slate-900 mb-3">Remote Queue</div>
      <div class="grid grid-cols-2 gap-3">
        <div class="bg-white rounded-xl border border-slate-200 p-4">
          <div class="text-sm font-medium text-slate-500 mb-1">Jobs Queued</div>
          <div class="text-base font-semibold text-slate-900">
            {{ settingsStore.remoteEngineInfo.jobs_queued }}
          </div>
        </div>
        <div class="bg-white rounded-xl border border-slate-200 p-4">
          <div class="text-sm font-medium text-slate-500 mb-1">Jobs Running</div>
          <div class="text-base font-semibold text-slate-900">
            {{ settingsStore.remoteEngineInfo.jobs_running }}
          </div>
        </div>
      </div>
    </div>

    <!-- Engine Status -->
    <div>
      <h2 class="text-lg font-semibold text-slate-900 mb-3">Engine Status</h2>
      <div class="bg-white rounded-xl border border-slate-200 p-4">
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-3">
            <span
              class="inline-flex items-center px-2.5 py-0.5 rounded-full text-sm font-medium"
              :class="settingsStore.isEngineReady ? 'bg-emerald-50 text-emerald-700' : 'bg-red-50 text-red-700'">
              {{ settingsStore.isEngineReady ? 'Ready' : 'Stopped' }}
            </span>
          </div>
        </div>
        <div class="grid grid-cols-2 gap-4 mb-4">
          <div>
            <div class="text-sm text-slate-500 mb-0.5">Uptime</div>
            <div class="text-base font-medium text-slate-900">{{ settingsStore.formattedUptime }}</div>
          </div>
          <div>
            <div class="text-sm text-slate-500 mb-0.5">Jobs Completed</div>
            <div class="text-base font-medium text-slate-900">{{ settingsStore.engineStatus?.completed_count ?? 0 }}</div>
          </div>
        </div>
        <div v-if="!settingsStore.isRemote" class="flex gap-2 pt-3 border-t border-slate-100">
          <Button severity="secondary" @click="handleResetEngine"> Reset Engine </Button>
          <Button severity="secondary" @click="handleClearVram"> Clear VRAM Cache </Button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue';
import { Message, Card, InputGroup, InputGroupAddon, InputText, Tag, Button } from 'primevue';
import { Wifi } from 'lucide-vue-next';
import { useSettingsStore } from '@/stores/settings';

const settingsStore = useSettingsStore();

const allocatedPct = computed(() => {
  if (!settingsStore.vramUsage?.total) return 0;
  return (settingsStore.vramUsage.allocated / settingsStore.vramUsage.total) * 100;
});

const cachedBytes = computed(() => {
  if (!settingsStore.vramUsage) return 0;
  return settingsStore.vramUsage.reserved - settingsStore.vramUsage.allocated;
});

const cachedPct = computed(() => {
  if (!settingsStore.vramUsage?.total) return 0;
  return (cachedBytes.value / settingsStore.vramUsage.total) * 100;
});

const freePct = computed(() => {
  if (!settingsStore.vramUsage?.total) return 0;
  return (settingsStore.vramUsage.free / settingsStore.vramUsage.total) * 100;
});

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const gb = bytes / 1024 ** 3;
  if (gb >= 1) return `${gb.toFixed(1)} GB`;
  const mb = bytes / 1024 ** 2;
  return `${mb.toFixed(0)} MB`;
}

async function refreshVram() {
  await settingsStore.loadVramUsage();
}

async function handleResetEngine() {
  await settingsStore.resetEngine();
}

async function handleClearVram() {
  await settingsStore.clearVramCache();
}

let vramInterval: ReturnType<typeof setInterval>;
let statusInterval: ReturnType<typeof setInterval>;

onMounted(async () => {
  await settingsStore.loadVramUsage();
  vramInterval = setInterval(() => settingsStore.loadVramUsage(), 5000);
  statusInterval = setInterval(() => settingsStore.loadEngineStatus(), 5000);
});

onUnmounted(() => {
  clearInterval(vramInterval);
  clearInterval(statusInterval);
});
</script>
