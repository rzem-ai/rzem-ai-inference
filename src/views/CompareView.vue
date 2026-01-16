<script setup lang="ts">
import { useCompareStore } from '@/stores/compare'
import { convertFileSrc } from '@tauri-apps/api/core'
import Button from 'primevue/button'
import Image from 'primevue/image'

const compareStore = useCompareStore()

const getImageSrc = (filePath: string) => {
  return convertFileSrc(filePath)
}

const handleRemove = (imageId: string) => {
  compareStore.removeFromCompare(imageId)
}

const getParameterDiff = (image: any, compareToImage: any, param: string) => {
  if (!compareToImage) return null
  const value = image[param]
  const compareValue = compareToImage[param]
  if (value !== compareValue) {
    return 'different'
  }
  return 'same'
}
</script>

<template>
  <div class="workspace-content compare-view">
    <div class="compare-header">
      <h1>Compare Images</h1>
      <div class="compare-actions">
        <span class="compare-count">
          {{ compareStore.compareCount }} / {{ compareStore.maxCompareImages }} images
        </span>
        <Button
          label="Clear All"
          icon="pi pi-times"
          severity="secondary"
          @click="compareStore.clearCompare"
          :disabled="compareStore.compareCount === 0"
        />
      </div>
    </div>

    <div v-if="compareStore.compareCount === 0" class="empty-state">
      <i class="pi pi-images" style="font-size: 3rem; color: #9ca3af"></i>
      <p>No images to compare</p>
      <p class="empty-hint">Add images from the gallery to compare them side by side</p>
    </div>

    <div v-else class="compare-grid" :style="{ gridTemplateColumns: `repeat(${compareStore.compareCount}, 1fr)` }">
      <div
        v-for="(image, index) in compareStore.compareImages"
        :key="image.id"
        class="compare-item"
      >
        <div class="compare-image-header">
          <span class="compare-index">#{{ index + 1 }}</span>
          <Button
            icon="pi pi-times"
            severity="danger"
            text
            rounded
            size="small"
            @click="handleRemove(image.id)"
          />
        </div>

        <div class="compare-image-container">
          <Image :src="getImageSrc(image.filePath)" :alt="image.prompt" preview />
        </div>

        <div class="compare-metadata">
          <div class="metadata-item">
            <span class="metadata-label">Prompt:</span>
            <span class="metadata-value">{{ image.prompt }}</span>
          </div>

          <div class="metadata-item">
            <span class="metadata-label">Model:</span>
            <span
              class="metadata-value"
              :class="getParameterDiff(image, compareStore.compareImages[0], 'modelName')"
            >
              {{ image.modelName }}
            </span>
          </div>

          <div class="metadata-item">
            <span class="metadata-label">Steps:</span>
            <span
              class="metadata-value"
              :class="getParameterDiff(image, compareStore.compareImages[0], 'steps')"
            >
              {{ image.steps ?? 'N/A' }}
            </span>
          </div>

          <div class="metadata-item">
            <span class="metadata-label">CFG:</span>
            <span
              class="metadata-value"
              :class="getParameterDiff(image, compareStore.compareImages[0], 'cfgScale')"
            >
              {{ image.cfgScale ?? 'N/A' }}
            </span>
          </div>

          <div class="metadata-item">
            <span class="metadata-label">Size:</span>
            <span
              class="metadata-value"
              :class="getParameterDiff(image, compareStore.compareImages[0], 'width')"
            >
              {{ image.width }}×{{ image.height }}
            </span>
          </div>

          <div class="metadata-item">
            <span class="metadata-label">Seed:</span>
            <span
              class="metadata-value"
              :class="getParameterDiff(image, compareStore.compareImages[0], 'seed')"
            >
              {{ image.seed ?? 'N/A' }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.compare-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.compare-header {
  padding: 1.5rem;
  border-bottom: 1px solid #e5e7eb;
  background: white;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.compare-header h1 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
}

.compare-actions {
  display: flex;
  gap: 1rem;
  align-items: center;
}

.compare-count {
  font-size: 0.875rem;
  color: #6b7280;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 1rem;
  color: #6b7280;
}

.empty-hint {
  font-size: 0.875rem;
  color: #9ca3af;
}

.compare-grid {
  display: grid;
  gap: 1rem;
  padding: 1rem;
  overflow-y: auto;
  flex: 1;
}

.compare-item {
  display: flex;
  flex-direction: column;
  background: white;
  border: 1px solid #e5e7eb;
  border-radius: 0.5rem;
  overflow: hidden;
}

.compare-image-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem;
  background: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
}

.compare-index {
  font-weight: 600;
  color: #374151;
}

.compare-image-container {
  aspect-ratio: 1;
  overflow: hidden;
  background: #f3f4f6;
}

.compare-image-container :deep(img) {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.compare-metadata {
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.metadata-item {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.metadata-label {
  font-size: 0.75rem;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
}

.metadata-value {
  font-size: 0.875rem;
  color: #374151;
  word-break: break-word;
}

.metadata-value.different {
  color: #dc2626;
  font-weight: 600;
}

.metadata-value.same {
  color: #059669;
}
</style>
