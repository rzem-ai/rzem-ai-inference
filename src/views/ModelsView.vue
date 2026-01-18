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
        <Button v-if="!isDownloaded" label="Download FLUX Schnell" icon="pi pi-download" :loading="isDownloading" @click="downloadModels" />
        <Button v-else label="Re-check Status" icon="pi pi-refresh" severity="secondary" @click="checkModels" />
        <Button label="Show Details" icon="pi pi-info-circle" severity="secondary" text @click="toggleDetails" />
      </div>

      <ProgressBar v-if="isDownloading" mode="indeterminate" class="mt-3" />

      <Message v-if="isDownloading" severity="info" class="mt-3">
        Downloading models from HuggingFace Hub. This may take several minutes depending on your internet speed.
      </Message>

      <!-- Model Files Status -->
      <div v-if="showDetails" class="model-status">
        <h3>Model Files Status</h3>
        <div v-if="modelStatus.length === 0" class="loading-status">
          <i class="pi pi-spin pi-spinner"></i> Loading status...
        </div>
        <div v-else class="status-list">
          <div v-for="file in modelStatus" :key="file.name" class="status-item">
            <i :class="['pi', file.exists ? 'pi-check-circle status-ok' : 'pi-times-circle status-missing']"></i>
            <div class="status-info">
              <span class="status-name">{{ file.name }}</span>
              <code class="status-path">{{ file.path }}</code>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="model-note">
      <p><strong>Note:</strong> Models are downloaded to <code>~/.cache/huggingface/hub/</code></p>
      <p>After download, the generation system will automatically use real FLUX models instead of stubs.</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import Button from 'primevue/button';
import ProgressBar from 'primevue/progressbar';
import Message from 'primevue/message';

interface ModelFileStatus {
  name: string;
  exists: boolean;
  path: string;
}

const isDownloaded = ref(false);
const isDownloading = ref(false);
const error = ref<string | null>(null);
const showDetails = ref(false);
const modelStatus = ref<ModelFileStatus[]>([]);

onMounted(async () => {
  await checkModels();
});

const checkModels = async () => {
  try {
    isDownloaded.value = await invoke<boolean>('check_models_downloaded');
    if (showDetails.value) {
      await loadModelStatus();
    }
  } catch (e) {
    error.value = `Failed to check models: ${e}`;
  }
};

const loadModelStatus = async () => {
  try {
    modelStatus.value = await invoke<ModelFileStatus[]>('get_model_status');
  } catch (e) {
    console.error('Failed to load model status:', e);
  }
};

const toggleDetails = async () => {
  showDetails.value = !showDetails.value;
  if (showDetails.value && modelStatus.value.length === 0) {
    await loadModelStatus();
  }
};

const downloadModels = async () => {
  isDownloading.value = true;
  error.value = null;

  try {
    const result = await invoke<string>('download_flux_schnell');
    console.log(result);
    await checkModels();
  } catch (e) {
    error.value = `Download failed: ${e}`;
  } finally {
    isDownloading.value = false;
  }
};
</script>

<style scoped>
@reference "tailwindcss";

.models-view {
  @apply p-8 max-w-3xl mx-auto;
}

.models-header {
  @apply mb-8;

  h1 {
    @apply m-0 mb-2 text-3xl font-semibold;
    color: var(--color-slate-50);
  }
}

.subtitle {
  @apply m-0;
  color: var(--color-slate-400);
}

.model-card {
  @apply border rounded-xl p-6 mb-4;
  background-color: var(--color-slate-800);
  border-color: var(--color-slate-700);
}

.model-header {
  @apply flex items-center gap-4 mb-4;

  h2 {
    @apply m-0 text-xl font-semibold;
    color: var(--color-slate-50);
  }
}

.badge {
  @apply py-1 px-3 rounded-full text-xs font-medium;

  &-success {
    background: rgba(34, 197, 94, 0.15);
    color: #22c55e;
  }

  &-warning {
    background: rgba(251, 191, 36, 0.15);
    color: #fbbf24;
  }
}

.model-info {
  @apply mb-6;

  p {
    @apply my-2;
    color: var(--color-slate-300);
  }

  strong {
    color: var(--color-slate-200);
  }
}

.model-actions {
  @apply flex gap-2;
}

.model-status {
  @apply mt-4 pt-4 border-t;
  border-color: var(--color-slate-700);

  h3 {
    @apply m-0 mb-3 text-sm font-semibold;
    color: var(--color-slate-300);
  }
}

.loading-status {
  @apply text-sm;
  color: var(--color-slate-400);
}

.status-list {
  @apply flex flex-col gap-2;
}

.status-item {
  @apply flex items-start gap-2 text-sm;
}

.status-ok {
  color: #22c55e;
}

.status-missing {
  color: #ef4444;
}

.status-info {
  @apply flex flex-col gap-0.5;
}

.status-name {
  @apply font-medium;
  color: var(--color-slate-200);
}

.status-path {
  @apply text-xs font-mono break-all;
  color: var(--color-slate-500);
}

.model-note {
  @apply rounded-xl p-4 text-sm;
  background-color: var(--color-slate-900);
  color: var(--color-slate-400);

  p {
    @apply my-2;
  }

  code {
    @apply py-0.5 px-1.5 rounded font-mono;
    background-color: var(--color-slate-800);
    color: var(--color-slate-300);
  }

  strong {
    color: var(--color-slate-300);
  }
}
</style>
