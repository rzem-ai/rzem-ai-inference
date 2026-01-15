import { onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { homeDir } from '@tauri-apps/api/path'
import { join } from '@tauri-apps/api/path'

export function useAppInit() {
  onMounted(async () => {
    try {
      // Get home directory
      const home = await homeDir()
      const dbPath = await join(home, '.flux-generator', 'gallery.db')

      // Initialize database
      await invoke('init_database', { dbPath })
      console.log('Database initialized successfully')
    } catch (error) {
      console.error('Failed to initialize app:', error)
    }
  })
}
