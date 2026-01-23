import { defineStore } from 'pinia'
import { ref, computed, onScopeDispose } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useGalleryStore } from './gallery'
import { useJobUpdates } from '@/composables/useWebSocket'

export type SamplerType = 'euler' | 'euler_a' | 'dpm_pp_2m'
export type SchedulerType = 'normal' | 'simple' | 'karras' | 'exponential'

export interface GenerationParams {
  prompt: string
  negative_prompt?: string
  steps: number
  cfg_scale: number
  width: number
  height: number
  seed: number
  model: string
  sampler?: SamplerType
  scheduler?: SchedulerType
}

/**
 * Job status matches backend Rust enum JobStatus with serde lowercase serialization
 * Backend: JobStatus::Pending -> "pending", JobStatus::Running -> "running", etc.
 */
export type JobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'

/**
 * Generation statistics from the backend
 */
export interface GenerationStats {
  model_load_ms?: number
  t5_load_ms?: number
  clip_load_ms?: number
  vae_load_ms?: number
  flux_load_ms?: number
  t5_encode_ms: number
  clip_encode_ms: number
  denoise_ms: number
  vae_decode_ms: number
  png_encode_ms: number
  total_ms: number
  t5_embedding_shape: number[]
  clip_embedding_shape: number[]
  latent_shape: number[]
  image_shape: number[]
  steps: number
  model_type: string
}

/**
 * Pipeline stages during generation
 */
export type PipelineStage =
  | 'loading_models'
  | 'encoding_t5'
  | 'encoding_clip'
  | 'denoising'
  | 'decoding_vae'
  | 'encoding_png'

export interface GenerationJob {
  id: string
  params: GenerationParams
  status: JobStatus
  /**
   * Progress value from backend (f32: 0.0-1.0)
   */
  progress: number
  /**
   * Current pipeline stage (from job-progress events)
   */
  currentStage?: PipelineStage
  /**
   * Human-readable status message
   */
  statusMessage?: string
  /**
   * Current denoising step (when in denoising stage)
   */
  currentStep?: number
  /**
   * Total denoising steps
   */
  totalSteps?: number
  created_at: number
  started_at?: number
  completed_at?: number
  result_path?: string
  error?: string
  stats?: GenerationStats
}

