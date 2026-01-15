export interface GenerationJob {
  id: string
  prompt: string
  status: 'Queued' | 'Running' | 'Completed' | 'Failed'
}

export interface GenerationParams {
  prompt: string
  negativePrompt?: string
  steps: number
  cfgScale: number
  width: number
  height: number
  seed: number
  model: string
}
