<template>
  <div class="flex items-center h-10 gap-3 px-4 border-t bg-surface-900 shrink border-surface-700">
    <!-- Demo mode indicator -->
    <div v-if="demoMode" class="absolute top-1 right-2 z-10 flex items-center gap-1 px-2 py-0.5 bg-purple-600 text-white text-xs rounded-full">
      <span class="animate-pulse">●</span> Demo Mode (Ctrl+Shift+D to exit)
    </div>

    <!-- Generation/Scan Status -->
    <div class="flex items-center gap-2" :class="{ active: currentStats?.isGenerating || isScanning }">
      <div class="status-icon" :class="(currentStats?.isGenerating || isScanning) ? 'generating' : 'idle'">
        <fa v-if="currentStats?.isGenerating || isScanning" :icon="['fal', 'spinner-third']" class="fa-spin" size="sm" />
        <fa v-else :icon="['fal', 'circle']" size="sm" />
      </div>
      <span class="text-xs font-medium text-surface-300">
        {{ statusText }}
      </span>
    </div>

    <div class="w-px h-4 bg-surface-700" />

    <!-- CPU Usage -->
    <div class="flex items-center gap-2">
      <fa :icon="['fal', 'microchip']" class="text-surface-400" size="sm" />
      <div class="flex items-center gap-2">
        <span class="w-8 text-xs stat-label text-surface-200">CPU</span>
        <ProgressBar class="w-20 border border-surface-700" :value="cpuUsage" :show-value="false" :dt="cpuProgressBarDt" v-tooltip.top="cpuUsageTip" />
      </div>
    </div>

    <div class="w-px h-4 bg-surface-700" />

    <!-- RAM Usage -->
    <div class="flex items-center gap-2">
      <fa :icon="['fal', 'memory']" class="text-surface-400" size="sm" />
      <div class="flex items-center gap-2">
        <span class="w-8 text-xs stat-label text-surface-200">RAM</span>
        <ProgressBar class="w-20 border border-surface-700" :value="ramUsage" :show-value="false" :dt="ramProgressBarDt" v-tooltip.top="ramUsageTip" />
      </div>
    </div>

    <!-- GPU Stats (if available) -->
    <template v-if="gpuName">
      <div class="w-px h-4 bg-surface-700" />

      <div class="flex items-center gap-2 gpu">
        <fa :icon="['fal', 'display']" class="text-surface-400" size="sm" />
        <div class="flex items-center gap-2">
          <span class="w-8 text-xs stat-label text-surface-200 gpu-name" :title="gpuNameTip">{{ gpuName }}</span>
          <ProgressBar class="w-20 border border-surface-700" :value="gpuUsage" :show-value="false" :dt="gpuProgressBarDt" v-tooltip.top="gpuUsageTip" />
        </div>
      </div>

      <div class="w-px h-4 bg-surface-700" />

      <!-- GPU VRAM -->
      <div class="flex items-center gap-2">
        <fa :icon="['fal', 'layer-group']" class="text-surface-400" size="sm" />
        <div class="flex items-center gap-2">
          <span class="w-8 text-xs stat-label text-surface-200">VRAM</span>
          <ProgressBar class="w-20 border border-surface-700" :value="vramUsage" :show-value="false" :dt="vramProgressBarDt" v-tooltip.top="vramUsageTip" />
        </div>
        <button
          type="button"
          @click="handleClearVram"
          :disabled="clearingVram"
          class="vram-clear-btn"
          v-tooltip.top="'Free VRAM'"
          aria-label="Free VRAM">
          <fa
            :icon="['fal', clearingVram ? 'spinner-third' : 'broom']"
            :class="{ 'fa-spin': clearingVram }"
            size="sm" />
        </button>
      </div>
    </template>

    <!-- Connection Status (server/client mode) -->
    <div class="ml-auto">
      <ConnectionStatus />
    </div>

    <!-- Refresh indicator -->
    <div class="flex items-center gap-2">
      <fa v-if="false" :icon="['fal', 'arrows-rotate']" class="text-surface-500" :class="{ 'fa-spin': isRefreshing }" size="sm" />
      <RouterLink :to="{ name: 'settings' }" class="w-full">
          <fa :icon="['fal', 'gear']" />
      </RouterLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@/utils/backend-bridge';
import ConnectionStatus from './ConnectionStatus.vue';
import ProgressBar from 'primevue/progressbar';
import { ProgressBarDesignTokens } from '@primeuix/themes/types/progressbar';

interface SystemStats {
  cpuUsage: number;
  memoryUsed: number;
  memoryTotal: number;
  memoryPercent: number;
  gpuMemoryUsed: number;
  gpuMemoryTotal: number;
  gpuUsagePercent: number;
  gpuName: string;
  isGenerating: boolean;
}

const BLANK_SYSTEM_STATUS: SystemStats = {
  cpuUsage: 0,
  memoryUsed: 0,
  memoryTotal: 0,
  memoryPercent: 0,
  gpuMemoryUsed: 0,
  gpuMemoryTotal: 0,
  gpuUsagePercent: 0,
  gpuName: 'RTX 5090',
  isGenerating: false,
};

