/**
 * Runtime configuration composable
 *
 * Provides access to the application's runtime mode (local, server, client)
 * and server URLs for client mode.
 */

import { ref, readonly } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface RuntimeConfig {
  mode: 'local' | 'server' | 'client';
  server_url?: string;
  ws_url?: string;
}

// Cached runtime config (fetched once at startup)
const config = ref<RuntimeConfig>({ mode: 'local', server_url: '', ws_url: '' });
const loading = ref(true);
const error = ref<string | null>(null);

/**
 * Get the runtime configuration
 *
 * This is fetched once at application startup and cached.
 */
export async function useRuntimeConfig() {
  // Return cached config if available
  if (config.value) {
    return {
      config: readonly(config),
      loading: readonly(loading),
      error: readonly(error),
      isLocal: config.value.mode === 'local',
      isServer: config.value.mode === 'server',
      isClient: config.value.mode === 'client',
      refresh: fetchConfig,
    };
  }

  // Fetch config on first call
  await fetchConfig();

  return {
    config: readonly(config),
    loading: readonly(loading),
    error: readonly(error),
    isLocal: config.value.mode === 'local',
    isServer: config.value.mode === 'server',
    isClient: config.value.mode === 'client',
    refresh: fetchConfig,
  };
}

/**
 * Fetch runtime configuration from Tauri backend
 */
async function fetchConfig() {
  try {
    loading.value = true;
    error.value = null;

    const result = await invoke<RuntimeConfig>('get_runtime_config');
    config.value = result;

    console.log('Runtime mode:', result.mode);
    if (result.mode === 'client') {
      console.log('Server URL:', result.server_url);
    }
  } catch (e) {
    error.value = `Failed to get runtime config: ${e}`;
    console.error(error.value);
    // Default to local mode on error
    config.value = { mode: 'local' };
  } finally {
    loading.value = false;
  }
}

/**
 * Initialize runtime config (call this in App.vue on mount)
 */
export async function initRuntimeConfig() {
  if (!config.value) {
    await fetchConfig();
  }
}

/**
 * Get the current mode (without async)
 *
 * Returns null if config hasn't been loaded yet
 */
export function getRuntimeMode(): 'local' | 'server' | 'client' | null {
  return config.value?.mode || null;
}

/**
 * Check if in client mode (without async)
 */
export function isClientMode(): boolean {
  return config.value?.mode === 'client';
}

/**
 * Check if in server mode (without async)
 */
export function isServerMode(): boolean {
  return config.value?.mode === 'server';
}

/**
 * Check if in local mode (without async)
 */
export function isLocalMode(): boolean {
  return config.value?.mode === 'local' || !config.value;
}
