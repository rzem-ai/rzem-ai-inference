import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Model, LoRA } from '@/types'

export const useModelsStore = defineStore('models', () => {
  // State
  const models = ref<Model[]>([
    {
      id: 'flux-schnell',
      name: 'Flux Schnell',
      type: 'flux-schnell',
      isDownloaded: true, // Stub model is "downloaded"
      isActive: true,
      createdAt: Date.now(),
    },
  ])

  const loras = ref<LoRA[]>([])

  const selectedModelId = ref<string>('flux-schnell')

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

  function removeModel(id: string) {
    const index = models.value.findIndex((m) => m.id === id)
    if (index !== -1) {
      models.value.splice(index, 1)
    }
  }

  function selectModel(id: string) {
    selectedModelId.value = id
  }

  function addLora(lora: LoRA) {
    loras.value.push(lora)
  }

  function removeLora(id: string) {
    const index = loras.value.findIndex((l) => l.id === id)
    if (index !== -1) {
      loras.value.splice(index, 1)
    }
  }

  function toggleLora(id: string) {
    const lora = loras.value.find((l) => l.id === id)
    if (lora) {
      lora.isActive = !lora.isActive
    }
  }

  function updateLoraStrength(id: string, strength: number) {
    const lora = loras.value.find((l) => l.id === id)
    if (lora) {
      lora.strength = strength
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
  }
})
