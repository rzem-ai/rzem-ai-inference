import { defineStore } from 'pinia'
import { ref } from 'vue'

export type ConnectionMode = 'local' | 'server' | 'client'

export const useSettingsStore = defineStore('settings', () => {
  // State
  const connectionMode = ref<ConnectionMode>('local')
  const serverUrl = ref<string>('')
  const serverPort = ref<number>(7860)
  const apiToken = ref<string>('')
  const outputPath = ref<string>('')
  const modelCachePath = ref<string>('')

  // Actions
  function setConnectionMode(mode: ConnectionMode) {
    connectionMode.value = mode
  }

  function setServerUrl(url: string) {
    serverUrl.value = url
  }

  function setApiToken(token: string) {
    apiToken.value = token
  }

  function setServerPort(port: number) {
    serverPort.value = port
  }

  function setOutputPath(path: string) {
    outputPath.value = path
  }

  function setModelCachePath(path: string) {
    modelCachePath.value = path
  }

  return {
    // State
    connectionMode,
    serverUrl,
    serverPort,
    apiToken,
    outputPath,
    modelCachePath,
    // Actions
    setConnectionMode,
    setServerUrl,
    setServerPort,
    setApiToken,
    setOutputPath,
    setModelCachePath
  }
})
