import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Model, LoRA } from '@/types'

interface ModelAvailability {
  id: string
  name: string
  is_downloaded: boolean
  has_quantized: boolean
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

  function addLora(lora: LoRA) {
    loras.value.push(lora)
  }

  function removeLora(id: string): boolean {
    const index = loras.value.findIndex((l) => l.id === id)
    if (index !== -1) {
      loras.value.splice(index, 1)
      return true
    }
    return false
  }

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
    // Getters
    activeModel,
    activeLoras,
    downloadedModels,
    // Actions
    addModel,
    removeModel,
    selectModel,
    addLora,
    removeLora,
    toggleLora,
    updateLoraStrength,
    refreshModelAvailability,
  }
})
