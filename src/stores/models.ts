import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import type { ModelInfo, LoRA, LoraFileInfo, LoraConfig } from '@/types';

interface BackendLoraInfo {
  id: string;
  name: string;
  path: string;
  trigger_words?: string;
  base_model?: string;
  size_bytes: number;
  created_at: number;
  metadata?: Record<string, string>;
}

interface BackendLoraFileInfo {
  path: string;
  size_bytes: number;
  weight_count: number;
  rank?: number;
  total_params: number;
}

function mapLoraInfo(info: BackendLoraInfo, existingLora?: LoRA): LoRA {
  return {
    id: info.id,
    name: info.name,
    path: info.path,
    triggerWords: info.trigger_words,
    baseModel: info.base_model,
    sizeBytes: info.size_bytes,
    createdAt: info.created_at,
    metadata: info.metadata,
    strength: existingLora?.strength ?? 1.0,
    isActive: existingLora?.isActive ?? false,
  };
}

function mapLoraFileInfo(info: BackendLoraFileInfo): LoraFileInfo {
  return {
    path: info.path,
    sizeBytes: info.size_bytes,
    weightCount: info.weight_count,
    rank: info.rank,
    totalParams: info.total_params,
  };
}

export const useModelsStore = defineStore('models', {
  state: () => ({
    models: [] as ModelInfo[],
    selectedModel: null as ModelInfo | null,
    scanning: false,
    scanProgress: { stage: '', message: '', progress: 0 },
    loras: [] as LoRA[],
    lorasLoading: false,
    lorasError: null as string | null,
  }),

  getters: {
    modelsByType(state) {
      const grouped: Record<string, ModelInfo[]> = {};
      for (const model of state.models) {
        if (!grouped[model.modelType]) grouped[model.modelType] = [];
        grouped[model.modelType].push(model);
      }
      // Sort each group by displayName
      for (const key of Object.keys(grouped)) {
        grouped[key].sort((a, b) => a.displayName.localeCompare(b.displayName));
      }
      return grouped;
    },

    typeOrder(): string[] {
      return ['checkpoint', 'text_encoder', 'vae', 'tokenizer', 'lora', 'scheduler'];
    },

    activeLoras(state): LoRA[] {
      return state.loras.filter((l) => l.isActive);
    },
  },

  actions: {
    async loadModels() {
      this.models = await invoke<ModelInfo[]>('get_all_models');
    },

    async scanDirectory(path: string) {
      this.scanning = true;
      this.scanProgress = { stage: 'scanning', message: 'Scanning...', progress: 0 };
      try {
        const result = await invoke<{ componentsFound: number; bundlesCreated: number }>('scan_directory_for_models', { directoryPath: path });
        await this.loadModels();
        return result;
      } finally {
        this.scanning = false;
        this.scanProgress = { stage: 'complete', message: 'Done', progress: 100 };
      }
    },

    async scanHfCache() {
      this.scanning = true;
      this.scanProgress = { stage: 'scanning', message: 'Scanning HF cache...', progress: 0 };
      try {
        const result = await invoke<{ componentsFound: number; bundlesCreated: number }>('scan_and_discover_models');
        await this.loadModels();
        return result;
      } finally {
        this.scanning = false;
        this.scanProgress = { stage: 'complete', message: 'Done', progress: 100 };
      }
    },

    selectModel(modelOrId: ModelInfo | string | null) {
      if (typeof modelOrId === 'string') {
        this.selectedModel = this.models.find((m) => m.id === modelOrId) ?? null;
      } else {
        this.selectedModel = modelOrId;
      }
    },

    async updateModel(modelId: string, displayName?: string, description?: string) {
      await invoke('update_model', { modelId, displayName: displayName ?? null, description: description ?? null });
      // Update local state
      const model = this.models.find((m) => m.id === modelId);
      if (model) {
        if (displayName !== undefined) model.displayName = displayName;
        if (description !== undefined) model.description = description;
      }
      if (this.selectedModel?.id === modelId) {
        if (displayName !== undefined) this.selectedModel.displayName = displayName;
        if (description !== undefined) this.selectedModel.description = description;
      }
    },

    async addTag(modelId: string, tag: string) {
      await invoke('add_model_tag', { modelId, tag });
      const model = this.models.find((m) => m.id === modelId);
      if (model && !model.tags.includes(tag)) model.tags.push(tag);
      if (this.selectedModel?.id === modelId && !this.selectedModel.tags.includes(tag)) {
        this.selectedModel.tags.push(tag);
      }
    },

    async removeTag(modelId: string, tag: string) {
      await invoke('remove_model_tag', { modelId, tag });
      const model = this.models.find((m) => m.id === modelId);
      if (model) model.tags = model.tags.filter((t) => t !== tag);
      if (this.selectedModel?.id === modelId) {
        this.selectedModel.tags = this.selectedModel.tags.filter((t) => t !== tag);
      }
    },

    async addExample(modelId: string, exampleType: 'image' | 'prompt', content: string) {
      const id = await invoke<string>('add_example', { entityType: 'model', entityId: modelId, exampleType, content });
      const model = this.models.find((m) => m.id === modelId);
      const example = { id, entityType: 'model' as const, entityId: modelId, exampleType, content, createdAt: new Date().toISOString() };
      if (model) model.examples.push(example);
      if (this.selectedModel?.id === modelId) this.selectedModel.examples.push(example);
    },

    async removeExample(modelId: string, exampleId: string) {
      await invoke('remove_example', { exampleId });
      const model = this.models.find((m) => m.id === modelId);
      if (model) model.examples = model.examples.filter((e) => e.id !== exampleId);
      if (this.selectedModel?.id === modelId) {
        this.selectedModel.examples = this.selectedModel.examples.filter((e) => e.id !== exampleId);
      }
    },

    // ============ LoRA Management ============

    async loadLoras() {
      this.lorasLoading = true;
      this.lorasError = null;
      try {
        const backendLoras = await invoke<BackendLoraInfo[]>('get_loras');
        this.loras = backendLoras.map((info) => {
          const existing = this.loras.find((l) => l.id === info.id);
          return mapLoraInfo(info, existing);
        });
      } catch (error) {
        this.lorasError = String(error);
        console.error('Failed to load LoRAs:', error);
      } finally {
        this.lorasLoading = false;
      }
    },

    async importLora(sourcePath: string, name: string, triggerWords?: string): Promise<LoRA | null> {
      try {
        const info = await invoke<BackendLoraInfo>('import_lora', {
          sourcePath,
          name,
          triggerWords: triggerWords || null,
        });
        const newLora = mapLoraInfo(info);
        this.loras.push(newLora);
        return newLora;
      } catch (error) {
        console.error('Failed to import LoRA:', error);
        throw error;
      }
    },

    async removeLora(id: string) {
      try {
        await invoke('remove_lora', { id });
        this.loras = this.loras.filter((l) => l.id !== id);
      } catch (error) {
        console.error('Failed to remove LoRA:', error);
        throw error;
      }
    },

    async getLoraFileInfo(path: string): Promise<LoraFileInfo> {
      const info = await invoke<BackendLoraFileInfo>('get_lora_file_info', { path });
      return mapLoraFileInfo(info);
    },

    toggleLora(id: string) {
      const lora = this.loras.find((l) => l.id === id);
      if (lora) lora.isActive = !lora.isActive;
    },

    updateLoraStrength(id: string, strength: number) {
      const lora = this.loras.find((l) => l.id === id);
      if (lora) lora.strength = Math.max(0, Math.min(2, strength));
    },

    getActiveLoraConfigs(): LoraConfig[] {
      return this.loras
        .filter((l) => l.isActive)
        .map((l) => ({ id: l.id, strength: l.strength }));
    },

    async refreshModelAvailability() {
      await this.loadModels();
    },
  },
});
