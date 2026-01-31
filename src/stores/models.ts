import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import type { Model, LoRA, LoraFileInfo, LoraConfig } from '@/types';

// Backend LoraInfo structure (snake_case from Rust)
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

// Backend LoraFileInfo structure
interface BackendLoraFileInfo {
  path: string;
  size_bytes: number;
  weight_count: number;
  rank?: number;
  total_params: number;
}

// Convert backend LoraInfo to frontend LoRA
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
    // Preserve existing frontend state or use defaults
    strength: existingLora?.strength ?? 1.0,
    isActive: existingLora?.isActive ?? false,
  };
}

// Convert backend LoraFileInfo to frontend
function mapLoraFileInfo(info: BackendLoraFileInfo): LoraFileInfo {
  return {
    path: info.path,
    sizeBytes: info.size_bytes,
    weightCount: info.weight_count,
    rank: info.rank,
    totalParams: info.total_params,
  };
}

// Selected model info for the Models view (shared between sidebar and detail panel)
export interface SelectedModelInfo {
  id: string;
  name: string;
  description: string;
  size: string;
  type: string;
  category: 'generation' | 'vision' | 'component';
  categoryLabel: string;
  format: string;
  license: string;
  source: string;
  tags: string[];
  icon: string;
  iconClass: string;
  isDownloaded: boolean;
  docsUrl?: string;
  defaultSettings?: {
    steps: number;
    guidance: number;
  };
  features?: string[];
  notes?: string;
  hasQuantized?: boolean;
  quantizedDownloaded?: boolean;
}

