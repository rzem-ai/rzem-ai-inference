import { defineStore } from 'pinia';
import { invoke } from '@/utils/backend-bridge';
import type { ModelInfo, BundleInfo as ApiBundleInfo, BundleItemInfo } from '@/types';

// ComponentRecord shape used by EnhancedModelSelector
export interface ComponentRecord {
  id: string;
  name: string;
  componentType: string;
  isAvailable: boolean;
  quantization: string | null;
  vramMb: number | null;
  architecture: string | null;
  family: string;
  filePath: string;
  fileSize: number;
}

// BundleInfo shape used by EnhancedModelSelector (extends API type with aliases)
export interface BundleInfo {
  id: string;
  name: string;
  displayName: string;
  description: string | null;
  isActive: boolean;
  isComplete: boolean;
  totalVramMb: number;
  tags: string[];
  items: BundleItemInfo[];
  components: BundleItemInfo[];  // alias for items
  examples: Array<{ id: string; entityType: string; entityId: string; exampleType: string; content: string; createdAt: string }>;
  createdAt: string;
  updatedAt: string;
}

function modelToComponent(model: ModelInfo): ComponentRecord {
  return {
    id: model.id,
    name: model.displayName,
    componentType: model.model_type,
    isAvailable: model.files.length > 0,
    quantization: model.quantization,
    vramMb: model.vramMb,
    architecture: model.architecture,
    family: model.family,
    filePath: model.files[0]?.path ?? '',
    fileSize: model.files[0]?.sizeBytes ?? 0,
  };
}

function apiBundleToBundle(b: ApiBundleInfo): BundleInfo {
  return {
    ...b,
    name: b.displayName,
    components: b.items,
  };
}

export const useBundlesStore = defineStore('bundles', {
  state: () => ({
    bundles: [] as BundleInfo[],
    models: [] as ModelInfo[],
    _initialized: false,
    isInitialized: false,
    isScanning: false,
    scanMessage: '',
    error: null as string | null,
  }),

  getters: {
    activeBundle(state): BundleInfo | null {
      return state.bundles.find((b) => b.isActive) ?? null;
    },

    transformerComponents(state): ComponentRecord[] {
      return state.models
        .filter((m) => m.model_type === 'transformer')
        .map(modelToComponent);
    },

    t5Components(state): ComponentRecord[] {
      return state.models
        .filter((m) => m.model_type === 'text_encoder' && (m.architecture?.toLowerCase().includes('t5') ?? false))
        .map(modelToComponent);
    },

    clipComponents(state): ComponentRecord[] {
      return state.models
        .filter((m) => m.model_type === 'text_encoder' && (m.architecture?.toLowerCase().includes('clip') ?? false))
        .map(modelToComponent);
    },

    vaeComponents(state): ComponentRecord[] {
      return state.models
        .filter((m) => m.model_type === 'ae' || m.model_type === 'vae')
        .map(modelToComponent);
    },
  },

  actions: {
    formatVram(vramMb: number): string {
      if (vramMb >= 1000) {
        return `${(vramMb / 1000).toFixed(1)} GB`;
      }
      return `${vramMb} MB`;
    },

    async initialize() {
      // Guard: only initialize once
      if (this.isInitialized) return;

      this.error = null;

      try {
        const [models, bundles] = await Promise.all([
          invoke<ModelInfo[]>('get_all_models'),
          invoke<ApiBundleInfo[]>('get_all_bundles'),
        ]);
        this.models = models;
        this.bundles = bundles.map(apiBundleToBundle);
        this._initialized = true;
        this.isInitialized = true;
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err);
        console.error('[BundlesStore] Initialization failed:', err);
        throw err;
      }
    },

    // Force reload (for manual refresh)
    async reload(): Promise<void> {
      this.isInitialized = false
      this._initialized = false
      await this.initialize()
    },

    async scanHfCache() {
      this._initialized = false;
      this.isScanning = true;
      this.scanMessage = 'Scanning HuggingFace cache...';
      try {
        const result = await invoke<{ componentsFound: number; bundlesCreated: number }>('scan_and_discover_models');
        await this.initialize();
        return result;
      } finally {
        this.isScanning = false;
        this.scanMessage = '';
      }
    },

    async scanDirectory(path: string) {
      this._initialized = false;
      this.isScanning = true;
      this.scanMessage = 'Scanning directory...';
      try {
        const result = await invoke<{ componentsFound: number; bundlesCreated: number }>('scan_directory_for_models', { directoryPath: path });
        await this.initialize();
        return result;
      } finally {
        this.isScanning = false;
        this.scanMessage = '';
      }
    },

    async createBundle(displayName: string, description: string | null, items: [string, string][]) {
      const id = await invoke<string>('create_bundle', { displayName, description, items });
      await this.initialize();
      return id;
    },

    async updateBundle(bundle_id: string, displayName?: string, description?: string) {
      await invoke('update_bundle', { bundle_id, displayName: displayName ?? null, description: description ?? null });
      // Update local state
      const bundle = this.bundles.find((b) => b.id === bundle_id);
      if (bundle) {
        if (displayName !== undefined) { bundle.name = displayName; bundle.displayName = displayName; }
        if (description !== undefined) { bundle.description = description; }
      }
    },

    async deleteBundle(bundle_id: string) {
      await invoke('delete_bundle', { bundle_id });
      this.bundles = this.bundles.filter((b) => b.id !== bundle_id);
    },

    async setActiveBundle(bundle_id: string) {
      await invoke('set_active_bundle', { bundle_id });
      this.bundles.forEach((b) => { b.isActive = b.id === bundle_id; });
    },

    async getCompatibleModels(baseModelId: string, targetType: string): Promise<ModelInfo[]> {
      return invoke<ModelInfo[]>('get_compatible_models', { baseModelId, targetType });
    },

    // Reload from backend
    async refresh() {
      this._initialized = false;
      await this.initialize();
    },
  },
});