export const useQueueStore = defineStore('queue', () => {
  // State
  const jobs = ref<GenerationJob[]>([])
  const historyJobs = ref<GenerationJob[]>([])
  const isPolling = ref(false)
  const pollingInterval = ref<number | null>(null)
  const error = ref<string | null>(null)

  // Set up unified job update listeners (works in both local and client modes)
  const jobUpdates = useJobUpdates()

  // Handle job update events
  const handleJobUpdate = async (payload: {
    job_id: string
    status: string
    progress?: number
    result_path?: string
    error?: string
    stats?: any
  }) => {
    const { job_id, status, progress, result_path, error: jobError, stats } = payload

    // Find and update job in local state
    let jobIndex = jobs.value.findIndex((j) => j.id === job_id)
    if (jobIndex === -1) {
      // Job not found in local state
      if (status === 'pending') {
        // This is likely a new job - refresh to get it
        await refreshJobs()
      }
      return
    }

    if (jobIndex !== -1) {
      // Update job properties - use object spread to ensure reactivity triggers
      const updatedJob = { ...jobs.value[jobIndex] }
      updatedJob.status = status as JobStatus
      if (progress !== undefined) {
        updatedJob.progress = progress
      }
      if (result_path) {
        updatedJob.result_path = result_path
      }
      if (jobError) {
        updatedJob.error = jobError
      }
      if (stats) {
        updatedJob.stats = stats
      }
      // Update started_at when job begins running
      if (status === 'running' && !updatedJob.started_at) {
        updatedJob.started_at = Math.floor(Date.now() / 1000)
      }
      // Update completed_at when job finishes
      if (status === 'completed' || status === 'failed' || status === 'cancelled') {
        updatedJob.completed_at = Math.floor(Date.now() / 1000)
      }

      // Replace job in array to trigger Vue reactivity
      jobs.value[jobIndex] = updatedJob

      // Refresh gallery when job completes successfully
      if (status === 'completed') {
        // Get gallery store inside the callback (after Pinia is initialized)
        const galleryStore = useGalleryStore()
        await galleryStore.loadImages()
      }
    }
  }

  // Handle job progress events
  const handleJobProgress = (payload: {
    job_id: string
    stage: string
    stage_progress: number
    overall_progress: number
    message: string
    eta_seconds?: number
    current_step?: number
    total_steps?: number
  }) => {
    const { job_id, stage, overall_progress, message, current_step, total_steps } = payload

    // Find and update job in local state
    const jobIndex = jobs.value.findIndex((j) => j.id === job_id)
    if (jobIndex !== -1) {
      jobs.value[jobIndex].progress = overall_progress
      jobs.value[jobIndex].currentStage = stage as PipelineStage
      jobs.value[jobIndex].statusMessage = message
      // Track denoising steps separately for the step progress bar
      if (current_step !== undefined) {
        jobs.value[jobIndex].currentStep = current_step
      }
      if (total_steps !== undefined) {
        jobs.value[jobIndex].totalSteps = total_steps
      }
    }
  }

  // Subscribe to job update events
  jobUpdates.onJobUpdate(handleJobUpdate)
  jobUpdates.onJobProgress(handleJobProgress)

  // Computed
  const pendingJobs = computed(() =>
    jobs.value.filter((j) => j.status === 'pending')
  )

  const runningJobs = computed(() =>
    jobs.value.filter((j) => j.status === 'running')
  )

  const completedJobs = computed(() =>
    jobs.value.filter((j) => j.status === 'completed')
  )

  const failedJobs = computed(() =>
    jobs.value.filter((j) => j.status === 'failed')
  )

  const queueLength = computed(() => pendingJobs.value.length)
  const hasRunningJobs = computed(() => runningJobs.value.length > 0)

  // Cleanup polling and event listeners on store disposal
  onScopeDispose(() => {
    stopPolling()
    jobUpdates.cleanup()
  })

  // Actions
  async function addToQueue(params: GenerationParams): Promise<string> {
    try {
      const jobId = await invoke<string>('client_add_to_queue', { params })
      // Note: No refreshJobs() call here - backend emits job-update events
      // which the event listener handles as the single source of truth
      error.value = null
      return jobId
    } catch (err) {
      const message = 'Failed to add to queue'
      error.value = message
      console.error(message, err)
      throw err
    }
  }

  async function refreshJobs(): Promise<void> {
    try {
      const result = await invoke<GenerationJob[]>('client_get_queue_jobs')
      jobs.value = result
      error.value = null
    } catch (err) {
      const message = 'Failed to refresh jobs'
      error.value = message
      console.error(message, err)
    }
  }

  async function getJob(jobId: string): Promise<GenerationJob | null> {
    try {
      const result = await invoke<GenerationJob | null>('client_get_queue_job', {
        jobId,
      })
      error.value = null
      return result
    } catch (err) {
      const message = 'Failed to get job'
      error.value = message
      console.error(message, err)
      return null
    }
  }

  async function cancelJob(jobId: string): Promise<boolean> {
    try {
      const cancelled = await invoke<boolean>('client_cancel_queue_job', { jobId })
      // Note: No refreshJobs() call here - backend emits job-update events
      // which the event listener handles as the single source of truth
      error.value = null
      return cancelled
    } catch (err) {
      const message = 'Failed to cancel job'
      error.value = message
      console.error(message, err)
      return false
    }
  }

  async function clearCompleted(): Promise<void> {
    try {
      await invoke('clear_completed_jobs')
      await refreshJobs()
      error.value = null
    } catch (err) {
      const message = 'Failed to clear completed jobs'
      error.value = message
      console.error(message, err)
    }
  }

  function startPolling(intervalMs: number = 1000): void {
    if (isPolling.value) return

    isPolling.value = true
    pollingInterval.value = window.setInterval(() => {
      refreshJobs()
    }, intervalMs)
  }

  function stopPolling(): void {
    if (!isPolling.value) return

    isPolling.value = false
    if (pollingInterval.value !== null) {
      clearInterval(pollingInterval.value)
      pollingInterval.value = null
    }
  }

  function moveCompletedToHistory(): void {
    // Move all completed and failed jobs from queue to history
    const completedOrFailed = jobs.value.filter(
      (j) => j.status === 'completed' || j.status === 'failed'
    )

    if (completedOrFailed.length > 0) {
      // Add to beginning of history (most recent first)
      historyJobs.value = [...completedOrFailed, ...historyJobs.value]

      // Remove from active queue
      jobs.value = jobs.value.filter(
        (j) => j.status !== 'completed' && j.status !== 'failed'
      )
    }
  }

  return {
    // State
    jobs,
    historyJobs,
    isPolling,
    error,

    // Computed
    pendingJobs,
    runningJobs,
    completedJobs,
    failedJobs,
    queueLength,
    hasRunningJobs,

    // Actions
    addToQueue,
    refreshJobs,
    getJob,
    cancelJob,
    clearCompleted,
    startPolling,
    stopPolling,
    moveCompletedToHistory,
  }
})
