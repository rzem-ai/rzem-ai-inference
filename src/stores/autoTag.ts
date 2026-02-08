import { defineStore } from 'pinia'
import { invoke } from '@/utils/backend-bridge'
import { listen, type UnlistenFn } from '@/utils/backend-bridge'

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

export const useAutoTagStore = defineStore('autoTag', {
  state: () => ({
    settings: {
      enabled: false,
      auto_tag_on_generation: false,
      preferred_backend: 'claude',
      min_confidence: 0.6,
      claude_api_key: undefined,
    } as AutoTagSettings,
    modelStatus: {
      is_downloaded: false,
      download_progress: null,
      model_size: 0,
      model_size_display: '',
      error: null,
    } as VisionModelStatus,
    isLoadingSettings: false,
    isTagging: false,
    isDownloading: false,
    lastError: null as string | null,
    recentTaggingResults: [] as TaggingResult[],
    isInitialized: false,
    // Event unlisteners
    unlisteners: [] as UnlistenFn[],
  }),

  getters: {
    isClaudeAvailable(state): boolean {
      return !!state.settings.claude_api_key && state.settings.claude_api_key.length > 0
    },

    isLocalAvailable(state): boolean {
      return state.modelStatus.is_downloaded
    },

    effectiveBackend(state): TaggingBackend | null {
      const preferred = state.settings.preferred_backend
      if (preferred === 'local' && state.modelStatus.is_downloaded) return 'local'
      if (preferred === 'claude' && state.settings.claude_api_key) return 'claude'
      // Fallback to other backend if preferred not available
      if (preferred === 'local' && state.settings.claude_api_key) return 'claude'
      if (preferred === 'claude' && state.modelStatus.is_downloaded) return 'local'
      return null
    },

    canAutoTag(state): boolean {
      const backend = this.effectiveBackend
      return state.settings.enabled && backend !== null
    },

    downloadProgressPercent(state): number {
      const progress = state.modelStatus.download_progress
      if (progress === null || state.modelStatus.model_size === 0) return 0
      return Math.round((progress / state.modelStatus.model_size) * 100)
    },
  },

  actions: {
    // Standard initialization method
    async initialize(): Promise<void> {
      // Guard: only initialize once
      if (this.isInitialized) {
        return
      }

      try {
        // Load initial data
        await this.loadSettings()
        await this.checkModelStatus()

        // Initialize event listeners
        await this.initializeEventListeners()

        this.isInitialized = true
      } catch (err) {
        console.error('[AutoTagStore] Initialization failed:', err)
        throw err
      }
    },

    // Cleanup (called on app close, not navigation)
    cleanup() {
      this.cleanupEventListeners()
      this.isInitialized = false
    },

    // Initialize event listeners
    async initializeEventListeners() {
      if (this.unlisteners.length > 0) return; // Already initialized

      // Listen for auto-tag complete events
      const unlisten1 = await listen<{
        image_id: string
        tags: TagWithConfidence[]
        backend: TaggingBackend
      }>('auto-tag-complete', (payload) => {
        const result: TaggingResult = {
          image_id: payload.image_id,
          tags: payload.tags,
          backend: payload.backend,
          success: true,
        }
        this.recentTaggingResults.unshift(result)
        // Keep only last 10 results
        if (this.recentTaggingResults.length > 10) {
          this.recentTaggingResults = this.recentTaggingResults.slice(0, 10)
        }
      })
      this.unlisteners.push(unlisten1)

      // Listen for auto-tag failed events
      const unlisten2 = await listen<{
        image_id: string
        error: string
        backend: TaggingBackend
      }>('auto-tag-failed', (payload) => {
        const result: TaggingResult = {
          image_id: payload.image_id,
          tags: [],
          backend: payload.backend,
          success: false,
          error: payload.error,
        }
        this.recentTaggingResults.unshift(result)
        if (this.recentTaggingResults.length > 10) {
          this.recentTaggingResults = this.recentTaggingResults.slice(0, 10)
        }
      })
      this.unlisteners.push(unlisten2)

      // Listen for vision model download progress
      const unlisten3 = await listen<{
        bytes_downloaded: number
        total_bytes: number
      }>('vision-model-download-progress', (payload) => {
        this.modelStatus.download_progress = payload.bytes_downloaded
        this.modelStatus.model_size = payload.total_bytes
      })
      this.unlisteners.push(unlisten3)
    },

    // Cleanup event listeners
    cleanupEventListeners() {
      this.unlisteners.forEach(fn => fn())
      this.unlisteners = []
    },

    async loadSettings(): Promise<void> {
      this.isLoadingSettings = true
      this.lastError = null
      try {
        const result = await invoke<AutoTagSettings>('get_auto_tag_settings')
        this.settings = result
      } catch (error) {
        console.error('Failed to load auto-tag settings:', error)
        this.lastError = 'Failed to load settings'
      } finally {
        this.isLoadingSettings = false
      }
    },

    async saveSettings(newSettings: Partial<AutoTagSettings>): Promise<boolean> {
      this.lastError = null
      try {
        // Merge with existing settings
        const updated = { ...this.settings, ...newSettings }
        await invoke('update_auto_tag_settings', { settings: updated })
        this.settings = updated
        return true
      } catch (error) {
        console.error('Failed to save auto-tag settings:', error)
        this.lastError = 'Failed to save settings'
        return false
      }
    },

    async checkModelStatus(): Promise<void> {
      try {
        const result = await invoke<VisionModelStatus>('check_vision_model_status')
        this.modelStatus = result
      } catch (error) {
        console.error('Failed to check vision model status:', error)
      }
    },

    async downloadModel(): Promise<boolean> {
      if (this.isDownloading) return false

      this.isDownloading = true
      this.lastError = null
      try {
        await invoke('download_vision_model')
        await this.checkModelStatus()
        return true
      } catch (error) {
        console.error('Failed to download vision model:', error)
        this.lastError = `Download failed: ${error}`
        return false
      } finally {
        this.isDownloading = false
      }
    },

    async clearLocks(): Promise<string> {
      try {
        const result = await invoke<string>('clear_vision_model_locks')
        return result
      } catch (error) {
        console.error('Failed to clear locks:', error)
        throw error
      }
    },

    async tagImages(
      imageIds: string[],
      backend?: TaggingBackend
    ): Promise<TaggingResult[]> {
      if (this.isTagging || imageIds.length === 0) return []

      this.isTagging = true
      this.lastError = null
      try {
        const results = await invoke<TaggingResult[]>('auto_tag_images', {
          imageIds,
          backend: backend ?? null,
        })
        // Add to recent results
        for (const result of results) {
          this.recentTaggingResults.unshift(result)
        }
        if (this.recentTaggingResults.length > 10) {
          this.recentTaggingResults = this.recentTaggingResults.slice(0, 10)
        }
        return results
      } catch (error) {
        console.error('Failed to tag images:', error)
        this.lastError = `Tagging failed: ${error}`
        return []
      } finally {
        this.isTagging = false
      }
    },

    clearRecentResults(): void {
      this.recentTaggingResults = []
    },
  },
})
