// ========== LoRA Types ==========
// 
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
  defaultStrength: number;
  strengthMin: number;
  strengthMax: number;
  downloadUrl?: string;
  civitaiModelId?: number;
  civitaiVersionId?: number;
}

// LoRA configuration for generation
export interface LoraConfig {
  id: string;
  strength: number;
}

export interface LoraFileInfo {
  path: string;
  sizeBytes: number;
  weightCount: number;
  rank?: number;
  totalParams: number;
  isFluxCompatible: boolean;
}
