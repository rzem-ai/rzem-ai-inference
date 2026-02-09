import { onMounted } from 'vue';
import { initRuntimeConfig, useRuntimeConfig } from './useRuntimeConfig';
import { initWebSocket } from './useWebSocket';
import { initFileServer } from '@/utils/backend-bridge';
import { useStoreInitialization } from './useStoreInitialization';

export function useAppInit() {
  const { isInitializing, initProgress, initError, initializeCriticalStores, initializeDataStores } = useStoreInitialization();

  onMounted(async () => {
    try {
      // 1. Initialize runtime configuration first
      await initRuntimeConfig();
      console.log('[App Init] Runtime config initialized');

      // 2. Initialize file server port (needed before any image URLs)
      await initFileServer();
      console.log('[App Init] File server initialized');

      // 3. Initialize WebSocket if in client mode
      const runtimeConfig = await useRuntimeConfig();
      if (runtimeConfig.isClient && runtimeConfig.config.value?.ws_url) {
        await initWebSocket(runtimeConfig.config.value.ws_url);
        console.log('[App Init] WebSocket initialized for client mode');
      }

      // 4. Initialize critical stores (blocking - app shell needs these)
      await initializeCriticalStores();
      console.log('[App Init] App is now interactive');

      // 5. Pre-load frequently used components (non-blocking)
      // This caches the webpack chunks so navigation is instant
      Promise.all([]).catch((err) => {
        console.error('[App Init] Component preload failed:', err);
      });

      // 6. Initialize data stores in background (non-blocking)
      // App becomes interactive immediately, data loads progressively
      initializeDataStores().catch((err) => {
        console.error('[App Init] Background data initialization failed:', err);
      });
    } catch (error) {
      console.error('[App Init] Failed to initialize app:', error);
    }
  });

  return {
    isInitializing,
    initProgress,
    initError,
  };
}
