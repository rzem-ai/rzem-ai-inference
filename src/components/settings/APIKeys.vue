<template>
  <div class="flex w-full">
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
              <Password v-model="apiKeys.hf.value" :feedback="false" :toggleMask="true" placeholder="hf_xxxxxxxxxxxxxxxxxxxx" class="token-input" fluid />
            </div>
            <small v-if="apiKeys.hf.message" :class="['save-message', apiKeys.hf.messageType]">
              {{ apiKeys.hf.message }}
            </small>
          </div>
        </template>
        <template #footer>
          <div class="flex justify-end">
            <Button :loading="apiKeys.hf.saving" :disabled="!hasChanged('hf')" size="small" @click="saveKey('hf')">
              <fa :icon="['fal', 'floppy-disk']" size="sm" />Save
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
              <fa :icon="['fal', 'floppy-disk']" size="sm" />Save
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
              <fa :icon="['fal', 'floppy-disk']" size="sm" />Save
            </Button>
          </div>
        </template>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import Badge from 'primevue/badge';
import Button from 'primevue/button';
import Password from 'primevue/password';

import Card from 'primevue/card';

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

const currentPreset = ref<string>('default');

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
</script>

<style scoped>
@reference "tailwindcss";
</style>
