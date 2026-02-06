import type { ExampleRecord } from './example';

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

export interface BundleItemInfo {
  id: string;
  modelId: string;
  role: string;
  modelDisplayName: string;
  modelFamily: string;
  model_type: string;
  modelVramMb: number | null;
  modelQuantization: string | null;
}
