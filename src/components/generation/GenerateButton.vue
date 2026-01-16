<script setup lang="ts">
import { computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useGenerationStore } from '@/stores/generation'
import type { GenerationJob } from '@/types'
import Button from 'primevue/button'

const store = useGenerationStore()

const canGenerate = computed(() => {
  return store.currentParams.prompt.trim().length > 0 && !store.isGenerating
})

const queueCount = computed(() => {
  return store.queuedJobs.length + store.runningJobs.length
})

const buttonLabel = computed(() => {
  if (store.isGenerating) {
    return 'Generating...'
  }
  if (queueCount.value > 0) {
    return `Generate (${queueCount.value} in queue)`
  }
  return 'Generate'
})

const handleGenerate = async () => {
  if (!canGenerate.value) return

  const params = store.currentParams

  // Create job
  const job: GenerationJob = {
    id: crypto.randomUUID(),
    prompt: params.prompt,
    status: 'Queued'
  }

  store.addJob(job)
  store.updateJobStatus(job.id, 'Running')

  try {
    // Call backend
    const result = await invoke<string>('generate_image', {
      prompt: params.prompt,
      steps: params.steps,
      width: params.width,
      height: params.height,
      seed: params.seed === -1 ? Math.floor(Math.random() * 2147483647) : params.seed
    })

    console.log('Generation result:', result)
    store.updateJobStatus(job.id, 'Completed')
  } catch (error) {
    console.error('Generation failed:', error)
    store.updateJobStatus(job.id, 'Failed')
  }
}
</script>

<template>
  <div class="generate-button-container">
    <Button
      :label="buttonLabel"
      @click="handleGenerate"
      :disabled="!canGenerate"
      severity="success"
      size="large"
      class="w-full"
    />

    <div v-if="queueCount > 0" class="queue-info">
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

.queue-info {
  text-align: center;
  font-size: 0.875rem;
  color: #6b7280;
}
</style>
