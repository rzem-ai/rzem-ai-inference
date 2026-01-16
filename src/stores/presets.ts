import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { GenerationPreset } from '@/types'
import { useGenerationStore } from './generation'
import { useModelsStore } from './models'

export const usePresetsStore = defineStore('presets', () => {
  // State
  const presets = ref<GenerationPreset[]>([
    {
      id: 'default',
      name: 'Default (Flux Schnell)',
      mode: 'txt2img',
      steps: 4,
      cfgScale: 1.0,
      width: 1024,
      height: 1024,
      seed: -1,
      modelId: 'flux-schnell',
      createdAt: Date.now(),
      updatedAt: Date.now(),
    },
    {
      id: 'fast-draft',
      name: 'Fast Draft',
      mode: 'txt2img',
      steps: 2,
      cfgScale: 1.0,
      width: 512,
      height: 512,
      seed: -1,
      modelId: 'flux-schnell',
      createdAt: Date.now(),
      updatedAt: Date.now(),
    },
    {
      id: 'high-quality',
      name: 'High Quality',
      mode: 'txt2img',
      steps: 8,
      cfgScale: 1.5,
      width: 1024,
      height: 1024,
      seed: -1,
      modelId: 'flux-schnell',
      createdAt: Date.now(),
      updatedAt: Date.now(),
    },
  ])

  // Actions
  function savePreset(name: string) {
    const generationStore = useGenerationStore()
    const modelsStore = useModelsStore()

    const preset: GenerationPreset = {
      id: crypto.randomUUID(),
      name,
      mode: generationStore.currentParams.mode,
      prompt: generationStore.currentParams.prompt,
      negativePrompt: generationStore.currentParams.negativePrompt,
      steps: generationStore.currentParams.steps,
      cfgScale: generationStore.currentParams.cfgScale,
      width: generationStore.currentParams.width,
      height: generationStore.currentParams.height,
      seed: generationStore.currentParams.seed,
      modelId: modelsStore.selectedModelId,
      loraIds: JSON.stringify(
        modelsStore.activeLoras.map((l) => ({
          id: l.id,
          strength: l.strength,
        }))
      ),
      createdAt: Date.now(),
      updatedAt: Date.now(),
    }

    presets.value.push(preset)
  }

  function loadPreset(id: string) {
    const preset = presets.value.find((p) => p.id === id)
    if (!preset) return

    const generationStore = useGenerationStore()
    const modelsStore = useModelsStore()

    generationStore.currentParams.mode = preset.mode
    generationStore.currentParams.prompt = preset.prompt || ''
    generationStore.currentParams.negativePrompt = preset.negativePrompt || ''
    generationStore.currentParams.steps = preset.steps
    generationStore.currentParams.cfgScale = preset.cfgScale
    generationStore.currentParams.width = preset.width
    generationStore.currentParams.height = preset.height
    generationStore.currentParams.seed = preset.seed || -1

    if (preset.modelId) {
      modelsStore.selectModel(preset.modelId)
    }
  }

  function deletePreset(id: string) {
    const index = presets.value.findIndex((p) => p.id === id)
    if (index !== -1) {
      presets.value.splice(index, 1)
    }
  }

  return {
    // State
    presets,
    // Actions
    savePreset,
    loadPreset,
    deletePreset,
  }
})
