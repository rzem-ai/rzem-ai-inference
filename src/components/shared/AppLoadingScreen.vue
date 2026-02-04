<template>
  <div
    v-if="isInitializing"
    class="fixed inset-0 z-50 flex items-center justify-center bg-surface-950"
  >
    <div class="flex flex-col items-center gap-6">
      <!-- Logo or Icon -->
      <div class="text-7xl">
        <i class="pi pi-spin pi-spinner text-blue-500" style="font-size: 3rem"></i>
      </div>

      <!-- Loading Message -->
      <div class="text-center">
        <h2 class="mb-2 text-2xl font-semibold text-surface-50">
          Loading Application
        </h2>
        <p class="text-sm text-surface-400">{{ currentStore }}</p>
      </div>

      <!-- Progress Bar -->
      <div class="w-80 h-3 overflow-hidden rounded-full bg-surface-800">
        <div
          class="h-full transition-all duration-300 bg-blue-500"
          :style="{ width: `${progress}%` }"
        />
      </div>

      <!-- Progress Text -->
      <p class="text-xs text-surface-500">
        {{ current }} / {{ total }} modules loaded
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  isInitializing: boolean
  current: number
  total: number
  currentStore: string
}>()

const progress = computed(() => {
  if (props.total === 0) return 0
  return (props.current / props.total) * 100
})
</script>
