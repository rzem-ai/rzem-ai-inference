export interface StyleDetail extends StyleInfo {
  loras: StyleLoraWithInfo[];
  examples: StyleExample[];
}

export interface StyleExample {
  id: string;
  styleId: string;
  exampleType: 'prompt' | 'image';
  content: string;
  generationParams?: string;
  createdAt: number;
}

export interface StyleInfo {
  id: string;
  name: string;
  description?: string;
  promptTemplate: string;
  defaultStrength: number;
  strengthMin: number;
  strengthMax: number;
  category?: string;
  thumbnailPath?: string;
  isFavorite: boolean;
  usageCount: number;
  createdAt: number;
  updatedAt: number;
}

export interface StyleLoraWithInfo {
  loraId: string;
  loraName: string;
  loraTriggerWords?: string;
  strength: number;
  priority: number;
}

export interface StyleRequest {
  name: string;
  description?: string;
  promptTemplate: string;
  defaultStrength: number;
  strengthMin: number;
  strengthMax: number;
  category?: string;
  isFavorite: boolean;
}
