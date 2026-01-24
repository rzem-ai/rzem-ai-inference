import { defineStore } from 'pinia'
import { ref, computed, onScopeDispose } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

/**
 * Backend preference for auto-tagging
 */
export type TaggingBackend = 'local' | 'claude'

/**
 * Tag category from the taxonomy
 */
export type TagCategory =
  | 'subject'
  | 'style'
  | 'mood'
  | 'lighting'
  | 'color'
  | 'composition'
  | 'setting'
  | 'quality'

/**
 * Tag with confidence score
 */
export interface TagWithConfidence {
  tag: string
  category: TagCategory
  confidence: number
}

/**
 * Auto-tagging settings persisted in the database
 */
export interface AutoTagSettings {
  enabled: boolean
  auto_tag_on_generation: boolean
  preferred_backend: TaggingBackend
  min_confidence: number
  claude_api_key?: string
}

/**
 * Result of a tagging operation
 */
export interface TaggingResult {
  image_id: string
  tags: TagWithConfidence[]
  backend: TaggingBackend
  success: boolean
  error?: string
}

/**
 * Vision model (Moondream) status (matches backend VisionModelStatus)
 */
export interface VisionModelStatus {
  is_downloaded: boolean
  download_progress: number | null
  model_size: number
  model_size_display: string
  error: string | null
}

