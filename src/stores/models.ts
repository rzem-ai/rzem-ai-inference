import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { Model, LoRA, LoraFileInfo, LoraConfig } from '@/types'

interface ModelAvailability {
  id: string
  name: string
  is_downloaded: boolean
  has_quantized: boolean
}

// Backend ModelRecord structure (camelCase from Rust serde)
interface BackendModelRecord {
  id: string
  name: string
  description: string
  type: string
  category: string
  categoryLabel: string
  path?: string
  sizeEstimate: string
  sizeBytes?: number
  format: string
  isDownloaded: boolean
  isActive: boolean
  source: string
  license: string
  docsUrl?: string
  tags: string[]
  icon: string
  iconClass: string
  defaultSettings?: any
  features?: string[]
  notes?: string
  hasQuantized: boolean
  quantizedDownloaded: boolean
  supportsLoras: boolean
  createdAt: number
  lastUsedAt?: number
  repoId?: string
  transformerFilename?: string
  quantizedFilename?: string
  quantizedRepos?: any
  stepMin?: number
  stepMax?: number
  vramFull?: number
  vramQuantized?: number
  modelFamily?: string
  componentType?: string
}

// Backend LoraInfo structure (snake_case from Rust)
interface BackendLoraInfo {
  id: string
  name: string
  path: string
  trigger_words?: string
  base_model?: string
  size_bytes: number
  created_at: number
  metadata?: Record<string, string>
}

// Backend LoraFileInfo structure
interface BackendLoraFileInfo {
  path: string
  size_bytes: number
  weight_count: number
  rank?: number
  total_params: number
}

// Convert backend ModelRecord to frontend Model
function mapModelRecord(record: BackendModelRecord): Model {
  // Infer type from model ID or family
  let type: Model['type'] = 'flux-schnell'
  if (record.id.includes('dev')) {
    type = 'flux-dev'
  } else if (record.id.includes('schnell')) {
    type = 'flux-schnell'
  }

  return {
    id: record.id,
    name: record.name,
    type,
    path: record.path,
    sizeBytes: record.sizeBytes,
    isDownloaded: record.isDownloaded,
    isActive: record.isActive,
    createdAt: record.createdAt,
    lastUsedAt: record.lastUsedAt,
    metadata: {
      description: record.description,
      category: record.category,
      format: record.format,
      source: record.source,
      license: record.license,
      docsUrl: record.docsUrl,
      repoId: record.repoId,
      supportsLoras: record.supportsLoras,
      vramFull: record.vramFull,
      vramQuantized: record.vramQuantized,
    },
    description: record.description,
    defaultSteps: record.stepMin || record.defaultSettings?.steps || 4,
    defaultGuidance: record.defaultSettings?.guidance || 3.5,
  }
}

// Convert backend LoraInfo to frontend LoRA
function mapLoraInfo(info: BackendLoraInfo, existingLora?: LoRA): LoRA {
  return {
    id: info.id,
    name: info.name,
    path: info.path,
    triggerWords: info.trigger_words,
    baseModel: info.base_model,
    sizeBytes: info.size_bytes,
    createdAt: info.created_at,
    metadata: info.metadata,
    // Preserve existing frontend state or use defaults
    strength: existingLora?.strength ?? 1.0,
    isActive: existingLora?.isActive ?? false,
  }
}

// Convert backend LoraFileInfo to frontend
function mapLoraFileInfo(info: BackendLoraFileInfo): LoraFileInfo {
  return {
    path: info.path,
    sizeBytes: info.size_bytes,
    weightCount: info.weight_count,
    rank: info.rank,
    totalParams: info.total_params,
  }
}

