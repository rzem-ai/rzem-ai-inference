<template>
  <div class="flex flex-col h-full bg-gray-900">
    <div class="workspace-header">
      <h1>Settings</h1>
      <p>Configure application settings and integrations</p>
    </div>
    <div class="h-full">
      <Tabs v-model:value="activeTab" class="h-full">
        <TabList>
          <Tab value="apikeys">
            <div class="flex flex-row items-center gap-2"> API Keys </div>
          </Tab>
          <Tab value="cache">
            <div class="flex flex-row items-center gap-2"> Cache </div>
          </Tab>
        </TabList>

        <TabPanels>
          <!-- API Keys Section -->
          <TabPanel value="apikeys">
            <div class="grid grid-cols-3 gap-4">
              <!-- HuggingFace -->
              <Card class="col-span-1">
                <template #title>
                  <div class="flex">
                    <div class="grow">HuggingFace</div>
                    <div class="shrink">
                      <Badge v-if="apiKeys.hf.value" value="Configured" severity="success" />
                      <span v-else class="badge badge-warning">Not Set</span>
                    </div>
                  </div>
                </template>
                <template #subtitle>
                  <div>
                    Required to download gated models like FLUX.
                    <a href="https://huggingface.co/settings/tokens" target="_blank" class="link"> Get your token </a>
                  </div>
                </template>
                <template #content>
                  <div class="flex flex-col gap-2">
                    <div class="flex items-start gap-2">
                      <Password
                        v-model="apiKeys.hf.value"
                        :feedback="false"
                        :toggleMask="true"
                        placeholder="hf_xxxxxxxxxxxxxxxxxxxx"
                        class="token-input"
                        fluid />
                    </div>
                    <small v-if="apiKeys.hf.message" :class="['save-message', apiKeys.hf.messageType]">
                      {{ apiKeys.hf.message }}
                    </small>
                  </div>
                </template>
                <template #footer>
                  <div class="flex justify-end">
                    <Button :loading="apiKeys.hf.saving" :disabled="!hasChanged('hf')" size="small" @click="saveKey('hf')">
                      <Save class="w-4 h-4" />Save
                    </Button>
                  </div>
                </template>
              </Card>

              <!-- Claude API -->
              <Card class="col-span-1">
                <template #title>
                  <div class="flex">
                    <div class="grow">Claude API</div>
                    <div class="shrink">
                      <Badge v-if="apiKeys.claude.value" value="Configured" severity="success" />
                      <span v-else class="badge badge-warning">Not Set</span>
                    </div>
                  </div>
                </template>
                <template #subtitle>
                  <div>
                    Anthropic API key for AI-powered features.
                    <a href="https://console.anthropic.com/settings/keys" target="_blank" class="link"> Get your key </a>
                  </div>
                </template>
                <template #content>
                  <div class="flex flex-col gap-2">
                    <div class="flex items-start gap-2">
                      <Password
                        v-model="apiKeys.claude.value"
                        :feedback="false"
                        :toggleMask="true"
                        placeholder="sk-ant-xxxxxxxxxxxxxxxxxxxx"
                        class="token-input"
                        fluid />
                    </div>
                    <small v-if="apiKeys.claude.message" :class="['save-message', apiKeys.claude.messageType]">
                      {{ apiKeys.claude.message }}
                    </small>
                  </div>
                </template>
                <template #footer>
                  <div class="flex justify-end">
                    <Button :loading="apiKeys.claude.saving" :disabled="!hasChanged('claude')" size="small" @click="saveKey('claude')">
                      <Save class="w-4 h-4" />Save
                    </Button>
                  </div>
                </template>
              </Card>

              <!-- Fal.ai -->
              <Card class="col-span-1">
                <template #title>
                  <div class="flex">
                    <div class="grow">Fal.ai</div>
                    <div class="shrink">
                      <Badge v-if="apiKeys.fal.value" value="Configured" severity="success" />
                      <span v-else class="badge badge-warning">Not Set</span>
                    </div>
                  </div>
                </template>
                <template #subtitle>
                  <div>
                    For cloud-based image generation when local inference is unavailable.
                    <a href="https://fal.ai/dashboard/keys" target="_blank" class="link"> Get your key </a>
                  </div>
                </template>
                <template #content>
                  <div class="flex flex-col gap-2">
                    <div class="flex items-start gap-2">
                      <Password
                        v-model="apiKeys.fal.value"
                        :feedback="false"
                        :toggleMask="true"
                        placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
                        class="token-input"
                        fluid />
                    </div>
                    <small v-if="apiKeys.fal.message" :class="['save-message', apiKeys.fal.messageType]">
                      {{ apiKeys.fal.message }}
                    </small>
                  </div>
                </template>
                <template #footer>
                  <div class="flex justify-end">
                    <Button :loading="apiKeys.fal.saving" :disabled="!hasChanged('fal')" size="small" @click="saveKey('fal')">
                      <Save class="w-4 h-4" />Save
                    </Button>
                  </div>
                </template>
              </Card>
            </div>
          </TabPanel>

          <TabPanel value="cache">
            <!-- Model Cache Section -->
            <div class="section">
              <h2>Model Cache</h2>
              <p class="section-description"> Manage model caching to speed up batch generation and control VRAM usage. </p>

              <!-- Cache Statistics -->
              <div class="stats-card">
                <div class="stats-header">
                  <h3>Cache Statistics</h3>
                  <Button icon="pi pi-refresh" severity="secondary" text rounded size="small" @click="loadCacheStats" :loading="cacheStatsLoading" />
                </div>
                <div class="stats-grid">
                  <div class="stat-item">
                    <span class="stat-value">{{ cacheStats.pipeline_reuses }}</span>
                    <span class="stat-label">Pipeline Reuses</span>
                  </div>
                  <div class="stat-item">
                    <span class="stat-value">{{ cacheStats.pipeline_recreations }}</span>
                    <span class="stat-label">Recreations</span>
                  </div>
                  <div class="stat-item">
                    <span class="stat-value">{{ cacheStats.embedding_hits }}</span>
                    <span class="stat-label">Embedding Hits</span>
                  </div>
                  <div class="stat-item">
                    <span class="stat-value">{{ cacheStats.embedding_misses }}</span>
                    <span class="stat-label">Embedding Misses</span>
                  </div>
                </div>
                <div class="stats-footer">
                  <span class="stat-detail">
                    <i class="pi pi-database"></i>
                    {{ cacheStats.cached_embeddings }} cached embeddings
                  </span>
                  <span v-if="cacheStats.current_model_type" class="stat-detail">
                    <i class="pi pi-box"></i>
                    Active: {{ cacheStats.current_model_type }}
                  </span>
                </div>
              </div>

              <!-- Cache Preset -->
              <div class="config-card">
                <h3>Cache Preset</h3>
                <p class="config-description"> Choose a preset based on your workflow. </p>
                <div class="preset-buttons">
                  <Button
                    v-for="preset in cachePresets"
                    :key="preset.id"
                    :label="preset.label"
                    :severity="currentPreset === preset.id ? 'primary' : 'secondary'"
                    :outlined="currentPreset !== preset.id"
                    size="small"
                    @click="applyPreset(preset.id)"
                    :loading="applyingPreset === preset.id" />
                </div>
                <small class="preset-hint">{{ getPresetHint(currentPreset) }}</small>
              </div>

              <!-- Advanced Settings -->
              <div class="config-card">
                <div class="config-header" @click="showAdvancedCache = !showAdvancedCache">
                  <h3>Advanced Settings</h3>
                  <i :class="['pi', showAdvancedCache ? 'pi-chevron-up' : 'pi-chevron-down']"></i>
                </div>

                <div v-if="showAdvancedCache" class="advanced-settings">
                  <div class="toggle-row">
                    <div class="toggle-info">
                      <span class="toggle-label">Keep VAE Loaded</span>
                      <span class="toggle-hint">Small model, fast decode (~500MB)</span>
                    </div>
                    <ToggleSwitch v-model="cacheConfig.keep_vae_loaded" @change="updateCacheConfig" />
                  </div>

                  <div class="toggle-row">
                    <div class="toggle-info">
                      <span class="toggle-label">Keep FLUX Loaded</span>
                      <span class="toggle-hint">Large transformer (~12GB VRAM)</span>
                    </div>
                    <ToggleSwitch v-model="cacheConfig.keep_flux_loaded" @change="updateCacheConfig" />
                  </div>

                  <div class="toggle-row">
                    <div class="toggle-info">
                      <span class="toggle-label">Keep T5 Loaded</span>
                      <span class="toggle-hint">Text encoder (~9GB VRAM)</span>
                    </div>
                    <ToggleSwitch v-model="cacheConfig.keep_t5_loaded" @change="updateCacheConfig" />
                  </div>

                  <div class="toggle-row">
                    <div class="toggle-info">
                      <span class="toggle-label">Keep CLIP Loaded</span>
                      <span class="toggle-hint">Text encoder (~400MB)</span>
                    </div>
                    <ToggleSwitch v-model="cacheConfig.keep_clip_loaded" @change="updateCacheConfig" />
                  </div>

                  <div class="slider-row">
                    <div class="slider-info">
                      <span class="slider-label">Embedding Cache Size</span>
                      <span class="slider-value">{{ cacheConfig.embedding_cache_size }} prompts</span>
                    </div>
                    <Slider v-model="cacheConfig.embedding_cache_size" :min="1" :max="50" :step="1" class="config-slider" @slideend="updateCacheConfig" />
                  </div>

                  <div class="slider-row">
                    <div class="slider-info">
                      <span class="slider-label">Idle Timeout</span>
                      <span class="slider-value">
                        {{ cacheConfig.idle_timeout_secs === 0 ? 'Disabled' : formatTimeout(cacheConfig.idle_timeout_secs) }}
                      </span>
                    </div>
                    <Slider v-model="cacheConfig.idle_timeout_secs" :min="0" :max="1800" :step="60" class="config-slider" @slideend="updateCacheConfig" />
                  </div>
                </div>
              </div>

              <!-- Clear Cache -->
              <div class="action-row">
                <Button label="Clear Model Cache" icon="pi pi-trash" severity="danger" outlined @click="clearCache" :loading="clearingCache" />
                <small class="action-hint">Free all cached models and embeddings from memory</small>
              </div>
            </div>
          </TabPanel>

          <TabPanel value="system">
            <!-- System Status -->
            <div class="section">
              <h2>System Status</h2>
              <Button label="Check Backend Health" icon="pi pi-heart" severity="secondary" @click="checkHealth" />
              <p v-if="healthStatus" class="status-message">Status: {{ healthStatus }}</p>
            </div>
          </TabPanel>
        </TabPanels>
      </Tabs>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import Badge from 'primevue/badge';
