import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Model, LoRA, LoraFileInfo, LoraConfig } from '@/types'

interface ModelAvailability {
  id: string
  name: string
  is_downloaded: boolean
  has_quantized: boolean
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

export const useModelsStore = defineStore('models', () => {
  // State
  const models = ref<Model[]>([
    {
      id: 'schnell',
      name: 'FLUX Schnell',
      type: 'flux-schnell',
      isDownloaded: false,
      isActive: true,
      createdAt: Date.now(),
      description: 'Fast generation (4 steps)',
      defaultSteps: 4,
      defaultGuidance: 1.0,
    },
    {
      id: 'dev',
      name: 'FLUX Dev',
      type: 'flux-dev',
      isDownloaded: false,
      isActive: false,
      createdAt: Date.now(),
      description: 'High quality (28+ steps)',
      defaultSteps: 28,
      defaultGuidance: 3.5,
    },
  ])

  const loras = ref<LoRA[]>([])

  const selectedModelId = ref<string>('schnell')

  // Getters
  const activeModel = computed(() =>
    models.value.find((m) => m.id === selectedModelId.value)
  )

  const activeLoras = computed(() => loras.value.filter((l) => l.isActive))

  const downloadedModels = computed(() =>
    models.value.filter((m) => m.isDownloaded)
  )

  // Actions
  function addModel(model: Model) {
    models.value.push(model)
  }

  function removeModel(id: string): boolean {
    const index = models.value.findIndex((m) => m.id === id)
    if (index !== -1) {
      models.value.splice(index, 1)
      return true
    }
    return false
  }

  function selectModel(id: string): boolean {
    const model = models.value.find((m) => m.id === id)
    if (model && model.isDownloaded) {
      // Deactivate all models
      models.value = models.value.map((m) => ({
        ...m,
        isActive: m.id === id,
      }))
      selectedModelId.value = id
      return true
    }
    return false
  }

  // ============ LoRA Backend Integration ============

  // Track loading state
  const lorasLoading = ref(false)
  const lorasError = ref<string | null>(null)

  // Load all LoRAs from backend
  async function loadLoras(): Promise<void> {
    lorasLoading.value = true
    lorasError.value = null
    try {
      const backendLoras = await invoke<BackendLoraInfo[]>('get_loras')
      // Map backend data, preserving frontend state for existing LoRAs
      loras.value = backendLoras.map((info) => {
        const existing = loras.value.find((l) => l.id === info.id)
        return mapLoraInfo(info, existing)
      })
    } catch (error) {
      lorasError.value = String(error)
      console.error('Failed to load LoRAs:', error)
    } finally {
      lorasLoading.value = false
    }
  }

  // Import a new LoRA from file
  async function importLora(
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
      loras.value.push(newLora)
      return newLora
    } catch (error) {
      console.error('Failed to import LoRA:', error)
      throw error
    }
  }

  // Remove a LoRA (deletes from disk via backend)
  async function removeLora(id: string): Promise<boolean> {
    try {
      await invoke('remove_lora', { id })
      const index = loras.value.findIndex((l) => l.id === id)
      if (index !== -1) {
        loras.value.splice(index, 1)
      }
      return true
    } catch (error) {
      console.error('Failed to remove LoRA:', error)
      throw error
    }
  }

  // Get file info before importing (preview)
  async function getLoraFileInfo(path: string): Promise<LoraFileInfo> {
    try {
      const info = await invoke<BackendLoraFileInfo>('get_lora_file_info', { path })
      return mapLoraFileInfo(info)
    } catch (error) {
      console.error('Failed to get LoRA file info:', error)
      throw error
    }
  }

  // ============ Local LoRA State Management ============

  function toggleLora(id: string): boolean {
    const index = loras.value.findIndex((l) => l.id === id)
    if (index !== -1) {
      loras.value[index] = {
        ...loras.value[index],
        isActive: !loras.value[index].isActive,
      }
      return true
    }
    return false
  }

  function updateLoraStrength(id: string, strength: number): boolean {
    // Clamp strength to valid range (0.0 to 2.0)
    const clampedStrength = Math.max(0, Math.min(2, strength))

    const index = loras.value.findIndex((l) => l.id === id)
    if (index !== -1) {
      loras.value[index] = {
        ...loras.value[index],
        strength: clampedStrength,
      }
      return true
    }
    return false
  }

  // Get active LoRAs as config for generation params
  function getActiveLoraConfigs(): LoraConfig[] {
    return loras.value
      .filter((l) => l.isActive)
      .map((l) => ({
        id: l.id,
        strength: l.strength,
      }))
  }

  // ============ Model Backend Integration ============

  // Fetch model availability from backend
  async function refreshModelAvailability(): Promise<void> {
    try {
      const availability = await invoke<ModelAvailability[]>('get_available_models')
      for (const avail of availability) {
        const model = models.value.find((m) => m.id === avail.id)
        if (model) {
          model.isDownloaded = avail.is_downloaded
        }
      }
    } catch (error) {
      console.error('Failed to refresh model availability:', error)
    }
  }

  return {
    // State
    models,
    loras,
    selectedModelId,
    lorasLoading,
    lorasError,
    // Getters
    activeModel,
    activeLoras,
    downloadedModels,
    // Model Actions
    addModel,
    removeModel,
    selectModel,
    refreshModelAvailability,
    // LoRA Actions (backend-connected)
    loadLoras,
    importLora,
    removeLora,
    getLoraFileInfo,
    // LoRA Actions (local state)
    toggleLora,
    updateLoraStrength,
    getActiveLoraConfigs,
  }
})
