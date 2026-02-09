import { defineStore } from 'pinia';
import { invoke } from '@/utils/backend-bridge';
import { useGalleryStore } from './gallery';
import { useJobUpdates } from '@/composables/useWebSocket';
import { GenerationJob, GenerationParams, JobStatus, PipelineStage, Sampler, Scheduler } from '@/types';

// ========== Valid Options ==========

const VALID_SAMPLERS: Sampler[] = ['euler', 'euler_a', 'dpm_pp_2m'];
const VALID_SCHEDULERS: Scheduler[] = ['normal', 'simple', 'karras', 'exponential'];

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

const defaultParams: GenerationParams = {
  prompt: 'A West Highland White Terrier in the style of a Pixar cartoon',
  negative_prompt: '',
  steps: 4,
  cfg_scale: 1.0,
  sampler: 'euler',
  scheduler: 'normal',
  width: 1024,
  height: 1024,
  seed: -1,
  bundle_id: undefined,
  model_component_id: '',
  t5_component_id: '',
  clip_component_id: '',
  vae_component_id: '',
  mode: 'txt2img',
};

function loadParams(): GenerationParams {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);

      // Normalize and validate sampler (fix old PascalCase values from before serde lowercase)
      if (parsed.sampler) {
        const normalized = parsed.sampler.toLowerCase().replace(/\+\+/g, '_pp_').replace(/-/g, '_') as Sampler;
        parsed.sampler = VALID_SAMPLERS.includes(normalized) ? normalized : defaultParams.sampler;
      }

      // Normalize and validate scheduler
      if (parsed.scheduler) {
        const normalized = parsed.scheduler.toLowerCase() as Scheduler;
        parsed.scheduler = VALID_SCHEDULERS.includes(normalized) ? normalized : defaultParams.scheduler;
      }

      return { ...defaultParams, ...parsed };
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
      if (state.currentParams.bundle_id) {
        return true;
      }
      return !!(
        state.currentParams.model_component_id &&
        state.currentParams.t5_component_id &&
        state.currentParams.clip_component_id &&
        state.currentParams.vae_component_id
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

      await this.jobUpdates.onJobUpdate(
        async (payload: {
          job_id: string;
          status: string;
          progress?: number;
          result_path?: string;
          error?: string;
          stats?: any;
          current_step?: number;
          total_steps?: number;
          stage?: string;
          preview_data?: string;
        }) => {
          const { job_id, status, progress, result_path, error: jobError, stats, current_step, total_steps, stage, preview_data } = payload;

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
          if (stage) {
            updatedJob.currentStage = stage as PipelineStage;
          }
          if (current_step !== undefined) {
            updatedJob.currentStep = current_step;
          }
          if (total_steps !== undefined) {
            updatedJob.totalSteps = total_steps;
          }
          if (preview_data && preview_data !== updatedJob.previewData) {
            updatedJob.previewData = preview_data;
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
        },
      );
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
        const result = await invoke<{ job_id: string }>('client_add_to_queue', { params });
        this.error = null;
        // Refresh jobs so the new job appears in the queue immediately
        await this.refreshJobs();
        return result.job_id;
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
