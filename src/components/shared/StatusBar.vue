<template>
  <div class="flex items-center h-full gap-3 px-4 border-t bg-surface-900 border-surface-700">
    <!-- Generation Status -->
    <div class="flex items-center gap-2" :class="{ active: stats?.is_generating }">
      <div class="status-icon" :class="stats?.is_generating ? 'generating' : 'idle'">
        <Loader2 v-if="stats?.is_generating" class="w-3 h-3 animate-spin" />
        <Circle v-else class="w-3 h-3" />
      </div>
      <span class="text-xs font-medium text-surface-300">{{ stats?.is_generating ? 'Generating' : 'Idle' }}</span>
    </div>

    <div class="w-px h-4 bg-surface-700" />

    <!-- CPU Usage -->
    <div class="flex items-center gap-2">
      <Cpu class="w-3.5 h-3.5 text-surface-400" />
      <div class="flex items-center gap-2">
        <span class="stat-label">CPU</span>
        <div class="stat-bar-container">
          <div class="stat-bar" :style="{ width: `${stats?.cpu_usage ?? 0}%` }" :class="getCpuBarClass()" />
        </div>
        <span class="font-mono text-xs text-right min-w-14" :class="getCpuTextClass()">{{ formatPercent(stats?.cpu_usage) }}</span>
      </div>
    </div>

    <div class="w-px h-4 bg-surface-700" />

    <!-- RAM Usage -->
    <div class="flex items-center gap-2">
      <MemoryStick class="w-3.5 h-3.5 text-surface-400" />
      <div class="flex items-center gap-2">
        <span class="stat-label">RAM</span>
        <div class="stat-bar-container">
          <div class="stat-bar" :style="{ width: `${stats?.memory_percent ?? 0}%` }" :class="getMemBarClass()" />
        </div>
        <span class="font-mono text-xs text-right min-w-14" :class="getMemTextClass()">
          {{ formatBytes(stats?.memory_used) }} / {{ formatBytes(stats?.memory_total) }}
        </span>
      </div>
    </div>

    <!-- GPU Stats (if available) -->
    <template v-if="stats?.gpu_name">
      <div class="w-px h-4 bg-surface-700" />

      <div class="flex items-center gap-2 gpu">
        <MonitorDot class="w-3.5 h-3.5 text-surface-400" />
        <div class="flex items-center gap-2">
          <span class="stat-label gpu-name" :title="stats.gpu_name">{{ truncateGpuName(stats.gpu_name) }}</span>
          <div class="stat-bar-container">
            <div class="stat-bar" :style="{ width: `${stats?.gpu_usage_percent ?? 0}%` }" :class="getGpuBarClass()" />
          </div>
          <span class="font-mono text-xs text-right min-w-14" :class="getGpuTextClass()">
            {{ formatPercent(stats?.gpu_usage_percent) }}
          </span>
        </div>
      </div>

      <!-- GPU VRAM -->
      <div class="flex items-center gap-2">
        <Layers class="w-3.5 h-3.5 text-surface-400" />
        <div class="flex items-center gap-2">
          <span class="stat-label">VRAM</span>
          <div class="stat-bar-container">
            <div class="stat-bar" :style="{ width: `${gpuMemPercent}%` }" :class="getVramBarClass()" />
          </div>
          <span class="font-mono text-xs text-right min-w-14" :class="getVramTextClass()">
            {{ formatBytes(stats?.gpu_memory_used) }} / {{ formatBytes(stats?.gpu_memory_total) }}
          </span>
        </div>
      </div>
    </template>

    <!-- Connection Status (server/client mode) -->
    <div class="ml-auto">
      <ConnectionStatus />
    </div>

    <!-- Refresh indicator -->
    <div class="flex items-center gap-2">
      <RefreshCw class="w-3 h-3 text-surface-500" :class="{ 'animate-spin': isRefreshing }" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Circle, Cpu, MemoryStick, MonitorDot, Layers, Loader2, RefreshCw } from 'lucide-vue-next';
import ConnectionStatus from './ConnectionStatus.vue';

interface SystemStats {
  cpu_usage: number;
  memory_used: number;
  memory_total: number;
  memory_percent: number;
  gpu_memory_used: number | null;
  gpu_memory_total: number | null;
  gpu_usage_percent: number | null;
  gpu_name: string | null;
  is_generating: boolean;
}

const stats = ref<SystemStats | null>(null);
const isRefreshing = ref(false);
let pollInterval: ReturnType<typeof setInterval> | null = null;

const gpuMemPercent = computed(() => {
  if (!stats.value?.gpu_memory_total || !stats.value?.gpu_memory_used) return 0;
  return (stats.value.gpu_memory_used / stats.value.gpu_memory_total) * 100;
});

