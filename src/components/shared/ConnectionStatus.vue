<template>
  <div
    v-if="showStatus"
    class="flex items-center gap-2 px-3 py-1 rounded text-xs"
    :class="statusClass"
  >
    <div class="flex items-center gap-1.5">
      <!-- Status dot -->
      <div class="w-2 h-2 rounded-full" :class="dotClass"></div>

      <!-- Mode label -->
      <span class="font-medium">{{ modeLabel }}</span>

      <!-- Server URL (client mode only) -->
      <span v-if="isClient && serverUrl" class="text-gray-400">
        {{ serverUrl }}
      </span>
    </div>

    <!-- Connection details popover trigger -->
    <button
      v-if="isClient"
      @click="showDetails = !showDetails"
      class="hover:text-gray-300 transition-colors"
      title="Connection details"
    >
      <svg
        class="w-3.5 h-3.5"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
        />
      </svg>
    </button>

    <!-- Details popover -->
    <div
      v-if="showDetails && isClient"
      class="absolute bottom-full right-0 mb-2 bg-gray-800 border border-gray-700 rounded-lg shadow-lg p-3 min-w-[250px] z-50"
    >
      <div class="text-xs space-y-2">
        <div>
          <div class="text-gray-400">Server URL:</div>
          <div class="text-gray-200 font-mono break-all">{{ serverUrl }}</div>
        </div>
        <div>
          <div class="text-gray-400">WebSocket:</div>
          <div class="text-gray-200 font-mono text-[10px] break-all">{{ wsUrl }}</div>
        </div>
        <div>
          <div class="text-gray-400">Status:</div>
          <div :class="connected ? 'text-green-400' : 'text-red-400'">
            {{ connected ? 'Connected' : 'Disconnected' }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRuntimeConfig } from '@/composables/useRuntimeConfig'

const showDetails = ref(false)
const config = ref<any>(null)

onMounted(async () => {
  const runtimeConfig = await useRuntimeConfig()
  config.value = runtimeConfig.config.value
})

const mode = computed(() => config.value?.mode)
const isClient = computed(() => mode.value === 'client')
const isServer = computed(() => mode.value === 'server')
const serverUrl = computed(() => config.value?.server_url)
const wsUrl = computed(() => config.value?.ws_url)

// For MVP, we assume connection is always good if we got the config
// In a full implementation, you'd track actual WebSocket connection state
const connected = computed(() => isClient.value && !!serverUrl.value)

const showStatus = computed(() => {
  // Only show in server or client mode
  return isServer.value || isClient.value
})

const modeLabel = computed(() => {
  if (isServer.value) return 'Server Mode'
  if (isClient.value) return 'Client Mode'
  return 'Local Mode'
})

const statusClass = computed(() => {
  if (isServer.value) {
    return 'bg-blue-900/30 text-blue-300 border border-blue-700/50'
  }
  if (isClient.value && connected.value) {
    return 'bg-green-900/30 text-green-300 border border-green-700/50'
  }
  if (isClient.value && !connected.value) {
    return 'bg-red-900/30 text-red-300 border border-red-700/50'
  }
  return 'bg-gray-900/30 text-gray-400 border border-gray-700/50'
})

const dotClass = computed(() => {
  if (isServer.value) return 'bg-blue-400 animate-pulse'
  if (isClient.value && connected.value) return 'bg-green-400 animate-pulse'
  if (isClient.value && !connected.value) return 'bg-red-400'
  return 'bg-gray-400'
})

// Close details when clicking outside
const handleClickOutside = (e: MouseEvent) => {
  const target = e.target as HTMLElement
  if (!target.closest('.connection-status')) {
    showDetails.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>
