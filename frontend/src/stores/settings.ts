import { defineStore } from 'pinia';
import type { PywebviewAPI } from '@/types/pywebview';
import type { VramUsage, EngineStatus, CachedModel, DataPaths, DiskUsage, GpuInfo, RemoteEngineInfo } from '@/types/inference';

let _api: PywebviewAPI | null = null;

export const useSettingsStore = defineStore('settings', {
  state: () => ({
    // Inference Engine
    gpuInfo: null as GpuInfo | null,
    cudaVersion: null as string | null,
    vramUsage: null as VramUsage | null,
    engineStatus: null as EngineStatus | null,

    // Connection mode
    connectionMode: 'local' as string,
    remoteEngineInfo: null as RemoteEngineInfo | null,

    // API Keys
    apiKeys: {} as Record<string, string | null>,

    // Model Cache
    dataPaths: null as DataPaths | null,
    cachedModels: [] as CachedModel[],
    cacheTotalSize: 0,
    diskUsage: null as DiskUsage | null,

    // Generation / Preview
    previewInterval: 5,
    previewMaxSize: 256,

    // AI Prompts
    aiPrompts: {
      style: {
        prompt: 'Analyze this image and describe its artistic style in detail. Then update my prompt to use the same style while keeping my current subject.',
        displayText: 'Analyze style from image',
      },
      both: {
        prompt: 'Analyze this image in exhaustive detail — subject, composition, lighting, color palette, artistic style, medium, mood, and any notable visual elements. Then update my prompt to reproduce this image as closely as possible.',
        displayText: 'Analyze style and subject from image',
      },
      subject: {
        prompt: 'Analyze this image and describe its subject in detail. Then update my prompt to use the same subject while keeping my current style.',
        displayText: 'Analyze subject from image',
      },
    } as Record<string, { prompt: string; displayText: string }>,

    loading: false,
  }),

  getters: {
    isEngineReady(state): boolean {
      return state.engineStatus?.ready ?? false;
    },

    isRemote(state): boolean {
      return state.connectionMode === 'remote';
    },

    formattedUptime(state): string {
      const secs = state.engineStatus?.uptime_seconds;
      if (secs == null) return '--';
      const h = Math.floor(secs / 3600);
      const m = Math.floor((secs % 3600) / 60);
      const s = secs % 60;
      if (h > 0) return `${h}h ${m}m ${s}s`;
      if (m > 0) return `${m}m ${s}s`;
      return `${s}s`;
    },
  },

  actions: {
    setApi(apiRef: PywebviewAPI) {
      _api = apiRef;
    },

    // ── Inference Engine ──

    async loadGpuInfo() {
      if (!_api) return;
      const res = await _api.get_gpu_info();
      if (res.status === 'success') {
        this.gpuInfo = {
          device_type: res.device_type ?? 'cpu',
          device_name: res.device_name ?? null,
          total_vram_gb: res.total_vram_gb ?? 0,
        };
      }
    },

    async loadCudaVersion() {
      if (!_api) return;
      const res = await _api.get_cuda_version();
      if (res.status === 'success') {
        this.cudaVersion = res.cuda_version ?? null;
      }
    },

    async loadVramUsage() {
      if (!_api) return;
      const res = await _api.get_vram_usage();
      if (res.status === 'success' && res.available) {
        this.vramUsage = {
          available: true,
          allocated: res.allocated ?? 0,
          reserved: res.reserved ?? 0,
          free: res.free ?? 0,
          total: res.total ?? 0,
        };
      }
    },

    async loadEngineStatus() {
      if (!_api) return;
      const res = await _api.get_engine_status();
      if (res.status === 'success') {
        this.engineStatus = {
          ready: res.ready ?? false,
          uptime_seconds: res.uptime_seconds ?? null,
          completed_count: res.completed_count ?? 0,
        };
        if (res.remote) {
          this.remoteEngineInfo = {
            device: res.device ?? 'unknown',
            jobs_queued: res.jobs_queued ?? 0,
            jobs_running: res.jobs_running ?? 0,
          };
        } else {
          this.remoteEngineInfo = null;
        }
      }
    },

    async loadConnectionMode() {
      if (!_api) return;
      const res = await _api.get_connection_mode();
      if (res.status === 'success') {
        this.connectionMode = res.mode ?? 'local';
      }
    },

    async clearVramCache() {
      if (!_api) return;
      await _api.clear_vram_cache();
      await this.loadVramUsage();
    },

    async resetEngine() {
      if (!_api) return;
      await _api.reset_engine();
      await this.loadEngineStatus();
      await this.loadVramUsage();
    },

    // ── API Keys ──

    async loadApiKey(key: string) {
      if (!_api) return;
      const res = await _api.get_setting({ key });
      if (res.status === 'success') {
        this.apiKeys[key] = res.value ?? null;
      }
    },

    async saveApiKey(key: string, value: string) {
      if (!_api) return;
      const res = await _api.set_setting({ key, value });
      if (res.status === 'success') {
        this.apiKeys[key] = value;
      }
    },

    async deleteApiKey(key: string) {
      if (!_api) return;
      const res = await _api.set_setting({ key, value: '' });
      if (res.status === 'success') {
        this.apiKeys[key] = null;
      }
    },

    // ── Generation / Preview ──

    async loadPreviewSettings() {
      if (!_api) return;
      const intervalRes = await _api.get_setting({ key: 'PREVIEW_INTERVAL' });
      if (intervalRes.status === 'success' && intervalRes.value) {
        this.previewInterval = parseInt(intervalRes.value, 10) || 5;
      }
      const sizeRes = await _api.get_setting({ key: 'PREVIEW_MAX_SIZE' });
      if (sizeRes.status === 'success' && sizeRes.value) {
        this.previewMaxSize = parseInt(sizeRes.value, 10) || 256;
      }
    },

    async savePreviewInterval(value: number) {
      if (!_api) return;
      const res = await _api.set_setting({ key: 'PREVIEW_INTERVAL', value: String(value) });
      if (res.status === 'success') {
        this.previewInterval = value;
      }
    },

    async savePreviewMaxSize(value: number) {
      if (!_api) return;
      const res = await _api.set_setting({ key: 'PREVIEW_MAX_SIZE', value: String(value) });
      if (res.status === 'success') {
        this.previewMaxSize = value;
      }
    },

    // ── AI Prompts ──

    async loadAiPrompts() {
      if (!_api) return;
      const keys = ['style', 'both', 'subject'] as const;
      for (const key of keys) {
        const promptRes = await _api.get_setting({ key: `AI_PROMPT_${key.toUpperCase()}` });
        if (promptRes.status === 'success' && promptRes.value) {
          this.aiPrompts[key].prompt = promptRes.value;
        }
        const displayRes = await _api.get_setting({ key: `AI_DISPLAY_${key.toUpperCase()}` });
        if (displayRes.status === 'success' && displayRes.value) {
          this.aiPrompts[key].displayText = displayRes.value;
        }
      }
    },

    async saveAiPrompt(key: string, prompt: string) {
      if (!_api) return;
      const res = await _api.set_setting({ key: `AI_PROMPT_${key.toUpperCase()}`, value: prompt });
      if (res.status === 'success') {
        this.aiPrompts[key].prompt = prompt;
      }
    },

    async saveAiDisplayText(key: string, displayText: string) {
      if (!_api) return;
      const res = await _api.set_setting({ key: `AI_DISPLAY_${key.toUpperCase()}`, value: displayText });
      if (res.status === 'success') {
        this.aiPrompts[key].displayText = displayText;
      }
    },

    // ── Model Cache ──

    async loadDataPaths() {
      if (!_api) return;
      const res = await _api.get_data_paths();
      if (res.status === 'success') {
        this.dataPaths = {
          data_dir: res.data_dir ?? '',
          output_dir: res.output_dir ?? '',
          hf_cache_dir: res.hf_cache_dir ?? null,
        };
      }
    },

    async loadCacheInfo() {
      if (!_api) return;
      const res = await _api.get_cache_info();
      if (res.status === 'success') {
        this.cachedModels = res.models ?? [];
        this.cacheTotalSize = res.total_size ?? 0;
      }
    },

    async deleteCachedModel(repoId: string) {
      if (!_api) return;
      const res = await _api.delete_cached_model({ repo_id: repoId });
      if (res.status === 'success') {
        this.cachedModels = this.cachedModels.filter(m => m.repo_id !== repoId);
        await this.loadCacheInfo();
      }
    },

    async loadDiskUsage() {
      if (!_api) return;
      const res = await _api.get_disk_usage();
      if (res.status === 'success') {
        this.diskUsage = {
          output_size: res.output_size ?? 0,
          hf_cache_size: res.hf_cache_size ?? 0,
        };
      }
    },

    // ── State setters ──

    setConnectionMode(mode: string) {
      this.connectionMode = mode;
    },

    setRemoteEngineInfo(info: RemoteEngineInfo | null) {
      this.remoteEngineInfo = info;
    },
  },
});
