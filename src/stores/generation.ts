import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { GenerationJob, GenerationParams } from '@/types'

export const useGenerationStore = defineStore('generation', () => {
  // State
  const jobs = ref<GenerationJob[]>([])
  const currentParams = ref<GenerationParams>({
    mode: 'txt2img',
    prompt: '',
    negativePrompt: '',
    steps: 20,
    cfgScale: 7.5,
    width: 1024,
    height: 1024,
    seed: -1,
    model: 'flux-schnell'
  })

  // Getters
  const queuedJobs = computed(() =>
    jobs.value.filter(job => job.status === 'Queued')
  )

  const runningJobs = computed(() =>
    jobs.value.filter(job => job.status === 'Running')
  )

  const completedJobs = computed(() =>
    jobs.value.filter(job => job.status === 'Completed')
  )

  // Actions
  function addJob(job: GenerationJob) {
    jobs.value.push(job)
  }

  function updateJobStatus(id: string, status: GenerationJob['status']): boolean {
    const job = jobs.value.find(j => j.id === id)
    if (job) {
      job.status = status
      return true
    }
    return false
  }

  function clearCompleted() {
    jobs.value = jobs.value.filter(job => job.status !== 'Completed')
  }

  return {
    // State
    jobs,
    currentParams,
    // Getters
    queuedJobs,
    runningJobs,
    completedJobs,
    // Actions
    addJob,
    updateJobStatus,
    clearCompleted
  }
})