import Button from 'primevue/button';
import Password from 'primevue/password';
import ToggleSwitch from 'primevue/toggleswitch';
import Tabs from 'primevue/tabs';
import TabList from 'primevue/tablist';
import Tab from 'primevue/tab';
import TabPanels from 'primevue/tabpanels';
import TabPanel from 'primevue/tabpanel';
import Slider from 'primevue/slider';
import Card from 'primevue/card';
import { Save } from 'lucide-vue-next';

const healthStatus = ref<string>('');

const activeTab = ref('apikeys');

// ========== Cache Management ==========

interface CacheStats {
  embedding_hits: number;
  embedding_misses: number;
  cached_embeddings: number;
  pipeline_reuses: number;
  pipeline_recreations: number;
  current_model_type: string | null;
  models_loaded: {
    t5: boolean;
    clip: boolean;
    vae: boolean;
    flux: boolean;
  };
}

interface CacheConfig {
  keep_vae_loaded: boolean;
  keep_flux_loaded: boolean;
  keep_t5_loaded: boolean;
  keep_clip_loaded: boolean;
  embedding_cache_size: number;
  idle_timeout_secs: number;
}

const cacheStats = reactive<CacheStats>({
  embedding_hits: 0,
  embedding_misses: 0,
  cached_embeddings: 0,
  pipeline_reuses: 0,
  pipeline_recreations: 0,
  current_model_type: null,
  models_loaded: { t5: false, clip: false, vae: false, flux: false },
});

