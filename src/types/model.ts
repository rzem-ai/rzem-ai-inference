import type { ExampleRecord } from './example';

export interface ModelFileInfo {
  id: string;
  modelId: string;
  path: string;
  resolvedPath: string;
  sha256: string | null;
  sizeBytes: number;
  isSymlink: boolean;
}

export interface ModelInfo {
  id: string;
  model_type: string;
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