export const useModelsStore = defineStore('models', {
  state: () => ({
    models: [] as Model[],
    loras: [] as LoRA[],
    selectedModelId: 'dev',
    selectedViewComponent: null as SelectedModelInfo | null,
    lorasLoading: false,
    lorasError: null as string | null,
    modelsLoading: false,
    modelsError: null as string | null,
  }),

  getters: {
    activeModel(state): Model | undefined {
      return state.models.find((m) => m.id === state.selectedModelId);
    },

    activeLoras(state): LoRA[] {
      return state.loras.filter((l) => l.isActive);
    },

    downloadedModels(state): Model[] {
      return state.models.filter((m) => m.isDownloaded);
    },
  },

  actions: {
    // ============ Model Backend Integration ============

    // Load all models from database
    // NOTE: Models are now derived from the bundle/component system
    // This loads bundles and creates model records from transformer components
    async loadModels(): Promise<void> {
      this.modelsLoading = true;
      this.modelsError = null;
      try {
        // Import bundle store types
        const { useBundlesStore } = await import('./bundles');
        const bundlesStore = useBundlesStore();

        // Load bundles from database
        await bundlesStore.loadBundles();

        // Create model records from bundles
        // Each complete bundle with a transformer becomes a "model"
        this.models = bundlesStore.completeBundles.map((bundle): Model => {
          const transformer = bundle.components.find((c) => c.componentType === 'transformer');

          return {
            id: bundle.id,
            name: bundle.name,
            type: bundle.modelFamily === 'flux' ? 'flux-schnell' : 'flux-schnell',
            path: transformer?.filePath,
            sizeBytes: transformer?.fileSize,
            isDownloaded: bundle.isComplete,
            isActive: bundle.isActive,
            createdAt: bundle.createdAt,
            lastUsedAt: bundle.updatedAt,
            metadata: {
              description: bundle.description || '',
              category: 'generation',
              format: transformer?.format || 'safetensors',
              source: 'huggingface',
              license: 'Apache 2.0',
              docsUrl: undefined,
              repoId: undefined,
              supportsLoras: transformer?.format === 'safetensors',
              vramFull: bundle.totalVramMb,
              vramQuantized: undefined,
            },
            description: bundle.description || '',
            defaultSteps: bundle.defaultSteps || 4,
            defaultGuidance: bundle.defaultGuidance || 3.5,
          };
        });

        // Set default selected model if none selected or selected model not available
        if (!this.selectedModelId || !this.models.find((m) => m.id === this.selectedModelId)) {
          // Prefer active bundle, then LoRA-compatible, then any downloaded
          const activeModel = this.models.find((m) => m.isActive);
          const loraCompatible = this.models.find((m) => m.isDownloaded && m.metadata?.supportsLoras);
          const anyDownloaded = this.models.find((m) => m.isDownloaded);
          this.selectedModelId = activeModel?.id || loraCompatible?.id || anyDownloaded?.id || this.models[0]?.id || 'dev';
        }
      } catch (error) {
        this.modelsError = String(error);
        console.error('Failed to load models:', error);
      } finally {
        this.modelsLoading = false;
      }
    },

    addModel(model: Model) {
      this.models.push(model);
    },

    removeModel(id: string): boolean {
      const index = this.models.findIndex((m) => m.id === id);
      if (index !== -1) {
        this.models.splice(index, 1);
        return true;
      }
      return false;
    },

    selectModel(id: string): boolean {
      const model = this.models.find((m) => m.id === id);
      if (model && model.isDownloaded) {
        // Deactivate all models
        this.models = this.models.map((m) => ({
          ...m,
          isActive: m.id === id,
        }));
        this.selectedModelId = id;
        return true;
      }
      return false;
    },

    // Set the selected component for the Models view detail panel
    setSelectedViewComponent(component: SelectedModelInfo | null) {
      this.selectedViewComponent = component;
    },

    // ============ LoRA Backend Integration ============

    // Load all LoRAs from backend
    async loadLoras(): Promise<void> {
      this.lorasLoading = true;
      this.lorasError = null;
      try {
        const backendLoras = await invoke<BackendLoraInfo[]>('get_loras');
        // Map backend data, preserving frontend state for existing LoRAs
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

    // Import a new LoRA from file
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

    // Remove a LoRA (deletes from disk via backend)
    async removeLora(id: string): Promise<boolean> {
      try {
        await invoke('remove_lora', { id });
        const index = this.loras.findIndex((l) => l.id === id);
        if (index !== -1) {
          this.loras.splice(index, 1);
        }
        return true;
      } catch (error) {
        console.error('Failed to remove LoRA:', error);
        throw error;
      }
    },

    // Get file info before importing (preview)
    async getLoraFileInfo(path: string): Promise<LoraFileInfo> {
      try {
        const info = await invoke<BackendLoraFileInfo>('get_lora_file_info', { path });
        return mapLoraFileInfo(info);
      } catch (error) {
        console.error('Failed to get LoRA file info:', error);
        throw error;
      }
    },

    // ============ Local LoRA State Management ============

    toggleLora(id: string): boolean {
      const lora = this.loras.find((l) => l.id === id);
      if (lora) {
        lora.isActive = !lora.isActive;
        return true;
      }
      return false;
    },

    updateLoraStrength(id: string, strength: number): boolean {
      const lora = this.loras.find((l) => l.id === id);
      if (lora) {
        // Clamp strength to valid range (0.0 to 2.0)
        lora.strength = Math.max(0, Math.min(2, strength));
        return true;
      }
      return false;
    },

    // Get active LoRAs as config for generation params
    getLoraConfigs(): LoraConfig[] {
      return this.loras.map((l) => ({
        id: l.id,
        strength: l.strength,
      }));
    },

    getActiveLoraConfigs(): LoraConfig[] {
      return this.loras
        .filter((l) => l.isActive)
        .map((l) => ({
          id: l.id,
          strength: l.strength,
        }));
    },

    // ============ Model Backend Integration ============

    // Fetch model availability from backend
    // NOTE: Now pulls from bundle system instead of removed model_bundles table
    async refreshModelAvailability(): Promise<void> {
      try {
        // Reload models from bundle system to get latest availability
        await this.loadModels();
      } catch (error) {
        console.error('Failed to refresh model availability:', error);
      }
    },
  },
});
