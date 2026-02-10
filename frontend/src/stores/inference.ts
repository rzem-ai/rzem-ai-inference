import { ref, reactive, computed } from 'vue';
import { defineStore } from 'pinia';
import type { PywebviewAPI } from '@/types/pywebview';
import type { ModelPreset, SubmitJobParams, LoraParam, InferenceEvent, GeneratedImage } from '@/types/inference';

export const useInferenceStore = defineStore('inference', () => {
  // ── API reference ──
  let api: PywebviewAPI | null = null;

  // ── Engine state ──
  const engineReady = ref(false);
  const engineStarting = ref(false);
  const modelStatus = ref<string | null>(null);

  // ── Job state ──
  const currentJobId = ref<string | null>(null);
  const isGenerating = ref(false);
  const progress = ref<{ step: number; totalSteps: number } | null>(null);
  const error = ref<string | null>(null);

  // ── Results ──
  const generatedImages = ref<GeneratedImage[]>([]);
  const latestImage = computed(() => generatedImages.value[0] ?? null);
  const previewDataUrl = ref<string | null>(null);

  // ── Presets & form params ──
  const presets = ref<ModelPreset[]>([]);
  const selectedPresetId = ref<string | null>(null);
  const selectedPreset = computed(() => presets.value.find((p) => p.id === selectedPresetId.value) ?? null);

  const params = reactive<SubmitJobParams>({
    prompt: 'Moebiusstyle, a planet with rings, space dust, psychedelic art',
    transformer_model: 'black-forest-labs/FLUX.1-dev',
    transformer_type: 'flux1_dev',
    vae_model: 'black-forest-labs/FLUX.1-dev',
    clip_tokenizer: 'openai/clip-vit-large-patch14',
    clip_encoder: 'openai/clip-vit-large-patch14',
    t5_tokenizer: 'google/t5-v1_1-xxl',
    t5_encoder: 'google/t5-v1_1-xxl',
    steps: 20,
    cfg_scale: 1.0,
    width: 1024,
    height: 1024,
    seed: -1,
    sampler: 'euler',
    scheduler: 'normal',
    loras: [],
  });

  // ── Event log (for debugging) ──
  const events = ref<InferenceEvent[]>([]);

  // ── Polling ──
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  // ── Actions ──

  function setApi(apiRef: PywebviewAPI) {
    api = apiRef;
  }

  async function startEngine() {
    if (!api || engineStarting.value) return;
    engineStarting.value = true;
    error.value = null;
    try {
      const res = await api.start_engine();
      if (res.status === 'error') {
        error.value = res.message ?? 'Failed to start engine';
        return;
      }
      engineReady.value = true;
      startPolling();
    } catch (e: any) {
      error.value = e.message ?? 'Failed to start engine';
    } finally {
      engineStarting.value = false;
    }
  }

  async function stopEngine() {
    if (!api) return;
    stopPolling();
    try {
      await api.stop_engine();
    } finally {
      engineReady.value = false;
      modelStatus.value = null;
    }
  }

  async function loadPresets() {
    if (!api) return;
    const res = await api.get_model_presets();
    if (res.status === 'success' && res.presets) {
      presets.value = res.presets;
    }
  }

  function applyPreset(preset: ModelPreset) {
    selectedPresetId.value = preset.id;
    params.transformer_model = preset.transformer_model;
    params.transformer_type = preset.transformer_type;
    params.vae_model = preset.vae_model;
    // Apply text encoder fields
    params.clip_tokenizer = preset.text_encoders.clip_tokenizer;
    params.clip_encoder = preset.text_encoders.clip_encoder;
    params.t5_tokenizer = preset.text_encoders.t5_tokenizer;
    params.t5_encoder = preset.text_encoders.t5_encoder;
    params.qwen3_tokenizer = preset.text_encoders.qwen3_tokenizer;
    params.qwen3_encoder = preset.text_encoders.qwen3_encoder;
  }

  async function submitJob() {
    if (!api || !engineReady.value || isGenerating.value) return;
    if (!params.prompt.trim()) {
      error.value = 'Prompt is required';
      return;
    }
    error.value = null;
    isGenerating.value = true;
    progress.value = null;

    console.log('submit job: ', params);

    // Build submission params, omitting undefined text encoder fields
    const jobParams: Record<string, any> = { ...params };

    console.log('submit jobParams: ', jobParams);

    // Clean out undefined keys so Python doesn't get "undefined" strings
    for (const key of Object.keys(jobParams)) {
      if (jobParams[key] === undefined || jobParams[key] === '') {
        delete jobParams[key];
      }
    }

    const res = await api.submit_job(jobParams);
    if (res.status === 'error') {
      error.value = res.message ?? 'Failed to submit job';
      isGenerating.value = false;
      return;
    }
    currentJobId.value = res.job_id ?? null;
  }

  async function cancelJob() {
    if (!api || !currentJobId.value) return;
    await api.cancel_job({ job_id: currentJobId.value });
    isGenerating.value = false;
    currentJobId.value = null;
    progress.value = null;
  }

  function addLora() {
    params.loras.push({ model_file: '', strength: 1.0 });
  }

  function removeLora(index: number) {
    params.loras.splice(index, 1);
  }

  // ── Polling ──

  function startPolling() {
    if (pollTimer) return;
    pollTimer = setInterval(pollEvents, 200);
  }

  function stopPolling() {
    if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  async function pollEvents() {
    if (!api) return;
    try {
      const res = await api.poll_events();
      if (res.status === 'success' && res.events?.length) {
        processEvents(res.events);
      }
    } catch {
      // Silently ignore poll errors
    }
  }

  async function processEvents(newEvents: InferenceEvent[]) {
    for (const event of newEvents) {
      events.value.push(event);

      switch (event.type) {
        case 'model_loading':
          console.log('model_loading:', event.data);
          modelStatus.value = event.data.message ?? 'Loading model...';
          break;

        case 'model_loaded':
          console.log('model_loaded:', event.data);
          modelStatus.value = 'Model loaded';
          break;

        case 'model_unloaded':
          modelStatus.value = null;
          break;

        case 'job_queued':
          // Job accepted into queue
          break;

        case 'job_started':
          console.log('job_started:', event.data);
          progress.value = { step: 0, totalSteps: params.steps };
          break;

        case 'job_progress':
          console.log('job_progress:', event.data);
          progress.value = {
            step: event.data.step ?? 0,
            totalSteps: event.data.total_steps ?? params.steps,
          };
          // Preview image handling could go here
          if (event.data.preview_path) {
            loadImageDataUrl(event.data.preview_path);
          }
          break;

        case 'job_completed': {
          isGenerating.value = false;
          currentJobId.value = null;
          progress.value = null;
          previewDataUrl.value = null;
          const img: GeneratedImage = {
            jobId: event.data.job_id,
            imagePath: event.data.image_path ?? '',
            seed: event.data.seed ?? -1,
            timestamp: event.data.timestamp ?? Date.now() / 1000,
          };
          if (img.imagePath && api) {
            const imgRes = await api.get_image_base64({
              image_path: img.imagePath,
            });
            if (imgRes.status === 'success' && imgRes.data_url) {
              img.dataUrl = imgRes.data_url;
            }
          }
          generatedImages.value.unshift(img);
          break;
        }

        case 'job_failed':
          isGenerating.value = false;
          currentJobId.value = null;
          progress.value = null;
          previewDataUrl.value = null;
          error.value = event.data.error ?? 'Generation failed';
          break;

        case 'job_cancelled':
          isGenerating.value = false;
          currentJobId.value = null;
          progress.value = null;
          previewDataUrl.value = null;
          break;

        default:
          console.log('event:', event.type, event.data);
      }
    }
  }

  function injectDebugEvent(eventType: string) {
    const dummyEvents: Record<string, InferenceEvent | null> = {
      default: null,
      model_loading: { type: 'model_loading', data: { message: 'Loading transformer model...' } },
      model_loaded: { type: 'model_loaded', data: {} },
      model_unloaded: { type: 'model_unloaded', data: {} },
      job_queued: { type: 'job_queued', data: { job_id: 'debug-001' } },
      job_started: { type: 'job_started', data: { job_id: 'debug-001' } },
      job_progress: { type: 'job_progress', data: { job_id: 'debug-001', step: 12, total_steps: 20 } },
      job_completed: { type: 'job_completed', data: { job_id: 'debug-001', seed: 42, timestamp: Date.now() / 1000 } },
      job_failed: { type: 'job_failed', data: { job_id: 'debug-001', error: 'CUDA out of memory. Tried to allocate 2.00 GiB' } },
      job_cancelled: { type: 'job_cancelled', data: { job_id: 'debug-001' } },
    };

    if (eventType === 'default') {
      // Reset all transient state
      isGenerating.value = false;
      currentJobId.value = null;
      progress.value = null;
      previewDataUrl.value = null;
      error.value = null;
      modelStatus.value = null;
      return;
    }

    // Set isGenerating for job events that need it for the UI to show progress
    if (['job_queued', 'job_started', 'job_progress'].includes(eventType)) {
      isGenerating.value = true;
      currentJobId.value = 'debug-001';
    }

    const event = dummyEvents[eventType];
    if (event) {
      processEvents([event]);
    }
  }

  async function loadImageDataUrl(imagePath: string) {
    if (!api) return;
    const res = await api.get_image_base64({ image_path: imagePath });
    if (res.status === 'success' && res.data_url) {
      previewDataUrl.value = res.data_url;
    }
  }

  return {
    // State
    engineReady,
    engineStarting,
    modelStatus,
    currentJobId,
    isGenerating,
    progress,
    error,
    generatedImages,
    latestImage,
    previewDataUrl,
    presets,
    selectedPresetId,
    selectedPreset,
    params,
    events,
    // Actions
    setApi,
    startEngine,
    stopEngine,
    loadPresets,
    applyPreset,
    submitJob,
    cancelJob,
    addLora,
    removeLora,
    startPolling,
    stopPolling,
    injectDebugEvent,
  };
});
