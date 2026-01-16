<script setup lang="ts">
import { ref } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'

// For now, we'll store the image path in the job
// In a future task, this will come from gallery
const imageSrc = ref<string | null>(null)

defineExpose({
  setImage: (path: string) => {
    // Convert filesystem path to asset URL
    imageSrc.value = convertFileSrc(path)
  }
})
</script>

<template>
  <div class="image-canvas">
    <div v-if="!imageSrc" class="canvas-empty">
      <p>Generated images will appear here</p>
    </div>

    <div v-else class="canvas-content">
      <img :src="imageSrc" alt="Generated image" class="generated-image" />
    </div>
  </div>
</template>

<style scoped>
.image-canvas {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f9fafb;
  border-radius: 0.5rem;
  overflow: hidden;
}

.canvas-empty {
  color: #9ca3af;
  font-size: 0.875rem;
}

.canvas-content {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
}

.generated-image {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  border-radius: 0.375rem;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
}
</style>
