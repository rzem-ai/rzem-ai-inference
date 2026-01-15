export interface GenerationJob {
  id: string
  prompt: string
  status: 'Queued' | 'Running' | 'Completed' | 'Failed'
}

export type GenerationMode = 'txt2img' | 'img2img' | 'inpainting'

export interface GenerationParams {
  mode: GenerationMode
  prompt: string
  negativePrompt?: string
  steps: number
  cfgScale: number
  width: number
  height: number
  seed: number
  model: string
  batchSize?: number
  // For img2img/inpainting
  sourceImage?: string
  strength?: number
  maskImage?: string
}

export interface GenerationProgress {
  jobId: string
  step: number
  totalSteps: number
  previewImage?: string
  status: 'Queued' | 'Preparing' | 'Generating' | 'Saving' | 'Completed' | 'Failed'
  error?: string
}

export interface GeneratedImage {
  id: string
  jobId: string
  filePath: string
  thumbnailPath?: string
  params: GenerationParams
  createdAt: number
}
