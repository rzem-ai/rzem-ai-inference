<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import Button from 'primevue/button'
import ProgressBar from 'primevue/progressbar'
import Message from 'primevue/message'

const isDownloaded = ref(false)
const isDownloading = ref(false)
const error = ref<string | null>(null)

onMounted(async () => {
  await checkModels()
})

const checkModels = async () => {
  try {
    isDownloaded.value = await invoke<boolean>('check_models_downloaded')
  } catch (e) {
    error.value = `Failed to check models: ${e}`
  }
}

const downloadModels = async () => {
  isDownloading.value = true
  error.value = null

  try {
    const result = await invoke<string>('download_flux_schnell')
    console.log(result)
    await checkModels()
  } catch (e) {
    error.value = `Download failed: ${e}`
  } finally {
    isDownloading.value = false
  }
}
</script>

<template>
  <div class="workspace-content models-view">
    <div class="models-header">
      <h1>Model Management</h1>
      <p class="subtitle">Download and manage FLUX models for local generation</p>
    </div>

    <Message v-if="error" severity="error" :closable="true" @close="error = null">
      {{ error }}
    </Message>

    <div class="model-card">
      <div class="model-header">
        <h2>FLUX.1 [schnell]</h2>
        <span v-if="isDownloaded" class="badge badge-success">Downloaded</span>
        <span v-else class="badge badge-warning">Not Downloaded</span>
      </div>

      <div class="model-info">
        <p><strong>Size:</strong> ~12 GB</p>
        <p><strong>Steps:</strong> 4 (fast)</p>
        <p><strong>License:</strong> Apache 2.0</p>
        <p><strong>Quality:</strong> Good, fast generation</p>
      </div>

      <div class="model-actions">
        <Button
          v-if="!isDownloaded"
          label="Download FLUX Schnell"
          icon="pi pi-download"
          :loading="isDownloading"
          @click="downloadModels"
        />
        <Button
          v-else
          label="Re-check Status"
          icon="pi pi-refresh"
          severity="secondary"
          @click="checkModels"
        />
      </div>

      <ProgressBar v-if="isDownloading" mode="indeterminate" class="mt-3" />

      <Message v-if="isDownloading" severity="info" class="mt-3">
        Downloading models from HuggingFace Hub. This may take several minutes depending on your internet speed.
      </Message>
    </div>

    <div class="model-note">
      <p><strong>Note:</strong> Models are downloaded to <code>~/.cache/huggingface/hub/</code></p>
      <p>After download, the generation system will automatically use real FLUX models instead of stubs.</p>
    </div>
  </div>
</template>

<style scoped>
.models-view {
  padding: 2rem;
  max-width: 800px;
  margin: 0 auto;
}

.models-header {
  margin-bottom: 2rem;
}

.models-header h1 {
  margin: 0 0 0.5rem 0;
  font-size: 2rem;
  font-weight: 600;
}

.subtitle {
  color: #6b7280;
  margin: 0;
}

.model-card {
  background: white;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  padding: 1.5rem;
  margin-bottom: 1rem;
}

.model-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1rem;
}

.model-header h2 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
}

.badge {
  padding: 0.25rem 0.75rem;
  border-radius: 12px;
  font-size: 0.75rem;
  font-weight: 500;
}

.badge-success {
  background: #d1fae5;
  color: #065f46;
}

.badge-warning {
  background: #fef3c7;
  color: #92400e;
}

.model-info {
  margin-bottom: 1.5rem;
}

.model-info p {
  margin: 0.5rem 0;
  color: #374151;
}

.model-actions {
  display: flex;
  gap: 0.5rem;
}

.model-note {
  background: #f3f4f6;
  border-radius: 8px;
  padding: 1rem;
  font-size: 0.875rem;
  color: #4b5563;
}

.model-note p {
  margin: 0.5rem 0;
}

.model-note code {
  background: #e5e7eb;
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
  font-family: monospace;
}
</style>
