import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { useGalleryStore } from './gallery';
import { useJobUpdates } from '@/composables/useWebSocket';

export type SamplerType = 'euler' | 'euler_a' | 'dpm_pp_2m';
export type SchedulerType = 'normal' | 'simple' | 'karras' | 'exponential';

// LoRA configuration for generation
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
  bundle_id?: string // If set, use bundle; otherwise use individual components
  model_component_id: string
  clip_component_id: string
  t5_component_id: string
  vae_component_id: string
  sampler?: SamplerType;
  scheduler?: SchedulerType;
  // LoRA adapters to apply during generation
  loras?: LoraConfig[];
}

/**
 * Job status matches backend Rust enum JobStatus with serde lowercase serialization
 * Backend: JobStatus::Pending -> "pending", JobStatus::Running -> "running", etc.
 */
export type JobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

/**
 * Generation statistics from the backend
 */
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

/**
 * Pipeline stages during generation
 */
export type PipelineStage = 'loading_models' | 'encoding_t5' | 'encoding_clip' | 'denoising' | 'decoding_vae' | 'encoding_png';

export interface GenerationJob {
  id: string;
  params: GenerationParams;
  status: JobStatus;
  /**
   * Progress value from backend (f32: 0.0-1.0)
   */
  progress: number;
  /**
   * Current pipeline stage (from job-progress events)
   */
  currentStage?: PipelineStage;
  /**
   * Human-readable status message
   */
  statusMessage?: string;
  /**
   * Current denoising step (when in denoising stage)
   */
  currentStep?: number;
  /**
   * Total denoising steps
   */
  totalSteps?: number;
  created_at: number;
  started_at?: number;
  completed_at?: number;
  result_path?: string;
  error?: string;
  stats?: GenerationStats;
  /**
   * Preview image data (base64-encoded JPEG)
   * Available during generation before final image completes
   */
  previewData?: string;
}

export const useQueueStore = defineStore('queue', {
  state: () => ({
    jobs: [] as GenerationJob[],
    historyJobs: [] as GenerationJob[],
    isPolling: false,
    pollingInterval: null as number | null,
    error: null as string | null,
    jobUpdates: null as ReturnType<typeof useJobUpdates> | null,
  }),

  getters: {
    pendingJobs(state): GenerationJob[] {
      return state.jobs.filter((j) => j.status === 'pending')
    },

    runningJobs(state): GenerationJob[] {
      return state.jobs.filter((j) => j.status === 'running')
    },

    completedJobs(state): GenerationJob[] {
      return state.jobs.filter((j) => j.status === 'completed')
    },

    failedJobs(state): GenerationJob[] {
      return state.jobs.filter((j) => j.status === 'failed')
    },

    queueLength(state): number {
      return state.jobs.filter((j) => j.status === 'pending').length
    },

    hasRunningJobs(state): boolean {
      return state.jobs.filter((j) => j.status === 'running').length > 0
    },
  },

  actions: {
    // Initialize event listeners
    async initializeEventListeners() {
      if (this.jobUpdates) return; // Already initialized

      this.jobUpdates = useJobUpdates();

      // Handle job update events
      await this.jobUpdates.onJobUpdate(async (payload: { job_id: string; status: string; progress?: number; result_path?: string; error?: string; stats?: any }) => {
        const { job_id, status, progress, result_path, error: jobError, stats } = payload;

        // Find and update job in local state
        let jobIndex = this.jobs.findIndex((j) => j.id === job_id);
        if (jobIndex === -1) {
          // Job not found in local state - refresh to get it
          await this.refreshJobs();
          // Re-check after refresh
          jobIndex = this.jobs.findIndex((j) => j.id === job_id);
          if (jobIndex === -1) {
            // Still not found - job may have been cancelled or cleared
            console.warn(`Job ${job_id} not found in local state after refresh`);
            return;
          }
        }

        // Job found - update its properties
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
        // Update started_at when job begins running
        if (status === 'running' && !updatedJob.started_at) {
          updatedJob.started_at = Math.floor(Date.now() / 1000);
        }
        // Update completed_at when job finishes
        if (status === 'completed' || status === 'failed' || status === 'cancelled') {
          updatedJob.completed_at = Math.floor(Date.now() / 1000);
        }

        // Replace job in array to trigger Vue reactivity
        this.jobs[jobIndex] = updatedJob;

        // Refresh gallery when job completes successfully
        if (status === 'completed') {
          const galleryStore = useGalleryStore();
          await galleryStore.loadImages();
        }
      });

      // Handle job progress events
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

        // Find and update job in local state
        const jobIndex = this.jobs.findIndex((j) => j.id === job_id);
        if (jobIndex !== -1) {
          this.jobs[jobIndex].progress = overall_progress;
          this.jobs[jobIndex].currentStage = stage as PipelineStage;
          this.jobs[jobIndex].statusMessage = message;
          // Track denoising steps separately for the step progress bar
          if (current_step !== undefined) {
            this.jobs[jobIndex].currentStep = current_step;
          }
          if (total_steps !== undefined) {
            this.jobs[jobIndex].totalSteps = total_steps;
          }
          // Update preview if included
          if (preview_data) {
            this.jobs[jobIndex].previewData = preview_data;
          }
        }
      });
    },

    // Cleanup event listeners
    cleanupEventListeners() {
      if (this.jobUpdates) {
        this.jobUpdates.cleanup();
        this.jobUpdates = null;
      }
    },

    async addToQueue(params: GenerationParams): Promise<string> {
      try {
        const jobId = await invoke<string>('client_add_to_queue', { params });
        // Note: No refreshJobs() call here - backend emits job-update events
        // which the event listener handles as the single source of truth
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
        // Note: No refreshJobs() call here - backend emits job-update events
        // which the event listener handles as the single source of truth
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
      // Move all completed and failed jobs from queue to history
      const completedOrFailed = this.jobs.filter((j) => j.status === 'completed' || j.status === 'failed');

      if (completedOrFailed.length > 0) {
        // Add to beginning of history (most recent first)
        this.historyJobs = [...completedOrFailed, ...this.historyJobs];

        // Remove from active queue
        this.jobs = this.jobs.filter((j) => j.status !== 'completed' && j.status !== 'failed');
      }
    },
  },
});