const cacheConfig = reactive<CacheConfig>({
  keep_vae_loaded: true,
  keep_flux_loaded: false,
  keep_t5_loaded: false,
  keep_clip_loaded: false,
  embedding_cache_size: 10,
  idle_timeout_secs: 300,
});

const cacheStatsLoading = ref(false);
const clearingCache = ref(false);
const showAdvancedCache = ref(false);
const currentPreset = ref<string>('default');
const applyingPreset = ref<string | null>(null);

const cachePresets = [
  { id: 'default', label: 'Default' },
  { id: 'keep_all', label: 'Keep All' },
  { id: 'memory_saver', label: 'Memory Saver' },
];

function getPresetHint(preset: string): string {
  switch (preset) {
    case 'default':
      return 'Balanced: Keeps VAE loaded, unloads large models after use. Good for most workflows.';
    case 'keep_all':
      return 'Performance: Keeps all models loaded for fast batch generation. Uses ~20GB+ VRAM.';
    case 'memory_saver':
      return 'Low Memory: Unloads all models after each generation. Best for limited VRAM.';
    default:
      return '';
  }
}

function formatTimeout(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  const mins = Math.floor(seconds / 60);
  return `${mins} min`;
}

async function loadCacheStats() {
  cacheStatsLoading.value = true;
  try {
    const stats = await invoke<CacheStats>('get_cache_stats');
    Object.assign(cacheStats, stats);
  } catch (error) {
    console.error('Failed to load cache stats:', error);
  } finally {
    cacheStatsLoading.value = false;
  }
}

async function loadCacheConfig() {
  try {
    const config = await invoke<CacheConfig>('get_cache_config');
    Object.assign(cacheConfig, {
      ...config,
      idle_timeout_secs: config.idle_timeout_secs ?? 0,
    });
    // Determine current preset based on config
    detectCurrentPreset();
  } catch (error) {
    console.error('Failed to load cache config:', error);
  }
}

