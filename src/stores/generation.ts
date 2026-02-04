import { defineStore } from 'pinia';
import type { GenerationJob, GenerationParams, GenerationProgress } from '@/types';

export interface UiSectionVisibility {
  quality: boolean;
  style: boolean;
  advanced: boolean;
}

const STORAGE_KEY = 'generation-params';
const SEED_RANDOMIZE_KEY = 'generation-randomize-seed';
const SECTION_VISIBILITY_KEY = 'generation-section-visibility';

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
    jobs: [] as GenerationJob[],
    currentParams: loadParams(),
    randomizeSeedOnGenerate: loadRandomizeSeed(),
    sectionVisibility: loadSectionVisibility(),
    activeProgress: {} as Record<string, GenerationProgress>,
    _unsubscribe: null as (() => void) | null,
    isInitialized: false,
    // Style support
    selectedStyleId: null as string | null,
    appliedTemplate: null as string | null,
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
    // Standard initialization method
    async initialize(): Promise<void> {
      // Guard: only initialize once
      if (this.isInitialized) {
        return
      }

      // Initialize persistence subscription
      this.initializePersistence()
      this.isInitialized = true
    },

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
            // Persist section visibility
            localStorage.setItem(SECTION_VISIBILITY_KEY, JSON.stringify(state.sectionVisibility));
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

    toggleSection(section: keyof UiSectionVisibility) {
      this.sectionVisibility[section] = !this.sectionVisibility[section];
    },

    setSectionVisibility(section: keyof UiSectionVisibility, visible: boolean) {
      this.sectionVisibility[section] = visible;
    },

    async applyStyle(styleId: string) {
      const { useStylesStore } = await import('./styles');
      const { useModelsStore } = await import('./models');

      const stylesStore = useStylesStore();
      const modelsStore = useModelsStore();

      // Load style detail if not already loaded
      if (!stylesStore.selectedStyle || stylesStore.selectedStyle.id !== styleId) {
        await stylesStore.loadStyleDetail(styleId);
      }

      const style = stylesStore.selectedStyle;
      if (!style) {
        throw new Error('Style not found');
      }

      // Apply template
      this.appliedTemplate = style.promptTemplate;
      this.selectedStyleId = styleId;

      // Apply style's LoRAs to models store
      // First, deactivate all LoRAs
      modelsStore.loras.forEach((lora) => {
        lora.isActive = false;
      });

      // Then activate and configure LoRAs from the style
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

      // Optionally clear LoRA activations
      // (commented out to preserve user's manual LoRA selections if they want)
      // const { useModelsStore } = require('./models');
      // const modelsStore = useModelsStore();
      // modelsStore.loras.forEach((lora) => {
      //   lora.isActive = false;
      // });
    },

    getFinalPrompt(userPrompt: string): string {
      if (this.appliedTemplate) {
        return this.appliedTemplate.replace(/\{\{prompt\}\}/g, userPrompt);
      }
      return userPrompt;
    },
  },
});
