import type { LoraConfig } from './lora';

export type GenerationMode = 'txt2img' | 'img2img' | 'inpainting';

/**
 * Job status matches backend Rust enum JobStatus with serde lowercase serialization
 * Backend: JobStatus::Pending -> "pending", JobStatus::Running -> "running", etc.
 */
export type JobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

/**
 * Pipeline stages during generation
 */
export type PipelineStage = 'loading_models' | 'encoding_t5' | 'encoding_clip' | 'denoising' | 'decoding_vae' | 'encoding_png';

export type SamplerType = 'euler' | 'euler_a' | 'dpm_pp_2m';
export type SchedulerType = 'normal' | 'simple' | 'karras' | 'exponential';

// Sampler types supported by FLUX
export type Sampler = 'euler' | 'euler_a' | 'dpm_pp_2m';

// Scheduler types supported by FLUX
export type Scheduler = 'normal' | 'simple' | 'karras' | 'exponential';

export interface GeneratedImage {
  id: string;
  jobId: string;
  filePath: string;
  thumbnailPath?: string;
  params: GenerationParams;
  createdAt: number;
}

// export interface GenerationJob {
//   id: string;
//   prompt: string;
//   status: 'Queued' | 'Running' | 'Completed' | 'Failed';
// }

export interface GenerationJob {
  id: string;
  params: GenerationParams;
  status: JobStatus;
  /**
   * Progress value from backend (f32: 0.0-1.0)
   */
  progress: number;
  /**
   * Current pipeline stage (from job-progress events)
   */
  currentStage?: PipelineStage;
  /**
   * Human-readable status message
   */
  statusMessage?: string;
  /**
   * Current denoising step (when in denoising stage)
   */
  currentStep?: number;
  /**
   * Total denoising steps
   */
  totalSteps?: number;
  created_at: number;
  started_at?: number;
  completed_at?: number;
  result_path?: string;
  error?: string;
  stats?: GenerationStats;
  /**
   * Preview image data (base64-encoded JPEG)
   * Available during generation before final image completes
   */
  previewData?: string;
}

export interface GenerationParams {
  mode: GenerationMode;
  prompt: string;
  negative_prompt?: string;
  steps: number;
  cfg_scale: number;
  sampler: Sampler;
  scheduler: Scheduler;
  width: number;
  height: number;
  seed: number;
  //model: string
  batch_size?: number;

  // For img2img/inpainting
  source_image?: string;
  strength?: number;
  mask_image?: string;

  // LoRA adapters to apply
  loras?: LoraConfig[];

  // Legacy model type hint
  model_type?: string;

  // Model/bundle selection for generation
  bundle_id?: string;
  model_component_id?: string;
  t5_component_id?: string;
  clip_component_id?: string;
  vae_component_id?: string;
}

export interface GenerationPreset {
  id: string;
  name: string;
  mode: GenerationMode;
  prompt?: string;
  negative_prompt?: string;
  steps: number;
  cfg_scale: number;
  sampler?: Sampler;
  scheduler?: Scheduler;
  width: number;
  height: number;
  seed?: number;
  modelId?: string;
  loraIds?: string; // JSON array of LoRA IDs with strengths
  createdAt: number;
  updatedAt: number;
}

export interface GenerationProgress {
  jobId: string;
  step: number;
  totalSteps: number;
  previewImage?: string;
  status: 'Queued' | 'Preparing' | 'Generating' | 'Saving' | 'Completed' | 'Failed';
  error?: string;
}

/**
 * Generation statistics from the backend
 */
export interface GenerationStats {
  model_load_ms?: number;
  t5_load_ms?: number;
  clip_load_ms?: number;
  vae_load_ms?: number;
  flux_load_ms?: number;
  t5_encode_ms: number;
  clip_encode_ms: number;
  denoise_ms: number;
  vae_decode_ms: number;
  png_encode_ms: number;
  total_ms: number;
  t5_embedding_shape: number[];
  clip_embedding_shape: number[];
  latent_shape: number[];
  image_shape: number[];
  steps: number;
  model_type: string;
}
