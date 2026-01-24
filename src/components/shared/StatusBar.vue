<template>
  <div class="relative flex items-center h-full gap-3 px-4 border-t bg-surface-900 border-surface-700">
    <!-- Demo mode indicator -->
    <div v-if="demoMode" class="absolute top-1 right-2 z-10 flex items-center gap-1 px-2 py-0.5 bg-purple-600 text-white text-xs rounded-full">
      <span class="animate-pulse">●</span> Demo Mode (Ctrl+Shift+D to exit)
    </div>

    <!-- Generation Status -->
    <div class="flex items-center gap-2" :class="{ active: currentStats?.is_generating }">
      <div class="status-icon" :class="currentStats?.is_generating ? 'generating' : 'idle'">
        <fa v-if="currentStats?.is_generating" :icon="['fal', 'spinner-third']" class="fa-spin" size="sm" />
        <fa v-else :icon="['fal', 'circle']" size="sm" />
      </div>
      <span class="text-xs font-medium text-surface-300">{{ currentStats?.is_generating ? 'Generating' : 'Idle' }}</span>
    </div>

    <div class="w-px h-4 bg-surface-700" />

    <!-- CPU Usage -->
    <div class="flex items-center gap-2">
      <fa :icon="['fal', 'microchip']" class="text-surface-400" size="sm" />
      <div class="flex items-center gap-2">
        <span class="w-8 text-xs stat-label text-surface-200">CPU</span>
        <div class="stat-bar-container">
          <div class="stat-bar" :style="{ width: `${currentStats?.cpu_usage ?? 0}%` }" :class="getCpuBarClass()" />
        </div>
        <span class="font-mono text-xs text-right min-w-14" :class="getCpuTextClass()">{{ formatPercent(currentStats?.cpu_usage) }}</span>
      </div>
    </div>

    <div class="w-px h-4 bg-surface-700" />

    <!-- RAM Usage -->
    <div class="flex items-center gap-2">
      <fa :icon="['fal', 'memory']" class="text-surface-400" size="sm" />
      <div class="flex items-center gap-2">
        <span class="w-8 text-xs stat-label text-surface-200">RAM</span>
        <div class="stat-bar-container">
          <div class="stat-bar" :style="{ width: `${currentStats?.memory_percent ?? 0}%` }" :class="getMemBarClass()" />
        </div>
        <span class="font-mono text-xs text-right min-w-14" :class="getMemTextClass()">
          {{ formatBytes(currentStats?.memory_used) }} / {{ formatBytes(currentStats?.memory_total) }}
        </span>
      </div>
    </div>

    <!-- GPU Stats (if available) -->
    <template v-if="currentStats?.gpu_name">
      <div class="w-px h-4 bg-surface-700" />

      <div class="flex items-center gap-2 gpu">
        <fa :icon="['fal', 'display']" class="text-surface-400" size="sm" />
        <div class="flex items-center gap-2">
          <span class="w-8 text-xs stat-label text-surface-200 gpu-name" :title="currentStats.gpu_name">{{ truncateGpuName(currentStats.gpu_name) }}</span>
          <div class="stat-bar-container">
            <div class="stat-bar" :style="{ width: `${currentStats?.gpu_usage_percent ?? 0}%` }" :class="getGpuBarClass()" />
          </div>
          <span class="font-mono text-xs text-right min-w-14" :class="getGpuTextClass()">
            {{ formatPercent(currentStats?.gpu_usage_percent) }}
          </span>
        </div>
      </div>

      <!-- GPU VRAM -->
      <div class="flex items-center gap-2">
        <fa :icon="['fal', 'layer-group']" class="text-surface-400" size="sm" />
        <div class="flex items-center gap-2">
          <span class="w-8 text-xs stat-label text-surface-200">VRAM</span>
          <div class="stat-bar-container">
            <div class="stat-bar" :style="{ width: `${gpuMemPercent}%` }" :class="getVramBarClass()" />
          </div>
          <span class="font-mono text-xs text-right min-w-14" :class="getVramTextClass()">
            {{ formatBytes(currentStats?.gpu_memory_used) }} / {{ formatBytes(currentStats?.gpu_memory_total) }}
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
      <fa :icon="['fal', 'arrows-rotate']" class="text-surface-500" :class="{ 'fa-spin': isRefreshing }" size="sm" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
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
      cpu_usage: 15 + Math.sin(phase / 10) * 10,
      memory_used: 8_589_934_592, // 8 GB
      memory_total: 32_212_254_720, // 30 GB
      memory_percent: 26.7,
      gpu_memory_used: 2_147_483_648, // 2 GB
      gpu_memory_total: 12_884_901_888, // 12 GB
      gpu_usage_percent: 5 + Math.random() * 5,
      gpu_name: 'RTX 4090',
      is_generating: false,
    };
  }

  // Phase 2 (100-199): Starting generation, ramping up
  if (phase < 200) {
    const progress = (phase - 100) / 100;
    return {
      cpu_usage: 15 + progress * 70,
      memory_used: 8_589_934_592 + progress * 4_294_967_296, // 8-12 GB
      memory_total: 32_212_254_720,
      memory_percent: 26.7 + progress * 13.3,
      gpu_memory_used: 2_147_483_648 + progress * 9_663_676_416, // 2-11 GB
      gpu_memory_total: 12_884_901_888,
      gpu_usage_percent: 5 + progress * 90,
      gpu_name: 'RTX 4090',
      is_generating: true,
    };
  }

  // Phase 3 (200-299): Active generation, high usage
  if (phase < 300) {
    const fluctuation = Math.sin((phase - 200) / 5) * 5;
    return {
      cpu_usage: 85 + fluctuation,
      memory_used: 12_884_901_888, // 12 GB
      memory_total: 32_212_254_720,
      memory_percent: 40,
      gpu_memory_used: 11_811_160_064, // 11 GB
      gpu_memory_total: 12_884_901_888,
      gpu_usage_percent: 95 + fluctuation,
      gpu_name: 'RTX 4090',
      is_generating: true,
    };
  }

  // Phase 4 (300-399): Finishing, ramping down
  const progress = (phase - 300) / 100;
  return {
    cpu_usage: 85 - progress * 70,
    memory_used: 12_884_901_888 - progress * 4_294_967_296, // 12-8 GB
    memory_total: 32_212_254_720,
    memory_percent: 40 - progress * 13.3,
    gpu_memory_used: 11_811_160_064 - progress * 9_663_676_416, // 11-2 GB
    gpu_memory_total: 12_884_901_888,
    gpu_usage_percent: 95 - progress * 90,
    gpu_name: 'RTX 4090',
    is_generating: progress < 0.8,
  };
});

