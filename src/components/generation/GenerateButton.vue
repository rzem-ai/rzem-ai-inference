<script setup lang="ts">
import { computed } from 'vue'
import { useGenerationStore } from '@/stores/generation'
import { useQueueStore } from '@/stores/queue'
import Button from 'primevue/button'

interface Props {
  queueCount?: number
}

// Add default value for queueCount
const props = withDefaults(defineProps<Props>(), {
  queueCount: 0
})

const emit = defineEmits<{
  generate: []
}>()

const store = useGenerationStore()
const queueStore = useQueueStore()

const canGenerate = computed(() => {
  return store.currentParams.prompt.trim().length > 0
})

const buttonLabel = computed(() => {
  if (queueStore.hasRunningJobs) {
    return 'Generating...'
  }
  return 'Generate'
})

const handleClick = () => {
  if (!canGenerate.value) return
  emit('generate')
}
</script>

<template>
  <div class="generate-button-container">
    <div class="button-wrapper">
      <Button
        :label="buttonLabel"
        @click="handleClick"
        :disabled="!canGenerate"
        severity="success"
        size="large"
        class="w-full"
      />
      <span v-if="props.queueCount > 0" class="queue-badge">
        {{ props.queueCount }}
      </span>
    </div>

    <div v-if="props.queueCount > 0" class="queue-info">
      {{ props.queueCount }} job{{ props.queueCount !== 1 ? 's' : '' }} in queue
    </div>
  </div>
</template>

<style scoped>
.generate-button-container {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.button-wrapper {
  position: relative;
  display: flex;
}

.queue-badge {
  position: absolute;
  top: -8px;
  right: -8px;
  background: var(--red-500);
  color: white;
  border-radius: 50%;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.75rem;
  font-weight: 600;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  z-index: 1;
}

.queue-info {
  text-align: center;
  font-size: 0.875rem;
  color: #6b7280;
}
</style>
