<template>
  <div class="flex w-full">
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

    <TabPanel value="system">
      <!-- System Status -->
      <div class="section">
        <h2>System Status</h2>
        <Button label="Check Backend Health" icon="pi pi-heart" severity="secondary" @click="checkHealth" />
        <p v-if="healthStatus" class="status-message">Status: {{ healthStatus }}</p>
      </div>
    </TabPanel>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

import Button from 'primevue/button';

import ToggleSwitch from 'primevue/toggleswitch';

import TabPanel from 'primevue/tabpanel';
import Slider from 'primevue/slider';

const healthStatus = ref<string>('');

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
</style>
