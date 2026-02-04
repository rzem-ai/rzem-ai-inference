import { defineStore } from 'pinia'

export type ConnectionMode = 'local' | 'server' | 'client'

export const useSettingsStore = defineStore('settings', {
  state: () => ({
    connectionMode: 'local' as ConnectionMode,
    serverUrl: '',
    serverPort: 7860,
    apiToken: '',
    outputPath: '',
    modelCachePath: '',
    isInitialized: false,
  }),

  actions: {
    // Minimal initialization (no backend calls needed)
    async initialize(): Promise<void> {
      if (this.isInitialized) return
      // No-op - settings loaded from runtime config
      this.isInitialized = true
    },

    setConnectionMode(mode: ConnectionMode) {
      this.connectionMode = mode
    },

    setServerUrl(url: string) {
      this.serverUrl = url
    },

    setApiToken(token: string) {
      this.apiToken = token
    },

    setServerPort(port: number) {
      this.serverPort = port
    },

    setOutputPath(path: string) {
      this.outputPath = path
    },

    setModelCachePath(path: string) {
      this.modelCachePath = path
    },
  },
})
