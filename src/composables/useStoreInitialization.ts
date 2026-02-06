import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { homeDir, join } from '@tauri-apps/api/path';

export interface InitProgress {
  current: number;
  total: number;
  currentStore: string;
}

export function useStoreInitialization() {
  const isInitializing = ref(false);
  const initProgress = ref<InitProgress>({ current: 0, total: 0, currentStore: '' });
  const initError = ref<string | null>(null);

  /**
   * Phase 1: Critical stores needed for app shell (blocking, fast)
   * These must complete before the app becomes interactive
   */
  async function initializeCriticalStores() {
    isInitializing.value = true;
    initError.value = null;

    const criticalStores = [
      {
        name: 'Database',
        init: async () => {
          const home = await homeDir();
          const dbPath = await join(home, '.rzem-ai-inference', 'inference.db');
          await invoke('init_database', { dbPath });
        },
      },
      {
        name: 'Settings',
        init: async () => {
          const { useSettingsStore } = await import('@/stores/settings');
          const settingsStore = useSettingsStore();
          await settingsStore.initialize();
        },
      },
      {
        name: 'Generation',
        init: async () => {
          const { useGenerationStore } = await import('@/stores/generation');
          const generationStore = useGenerationStore();
          await generationStore.initialize();
        },
      },
      {
        name: 'Presets',
        init: async () => {
          const { usePresetsStore } = await import('@/stores/presets');
          const presetsStore = usePresetsStore();
          await presetsStore.initialize();
        },
      },
      {
        name: 'Windows',
        init: async () => {
          const { useWindowsStore } = await import('@/stores/windows');
          const windowsStore = useWindowsStore();
          await windowsStore.initialize();
        },
      },
      {
        name: 'Styles',
        init: async () => {
          const { useStylesStore } = await import('@/stores/styles');
          const stylesStore = useStylesStore();
          await stylesStore.initialize();
        },
      },
      {
        name: 'Gallery',
        init: async () => {
          const { useGalleryStore } = await import('@/stores/gallery');
          const galleryStore = useGalleryStore();
          await galleryStore.initialize();
        },
      },
      {
        name: 'Chatbot',
        init: async () => {
          const { useChatbotStore } = await import('@/stores/chatbot');
          const chatbotStore = useChatbotStore();
          await chatbotStore.initialize();
        },
      },
    ];

    initProgress.value.total = criticalStores.length;

    try {
      for (let i = 0; i < criticalStores.length; i++) {
        const store = criticalStores[i];
        initProgress.value.current = i + 1;
        initProgress.value.currentStore = store.name;

        console.log(`[App Init] Initializing ${store.name}...`);
        await store.init();
        console.log(`[App Init] ${store.name} initialized`);
      }

      console.log('[App Init] Critical stores initialized');
    } catch (err) {
      initError.value = err instanceof Error ? err.message : String(err);
      console.error('[App Init] Critical store initialization failed:', err);
      throw err;
    } finally {
      isInitializing.value = false;
    }
  }

  /**
   * Phase 2: Data stores (non-blocking, background)
   * These load in PARALLEL in the background after the app becomes interactive
   */
  async function initializeDataStores() {
    const dataStores = [
      {
        name: 'Models',
        init: async () => {
          const { useModelsStore } = await import('@/stores/models');
          const modelsStore = useModelsStore();
          await modelsStore.initialize();
        },
      },
      {
        name: 'Bundles',
        init: async () => {
          const { useBundlesStore } = await import('@/stores/bundles');
          const bundlesStore = useBundlesStore();
          await bundlesStore.initialize();
        },
      },

      {
        name: 'Folders',
        init: async () => {
          const { useFoldersStore } = await import('@/stores/folders');
          const foldersStore = useFoldersStore();
          await foldersStore.initialize();
        },
      },
      {
        name: 'Tags',
        init: async () => {
          const { useTagsStore } = await import('@/stores/tags');
          const tagsStore = useTagsStore();
          await tagsStore.initialize();
        },
      },
      {
        name: 'Styles',
        init: async () => {
          const { useStylesStore } = await import('@/stores/styles');
          const stylesStore = useStylesStore();
          await stylesStore.initialize();
        },
      },
      {
        name: 'Auto-Tag',
        init: async () => {
          const { useAutoTagStore } = await import('@/stores/autoTag');
          const autoTagStore = useAutoTagStore();
          await autoTagStore.initialize();
        },
      },
    ];

    console.log('[App Init] Starting parallel background data store initialization...');

    try {
      // Initialize all stores in parallel for maximum speed
      await Promise.all(
        dataStores.map(async (store) => {
          try {
            console.log(`[App Init] Initializing ${store.name}...`);
            await store.init();
            console.log(`[App Init] ${store.name} initialized`);
          } catch (err) {
            console.error(`[App Init] ${store.name} initialization failed:`, err);
            // Continue with other stores even if one fails
          }
        }),
      );

      console.log('[App Init] All data stores initialized');
    } catch (err) {
      console.error('[App Init] Data store initialization failed:', err);
      // Non-fatal - app can still function
    }
  }

  return {
    isInitializing,
    initProgress,
    initError,
    initializeCriticalStores,
    initializeDataStores,
  };
}
