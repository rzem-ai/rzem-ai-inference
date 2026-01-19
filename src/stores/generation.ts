import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { GenerationJob, GenerationParams, GenerationProgress } from '@/types'

export const useGenerationStore = defineStore('generation', () => {
  // State
  const jobs = ref<GenerationJob[]>([])
  const currentParams = ref<GenerationParams>({
    mode: 'txt2img',
    prompt: 'A West Highland White Terrier in the style of a Pixar cartoon',
    negativePrompt: '',
    steps: 4,  // Flux Schnell default
    cfgScale: 1.0,  // Flux uses CFG=1 typically
    sampler: 'euler',  // Default sampler
    scheduler: 'normal',  // Default scheduler
    width: 1024,
    height: 1024,
    seed: -1,
    model: 'schnell',  // Model ID: 'schnell' or 'dev'
    batchSize: 1
  })

  // When true, a new random seed is generated for each generation
  // When false, the current seed value is used (locked)
  const randomizeSeedOnGenerate = ref(true)

  const activeProgress = ref<Record<string, GenerationProgress>>({})

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

  // Getter for active generation
  const isGenerating = computed(() => runningJobs.value.length > 0)

  // Getter for progress of specific job
  const getProgress = (jobId: string) => activeProgress.value[jobId]

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

  function updateProgress(jobId: string, progress: GenerationProgress) {
    activeProgress.value[jobId] = progress
  }

  function clearProgress(jobId: string) {
    delete activeProgress.value[jobId]
  }

  return {
    // State
    jobs,
    currentParams,
    activeProgress,
    randomizeSeedOnGenerate,
    // Getters
    queuedJobs,
    runningJobs,
    completedJobs,
    isGenerating,
    getProgress,
    // Actions
    addJob,
    updateJobStatus,
    clearCompleted,
    updateProgress,
    clearProgress
  }
})
