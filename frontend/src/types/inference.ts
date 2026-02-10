/** Inference-specific types matching the Python backend. */

export type ApiResponse<T = {}> = { status: "success" | "error"; message?: string } & T;

export type TransformerType = "flux1_dev" | "flux2_dev" | "z_image" | "qwen_image";

export type EventType =
  | "job_queued"
  | "job_started"
  | "job_progress"
  | "job_completed"
  | "job_failed"
  | "job_cancelled"
  | "model_loading"
  | "model_loaded"
  | "model_unloaded";

export interface InferenceEvent {
  type: EventType;
  data: Record<string, any>;
}

export type BundleTier = "performance" | "balanced" | "quality";

export interface ModelBundle {
  id: string;
  label: string;
  description: string;
  transformer_type: TransformerType;
  tier: BundleTier;
  transformer_model: string;
  vae_model: string;
  clip_tokenizer?: string;
  clip_encoder?: string;
  t5_tokenizer?: string;
  t5_encoder?: string;
  qwen3_tokenizer?: string;
  qwen3_encoder?: string;
  vram_estimate_gb: number;
  is_default: boolean;
}

export interface LoraParam {
  model_file: string;
  strength: number;
}

export interface SubmitJobParams {
  prompt: string;
  transformer_model: string;
  transformer_type: TransformerType;
  vae_model: string;
  steps: number;
  cfg_scale: number;
  width: number;
  height: number;
  seed: number;
  sampler: string;
  scheduler: string;
  loras: LoraParam[];
  // Optional text encoder overrides
  clip_tokenizer?: string;
  clip_encoder?: string;
  t5_tokenizer?: string;
  t5_encoder?: string;
  qwen3_tokenizer?: string;
  qwen3_encoder?: string;
}

export interface GeneratedImage {
  jobId: string;
  imagePath: string;
  dataUrl?: string;
  seed: number;
  timestamp: number;
}
