<script setup lang="ts">
import { computed } from 'vue'
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

const handleGenerate = () => {
  if (!canGenerate.value) return

  // Create job
  const job: GenerationJob = {
    id: crypto.randomUUID(),
    prompt: store.currentParams.prompt,
    status: 'Queued'
  }

  store.addJob(job)

  // TODO: Dispatch to backend in next task
  console.log('Job added to queue:', job)
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
