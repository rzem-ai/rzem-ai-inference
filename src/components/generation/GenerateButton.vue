<script setup lang="ts">
import { computed } from 'vue'
import { useGenerationStore } from '@/stores/generation'
import { useQueueStore } from '@/stores/queue'
import type { GenerationParams } from '@/stores/queue'
import Button from 'primevue/button'
import type ImageCanvas from './ImageCanvas.vue'

// Accept queue count from parent
defineProps<{
  canvasRef?: InstanceType<typeof ImageCanvas> | null
  queueCount?: number
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

const handleGenerate = async () => {
  if (!canGenerate.value) return

  const params = store.currentParams

  // Build queue params from current generation params
  const queueParams: GenerationParams = {
    prompt: params.prompt,
    negative_prompt: params.negativePrompt || undefined,
    steps: params.steps,
    cfg_scale: params.cfgScale,
    width: params.width,
    height: params.height,
    seed: params.seed === -1 ? Math.floor(Math.random() * 2147483647) : params.seed,
    model: params.model
  }

  try {
    // Add to queue - backend will handle processing
    const jobId = await queueStore.addToQueue(queueParams)
    console.log('Job added to queue:', jobId)
  } catch (error) {
    console.error('Failed to add to queue:', error)
  }
}
</script>

<template>
  <div class="generate-button-container">
    <div class="button-wrapper">
      <Button
        :label="buttonLabel"
        @click="handleGenerate"
        :disabled="!canGenerate"
        severity="success"
        size="large"
        class="w-full"
      />
      <span v-if="queueCount && queueCount > 0" class="queue-badge">
        {{ queueCount }}
      </span>
    </div>

    <div v-if="queueCount && queueCount > 0" class="queue-info">
      {{ queueCount }} job{{ queueCount !== 1 ? 's' : '' }} in queue
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