const fetchStats = async () => {
  isRefreshing.value = true;
  try {
    stats.value = await invoke<SystemStats>('get_system_stats');
  } catch (e) {
    console.error('Failed to get system stats:', e);
  } finally {
    isRefreshing.value = false;
  }
};

const formatPercent = (value: number | null | undefined): string => {
  if (value == null) return '--';
  return `${Math.round(value)}%`;
};

const formatBytes = (bytes: number | null | undefined): string => {
  if (bytes == null) return '--';

  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let unitIndex = 0;
  let value = bytes;

  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex++;
  }

  return `${value.toFixed(1)}${units[unitIndex]}`;
};

const truncateGpuName = (name: string): string => {
  // Shorten common GPU names
  return name.replace('NVIDIA GeForce ', '').replace('NVIDIA ', '').replace('AMD Radeon ', '').slice(0, 12);
};

// Color classes based on usage levels
const getCpuBarClass = () => {
  const usage = stats.value?.cpu_usage ?? 0;
  if (usage >= 90) return 'bg-status-red';
  if (usage >= 70) return 'bg-status-yellow';
  return 'bg-status-green';
};

const getCpuTextClass = () => {
  const usage = stats.value?.cpu_usage ?? 0;
  if (usage >= 90) return 'text-status-red';
  if (usage >= 70) return 'text-status-yellow';
  return 'text-status-gray';
};

const getMemBarClass = () => {
  const usage = stats.value?.memory_percent ?? 0;
  if (usage >= 90) return 'bg-status-red';
  if (usage >= 75) return 'bg-status-yellow';
  return 'bg-status-blue';
};

const getMemTextClass = () => {
  const usage = stats.value?.memory_percent ?? 0;
  if (usage >= 90) return 'text-status-red';
  if (usage >= 75) return 'text-status-yellow';
  return 'text-status-gray';
};

const getGpuBarClass = () => {
  const usage = stats.value?.gpu_usage_percent ?? 0;
  if (usage >= 95) return 'bg-status-red';
  if (usage >= 80) return 'bg-status-purple';
  return 'bg-status-green';
};

const getGpuTextClass = () => {
  const usage = stats.value?.gpu_usage_percent ?? 0;
  if (usage >= 95) return 'text-status-red';
  if (usage >= 80) return 'text-status-purple';
  return 'text-status-gray';
};

const getVramBarClass = () => {
  const usage = gpuMemPercent.value;
  if (usage >= 95) return 'bg-status-red';
  if (usage >= 80) return 'bg-status-purple';
  return 'bg-status-blue';
};

const getVramTextClass = () => {
  const usage = gpuMemPercent.value;
  if (usage >= 95) return 'text-status-red';
  if (usage >= 80) return 'text-status-purple';
  return 'text-status-gray';
};

onMounted(() => {
  fetchStats();

  // Adaptive polling: faster when generating, slower when idle
  // This reduces unnecessary updates when nothing is happening
  pollInterval = setInterval(() => {
    const isGenerating = stats.value?.is_generating ?? false;

    // Poll more frequently during generation (2s), less when idle (6s)
    if (isGenerating) {
      fetchStats();
    } else {
      // Only update every 3rd interval when idle (6 seconds)
      const now = Date.now();
      if (!lastIdleUpdate || now - lastIdleUpdate >= 6000) {
        fetchStats();
        lastIdleUpdate = now;
      }
    }
  }, 2000);
});

onUnmounted(() => {
  if (pollInterval) {
    clearInterval(pollInterval);
  }
});

let lastIdleUpdate = 0;
</script>

<style scoped>
@reference 'tailwindcss';

.bg-status-red {
  @apply bg-red-500;
}

.bg-status-yellow {
  @apply bg-amber-500;
}

.bg-status-green {
  @apply bg-emerald-500;
}

.bg-status-blue {
  @apply bg-blue-500;
}

.bg-status-purple {
  @apply bg-purple-500;
}

.text-status-red {
  @apply bg-red-500;
}

.text-status-yellow {
  @apply bg-amber-500;
}

.text-status-green {
  @apply bg-emerald-500;
}

.text-status-gray {
  @apply bg-gray-200;
}

.flex items-center gap-2 {
  &.active {
    color: green; /* @apply text-emerald-400; */
  }

  &.gpu {
    @apply max-w-40;
  }
}

.status-icon {
  &.generating {
    color: green; /* @apply text-emerald-400; */
  }

  &.idle {
    color: gray; /* @apply text-surface-500; */
  }
}

.stat-label {
  @apply text-xs  w-8;
  color: gray; /* text-surface-400 */

  &.gpu-name {
    @apply w-auto max-w-20 truncate;
  }
}
</style>
