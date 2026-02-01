export interface GenerationJob {
  id: string;
  prompt: string;
  status: 'Queued' | 'Running' | 'Completed' | 'Failed';
}

export type GenerationMode = 'txt2img' | 'img2img' | 'inpainting';

// Sampler types supported by FLUX
export type Sampler = 'euler' | 'euler_a' | 'dpm_pp_2m';
// Scheduler types supported by FLUX
export type Scheduler = 'normal' | 'simple' | 'karras' | 'exponential';

export interface GenerationParams {
  mode: GenerationMode;
  prompt: string;
  negativePrompt?: string;
  steps: number;
  cfgScale: number;
  sampler: Sampler;
  scheduler: Scheduler;
  width: number;
  height: number;
  seed: number;
  //model: string
  batchSize?: number;

  // For img2img/inpainting
  sourceImage?: string;
  strength?: number;
  maskImage?: string;

  // LoRA adapters to apply
  loras?: LoraConfig[];

  // Legacy model type hint
  modelType?: string;

  // Model/bundle selection for generation
  bundleId?: string;
  modelComponentId?: string;
  t5ComponentId?: string;
  clipComponentId?: string;
  vaeComponentId?: string;
}

export interface GenerationProgress {
  jobId: string;
  step: number;
  totalSteps: number;
  previewImage?: string;
  status: 'Queued' | 'Preparing' | 'Generating' | 'Saving' | 'Completed' | 'Failed';
  error?: string;
}

export interface GeneratedImage {
  id: string;
  jobId: string;
  filePath: string;
  thumbnailPath?: string;
  params: GenerationParams;
  createdAt: number;
}

export interface GenerationPreset {
  id: string;
  name: string;
  mode: GenerationMode;
  prompt?: string;
  negativePrompt?: string;
  steps: number;
  cfgScale: number;
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

// ========== LoRA Types ==========

export interface LoraConfig {
  id: string;
  strength: number;
}

export interface LoRA {
  id: string;
  name: string;
  path: string;
  triggerWords?: string;
  baseModel?: string;
  sizeBytes: number;
  createdAt: number;
  metadata?: Record<string, string>;
  strength: number;
  isActive: boolean;
}

export interface LoraFileInfo {
  path: string;
  sizeBytes: number;
  weightCount: number;
  rank?: number;
  totalParams: number;
}

// ========== Model Manager Types ==========

export interface ModelFileInfo {
  id: string;
  modelId: string;
  path: string;
  resolvedPath: string;
  sha256: string | null;
  sizeBytes: number;
  isSymlink: boolean;
}

export interface ModelPrefsBase {
  modelId: string;
  preferredSteps: number | null;
  preferredCfg: number | null;
}

export interface ModelPrefsLora {
  modelId: string;
  strengthMin: number | null;
  strengthMax: number | null;
  strengthDefault: number | null;
}

export interface ExampleRecord {
  id: string;
  entityType: 'model' | 'bundle';
  entityId: string;
  exampleType: 'image' | 'prompt';
  content: string;
  createdAt: string;
}

export interface ModelInfo {
  id: string;
  modelType: string;
  family: string;
  displayName: string;
  description: string | null;
  files: ModelFileInfo[];
  tags: string[];
  examples: ExampleRecord[];
  prefsBase: ModelPrefsBase | null;
  prefsLora: ModelPrefsLora | null;
  triggerWords: string[];
  architecture: string | null;
  quantization: string | null;
  vramMb: number | null;
  createdAt: string;
  updatedAt: string;
}

export interface BundleItemInfo {
  id: string;
  modelId: string;
  role: string;
  modelDisplayName: string;
  modelFamily: string;
  modelType: string;
  modelVramMb: number | null;
  modelQuantization: string | null;
}

export interface BundleInfo {
  id: string;
  displayName: string;
  description: string | null;
  isActive: boolean;
  isComplete: boolean;
  totalVramMb: number;
  tags: string[];
  items: BundleItemInfo[];
  examples: ExampleRecord[];
  createdAt: string;
  updatedAt: string;
}

export type ScanResult = {
  componentsFound: number;
  bundlesCreated: number;
};
