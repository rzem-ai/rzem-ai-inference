import { onMounted } from 'vue';
import { initRuntimeConfig, useRuntimeConfig } from './useRuntimeConfig';
import { initWebSocket } from './useWebSocket';
import { useStoreInitialization } from './useStoreInitialization';

export function useAppInit() {
  const { isInitializing, initProgress, initError, initializeCriticalStores, initializeDataStores } = useStoreInitialization();

  onMounted(async () => {
    try {
      // 1. Initialize runtime configuration first
      await initRuntimeConfig();
      console.log('[App Init] Runtime config initialized');

      // 2. Initialize WebSocket if in client mode
      const runtimeConfig = await useRuntimeConfig();
      if (runtimeConfig.isClient && runtimeConfig.config.value?.ws_url) {
        await initWebSocket(runtimeConfig.config.value.ws_url);
        console.log('[App Init] WebSocket initialized for client mode');
      }

      // 3. Initialize critical stores (blocking - app shell needs these)
      await initializeCriticalStores();
      console.log('[App Init] App is now interactive');

      // 4. Pre-load frequently used components (non-blocking)
      // This caches the webpack chunks so navigation is instant
      Promise.all([
        import('@/views/GalleryView.vue'),
        import('@/views/GenerateView.vue'),
        import('@/views/StylesView.vue'),
      ]).catch(err => {
        console.error('[App Init] Component preload failed:', err);
      });

      // 5. Initialize data stores in background (non-blocking)
      // App becomes interactive immediately, data loads progressively
      initializeDataStores().catch(err => {
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
