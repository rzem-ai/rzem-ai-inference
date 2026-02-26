import { defineStore } from 'pinia';
import { getApiAsync } from '@/bridge';
import type { VramUsage, EngineStatus, CachedModel, DataPaths, DiskUsage, GpuInfo, RemoteEngineInfo } from '@/types/inference';

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

    // Claude Model
    claudeModel: 'claude-sonnet-4-6' as string,

    // AI Provider
    aiProvider: 'claude' as string,

    // Perplexity Model
    perplexityModel: 'anthropic/claude-sonnet-4-6' as string,

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
    // ── Inference Engine ──

    async loadGpuInfo() {
      const api = await getApiAsync();
      const res = await api.get_gpu_info();
      if (res.status === 'success') {
        this.gpuInfo = {
          device_type: res.device_type ?? 'cpu',
          device_name: res.device_name ?? null,
          total_vram_gb: res.total_vram_gb ?? 0,
        };
      }
    },

    async loadCudaVersion() {
      const api = await getApiAsync();
      const res = await api.get_cuda_version();
      if (res.status === 'success') {
        this.cudaVersion = res.cuda_version ?? null;
      }
    },

    async loadVramUsage() {
      const api = await getApiAsync();
      const res = await api.get_vram_usage();
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
      const api = await getApiAsync();
      const res = await api.get_engine_status();
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
      const api = await getApiAsync();
      const res = await api.get_connection_mode();
      if (res.status === 'success') {
        this.connectionMode = res.mode ?? 'local';
      }
    },

    async clearVramCache() {
      const api = await getApiAsync();
      await api.clear_vram_cache();
      await this.loadVramUsage();
    },

    async resetEngine() {
      const api = await getApiAsync();
      await api.reset_engine();
      await this.loadEngineStatus();
      await this.loadVramUsage();
    },

    // ── API Keys ──

    async loadApiKey(key: string) {
      const api = await getApiAsync();
      const res = await api.get_setting({ key });
      if (res.status === 'success') {
        this.apiKeys[key] = res.value ?? null;
      }
    },

    async saveApiKey(key: string, value: string) {
      const api = await getApiAsync();
      const res = await api.set_setting({ key, value });
      if (res.status === 'success') {
        this.apiKeys[key] = value;
      }
    },

    async deleteApiKey(key: string) {
      const api = await getApiAsync();
      const res = await api.set_setting({ key, value: '' });
      if (res.status === 'success') {
        this.apiKeys[key] = null;
      }
    },

    // ── Generation / Preview ──

    async loadPreviewSettings() {
      const api = await getApiAsync();
      const intervalRes = await api.get_setting({ key: 'PREVIEW_INTERVAL' });
      if (intervalRes.status === 'success' && intervalRes.value) {
        this.previewInterval = parseInt(intervalRes.value, 10) || 5;
      }
      const sizeRes = await api.get_setting({ key: 'PREVIEW_MAX_SIZE' });
      if (sizeRes.status === 'success' && sizeRes.value) {
        this.previewMaxSize = parseInt(sizeRes.value, 10) || 256;
      }
    },

    async savePreviewInterval(value: number) {
      const api = await getApiAsync();
      const res = await api.set_setting({ key: 'PREVIEW_INTERVAL', value: String(value) });
      if (res.status === 'success') {
        this.previewInterval = value;
      }
    },

    async savePreviewMaxSize(value: number) {
      const api = await getApiAsync();
      const res = await api.set_setting({ key: 'PREVIEW_MAX_SIZE', value: String(value) });
      if (res.status === 'success') {
        this.previewMaxSize = value;
      }
    },

    // ── AI Prompts ──

    async loadAiPrompts() {
      const api = await getApiAsync();
      const keys = ['style', 'both', 'subject'] as const;
      for (const key of keys) {
        const promptRes = await api.get_setting({ key: `AI_PROMPT_${key.toUpperCase()}` });
        if (promptRes.status === 'success' && promptRes.value) {
          this.aiPrompts[key].prompt = promptRes.value;
        }
        const displayRes = await api.get_setting({ key: `AI_DISPLAY_${key.toUpperCase()}` });
        if (displayRes.status === 'success' && displayRes.value) {
          this.aiPrompts[key].displayText = displayRes.value;
        }
      }
    },

    async saveAiPrompt(key: string, prompt: string) {
      const api = await getApiAsync();
      const res = await api.set_setting({ key: `AI_PROMPT_${key.toUpperCase()}`, value: prompt });
      if (res.status === 'success') {
        this.aiPrompts[key].prompt = prompt;
      }
    },

    async saveAiDisplayText(key: string, displayText: string) {
      const api = await getApiAsync();
      const res = await api.set_setting({ key: `AI_DISPLAY_${key.toUpperCase()}`, value: displayText });
      if (res.status === 'success') {
        this.aiPrompts[key].displayText = displayText;
      }
    },

    // ── Claude Model ──

    async loadClaudeModel() {
      const api = await getApiAsync();
      const res = await api.get_setting({ key: 'CLAUDE_MODEL' });
      if (res.status === 'success' && res.value) {
        this.claudeModel = res.value;
      }
    },

    async saveClaudeModel(model: string) {
      const api = await getApiAsync();
      const res = await api.set_setting({ key: 'CLAUDE_MODEL', value: model });
      if (res.status === 'success') {
        this.claudeModel = model;
      }
    },

    // ── AI Provider ──

    async loadAiProvider() {
      const api = await getApiAsync();
      const res = await api.get_setting({ key: 'AI_PROVIDER' });
      if (res.status === 'success' && res.value) {
        this.aiProvider = res.value;
      }
    },

    async saveAiProvider(provider: string) {
      const api = await getApiAsync();
      const res = await api.set_setting({ key: 'AI_PROVIDER', value: provider });
      if (res.status === 'success') {
        this.aiProvider = provider;
      }
    },

    // ── Perplexity Model ──

    async loadPerplexityModel() {
      const api = await getApiAsync();
      const res = await api.get_setting({ key: 'PERPLEXITY_MODEL' });
      if (res.status === 'success' && res.value) {
        this.perplexityModel = res.value;
      }
    },

    async savePerplexityModel(model: string) {
      const api = await getApiAsync();
      const res = await api.set_setting({ key: 'PERPLEXITY_MODEL', value: model });
      if (res.status === 'success') {
        this.perplexityModel = model;
      }
    },

    // ── Model Cache ──

    async loadDataPaths() {
      const api = await getApiAsync();
      const res = await api.get_data_paths();
      if (res.status === 'success') {
        this.dataPaths = {
          data_dir: res.data_dir ?? '',
          output_dir: res.output_dir ?? '',
          hf_cache_dir: res.hf_cache_dir ?? null,
        };
      }
    },

    async browseOutputDirectory() {
      const api = await getApiAsync();
      const res = await api.browse_output_directory();
      if (res.status === 'success' && res.changed && res.output_dir) {
        if (this.dataPaths) {
          this.dataPaths.output_dir = res.output_dir;
        }
      }
      return res;
    },

    async loadCacheInfo() {
      const api = await getApiAsync();
      const res = await api.get_cache_info();
      if (res.status === 'success') {
        this.cachedModels = res.models ?? [];
        this.cacheTotalSize = res.total_size ?? 0;
      }
    },

    async deleteCachedModel(repoId: string) {
      const api = await getApiAsync();
      const res = await api.delete_cached_model({ repo_id: repoId });
      if (res.status === 'success') {
        this.cachedModels = this.cachedModels.filter(m => m.repo_id !== repoId);
        await this.loadCacheInfo();
      }
    },

    async loadDiskUsage() {
      const api = await getApiAsync();
      const res = await api.get_disk_usage();
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