const stats = ref<SystemStats>(BLANK_SYSTEM_STATUS);
const isRefreshing = ref(false);
const clearingVram = ref(false);
let pollInterval: ReturnType<typeof setInterval> | null = null;

// Scan progress state
// Use scan state from bundles store (event listener set up in store.initialize())
import { useBundlesStore } from '@/stores/bundles';
const bundlesStore = useBundlesStore();

const isScanning = computed(() => bundlesStore.isScanning);
const scanMessage = computed(() => bundlesStore.scanMessage);

// Demo mode state
const demoMode = ref(false);
const demoPhase = ref(0);
let demoInterval: ReturnType<typeof setInterval> | null = null;

// Demo stats that cycle through different scenarios
const demoStats = computed<SystemStats>(() => {
  const phase = demoPhase.value % 400; // 4 phases of 100 ticks each

  // Phase 1 (0-99): Idle, low usage
  if (phase < 100) {
    return {
      cpuUsage: 15 + Math.sin(phase / 10) * 10,
      memoryUsed: 8_589_934_592, // 8 GB
      memoryTotal: 32_212_254_720, // 30 GB
      memoryPercent: 26.7,
      gpuMemoryUsed: 9_147_483_648, // 2 GB
      gpuMemoryTotal: 12_884_901_888, // 12 GB
      gpuUsagePercent: 5 + Math.random() * 5,
      gpuName: 'RTX 4090',
      isGenerating: false,
    };
  }

  // Phase 2 (100-199): Starting generation, ramping up
  if (phase < 200) {
    const progress = (phase - 100) / 100;
    return {
      cpuUsage: 15 + progress * 70,
      memoryUsed: 8_589_934_592 + progress * 4_294_967_296, // 8-12 GB
      memoryTotal: 32_212_254_720,
      memoryPercent: 26.7 + progress * 13.3,
      gpuMemoryUsed: 2_147_483_648 + progress * 9_663_676_416, // 2-11 GB
      gpuMemoryTotal: 12_884_901_888,
      gpuUsagePercent: 5 + progress * 90,
      gpuName: 'RTX 4090',
      isGenerating: true,
    };
  }

  // Phase 3 (200-299): Active generation, high usage
  if (phase < 300) {
    const fluctuation = Math.sin((phase - 200) / 5) * 5;
    return {
      cpuUsage: 85 + fluctuation,
      memoryUsed: 12_884_901_888, // 12 GB
      memoryTotal: 32_212_254_720,
      memoryPercent: 40,
      gpuMemoryUsed: 11_811_160_064, // 11 GB
      gpuMemoryTotal: 12_884_901_888,
      gpuUsagePercent: 95 + fluctuation,
      gpuName: 'RTX 4090',
      isGenerating: true,
    };
  }

  // Phase 4 (300-399): Finishing, ramping down
  const progress = (phase - 300) / 100;
  return {
    cpuUsage: 85 - progress * 70,
    memoryUsed: 12_884_901_888 - progress * 4_294_967_296, // 12-8 GB
    memoryTotal: 32_212_254_720,
    memoryPercent: 40 - progress * 13.3,
    gpuMemoryUsed: 11_811_160_064 - progress * 9_663_676_416, // 11-2 GB
    gpuMemoryTotal: 12_884_901_888,
    gpuUsagePercent: 95 - progress * 90,
    gpuName: 'RTX 4090',
    isGenerating: progress < 0.8,
  };
});

// Use demo stats when in demo mode, otherwise use real stats
const currentStats = computed(() => {
  return demoMode.value ? demoStats.value : stats.value;
});

// Status text shows scan progress or generation status
const statusText = computed(() => {
  if (isScanning.value) {
    return scanMessage.value || 'Scanning models...';
  }
  return currentStats.value.isGenerating ? 'Drawing' : 'Idle';
});

const cpuUsage = computed(() => {
  return Math.floor(currentStats.value.cpuUsage);
});

const cpuUsageTip = computed(() => {
  return formatPercent(cpuUsage.value);
});

const cpuProgressBarDt = computed(() => {
  const usage = cpuUsage.value;

  let barColor = '';
  if (usage >= 90) {
    barColor = 'rgb(239, 68, 68)'; // red-500
  } else if (usage >= 70) {
    barColor = 'rgb(245, 158, 11)'; // amber-500
  } else {
    barColor = 'rgb(34, 197, 94)'; // green-500
  }

  return {
    label: {
      fontSize: '0.55rem',
      fontWeight: '400',
    },
    value: {
      background: barColor,
    } as ProgressBarDesignTokens,
  };
});

const ramUsage = computed(() => {
  return Math.floor(currentStats.value.memoryPercent);
});

const ramUsageTip = computed(() => {
  return `${formatBytes(Math.floor(currentStats.value.memoryUsed))} / ${formatBytes(Math.floor(currentStats.value.memoryTotal))}`;
});