export const useModelsStore = defineStore('models', {
  state: () => ({
    models: [] as Model[],
    loras: [] as LoRA[],
    selectedModelId: 'dev',
    lorasLoading: false,
    lorasError: null as string | null,
    modelsLoading: false,
    modelsError: null as string | null,
  }),

  getters: {
    activeModel(state): Model | undefined {
      return state.models.find((m) => m.id === state.selectedModelId)
    },

    activeLoras(state): LoRA[] {
      return state.loras.filter((l) => l.isActive)
    },

    downloadedModels(state): Model[] {
      return state.models.filter((m) => m.isDownloaded)
    },
  },

  actions: {
    // ============ Model Backend Integration ============

    // Load all models from database
    async loadModels(): Promise<void> {
      this.modelsLoading = true
      this.modelsError = null
      try {
        const backendModels = await invoke<BackendModelRecord[]>('get_all_models')

        // Filter to only generation models (transformers)
        const generationModels = backendModels.filter(
          (m) => m.category === 'generation' && m.componentType === 'transformer'
        )

        this.models = generationModels.map(mapModelRecord)

        // Set default selected model if none selected or selected model not available
        if (!this.selectedModelId || !this.models.find((m) => m.id === this.selectedModelId)) {
          // Prefer LoRA-compatible models, then downloaded models
          const loraCompatible = this.models.find((m) => m.isDownloaded && m.metadata?.supportsLoras)
          const anyDownloaded = this.models.find((m) => m.isDownloaded)
          this.selectedModelId = loraCompatible?.id || anyDownloaded?.id || this.models[0]?.id || 'dev'
        }
      } catch (error) {
        this.modelsError = String(error)
        console.error('Failed to load models:', error)
      } finally {
        this.modelsLoading = false
      }
    },

    addModel(model: Model) {
      this.models.push(model)
    },

    removeModel(id: string): boolean {
      const index = this.models.findIndex((m) => m.id === id)
      if (index !== -1) {
        this.models.splice(index, 1)
        return true
      }
      return false
    },

    selectModel(id: string): boolean {
      const model = this.models.find((m) => m.id === id)
      if (model && model.isDownloaded) {
        // Deactivate all models
        this.models = this.models.map((m) => ({
          ...m,
          isActive: m.id === id,
        }))
        this.selectedModelId = id
        return true
      }
      return false
    },

    // ============ LoRA Backend Integration ============

    // Load all LoRAs from backend
    async loadLoras(): Promise<void> {
      this.lorasLoading = true
      this.lorasError = null
      try {
        const backendLoras = await invoke<BackendLoraInfo[]>('get_loras')
        // Map backend data, preserving frontend state for existing LoRAs
        this.loras = backendLoras.map((info) => {
          const existing = this.loras.find((l) => l.id === info.id)
          return mapLoraInfo(info, existing)
        })
      } catch (error) {
        this.lorasError = String(error)
        console.error('Failed to load LoRAs:', error)
      } finally {
        this.lorasLoading = false
      }
    },

    // Import a new LoRA from file
    async importLora(
      sourcePath: string,
      name: string,
      triggerWords?: string
    ): Promise<LoRA | null> {
      try {
        const info = await invoke<BackendLoraInfo>('import_lora', {
          sourcePath,
          name,
          triggerWords: triggerWords || null,
        })
        const newLora = mapLoraInfo(info)
        this.loras.push(newLora)
        return newLora
      } catch (error) {
        console.error('Failed to import LoRA:', error)
        throw error
      }
    },

    // Remove a LoRA (deletes from disk via backend)
    async removeLora(id: string): Promise<boolean> {
      try {
        await invoke('remove_lora', { id })
        const index = this.loras.findIndex((l) => l.id === id)
        if (index !== -1) {
          this.loras.splice(index, 1)
        }
        return true
      } catch (error) {
        console.error('Failed to remove LoRA:', error)
        throw error
      }
    },

    // Get file info before importing (preview)
    async getLoraFileInfo(path: string): Promise<LoraFileInfo> {
      try {
        const info = await invoke<BackendLoraFileInfo>('get_lora_file_info', { path })
        return mapLoraFileInfo(info)
      } catch (error) {
        console.error('Failed to get LoRA file info:', error)
        throw error
      }
    },

    // ============ Local LoRA State Management ============

    toggleLora(id: string): boolean {
      const lora = this.loras.find((l) => l.id === id)
      if (lora) {
        lora.isActive = !lora.isActive
        return true
      }
      return false
    },

    updateLoraStrength(id: string, strength: number): boolean {
      const lora = this.loras.find((l) => l.id === id)
      if (lora) {
        // Clamp strength to valid range (0.0 to 2.0)
        lora.strength = Math.max(0, Math.min(2, strength))
        return true
      }
      return false
    },

    // Get active LoRAs as config for generation params
    getActiveLoraConfigs(): LoraConfig[] {
      return this.loras
        .filter((l) => l.isActive)
        .map((l) => ({
          id: l.id,
          strength: l.strength,
        }))
    },

    // ============ Model Backend Integration ============

    // Fetch model availability from backend
    async refreshModelAvailability(): Promise<void> {
      try {
        const availability = await invoke<ModelAvailability[]>('get_available_models')
        for (const avail of availability) {
          const model = this.models.find((m) => m.id === avail.id)
          if (model) {
            model.isDownloaded = avail.is_downloaded
          }
        }
      } catch (error) {
        console.error('Failed to refresh model availability:', error)
      }
    },
  },
})
