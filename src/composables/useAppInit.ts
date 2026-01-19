import { onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { homeDir } from '@tauri-apps/api/path';
import { join } from '@tauri-apps/api/path';
import { useModelsStore } from '@/stores/models';

export function useAppInit() {
  const modelsStore = useModelsStore();
  
  onMounted(async () => {
    try {
      // Get home directory
      const home = await homeDir();
      const dbPath = await join(home, '.rzem-ai-inference', 'gallery.db');

      // Initialize database
      await invoke('init_database', { dbPath });
      console.log('Database initialized successfully');

      // Refresh model availability from backend
      await modelsStore.refreshModelAvailability();
      console.log('Model availability refreshed');
    } catch (error) {
      console.error('Failed to initialize app:', error);
    }
  });
}
