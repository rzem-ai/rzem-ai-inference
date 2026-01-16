import { defineStore } from 'pinia'
import { ref, computed, onScopeDispose } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export interface GenerationParams {
  prompt: string
  negative_prompt?: string
  steps: number
  cfg_scale: number
  width: number
  height: number
  seed: number
  model: string
}

/**
 * Job status matches backend Rust enum JobStatus with serde lowercase serialization
 * Backend: JobStatus::Pending -> "pending", JobStatus::Running -> "running", etc.
 */
export type JobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'

export interface GenerationJob {
  id: string
  params: GenerationParams
  status: JobStatus
  /**
   * Progress value from backend (f32: 0.0-1.0)
   */
  progress: number
  created_at: number
  started_at?: number
  completed_at?: number
  result_path?: string
  error?: string
}

export const useQueueStore = defineStore('queue', () => {
  // State
  const jobs = ref<GenerationJob[]>([])
  const isPolling = ref(false)
  const pollingInterval = ref<number | null>(null)
  const error = ref<string | null>(null)

  // Listen for job updates from backend
  const unlistenPromise = listen<{
    job_id: string
    status: JobStatus
    progress?: number
    result_path?: string
    error?: string
  }>('job-update', (event) => {
    const { job_id, status, progress, result_path, error: jobError } = event.payload

    // Find and update job in local state
    const jobIndex = jobs.value.findIndex((j) => j.id === job_id)
    if (jobIndex !== -1) {
      jobs.value[jobIndex].status = status
      if (progress !== undefined) {
        jobs.value[jobIndex].progress = progress
      }
      if (result_path) {
        jobs.value[jobIndex].result_path = result_path
      }
      if (jobError) {
        jobs.value[jobIndex].error = jobError
      }
      if (status === 'completed' || status === 'failed') {
        jobs.value[jobIndex].completed_at = Math.floor(Date.now() / 1000)
      }
    }
  })

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

  // Cleanup polling and event listener on store disposal
  onScopeDispose(async () => {
    stopPolling()
    const unlisten = await unlistenPromise
    unlisten()
  })

  // Actions
  async function addToQueue(params: GenerationParams): Promise<string> {
    try {
      const jobId = await invoke<string>('add_to_queue', { params })
      await refreshJobs()
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
      const result = await invoke<GenerationJob[]>('get_queue_jobs')
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
      const result = await invoke<GenerationJob | null>('get_queue_job', {
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
      const cancelled = await invoke<boolean>('cancel_queue_job', { jobId })
      if (cancelled) {
        await refreshJobs()
      }
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

  return {
    // State
    jobs,
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
  }
})
