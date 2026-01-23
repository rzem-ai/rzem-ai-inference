export interface GenerationJob {
  id: string
  prompt: string
  status: 'Queued' | 'Running' | 'Completed' | 'Failed'
}

export type GenerationMode = 'txt2img' | 'img2img' | 'inpainting'

// Sampler types supported by FLUX
export type Sampler = 'euler' | 'euler_a' | 'dpm_pp_2m'
// Scheduler types supported by FLUX
export type Scheduler = 'normal' | 'simple' | 'karras' | 'exponential'

// LoRA configuration for generation (just id + strength)
export interface LoraConfig {
  id: string
  strength: number
}

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
  // LoRA adapters to apply
  loras?: LoraConfig[]
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

// LoRA metadata from backend (matches LoraInfo)
export interface LoRA {
  id: string
  name: string
  path: string
  triggerWords?: string
  baseModel?: string
  sizeBytes: number
  createdAt: number
  metadata?: Record<string, string>
  // Frontend-only state (not stored in backend)
  strength: number
  isActive: boolean
}

// Preview info for a LoRA file before import
export interface LoraFileInfo {
  path: string
  sizeBytes: number
  weightCount: number
  rank?: number
  totalParams: number
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
