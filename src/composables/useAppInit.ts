import { onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { homeDir } from '@tauri-apps/api/path';
import { join } from '@tauri-apps/api/path';
import { useModelsStore } from '@/stores/models';
import { initRuntimeConfig, useRuntimeConfig } from './useRuntimeConfig';
import { initWebSocket } from './useWebSocket';

export function useAppInit() {
  const modelsStore = useModelsStore();

  onMounted(async () => {
    try {
      // Initialize runtime configuration first
      await initRuntimeConfig();
      console.log('Runtime config initialized');

      // Initialize WebSocket if in client mode
      const runtimeConfig = await useRuntimeConfig();
      if (runtimeConfig.isClient && runtimeConfig.config.value?.ws_url) {
        await initWebSocket(runtimeConfig.config.value.ws_url);
        console.log('WebSocket initialized for client mode');
      }

      // Get home directory
      const home = await homeDir();
      const dbPath = await join(home, '.rzem-ai-inference', 'inference.db');

      // Initialize database
      await invoke('init_database', { dbPath });
      console.log('Database initialized successfully');

      // Scan and discover models (creates components and bundles in database)
      console.log('Starting model scan...');
      await invoke('scan_and_discover_models');
      console.log('Model scan completed');

      // Refresh model availability from backend
      await modelsStore.refreshModelAvailability();
      console.log('Model availability refreshed');
    } catch (error) {
      console.error('Failed to initialize app:', error);
    }
  });
}
