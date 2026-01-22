import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import type { GenerationJob, GenerationParams, GenerationProgress } from '@/types'

const STORAGE_KEY = 'generation-params'
const SEED_RANDOMIZE_KEY = 'generation-randomize-seed'

const defaultParams: GenerationParams = {
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
}

function loadParams(): GenerationParams {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) {
      return { ...defaultParams, ...JSON.parse(stored) }
    }
  } catch (e) {
    console.warn('Failed to load generation params from localStorage:', e)
  }
  return { ...defaultParams }
}

function loadRandomizeSeed(): boolean {
  try {
    const stored = localStorage.getItem(SEED_RANDOMIZE_KEY)
    if (stored !== null) {
      return JSON.parse(stored)
    }
  } catch (e) {
    console.warn('Failed to load randomize seed setting from localStorage:', e)
  }
  return true
}

export const useGenerationStore = defineStore('generation', () => {
  // State
  const jobs = ref<GenerationJob[]>([])
  const currentParams = ref<GenerationParams>(loadParams())

  // When true, a new random seed is generated for each generation
  // When false, the current seed value is used (locked)
  const randomizeSeedOnGenerate = ref(loadRandomizeSeed())

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

  // Persist params to localStorage on change
  watch(currentParams, (params) => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(params))
    } catch (e) {
      console.warn('Failed to save generation params to localStorage:', e)
    }
  }, { deep: true })

  watch(randomizeSeedOnGenerate, (value) => {
    try {
      localStorage.setItem(SEED_RANDOMIZE_KEY, JSON.stringify(value))
    } catch (e) {
      console.warn('Failed to save randomize seed setting to localStorage:', e)
    }
  })

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
