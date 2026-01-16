<script setup lang="ts">
import { ref } from 'vue'
import PromptInput from '@/components/generation/PromptInput.vue'
import ModelSelector from '@/components/generation/ModelSelector.vue'
import PresetSelector from '@/components/generation/PresetSelector.vue'
import ParameterControls from '@/components/generation/ParameterControls.vue'
import GenerateButton from '@/components/generation/GenerateButton.vue'
import QueuePanel from '@/components/queue/QueuePanel.vue'
import ImageCanvas from '@/components/generation/ImageCanvas.vue'
import { useQueueStore } from '@/stores/queue'

const canvasRef = ref<InstanceType<typeof ImageCanvas> | null>(null)
const queueStore = useQueueStore()

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
        <GenerateButton :canvas-ref="canvasRef" :queue-count="queueStore.queueLength" />
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
  display: flex;
  flex: 1;
  overflow: hidden;
}

.panel {
  padding: 1rem;
  overflow-y: auto;
  border-right: 1px solid #e9ecef;
}

.panel:last-child {
  border-right: none;
}

.left-panel {
  width: 35%;
}

.center-panel {
  width: 25%;
}

.right-panel {
  width: 40%;
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
