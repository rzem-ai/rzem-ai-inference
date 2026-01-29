import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

// ===== TypeScript Interfaces (mapped from Rust) =====

export interface ComponentInfo {
  id: string
  role: string
  name: string
  componentType: string
  format: string
  filePath: string
  fileSize: number
  quantization?: string
  vramMb?: number
  isAvailable: boolean
  isRequired: boolean
  priority: number
}

export interface BundleInfo {
  id: string
  name: string
  description?: string
  bundleType: 'auto_discovered' | 'user_created' | 'system'
  modelFamily: string
  defaultSteps?: number
  defaultGuidance?: number
  stepMin?: number
  stepMax?: number
  totalVramMb?: number
  isComplete: boolean
  isActive: boolean
  createdAt: number
  updatedAt: number
  validationErrors?: string
  components: ComponentInfo[]
}

export interface ComponentRecord {
  id: string
  componentType: string
  format: string
  filePath: string
  fileSize: number
  fileHash?: string
  name: string
  repoId?: string
  repoSnapshot?: string
  architecture?: string
  quantization?: string
  supportsLoras: boolean
  isSharded: boolean
  shardCount?: number
  vramMb?: number
  discoveredAt: number
  lastVerifiedAt?: number
  isAvailable: boolean
  metadata?: any
}

export interface ScanResult {
  componentsFound: number
  componentsAdded: number
  bundlesCreated: number
}

export interface CreateBundleParams {
  name: string
  description?: string
  modelFamily: string
  componentMap: Record<string, string>
}

// ===== Store Definition (Options API) =====

export const useBundlesStore = defineStore('bundles', {
  state: () => ({
    bundles: [] as BundleInfo[],
    activeBundle: null as BundleInfo | null,
    availableComponents: [] as ComponentRecord[],
    isLoading: false,
    isScanning: false,
    lastScanResult: null as ScanResult | null,
    error: null as string | null,
  }),

  getters: {
    // Get bundles by family
    fluxBundles(state): BundleInfo[] {
      return state.bundles.filter(b => b.modelFamily === 'flux')
    },

    zindexBundles(state): BundleInfo[] {
      return state.bundles.filter(b => b.modelFamily === 'zindex')
    },

    // Get bundles by type
    autoDiscoveredBundles(state): BundleInfo[] {
      return state.bundles.filter(b => b.bundleType === 'auto_discovered')
    },

    userCreatedBundles(state): BundleInfo[] {
      return state.bundles.filter(b => b.bundleType === 'user_created')
    },

    // Get complete bundles only
    completeBundles(state): BundleInfo[] {
      return state.bundles.filter(b => b.isComplete)
    },

    // Get components by type (deduplicated by hash)
    transformerComponents(state): ComponentRecord[] {
      return this.deduplicateComponents(
        state.availableComponents.filter(c => c.componentType === 'transformer')
      )
    },

    t5Components(state): ComponentRecord[] {
      return this.deduplicateComponents(
        state.availableComponents.filter(c => c.componentType === 't5_encoder')
      )
    },

    clipComponents(state): ComponentRecord[] {
      return this.deduplicateComponents(
        state.availableComponents.filter(c => c.componentType === 'clip_encoder')
      )
    },

    vaeComponents(state): ComponentRecord[] {
      return this.deduplicateComponents(
        state.availableComponents.filter(c => c.componentType === 'vae')
      )
    },

    // Deduplicate components by file hash
    // For components with same hash, prefer the first one (usually from primary repo)
    deduplicateComponents: () => (components: ComponentRecord[]) => {
      const seen = new Map<string, ComponentRecord>()

      for (const comp of components) {
        // Skip components without hash (shouldn't happen but be safe)
        if (!comp.fileHash) {
          seen.set(comp.id, comp)
          continue
        }

        // If we've seen this hash before, skip this component
        if (seen.has(comp.fileHash)) {
          continue
        }

        // First time seeing this hash, keep it
        seen.set(comp.fileHash, comp)
      }

      return Array.from(seen.values())
    },

    // Get total VRAM formatted
    formatVram: () => (vramMb?: number) => {
      if (!vramMb) return 'Unknown'
      if (vramMb >= 1000) {
        return `${(vramMb / 1000).toFixed(1)} GB`
      }
      return `${vramMb} MB`
    },
  },

  actions: {
    // Load all bundles from database
    async loadBundles() {
      this.isLoading = true
      this.error = null

      try {
        this.bundles = await invoke<BundleInfo[]>('get_all_bundles')
        this.activeBundle = this.bundles.find(b => b.isActive) || null
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err)
        console.error('Failed to load bundles:', err)
      } finally {
        this.isLoading = false
      }
    },

    // Load available components
    async loadComponents() {
      this.isLoading = true
      this.error = null

      try {
        this.availableComponents = await invoke<ComponentRecord[]>('get_available_components')
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err)
        console.error('Failed to load components:', err)
      } finally {
        this.isLoading = false
      }
    },

    // Scan HuggingFace cache for models
    async scanModels() {
      this.isScanning = true
      this.error = null

      try {
        this.lastScanResult = await invoke<ScanResult>('scan_and_discover_models')

        // Reload bundles and components after scan
        await Promise.all([
          this.loadBundles(),
          this.loadComponents(),
        ])

        return this.lastScanResult
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err)
        console.error('Failed to scan models:', err)
        throw err
      } finally {
        this.isScanning = false
      }
    },

    // Get a specific bundle
    async getBundle(bundleId: string) {
      this.error = null

      try {
        return await invoke<BundleInfo>('get_bundle', { bundleId })
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err)
        console.error('Failed to get bundle:', err)
        throw err
      }
    },

    // Create a new bundle
    async createBundle(params: CreateBundleParams) {
      this.error = null

      try {
        const bundleId = await invoke<string>('create_bundle', {
          name: params.name,
          description: params.description || null,
          modelFamily: params.modelFamily,
          componentMap: params.componentMap,
        })

        await this.loadBundles()
        return bundleId
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err)
        console.error('Failed to create bundle:', err)
        throw err
      }
    },

    // Update bundle metadata
    async updateBundle(bundleId: string, name?: string, description?: string) {
      this.error = null

      try {
        await invoke('update_bundle', {
          bundleId,
          name: name || null,
          description: description || null,
        })

        await this.loadBundles()
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err)
        console.error('Failed to update bundle:', err)
        throw err
      }
    },

    // Delete a bundle
    async deleteBundle(bundleId: string) {
      this.error = null

      try {
        await invoke('delete_bundle', { bundleId })
        await this.loadBundles()
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err)
        console.error('Failed to delete bundle:', err)
        throw err
      }
    },

    // Set active bundle
    async setActiveBundle(bundleId: string) {
      this.error = null

      try {
        await invoke('set_active_bundle', { bundleId })
        await this.loadBundles()
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err)
        console.error('Failed to set active bundle:', err)
        throw err
      }
    },

    // Initialize store - call from component on mount
    async initialize() {
      await Promise.all([
        this.loadBundles(),
        this.loadComponents(),
      ])
    },

    // Clear error
    clearError() {
      this.error = null
    },
  },
})
