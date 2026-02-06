import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { useGalleryStore } from './gallery';
import { useJobUpdates } from '@/composables/useWebSocket';

// ========== Queue/Job Types (from backend) ==========

export type SamplerType = 'euler' | 'euler_a' | 'dpm_pp_2m';
export type SchedulerType = 'normal' | 'simple' | 'karras' | 'exponential';

export interface LoraConfig {
  id: string;
  strength: number;
}

export interface GenerationParams {
  prompt: string;
  negative_prompt?: string;
  steps: number;
  cfg_scale: number;
  width: number;
  height: number;
  seed: number;
  bundle_id?: string;
  model_component_id: string;
  clip_component_id: string;
  t5_component_id: string;
  vae_component_id: string;
  sampler?: SamplerType;
  scheduler?: SchedulerType;
  loras?: LoraConfig[];
}

export type JobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface GenerationStats {
  model_load_ms?: number;
  t5_load_ms?: number;
  clip_load_ms?: number;
  vae_load_ms?: number;
  flux_load_ms?: number;
  t5_encode_ms: number;
  clip_encode_ms: number;
  denoise_ms: number;
  vae_decode_ms: number;
  png_encode_ms: number;
  total_ms: number;
  t5_embedding_shape: number[];
  clip_embedding_shape: number[];
  latent_shape: number[];
  image_shape: number[];
  steps: number;
  model_type: string;
}

export type PipelineStage = 'loading_models' | 'encoding_t5' | 'encoding_clip' | 'denoising' | 'decoding_vae' | 'encoding_png';

export interface GenerationJob {
  id: string;
  params: GenerationParams;
  status: JobStatus;
  progress: number;
  currentStage?: PipelineStage;
  statusMessage?: string;
  currentStep?: number;
  totalSteps?: number;
  created_at: number;
  started_at?: number;
  completed_at?: number;
  result_path?: string;
  error?: string;
  stats?: GenerationStats;
  previewData?: string;
}

// ========== UI-Specific Types ==========

export interface UiSectionVisibility {
  quality: boolean;
  style: boolean;
  advanced: boolean;
}

// ========== UI Params (for form state) ==========

const STORAGE_KEY = 'generation-params';
const SEED_RANDOMIZE_KEY = 'generation-randomize-seed';
const SECTION_VISIBILITY_KEY = 'generation-section-visibility';

export interface UiGenerationParams {
  prompt: string;
  negativePrompt: string;
  steps: number;
  cfgScale: number;
  width: number;
  height: number;
  seed: number;
  sampler: SamplerType;
  scheduler: SchedulerType;
  bundleId?: string;
  modelComponentId: string;
  t5ComponentId: string;
  clipComponentId: string;
  vaeComponentId: string;
}

const defaultParams: UiGenerationParams = {
  prompt: 'A West Highland White Terrier in the style of a Pixar cartoon',
  negativePrompt: '',
  steps: 4,
  cfgScale: 1.0,
  sampler: 'euler',
  scheduler: 'normal',
  width: 1024,
  height: 1024,
  seed: -1,
  bundleId: undefined,
  modelComponentId: '',
  t5ComponentId: '',
  clipComponentId: '',
  vaeComponentId: '',
};

function loadParams(): UiGenerationParams {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      return { ...defaultParams, ...JSON.parse(stored) };
    }
  } catch (e) {
    console.warn('Failed to load generation params from localStorage:', e);
  }
  return { ...defaultParams };
}

function loadRandomizeSeed(): boolean {
  try {
    const stored = localStorage.getItem(SEED_RANDOMIZE_KEY);
    if (stored !== null) {
      return JSON.parse(stored);
    }
  } catch (e) {
    console.warn('Failed to load randomize seed setting from localStorage:', e);
  }
  return true;
}

function loadSectionVisibility(): UiSectionVisibility {
  try {
    const stored = localStorage.getItem(SECTION_VISIBILITY_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      // Merge with defaults to handle missing fields gracefully
      return {
        quality: parsed.quality ?? true,
        style: parsed.style ?? false,
        advanced: parsed.advanced ?? false,
      };
    }
  } catch (e) {
    console.warn('Failed to load section visibility from localStorage:', e);
  }
  // Defaults: quality visible, style and advanced hidden
  return { quality: true, style: false, advanced: false };
}

