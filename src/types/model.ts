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
  architecture: string | null;
  componentType: string;
  createdAt: string;
  description: string | null;
  name: string;
  examples: ExampleRecord[];
  family: string;
  files: ModelFileInfo[];
  prefsBase: ModelPrefsBase | null;
  prefsLora: ModelPrefsLora | null;
  quantization: string | null;
  tags: string[];
  triggerWords: string[];
  updatedAt: string;
  vramMb: number | null;
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