// Use demo stats when in demo mode, otherwise use real stats
const currentStats = computed(() => {
  return demoMode.value ? demoStats.value : stats.value;
});

const gpuMemPercent = computed(() => {
  if (!currentStats.value?.gpu_memory_total || !currentStats.value?.gpu_memory_used) return 0;
  return (currentStats.value.gpu_memory_used / currentStats.value.gpu_memory_total) * 100;
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

// Color classes based on usage levels
const getCpuBarClass = () => {
  const usage = currentStats.value?.cpu_usage ?? 0;
  if (usage >= 90) return 'bg-status-red';
  if (usage >= 70) return 'bg-status-yellow';
  return 'bg-status-green';
};

const getCpuTextClass = () => {
  const usage = currentStats.value?.cpu_usage ?? 0;
  if (usage >= 90) return 'text-status-red';
  if (usage >= 70) return 'text-status-yellow';
  return 'text-status-gray';
};

const getMemBarClass = () => {
  const usage = currentStats.value?.memory_percent ?? 0;
  if (usage >= 90) return 'bg-status-red';
  if (usage >= 75) return 'bg-status-yellow';
  return 'bg-status-blue';
};

const getMemTextClass = () => {
  const usage = currentStats.value?.memory_percent ?? 0;
  if (usage >= 90) return 'text-status-red';
  if (usage >= 75) return 'text-status-yellow';
  return 'text-status-gray';
};

const getGpuBarClass = () => {
  const usage = currentStats.value?.gpu_usage_percent ?? 0;
  if (usage >= 95) return 'bg-status-red';
  if (usage >= 80) return 'bg-status-purple';
  return 'bg-status-green';
};

const getGpuTextClass = () => {
  const usage = currentStats.value?.gpu_usage_percent ?? 0;
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

.stat-label  {


  &.gpu-name {
    @apply w-auto max-w-20 truncate;
  }
}
</style>