function detectCurrentPreset() {
  const { keep_vae_loaded, keep_flux_loaded, keep_t5_loaded } = cacheConfig;

  if (keep_vae_loaded && keep_flux_loaded && keep_t5_loaded) {
    currentPreset.value = 'keep_all';
  } else if (!keep_vae_loaded && !keep_flux_loaded && !keep_t5_loaded) {
    currentPreset.value = 'memory_saver';
  } else {
    currentPreset.value = 'default';
  }
}

async function applyPreset(preset: string) {
  applyingPreset.value = preset;
  try {
    await invoke('set_cache_preset', { preset });
    currentPreset.value = preset;
    await loadCacheConfig(); // Reload to get new values
  } catch (error) {
    console.error('Failed to apply preset:', error);
  } finally {
    applyingPreset.value = null;
  }
}

async function updateCacheConfig() {
  try {
    await invoke('set_cache_config', {
      keep_vae_loaded: cacheConfig.keep_vae_loaded,
      keep_flux_loaded: cacheConfig.keep_flux_loaded,
      keep_t5_loaded: cacheConfig.keep_t5_loaded,
      keep_clip_loaded: cacheConfig.keep_clip_loaded,
      embedding_cache_size: cacheConfig.embedding_cache_size,
      idle_timeout_secs: cacheConfig.idle_timeout_secs,
    });
    detectCurrentPreset();
  } catch (error) {
    console.error('Failed to update cache config:', error);
  }
}

async function clearCache() {
  clearingCache.value = true;
  try {
    await invoke('clear_model_cache');
    await loadCacheStats();
  } catch (error) {
    console.error('Failed to clear cache:', error);
  } finally {
    clearingCache.value = false;
  }
}

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
  await Promise.all([loadAllKeys(), loadCacheStats(), loadCacheConfig()]);
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

/* PrimeVue Tabs overrides */
:deep(.p-tabs) {
  @apply flex h-full flex-col;
}

:deep(.p-tabpanels) {
  @apply h-full p-0;
}

:deep(.p-tabpanel) {
  @apply h-full p-4;
}

:deep(.p-card) {
  @apply border border-gray-600;
}

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
  @apply underline text-blue-400;

  &:hover {
    @apply text-blue-500;
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

/* Cache Management Styles */
.section-description {
  @apply mb-4 text-sm;
  color: var(--color-slate-400);
}

.stats-card,
.config-card {
  @apply p-4 mb-4 border rounded-xl;
  border-color: var(--color-slate-700);
  background-color: var(--color-slate-800);

  h3 {
    @apply m-0 text-sm font-semibold;
    color: var(--color-slate-200);
  }
}

.stats-header {
  @apply flex items-center justify-between mb-3;
}

.stats-grid {
  @apply grid grid-cols-4 gap-3 mb-3;
}

.stat-item {
  @apply flex flex-col items-center p-3 rounded-lg;
  background-color: var(--color-slate-900);

  .stat-value {
    @apply text-xl font-bold;
    color: var(--color-teal-400);
  }

  .stat-label {
    @apply text-xs mt-1;
    color: var(--color-slate-400);
  }
}

.stats-footer {
  @apply flex gap-4 pt-3 border-t;
  border-color: var(--color-slate-700);
}

.stat-detail {
  @apply flex items-center gap-2 text-xs;
  color: var(--color-slate-400);

  i {
    color: var(--color-slate-500);
  }
}

.config-description {
  @apply mt-2 mb-3 text-sm;
  color: var(--color-slate-400);
}

.preset-buttons {
  @apply flex gap-2 mb-2;
}

.preset-hint {
  @apply block text-xs;
  color: var(--color-slate-500);
}

.config-header {
  @apply flex items-center justify-between cursor-pointer select-none;

  i {
    color: var(--color-slate-400);
  }
}

.advanced-settings {
  @apply mt-4 pt-4 border-t;
  border-color: var(--color-slate-700);
}

.toggle-row {
  @apply flex items-center justify-between py-3;

  &:not(:last-child) {
    @apply border-b;
    border-color: var(--color-slate-700);
  }
}

.toggle-info {
  @apply flex flex-col;
}

.toggle-label {
  @apply text-sm font-medium;
  color: var(--color-slate-200);
}

.toggle-hint {
  @apply text-xs mt-0.5;
  color: var(--color-slate-500);
}

.slider-row {
  @apply py-3;

  &:not(:last-child) {
    @apply border-b;
    border-color: var(--color-slate-700);
  }
}

.slider-info {
  @apply flex items-center justify-between mb-2;
}

.slider-label {
  @apply text-sm font-medium;
  color: var(--color-slate-200);
}

.slider-value {
  @apply text-sm font-mono;
  color: var(--color-teal-400);
}

.config-slider {
  @apply w-full;
}

.action-row {
  @apply flex items-center gap-4;
}

.action-hint {
  @apply text-xs;
  color: var(--color-slate-500);
}
</style>
