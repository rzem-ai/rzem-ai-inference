<template>
  <div class="workspace">
    <div class="workspace-header">
      <h1>Settings</h1>
      <p>Configure application settings and integrations</p>
    </div>
    <div class="workspace-content">
      <!-- API Keys Section -->
      <div class="section">
        <h2>API Keys</h2>

        <!-- HuggingFace -->
        <div class="api-key-card">
          <div class="api-key-header">
            <h3>HuggingFace</h3>
            <span v-if="apiKeys.hf.value" class="badge badge-success">Configured</span>
            <span v-else class="badge badge-warning">Not Set</span>
          </div>
          <p class="api-key-description">
            Required to download gated models like FLUX.
            <a href="https://huggingface.co/settings/tokens" target="_blank" class="link">
              Get your token
            </a>
          </p>
          <div class="form-group">
            <div class="input-row">
              <Password
                v-model="apiKeys.hf.value"
                :feedback="false"
                :toggleMask="true"
                placeholder="hf_xxxxxxxxxxxxxxxxxxxx"
                class="token-input"
              />
              <Button
                label="Save"
                icon="pi pi-check"
                :loading="apiKeys.hf.saving"
                :disabled="!hasChanged('hf')"
                @click="saveKey('hf')"
              />
            </div>
            <small v-if="apiKeys.hf.message" :class="['save-message', apiKeys.hf.messageType]">
              {{ apiKeys.hf.message }}
            </small>
          </div>
        </div>

        <!-- Claude API -->
        <div class="api-key-card">
          <div class="api-key-header">
            <h3>Claude API</h3>
            <span v-if="apiKeys.claude.value" class="badge badge-success">Configured</span>
            <span v-else class="badge badge-warning">Not Set</span>
          </div>
          <p class="api-key-description">
            Anthropic API key for AI-powered features.
            <a href="https://console.anthropic.com/settings/keys" target="_blank" class="link">
              Get your key
            </a>
          </p>
          <div class="form-group">
            <div class="input-row">
              <Password
                v-model="apiKeys.claude.value"
                :feedback="false"
                :toggleMask="true"
                placeholder="sk-ant-xxxxxxxxxxxxxxxxxxxx"
                class="token-input"
              />
              <Button
                label="Save"
                icon="pi pi-check"
                :loading="apiKeys.claude.saving"
                :disabled="!hasChanged('claude')"
                @click="saveKey('claude')"
              />
            </div>
            <small v-if="apiKeys.claude.message" :class="['save-message', apiKeys.claude.messageType]">
              {{ apiKeys.claude.message }}
            </small>
          </div>
        </div>

        <!-- Fal.ai -->
        <div class="api-key-card">
          <div class="api-key-header">
            <h3>Fal.ai</h3>
            <span v-if="apiKeys.fal.value" class="badge badge-success">Configured</span>
            <span v-else class="badge badge-warning">Not Set</span>
          </div>
          <p class="api-key-description">
            For cloud-based image generation when local inference is unavailable.
            <a href="https://fal.ai/dashboard/keys" target="_blank" class="link">
              Get your key
            </a>
          </p>
          <div class="form-group">
            <div class="input-row">
              <Password
                v-model="apiKeys.fal.value"
                :feedback="false"
                :toggleMask="true"
                placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
                class="token-input"
              />
              <Button
                label="Save"
                icon="pi pi-check"
                :loading="apiKeys.fal.saving"
                :disabled="!hasChanged('fal')"
                @click="saveKey('fal')"
              />
            </div>
            <small v-if="apiKeys.fal.message" :class="['save-message', apiKeys.fal.messageType]">
              {{ apiKeys.fal.message }}
            </small>
          </div>
        </div>
      </div>

      <!-- System Status -->
      <div class="section">
        <h2>System Status</h2>
        <Button
          label="Check Backend Health"
          icon="pi pi-heart"
          severity="secondary"
          @click="checkHealth"
        />
        <p v-if="healthStatus" class="status-message">Status: {{ healthStatus }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import Button from 'primevue/button';
import Password from 'primevue/password';

const healthStatus = ref<string>('');

interface ApiKeyState {
  value: string;
  original: string;
  saving: boolean;
  message: string;
  messageType: 'success' | 'error';
  getCommand: string;
  setCommand: string;
  setParam: string;
}

const apiKeys = reactive<Record<string, ApiKeyState>>({
  hf: {
    value: '',
    original: '',
    saving: false,
    message: '',
    messageType: 'success',
    getCommand: 'get_hf_token',
    setCommand: 'set_hf_token',
    setParam: 'token',
  },
  claude: {
    value: '',
    original: '',
    saving: false,
    message: '',
    messageType: 'success',
    getCommand: 'get_claude_api_key',
    setCommand: 'set_claude_api_key',
    setParam: 'key',
  },
  fal: {
    value: '',
    original: '',
    saving: false,
    message: '',
    messageType: 'success',
    getCommand: 'get_fal_key',
    setCommand: 'set_fal_key',
    setParam: 'key',
  },
});

function hasChanged(key: string): boolean {
  return apiKeys[key].value !== apiKeys[key].original;
}

onMounted(async () => {
  await loadAllKeys();
});

async function loadAllKeys() {
  for (const key in apiKeys) {
    try {
      const result = await invoke<string | null>(apiKeys[key].getCommand);
      apiKeys[key].value = result || '';
      apiKeys[key].original = apiKeys[key].value;
    } catch (error) {
      console.error(`Failed to load ${key} key:`, error);
    }
  }
}

async function saveKey(key: string) {
  const apiKey = apiKeys[key];
  apiKey.saving = true;
  apiKey.message = '';

  try {
    const valueToSave = apiKey.value.trim() || null;
    await invoke(apiKey.setCommand, { [apiKey.setParam]: valueToSave });
    apiKey.original = apiKey.value;
    apiKey.message = 'Saved successfully';
    apiKey.messageType = 'success';
  } catch (error) {
    apiKey.message = `Failed to save: ${error}`;
    apiKey.messageType = 'error';
  } finally {
    apiKey.saving = false;
    setTimeout(() => {
      apiKey.message = '';
    }, 3000);
  }
}

async function checkHealth() {
  try {
    const result = await invoke<string>('health_check');
    healthStatus.value = result;
  } catch (error) {
    healthStatus.value = `Error: ${error}`;
  }
}
</script>

<style scoped>
@reference "tailwindcss";

.workspace {
  @apply flex flex-col h-full;
  background-color: var(--color-slate-950);
}

.workspace-header {
  @apply py-6 px-8 border-b;
  border-color: var(--color-slate-800);

  h1 {
    @apply m-0 text-2xl font-semibold;
    color: var(--color-slate-50);
  }

  p {
    @apply mt-1 mb-0 text-sm;
    color: var(--color-slate-400);
  }
}

.workspace-content {
  @apply flex-1 p-8 overflow-y-auto max-w-3xl;
}

.section {
  @apply mb-8;

  h2 {
    @apply m-0 mb-4 text-lg font-semibold;
    color: var(--color-slate-100);
  }
}

.api-key-card {
  @apply p-4 mb-4 border rounded-xl;
  border-color: var(--color-slate-700);
  background-color: var(--color-slate-800);

  h3 {
    @apply m-0 text-base font-semibold;
    color: var(--color-slate-50);
  }
}

.api-key-header {
  @apply flex items-center gap-3 mb-2;
}

.badge {
  @apply py-0.5 px-2 rounded text-xs font-medium;

  &-success {
    background: rgba(34, 197, 94, 0.15);
    color: #22c55e;
  }

  &-warning {
    background: rgba(251, 191, 36, 0.15);
    color: #fbbf24;
  }
}

.api-key-description {
  @apply mb-3 text-sm;
  color: var(--color-slate-400);
}

.link {
  color: var(--color-teal-400);
  @apply underline;

  &:hover {
    color: var(--color-teal-500);
  }
}

.form-group {
  @apply flex flex-col gap-2;
}

.input-row {
  @apply flex gap-2 items-start;
}

.token-input {
  @apply flex-1;

  :deep(input) {
    @apply w-full font-mono text-sm;
  }
}

.save-message {
  @apply text-xs;

  &.success {
    color: #22c55e;
  }

  &.error {
    color: #ef4444;
  }
}

.status-message {
  @apply mt-4 py-3 px-5 rounded text-sm;
  background: rgba(34, 197, 94, 0.15);
  border: 1px solid rgba(34, 197, 94, 0.3);
  color: #22c55e;
}
</style>
