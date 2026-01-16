<script setup lang="ts">
import { ref } from 'vue'
import { useToast } from 'primevue/usetoast'
import PromptInput from '@/components/generation/PromptInput.vue'
import ModelSelector from '@/components/generation/ModelSelector.vue'
import PresetSelector from '@/components/generation/PresetSelector.vue'
import ParameterControls from '@/components/generation/ParameterControls.vue'
import GenerateButton from '@/components/generation/GenerateButton.vue'
import QueuePanel from '@/components/queue/QueuePanel.vue'
import ImageCanvas from '@/components/generation/ImageCanvas.vue'
import { useQueueStore } from '@/stores/queue'
import { useGenerationStore } from '@/stores/generation'
import type { GenerationParams } from '@/stores/queue'

const canvasRef = ref<InstanceType<typeof ImageCanvas> | null>(null)
const queueStore = useQueueStore()
const generationStore = useGenerationStore()
const toast = useToast()

const handleGenerate = async () => {
  const params = generationStore.currentParams

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
    await queueStore.addToQueue(queueParams)
  } catch (error) {
    console.error('Failed to add to queue:', error)
    toast.add({
      severity: 'error',
      summary: 'Generation Failed',
      detail: queueStore.error || 'Failed to add generation to queue',
      life: 5000
    })
  }
}

defineExpose({
  canvasRef
})
</script>

<template>
  <div class="workspace">
    <div class="workspace-header">
      <h1>Generate</h1>
      <p>Text-to-image, Image-to-image, and Inpainting</p>
    </div>
    <div class="workspace-content">
      <div class="panel left-panel">
        <h2>Generate</h2>
        <PromptInput />
        <div class="divider"></div>
        <ModelSelector />
        <PresetSelector />
        <div class="divider"></div>
        <ParameterControls />
        <div class="divider"></div>
        <GenerateButton
          :queue-count="queueStore.queueLength"
          @generate="handleGenerate"
        />
      </div>
      <div class="panel center-panel">
        <QueuePanel />
      </div>
      <div class="panel right-panel">
        <h2>Canvas</h2>
        <ImageCanvas ref="canvasRef" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: white;
}

.workspace-header {
  padding: 1.5rem 2rem;
  border-bottom: 1px solid #e9ecef;
}

.workspace-header h1 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
}

.workspace-header p {
  margin: 0.25rem 0 0 0;
  color: #6c757d;
  font-size: 0.875rem;
}

.workspace-content {
  display: grid;
  grid-template-columns: 350px 1fr 400px;
  gap: 1rem;
  flex: 1;
  overflow: hidden;
}

.panel {
  padding: 1rem;
  overflow-y: auto;
}

.panel h2 {
  margin: 0 0 1rem 0;
  font-size: 1rem;
  font-weight: 600;
}

.divider {
  height: 1px;
  background: #e5e7eb;
  margin: 1.5rem 0;
}
</style>