export const useAutoTagStore = defineStore('autoTag', () => {
  // State
  const settings = ref<AutoTagSettings>({
    enabled: false,
    auto_tag_on_generation: false,
    preferred_backend: 'claude',
    min_confidence: 0.6,
    claude_api_key: undefined,
  })
  const modelStatus = ref<VisionModelStatus>({
    is_downloaded: false,
    download_progress: null,
    model_size: 0,
    model_size_display: '',
    error: null,
  })
  const isLoadingSettings = ref(false)
  const isTagging = ref(false)
  const isDownloading = ref(false)
  const lastError = ref<string | null>(null)
  const recentTaggingResults = ref<TaggingResult[]>([])

  // Computed
  const isClaudeAvailable = computed(() => {
    return !!settings.value.claude_api_key && settings.value.claude_api_key.length > 0
  })

  const isLocalAvailable = computed(() => {
    return modelStatus.value.is_downloaded
  })

  const effectiveBackend = computed((): TaggingBackend | null => {
    const preferred = settings.value.preferred_backend
    if (preferred === 'local' && isLocalAvailable.value) return 'local'
    if (preferred === 'claude' && isClaudeAvailable.value) return 'claude'
    // Fallback to other backend if preferred not available
    if (preferred === 'local' && isClaudeAvailable.value) return 'claude'
    if (preferred === 'claude' && isLocalAvailable.value) return 'local'
    return null
  })

  const canAutoTag = computed(() => {
    return settings.value.enabled && effectiveBackend.value !== null
  })

  const downloadProgressPercent = computed(() => {
    const progress = modelStatus.value.download_progress
    if (progress === null || modelStatus.value.model_size === 0) return 0
    return Math.round((progress / modelStatus.value.model_size) * 100)
  })

  // Listen for auto-tag events from backend
  const unlistenComplete = listen<{
    image_id: string
    tags: TagWithConfidence[]
    backend: TaggingBackend
  }>('auto-tag-complete', (event) => {
    const result: TaggingResult = {
      image_id: event.payload.image_id,
      tags: event.payload.tags,
      backend: event.payload.backend,
      success: true,
    }
    recentTaggingResults.value.unshift(result)
    // Keep only last 10 results
    if (recentTaggingResults.value.length > 10) {
      recentTaggingResults.value = recentTaggingResults.value.slice(0, 10)
    }
  })

  const unlistenFailed = listen<{
    image_id: string
    error: string
    backend: TaggingBackend
  }>('auto-tag-failed', (event) => {
    const result: TaggingResult = {
      image_id: event.payload.image_id,
      tags: [],
      backend: event.payload.backend,
      success: false,
      error: event.payload.error,
    }
    recentTaggingResults.value.unshift(result)
    if (recentTaggingResults.value.length > 10) {
      recentTaggingResults.value = recentTaggingResults.value.slice(0, 10)
    }
  })

  const unlistenDownloadProgress = listen<{
    bytes_downloaded: number
    total_bytes: number
  }>('vision-model-download-progress', (event) => {
    modelStatus.value.download_progress = event.payload.bytes_downloaded
    modelStatus.value.model_size = event.payload.total_bytes
  })

  // Cleanup listeners on store disposal
  onScopeDispose(async () => {
    const unlisten1 = await unlistenComplete
    unlisten1()
    const unlisten2 = await unlistenFailed
    unlisten2()
    const unlisten3 = await unlistenDownloadProgress
    unlisten3()
  })

  // Actions
  async function loadSettings(): Promise<void> {
    isLoadingSettings.value = true
    lastError.value = null
    try {
      const result = await invoke<AutoTagSettings>('get_auto_tag_settings')
      settings.value = result
    } catch (error) {
      console.error('Failed to load auto-tag settings:', error)
      lastError.value = 'Failed to load settings'
    } finally {
      isLoadingSettings.value = false
    }
  }

  async function saveSettings(newSettings: Partial<AutoTagSettings>): Promise<boolean> {
    lastError.value = null
    try {
      // Merge with existing settings
      const updated = { ...settings.value, ...newSettings }
      await invoke('update_auto_tag_settings', { settings: updated })
      settings.value = updated
      return true
    } catch (error) {
      console.error('Failed to save auto-tag settings:', error)
      lastError.value = 'Failed to save settings'
      return false
    }
  }

  async function checkModelStatus(): Promise<void> {
    try {
      const result = await invoke<VisionModelStatus>('check_vision_model_status')
      modelStatus.value = result
    } catch (error) {
      console.error('Failed to check vision model status:', error)
    }
  }

  async function downloadModel(): Promise<boolean> {
    if (isDownloading.value) return false

    isDownloading.value = true
    lastError.value = null
    try {
      await invoke('download_vision_model')
      await checkModelStatus()
      return true
    } catch (error) {
      console.error('Failed to download vision model:', error)
      lastError.value = `Download failed: ${error}`
      return false
    } finally {
      isDownloading.value = false
    }
  }

  async function clearLocks(): Promise<string> {
    try {
      const result = await invoke<string>('clear_vision_model_locks')
      return result
    } catch (error) {
      console.error('Failed to clear locks:', error)
      throw error
    }
  }

  async function tagImages(
    imageIds: string[],
    backend?: TaggingBackend
  ): Promise<TaggingResult[]> {
    if (isTagging.value || imageIds.length === 0) return []

    isTagging.value = true
    lastError.value = null
    try {
      const results = await invoke<TaggingResult[]>('auto_tag_images', {
        imageIds,
        backend: backend ?? null,
      })
      // Add to recent results
      for (const result of results) {
        recentTaggingResults.value.unshift(result)
      }
      if (recentTaggingResults.value.length > 10) {
        recentTaggingResults.value = recentTaggingResults.value.slice(0, 10)
      }
      return results
    } catch (error) {
      console.error('Failed to tag images:', error)
      lastError.value = `Tagging failed: ${error}`
      return []
    } finally {
      isTagging.value = false
    }
  }

  function clearRecentResults(): void {
    recentTaggingResults.value = []
  }

  return {
    // State
    settings,
    modelStatus,
    isLoadingSettings,
    isTagging,
    isDownloading,
    lastError,
    recentTaggingResults,
    // Computed
    isClaudeAvailable,
    isLocalAvailable,
    effectiveBackend,
    canAutoTag,
    downloadProgressPercent,
    // Actions
    loadSettings,
    saveSettings,
    checkModelStatus,
    downloadModel,
    clearLocks,
    tagImages,
    clearRecentResults,
  }
})
