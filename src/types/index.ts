export interface GenerationJob {
  id: string
  prompt: string
  status: 'Queued' | 'Running' | 'Completed' | 'Failed'
}

export type GenerationMode = 'txt2img' | 'img2img' | 'inpainting'

export type Sampler = 'euler' | 'euler_a' | 'heun' | 'dpm_2' | 'dpm_2_a' | 'lms' | 'dpmpp_2m' | 'dpmpp_2s_a' | 'dpmpp_sde'
export type Scheduler = 'normal' | 'karras' | 'exponential' | 'sgm_uniform' | 'simple' | 'ddim_uniform' | 'beta'

export interface GenerationParams {
  mode: GenerationMode
  prompt: string
  negativePrompt?: string
  steps: number
  cfgScale: number
  sampler: Sampler
  scheduler: Scheduler
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

export interface Model {
  id: string
  name: string
  type: 'flux-schnell' | 'flux-dev' | 'flux-pro' | 'sdxl' | 'sd15'
  path?: string
  sizeBytes?: number
  isDownloaded: boolean
  isActive: boolean
  createdAt: number
  lastUsedAt?: number
  metadata?: Record<string, any>
  description?: string
  defaultSteps?: number
  defaultGuidance?: number
}

export interface LoRA {
  id: string
  name: string
  path: string
  triggerWords?: string
  baseModel?: string
  sizeBytes?: number
  strength: number
  isActive: boolean
  createdAt: number
  metadata?: Record<string, any>
}

export interface GenerationPreset {
  id: string
  name: string
  mode: GenerationMode
  prompt?: string
  negativePrompt?: string
  steps: number
  cfgScale: number
  sampler?: Sampler
  scheduler?: Scheduler
  width: number
  height: number
  seed?: number
  modelId?: string
  loraIds?: string // JSON array of LoRA IDs with strengths
  createdAt: number
  updatedAt: number
}
