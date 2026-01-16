import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

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

export type JobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'

export interface GenerationJob {
  id: string
  params: GenerationParams
  status: JobStatus
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

  // Actions
  async function addToQueue(params: GenerationParams): Promise<string> {
    try {
      const jobId = await invoke<string>('add_to_queue', { params })
      await refreshJobs()
      return jobId
    } catch (error) {
      console.error('Failed to add to queue:', error)
      throw error
    }
  }

  async function refreshJobs(): Promise<void> {
    try {
      const result = await invoke<GenerationJob[]>('get_queue_jobs')
      jobs.value = result
    } catch (error) {
      console.error('Failed to refresh jobs:', error)
    }
  }

  async function getJob(jobId: string): Promise<GenerationJob | null> {
    try {
      const result = await invoke<GenerationJob | null>('get_queue_job', {
        jobId,
      })
      return result
    } catch (error) {
      console.error('Failed to get job:', error)
      return null
    }
  }

  async function cancelJob(jobId: string): Promise<boolean> {
    try {
      const cancelled = await invoke<boolean>('cancel_queue_job', { jobId })
      if (cancelled) {
        await refreshJobs()
      }
      return cancelled
    } catch (error) {
      console.error('Failed to cancel job:', error)
      return false
    }
  }

  async function clearCompleted(): Promise<void> {
    try {
      await invoke('clear_completed_jobs')
      await refreshJobs()
    } catch (error) {
      console.error('Failed to clear completed jobs:', error)
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