export const useGenerationStore = defineStore('generation', {
  state: () => ({
    // Queue state (from queue store)
    jobs: [] as GenerationJob[],
    historyJobs: [] as GenerationJob[],
    isPolling: false,
    pollingInterval: null as number | null,
    error: null as string | null,
    jobUpdates: null as ReturnType<typeof useJobUpdates> | null,

    // UI form state (from generation store)
    currentParams: loadParams(),
    randomizeSeedOnGenerate: loadRandomizeSeed(),
    sectionVisibility: loadSectionVisibility(),
    _unsubscribe: null as (() => void) | null,
    isInitialized: false,

    // Style support
    selectedStyleId: null as string | null,
    appliedTemplate: null as string | null,
  }),

  getters: {
    // Queue getters (from queue store)
    pendingJobs(state): GenerationJob[] {
      return state.jobs.filter((j) => j.status === 'pending');
    },

    runningJobs(state): GenerationJob[] {
      return state.jobs.filter((j) => j.status === 'running');
    },

    completedJobs(state): GenerationJob[] {
      return state.jobs.filter((j) => j.status === 'completed');
    },

    failedJobs(state): GenerationJob[] {
      return state.jobs.filter((j) => j.status === 'failed');
    },

    queueLength(state): number {
      return state.jobs.filter((j) => j.status === 'pending').length;
    },

    hasRunningJobs(state): boolean {
      return state.jobs.filter((j) => j.status === 'running').length > 0;
    },

    // Legacy alias for compatibility
    isGenerating(state): boolean {
      return state.jobs.filter((j) => j.status === 'running').length > 0;
    },

    // UI configuration validation
    isValidConfiguration(state): boolean {
      if (state.currentParams.bundleId) {
        return true;
      }
      return !!(
        state.currentParams.modelComponentId &&
        state.currentParams.t5ComponentId &&
        state.currentParams.clipComponentId &&
        state.currentParams.vaeComponentId
      );
    },
  },

  actions: {
    // ========== Initialization (merged) ==========

    async initialize(): Promise<void> {
      if (this.isInitialized) {
        return;
      }

      try {
        // Load initial queue data
        await this.refreshJobs();

        // Initialize event listeners
        await this.initializeEventListeners();

        // Initialize UI persistence
        this.initializePersistence();

        this.isInitialized = true;
      } catch (err) {
        console.error('[GenerationStore] Initialization failed:', err);
        throw err;
      }
    },

    cleanup() {
      this.cleanupEventListeners();
      this.cleanupPersistence();
      this.isInitialized = false;
    },

    // ========== Queue Event Listeners (from queue store) ==========

    async initializeEventListeners() {
      if (this.jobUpdates) return;

      this.jobUpdates = useJobUpdates();

      await this.jobUpdates.onJobUpdate(async (payload: { job_id: string; status: string; progress?: number; result_path?: string; error?: string; stats?: any }) => {
        const { job_id, status, progress, result_path, error: jobError, stats } = payload;

        let jobIndex = this.jobs.findIndex((j) => j.id === job_id);
        if (jobIndex === -1) {
          await this.refreshJobs();
          jobIndex = this.jobs.findIndex((j) => j.id === job_id);
          if (jobIndex === -1) {
            console.warn(`Job ${job_id} not found in local state after refresh`);
            return;
          }
        }

        const updatedJob = { ...this.jobs[jobIndex] };
        updatedJob.status = status as JobStatus;
        if (progress !== undefined) {
          updatedJob.progress = progress;
        }
        if (result_path) {
          updatedJob.result_path = result_path;
        }
        if (jobError) {
          updatedJob.error = jobError;
        }
        if (stats) {
          updatedJob.stats = stats;
        }
        if (status === 'running' && !updatedJob.started_at) {
          updatedJob.started_at = Math.floor(Date.now() / 1000);
        }
        if (status === 'completed' || status === 'failed' || status === 'cancelled') {
          updatedJob.completed_at = Math.floor(Date.now() / 1000);
        }

        this.jobs[jobIndex] = updatedJob;

        if (status === 'completed') {
          const galleryStore = useGalleryStore();
          await galleryStore.loadImages();
        }
      });

      await this.jobUpdates.onJobProgress((payload: {
        job_id: string;
        stage: string;
        stage_progress: number;
        overall_progress: number;
        message: string;
        eta_seconds?: number;
        current_step?: number;
        total_steps?: number;
        preview_data?: string;
      }) => {
        const { job_id, stage, overall_progress, message, current_step, total_steps, preview_data } = payload;

        const jobIndex = this.jobs.findIndex((j) => j.id === job_id);
        if (jobIndex !== -1) {
          this.jobs[jobIndex].progress = overall_progress;
          this.jobs[jobIndex].currentStage = stage as PipelineStage;
          this.jobs[jobIndex].statusMessage = message;
          if (current_step !== undefined) {
            this.jobs[jobIndex].currentStep = current_step;
          }
          if (total_steps !== undefined) {
            this.jobs[jobIndex].totalSteps = total_steps;
          }
          if (preview_data && preview_data !== this.jobs[jobIndex].previewData) {
            console.log(`[Generation Store] Preview UPDATE for job ${job_id.substring(0,8)} at step ${current_step}/${total_steps}, size: ${preview_data.length} chars`);
            this.jobs[jobIndex].previewData = preview_data;
          }
        }
      });
    },

    cleanupEventListeners() {
      if (this.jobUpdates) {
        this.jobUpdates.cleanup();
        this.jobUpdates = null;
      }
    },

    // ========== Queue Management Actions (from queue store) ==========

    async addToQueue(params: GenerationParams): Promise<string> {
      try {
        const jobId = await invoke<string>('client_add_to_queue', { params });
        this.error = null;
        return jobId;
      } catch (err) {
        const message = 'Failed to add to queue';
        this.error = message;
        console.error(message, err);
        throw err;
      }
    },

    async refreshJobs(): Promise<void> {
      try {
        const result = await invoke<GenerationJob[]>('client_get_queue_jobs');
        this.jobs = result;
        this.error = null;
      } catch (err) {
        const message = 'Failed to refresh jobs';
        this.error = message;
        console.error(message, err);
      }
    },

    async getJob(jobId: string): Promise<GenerationJob | null> {
      try {
        const result = await invoke<GenerationJob | null>('client_get_queue_job', {
          jobId,
        });
        this.error = null;
        return result;
      } catch (err) {
        const message = 'Failed to get job';
        this.error = message;
        console.error(message, err);
        return null;
      }
    },

    async cancelJob(jobId: string): Promise<boolean> {
      try {
        const cancelled = await invoke<boolean>('client_cancel_queue_job', { jobId });
        this.error = null;
        return cancelled;
      } catch (err) {
        const message = 'Failed to cancel job';
        this.error = message;
        console.error(message, err);
        return false;
      }
    },

    async clearCompleted(): Promise<void> {
      try {
        await invoke('clear_completed_jobs');
        await this.refreshJobs();
        this.error = null;
      } catch (err) {
        const message = 'Failed to clear completed jobs';
        this.error = message;
        console.error(message, err);
      }
    },

    startPolling(intervalMs: number = 1000): void {
      if (this.isPolling) return;

      this.isPolling = true;
      this.pollingInterval = window.setInterval(() => {
        this.refreshJobs();
      }, intervalMs);
    },

    stopPolling(): void {
      if (!this.isPolling) return;

      this.isPolling = false;
      if (this.pollingInterval !== null) {
        clearInterval(this.pollingInterval);
        this.pollingInterval = null;
      }
    },

    moveCompletedToHistory(): void {
      const completedOrFailed = this.jobs.filter((j) => j.status === 'completed' || j.status === 'failed');

      if (completedOrFailed.length > 0) {
        this.historyJobs = [...completedOrFailed, ...this.historyJobs];
        this.jobs = this.jobs.filter((j) => j.status !== 'completed' && j.status !== 'failed');
      }
    },

    // ========== UI Persistence (from generation store) ==========

    initializePersistence() {
      if (this._unsubscribe) return;

      this._unsubscribe = this.$subscribe(
        (_mutation, state) => {
          try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify(state.currentParams));
            localStorage.setItem(SEED_RANDOMIZE_KEY, JSON.stringify(state.randomizeSeedOnGenerate));
            localStorage.setItem(SECTION_VISIBILITY_KEY, JSON.stringify(state.sectionVisibility));
          } catch (e) {
            console.warn('Failed to save generation params to localStorage:', e);
          }
        },
        { detached: true },
      );
    },

    cleanupPersistence() {
      if (this._unsubscribe) {
        this._unsubscribe();
        this._unsubscribe = null;
      }
    },

    // ========== UI Section Visibility ==========

    toggleSection(section: keyof UiSectionVisibility) {
      this.sectionVisibility[section] = !this.sectionVisibility[section];
    },

    setSectionVisibility(section: keyof UiSectionVisibility, visible: boolean) {
      this.sectionVisibility[section] = visible;
    },

    // ========== Style Management ==========

    async applyStyle(styleId: string) {
      const { useStylesStore } = await import('./styles');
      const { useModelsStore } = await import('./models');

      const stylesStore = useStylesStore();
      const modelsStore = useModelsStore();

      if (!stylesStore.selectedStyle || stylesStore.selectedStyle.id !== styleId) {
        await stylesStore.loadStyleDetail(styleId);
      }

      const style = stylesStore.selectedStyle;
      if (!style) {
        throw new Error('Style not found');
      }

      this.appliedTemplate = style.promptTemplate;
      this.selectedStyleId = styleId;

      modelsStore.loras.forEach((lora) => {
        lora.isActive = false;
      });

      style.loras.forEach((styleLora) => {
        const lora = modelsStore.loras.find((l) => l.id === styleLora.loraId);
        if (lora) {
          lora.isActive = true;
          lora.strength = styleLora.strength;
        }
      });
    },

    clearStyle() {
      this.selectedStyleId = null;
      this.appliedTemplate = null;
    },

    getFinalPrompt(userPrompt: string): string {
      if (this.appliedTemplate) {
        return this.appliedTemplate.replace(/\{\{prompt\}\}/g, userPrompt);
      }
      return userPrompt;
    },
  },
});
