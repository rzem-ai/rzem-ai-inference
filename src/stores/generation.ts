import { defineStore } from 'pinia';
import type { GenerationJob, GenerationParams, GenerationProgress } from '@/types';

const STORAGE_KEY = 'generation-params';
const SEED_RANDOMIZE_KEY = 'generation-randomize-seed';

const defaultParams: GenerationParams = {
  mode: 'txt2img',
  prompt: 'A West Highland White Terrier in the style of a Pixar cartoon',
  negativePrompt: '',
  steps: 4, // Flux Schnell default
  cfgScale: 1.0, // Flux uses CFG=1 typically
  sampler: 'euler', // Default sampler
  scheduler: 'normal', // Default scheduler
  width: 1024,
  height: 1024,
  seed: -1,
  modelType: 'schnell', // Model ID: 'schnell' or 'dev'
  batchSize: 1,
  // Bundle system (new)
  bundleId: undefined,
  modelComponentId: '',
  t5ComponentId: '',
  clipComponentId: '',
  vaeComponentId: '',
};

function loadParams(): GenerationParams {
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

export const useGenerationStore = defineStore('generation', {
  state: () => ({
    jobs: [] as GenerationJob[],
    currentParams: loadParams(),
    randomizeSeedOnGenerate: loadRandomizeSeed(),
    activeProgress: {} as Record<string, GenerationProgress>,
    _unsubscribe: null as (() => void) | null,
  }),

  getters: {
    queuedJobs(state): GenerationJob[] {
      return state.jobs.filter((job) => job.status === 'Queued');
    },

    runningJobs(state): GenerationJob[] {
      return state.jobs.filter((job) => job.status === 'Running');
    },

    completedJobs(state): GenerationJob[] {
      return state.jobs.filter((job) => job.status === 'Completed');
    },

    isGenerating(state): boolean {
      return state.jobs.filter((job) => job.status === 'Running').length > 0;
    },

    getProgress(state) {
      return (jobId: string) => state.activeProgress[jobId];
    },

    isValidConfiguration(state): boolean {
      // Bundle mode: bundleId must be set
      if (state.currentParams.bundleId) {
        return true;
      }

      // Individual mode: all component IDs must be set
      return !!(
        state.currentParams.modelComponentId &&
        state.currentParams.t5ComponentId &&
        state.currentParams.clipComponentId &&
        state.currentParams.vaeComponentId
      );
    },
  },

  actions: {
    // Initialize automatic localStorage persistence
    initializePersistence() {
      if (this._unsubscribe) return; // Already initialized

      // Subscribe to state changes for automatic persistence
      this._unsubscribe = this.$subscribe(
        (_mutation, state) => {
          try {
            // Persist currentParams
            localStorage.setItem(STORAGE_KEY, JSON.stringify(state.currentParams));
            // Persist randomizeSeedOnGenerate
            localStorage.setItem(SEED_RANDOMIZE_KEY, JSON.stringify(state.randomizeSeedOnGenerate));
          } catch (e) {
            console.warn('Failed to save generation params to localStorage:', e);
          }
        },
        { detached: true },
      );
    },

    // Cleanup persistence subscription
    cleanupPersistence() {
      if (this._unsubscribe) {
        this._unsubscribe();
        this._unsubscribe = null;
      }
    },

    addJob(job: GenerationJob) {
      this.jobs.push(job);
    },

    updateJobStatus(id: string, status: GenerationJob['status']): boolean {
      const job = this.jobs.find((j) => j.id === id);
      if (job) {
        job.status = status;
        return true;
      }
      return false;
    },

    clearCompleted() {
      this.jobs = this.jobs.filter((job) => job.status !== 'Completed');
    },

    updateProgress(jobId: string, progress: GenerationProgress) {
      this.activeProgress[jobId] = progress;
    },

    clearProgress(jobId: string) {
      delete this.activeProgress[jobId];
    },
  },
});
