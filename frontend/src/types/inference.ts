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
  steps: number;
  cfg_scale: number;
  sampler: string;
  scheduler: string;
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
  bundle_id?: string;
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

// ── Gallery types (matching database schema) ──

export type ImageStatus = "completed" | "failed";

export interface GalleryImage {
  id: string;
  file_path: string;
  thumbnail_path: string | null;
  prompt: string;
  negative_prompt: string | null;
  width: number;
  height: number;
  file_size: number | null;
  steps: number;
  cfg_scale: number;
  seed: number;
  bundle_id: string | null;
  model_config: string | null;
  loras: string | null;
  favorite: number;
  generation_time_ms: number | null;
  status: ImageStatus;
  created_at: number;
  updated_at: number;
}

export interface Folder {
  id: string;
  name: string;
  parent_id: string | null;
  color: string | null;
  icon: string | null;
  sort_order: number;
  created_at: number;
  updated_at: number;
}

export interface Tag {
  id: number;
  name: string;
  color: string | null;
  category: string | null;
}

// ── Style types (matching database schema) ──

export interface Style {
  id: string;
  name: string;
  description: string | null;
  prompt_template: string;
  negative_prompt: string | null;
  default_strength: number;
  strength_min: number;
  strength_max: number;
  category: string | null;
  thumbnail_path: string | null;
  is_favorite: number;
  usage_count: number;
  created_at: number;
  updated_at: number;
}

export interface StyleLoRA {
  id: number;
  style_id: string;
  lora_id: string;
  strength: number;
  priority: number;
  lora_name: string;
  lora_path: string;
}

export interface LoRA {
  id: string;
  name: string;
  path: string;
  trigger_words: string | null;
  base_model: string | null;
  size_bytes: number | null;
  strength: number;
  is_active: number;
  created_at: number;
  metadata: string | null;
}