const ramProgressBarDt = computed(() => {
  const usage = ramUsage.value;

  let barColor = '';
  if (usage >= 90) {
    barColor = 'rgb(239, 68, 68)'; // red-500
  } else if (usage >= 70) {
    barColor = 'rgb(245, 158, 11)'; // amber-500
  } else {
    barColor = 'oklch(62.3% 0.214 259.815)'; // blue-500
  }

  return {
    label: {
      fontSize: '0.55rem',
      fontWeight: '400',
    },
    value: {
      background: barColor,
    } as ProgressBarDesignTokens,
  };
});

const gpuName = computed(() => {
  return truncateGpuName(currentStats.value.gpuName);
});

const gpuNameTip = computed(() => {
  return currentStats.value.gpuName;
});

const gpuUsage = computed(() => {
  return Math.floor(currentStats.value.gpuUsagePercent);
});

const gpuUsageTip = computed(() => {
  return formatPercent(gpuUsage.value);
});

const gpuProgressBarDt = computed(() => {
  const usage = gpuUsage.value;

  let barColor = '';
  if (usage >= 90) {
    barColor = 'rgb(239, 68, 68)'; // red-500
  } else if (usage >= 70) {
    barColor = 'rgb(245, 158, 11)'; // amber-500
  } else {
    barColor = 'rgb(34, 197, 94)'; // green-500
  }

  return {
    label: {
      fontSize: '0.55rem',
      fontWeight: '400',
    },
    value: {
      background: barColor,
    } as ProgressBarDesignTokens,
  };
});

const vramUsage = computed(() => {
  let usage = 0;
  if (currentStats.value?.gpuMemoryTotal && currentStats.value?.gpuMemoryUsed) {
    usage = Math.floor((currentStats.value.gpuMemoryUsed / currentStats.value.gpuMemoryTotal) * 100);
  }

  return usage;
});

const vramUsageTip = computed(() => {
  let used = '0';
  if (currentStats.value?.gpuMemoryTotal && currentStats.value?.gpuMemoryUsed) {
    used = formatBytes(Math.floor(currentStats.value.gpuMemoryUsed));
  }

  let total = '0';
  if (currentStats.value?.gpuMemoryTotal && currentStats.value?.gpuMemoryUsed) {
    total = formatBytes(Math.floor(currentStats.value.gpuMemoryTotal));
  }

  return `${used} / ${total}`;
});

const vramProgressBarDt = computed(() => {
  const usage = Math.floor(vramUsage.value);

  let barColor = '';
  if (usage >= 95) {
    barColor = 'rgb(239, 68, 68)'; // red-500
  } else if (usage >= 80) {
    barColor = 'rgb(245, 158, 11)'; // amber-500
  } else {
    barColor = 'oklch(62.7% 0.265 303.9)'; // blue-500
  }

  return {
    label: {
      fontSize: '0.55rem',
      fontWeight: '400',
    },
    value: {
      background: barColor,
    } as ProgressBarDesignTokens,
  };
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

const handleClearVram = async () => {
  clearingVram.value = true;
  try {
    await invoke('clear_model_cache');
    await fetchStats();
  } catch (e) {
    console.error('Failed to clear VRAM:', e);
  } finally {
    clearingVram.value = false;
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

// Toggle demo mode with Ctrl+Shift+D
const handleKeydown = (e: KeyboardEvent) => {
  if (e.ctrlKey && e.shiftKey && e.key === 'D') {
    e.preventDefault();
    demoMode.value = !demoMode.value;

    if (demoMode.value) {
      // Start animating through phases
      demoPhase.value = 0;
      demoInterval = setInterval(() => {
        demoPhase.value = (demoPhase.value + 1) % 400;
      }, 100); // Update every 100ms for smooth animation
      console.log('Demo mode enabled - press Ctrl+Shift+D to disable');
    } else {
      // Stop animation
      if (demoInterval) {
        clearInterval(demoInterval);
        demoInterval = null;
      }
      console.log('Demo mode disabled');
    }
  }
};

onMounted(async () => {
  fetchStats();

  // Adaptive polling: faster when generating, slower when idle
  // This reduces unnecessary updates when nothing is happening
  pollInterval = setInterval(() => {
    const isGenerating = stats.value?.isGenerating ?? false;

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

  // Add keyboard listener for demo mode
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  if (pollInterval) {
    clearInterval(pollInterval);
  }
  if (demoInterval) {
    clearInterval(demoInterval);
  }
  window.removeEventListener('keydown', handleKeydown);
});

let lastIdleUpdate = 0;
</script>

<style scoped>
@reference 'tailwindcss';

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
  &.gpu-name {
    @apply w-auto max-w-20 truncate;
  }
}

.vram-clear-btn {
  @apply text-gray-500 hover:text-blue-500 transition-colors cursor-pointer bg-transparent border-0 p-0;

  &:disabled {
    @apply cursor-not-allowed opacity-70;
  }
}
</style>
