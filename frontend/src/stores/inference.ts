import { defineStore } from 'pinia';
import type { PywebviewAPI } from '@/types/pywebview';
import type { ModelBundle, SubmitJobParams, LoraParam, InferenceEvent, GeneratedImage, StyleLoRA } from '@/types/inference';

// Non-reactive module-level state (no reactivity tracking needed)
let _api: PywebviewAPI | null = null;
let _pollTimer: ReturnType<typeof setInterval> | null = null;
let _debugImages: { output: string | null; previews: Record<string, string> } | null = null;

export interface AspectRatio {
  label: string;
  width: number;
  height: number;
}

export const ASPECT_RATIOS: AspectRatio[] = [
  { label: '1:2', width: 512, height: 1024 },
  { label: '9:16', width: 576, height: 1024 },
  { label: '3:4', width: 768, height: 1024 },
  { label: '1:1', width: 1024, height: 1024 },
  { label: '4:3', width: 1024, height: 768 },
  { label: '16:9', width: 1024, height: 576 },
  { label: '2:1', width: 1024, height: 512 },
];

export const useInferenceStore = defineStore('inference', {
  state: () => ({
    // Engine state
    engineReady: false,
    engineStarting: false,
    modelStatus: null as string | null,

    // Job state
    currentJobId: null as string | null,
    isGenerating: false,
    progress: null as { step: number; totalSteps: number; aspectRatio: string; width: number; height: number } | null,
    error: null as string | null,

    // Results
    generatedImages: [] as GeneratedImage[],
    previewDataUrl: null as string | null,

    // Bundles & form params
    bundles: [] as ModelBundle[],
    selectedBundleId: null as string | null,

    // GPU info
    gpuDeviceType: null as string | null,
    gpuDeviceName: null as string | null,
    gpuTotalVramGb: 0,

    // Style
    selectedStyleId: null as string | null,
    stylePromptTemplate: null as string | null,
    styleNegativePrompt: null as string | null,

    // UI state
    chatbotOpen: false,
    activeAspectRatio: '1:1' as string | null,
    advancedOpen: false,
    devMode: false,

    params: {
      prompt: 'a planet with rings, space dust, psychedelic art',
      transformer_model: 'black-forest-labs/FLUX.1-dev',
      transformer_type: 'flux1_dev',
      vae_model: 'black-forest-labs/FLUX.1-dev',
      clip_tokenizer: 'openai/clip-vit-large-patch14',
      clip_encoder: 'openai/clip-vit-large-patch14',
      t5_tokenizer: 'google/t5-v1_1-xxl',
      t5_encoder: 'google/t5-v1_1-xxl',
      steps: 28,
      cfg_scale: 3.5,
      width: 1024,
      height: 1024,
      seed: -1,
      sampler: 'euler',
      scheduler: 'simple',
      loras: [] as LoraParam[],
    } as SubmitJobParams,

    // Batch state
    batchActive: false,
    batchTotal: 0,
    batchCompleted: 0,
    batchJobIds: [] as string[],

    // Event log (for debugging)
    events: [] as InferenceEvent[],
  }),

  getters: {
    latestImage(state): GeneratedImage | null {
      return state.generatedImages[0] ?? null;
    },

    selectedBundle(state): ModelBundle | null {
      return state.bundles.find((b) => b.id === state.selectedBundleId) ?? null;
    },

    bundlesByType(state): { label: string; items: ModelBundle[] }[] {
      const typeLabels: Record<string, string> = {
        flux1_dev: 'FLUX.1 Dev',
        flux2_dev: 'FLUX.2 Dev',
        z_image: 'Z-Image',
        qwen_image: 'Qwen-Image',
      };
      const groups = new Map<string, ModelBundle[]>();
      for (const b of state.bundles) {
        const list = groups.get(b.transformer_type) ?? [];
        list.push(b);
        groups.set(b.transformer_type, list);
      }
      return Array.from(groups.entries()).map(([type, items]) => ({
        label: typeLabels[type] ?? type,
        items,
      }));
    },
  },

  actions: {
    setApi(apiRef: PywebviewAPI) {
      _api = apiRef;
    },

    async loadGpuInfo() {
      if (!_api) return;
      try {
        const res = await _api.get_gpu_info();
        if (res.status === 'success') {
          this.gpuDeviceType = res.device_type ?? null;
          this.gpuDeviceName = res.device_name ?? null;
          this.gpuTotalVramGb = res.total_vram_gb ?? 0;
        }
      } catch {
        // GPU info is non-critical — fail silently
      }
    },

    async startEngine() {
      if (!_api || this.engineStarting) return;
      this.engineStarting = true;
      this.error = null;
      try {
        const res = await _api.start_engine();
        if (res.status === 'error') {
          this.error = res.message ?? 'Failed to start engine';
          return;
        }
        this.engineReady = true;
        this.startPolling();
      } catch (e: any) {
        this.error = e.message ?? 'Failed to start engine';
      } finally {
        this.engineStarting = false;
      }
    },

    async stopEngine() {
      if (!_api) return;
      this.stopPolling();
      try {
        await _api.stop_engine();
      } finally {
        this.engineReady = false;
        this.modelStatus = null;
      }
    },

    async loadBundles() {
      if (!_api) return;
      const res = await _api.get_bundles();
      if (res.status === 'success' && res.bundles) {
        this.bundles = res.bundles;
      }
    },

    applyBundle(bundle: ModelBundle) {
      this.selectedBundleId = bundle.id;
      this.params.transformer_model = bundle.transformer_model;
      this.params.transformer_type = bundle.transformer_type;
      this.params.vae_model = bundle.vae_model;
      this.params.clip_tokenizer = bundle.clip_tokenizer;
      this.params.clip_encoder = bundle.clip_encoder;
      this.params.t5_tokenizer = bundle.t5_tokenizer;
      this.params.t5_encoder = bundle.t5_encoder;
      this.params.qwen3_tokenizer = bundle.qwen3_tokenizer;
      this.params.qwen3_encoder = bundle.qwen3_encoder;
      this.params.steps = bundle.steps;
      this.params.cfg_scale = bundle.cfg_scale;
      this.params.sampler = bundle.sampler;
      this.params.scheduler = bundle.scheduler;
    },

    async submitJob() {
      if (!_api || !this.engineReady || this.isGenerating) return;
      if (!this.params.prompt.trim()) {
        this.error = 'Prompt is required';
        return;
      }
      this.error = null;
      this.isGenerating = true;
      this.progress = null;

      console.log('submit job: ', this.params);

      const jobParams: Record<string, any> = { ...this.params };
      if (this.selectedBundleId) {
        jobParams.bundle_id = this.selectedBundleId;
      }
      if (this.stylePromptTemplate) {
        jobParams.prompt = this.stylePromptTemplate.replace('{prompt}', this.params.prompt);
      }
      if (this.styleNegativePrompt) {
        jobParams.negative_prompt = this.styleNegativePrompt;
      }

      console.log('submit jobParams: ', jobParams);

      // Clean out undefined keys so Python doesn't get "undefined" strings
      for (const key of Object.keys(jobParams)) {
        if (jobParams[key] === undefined || jobParams[key] === '') {
          delete jobParams[key];
        }
      }

      const res = await _api.submit_job(jobParams);
      if (res.status === 'error') {
        this.error = res.message ?? 'Failed to submit job';
        this.isGenerating = false;
        return;
      }
      this.currentJobId = res.job_id ?? null;
    },

    async cancelJob() {
      if (!_api || !this.currentJobId) return;
      if (this.batchActive) {
        return this.cancelBatch();
      }
      await _api.cancel_job({ job_id: this.currentJobId });
      this.isGenerating = false;
      this.currentJobId = null;
      this.progress = null;
    },

    // ── Batch ──

    async submitBatch(prompts: string[]) {
      if (!_api || !this.engineReady || this.isGenerating) return;
      if (!prompts.length) return;

      this.error = null;
      this.batchActive = true;
      this.batchTotal = prompts.length;
      this.batchCompleted = 0;
      this.batchJobIds = [];
      this.isGenerating = true;

      // Freeze seed: use current seed or generate a random one
      const frozenSeed = this.params.seed >= 0 ? this.params.seed : Math.floor(Math.random() * 2147483647);

      for (const prompt of prompts) {
        const jobParams: Record<string, any> = { ...this.params, prompt, seed: frozenSeed };
        if (this.selectedBundleId) {
          jobParams.bundle_id = this.selectedBundleId;
        }
        if (this.stylePromptTemplate) {
          jobParams.prompt = this.stylePromptTemplate.replace('{prompt}', prompt);
        }
        if (this.styleNegativePrompt) {
          jobParams.negative_prompt = this.styleNegativePrompt;
        }

        for (const key of Object.keys(jobParams)) {
          if (jobParams[key] === undefined || jobParams[key] === '') {
            delete jobParams[key];
          }
        }

        const res = await _api.submit_job(jobParams);
        if (res.status === 'success' && res.job_id) {
          this.batchJobIds.push(res.job_id);
        }
      }
    },

    async cancelBatch() {
      if (!_api) return;
      for (const jobId of this.batchJobIds) {
        await _api.cancel_job({ job_id: jobId });
      }
      this.batchActive = false;
      this.batchJobIds = [];
      this.batchTotal = 0;
      this.batchCompleted = 0;
      this.isGenerating = false;
      this.currentJobId = null;
      this.progress = null;
      this.previewDataUrl = null;
    },

    applyStyle(styleId: string, promptTemplate: string, negativePrompt: string | null, loras: StyleLoRA[]) {
      this.selectedStyleId = styleId;
      this.stylePromptTemplate = promptTemplate;
      this.styleNegativePrompt = negativePrompt;
      this.params.loras = loras.map((l) => ({
        model_file: l.lora_path,
        strength: l.strength,
      }));
    },

    clearStyle() {
      this.selectedStyleId = null;
      this.stylePromptTemplate = null;
      this.styleNegativePrompt = null;
      this.params.loras = [];
    },

    addLora() {
      this.params.loras.push({ model_file: '', strength: 1.0 });
    },

    removeLora(index: number) {
      this.params.loras.splice(index, 1);
    },

    // ── UI toggles ──

    toggleChatbot() {
      this.chatbotOpen = !this.chatbotOpen;
    },

    setAspectRatio(label: string, width: number, height: number) {
      this.activeAspectRatio = label;
      this.params.width = width;
      this.params.height = height;
    },

    toggleAdvanced() {
      this.advancedOpen = !this.advancedOpen;
    },

    toggleDevMode() {
      this.devMode = !this.devMode;
    },

    // ── Polling ──

    startPolling() {
      if (_pollTimer) return;
      _pollTimer = setInterval(() => this.pollEvents(), 200);
    },

    stopPolling() {
      if (_pollTimer) {
        clearInterval(_pollTimer);
        _pollTimer = null;
      }
    },

    async pollEvents() {
      if (!_api) return;
      try {
        const res = await _api.poll_events();
        if (res.status === 'success' && res.events?.length) {
          this.processEvents(res.events);
        }
      } catch {
        // Silently ignore poll errors
      }
    },

    async processEvents(newEvents: InferenceEvent[]) {
      for (const event of newEvents) {
        this.events.push(event);

        switch (event.type) {
          case 'model_loading':
            console.log('model_loading:', event.data);
            this.modelStatus = event.data.message ?? 'Loading model...';
            break;

          case 'model_loaded':
            console.log('model_loaded:', event.data);
            this.modelStatus = 'Model loaded';
            break;

          case 'model_unloaded':
            this.modelStatus = null;
            break;

          case 'server_connected':
            console.log('server_connected:', event.data);
            this.engineReady = true;
            break;

          case 'server_disconnected':
            console.log('server_disconnected:', event.data);
            this.engineReady = false;
            this.error = event.data.message ?? 'Disconnected from remote server';
            break;

          case 'job_queued':
            break;

          case 'job_started':
            console.log('job_started:', event.data);
            this.currentJobId = event.data.job_id ?? this.currentJobId;
            this.isGenerating = true;
            this.progress = {
              step: 0,
              totalSteps: this.params.steps,
              aspectRatio: '1/1',
              width: event.data.width,
              height: event.data.height,
            };
            break;

          case 'job_progress':
            console.log('job_progress:', event.data);
            this.progress = {
              step: event.data.step ?? 0,
              totalSteps: event.data.total_steps ?? this.params.steps,
              aspectRatio: `${event.data.width}/${event.data.height}`,
              width: event.data.width,
              height: event.data.height,
            };
            if (event.data.preview_path) {
              this.loadImageDataUrl(event.data.preview_path);
            }
            break;

          case 'job_completed': {
            this.progress = null;
            this.previewDataUrl = null;
            const img: GeneratedImage = {
              jobId: event.data.job_id,
              imagePath: event.data.image_path ?? '',
              seed: event.data.seed ?? -1,
              timestamp: event.data.timestamp ?? Date.now() / 1000,
              width: event.data.width,
              height: event.data.height,
            };
            if (img.imagePath && _api) {
              const imgRes = await _api.get_image_base64({
                image_path: img.imagePath,
              });
              if (imgRes.status === 'success' && imgRes.data_url) {
                img.dataUrl = imgRes.data_url;
              }
            }
            this.generatedImages.unshift(img);

            if (this.batchActive) {
              this.batchCompleted++;
              if (this.batchCompleted >= this.batchTotal) {
                this.batchActive = false;
                this.isGenerating = false;
                this.currentJobId = null;
              }
            } else {
              this.isGenerating = false;
              this.currentJobId = null;
            }
            break;
          }

          case 'job_failed':
            this.progress = null;
            this.previewDataUrl = null;
            this.error = event.data.error ?? 'Generation failed';

            if (this.batchActive) {
              this.batchCompleted++;
              if (this.batchCompleted >= this.batchTotal) {
                this.batchActive = false;
                this.isGenerating = false;
                this.currentJobId = null;
              }
            } else {
              this.isGenerating = false;
              this.currentJobId = null;
            }
            break;

          case 'job_cancelled':
            this.progress = null;
            this.previewDataUrl = null;

            if (this.batchActive) {
              this.batchCompleted++;
              if (this.batchCompleted >= this.batchTotal) {
                this.batchActive = false;
                this.isGenerating = false;
                this.currentJobId = null;
              }
            } else {
              this.isGenerating = false;
              this.currentJobId = null;
            }
            break;

          default:
            console.log('event:', event.type, event.data);
        }
      }
    },

    async injectDebugEvent(eventType: string) {
      if (eventType === 'default') {
        this.isGenerating = false;
        this.currentJobId = null;
        this.progress = null;
        this.previewDataUrl = null;
        this.error = null;
        this.modelStatus = null;
        return;
      }

      // Lazily fetch debug image paths from the backend
      if (!_debugImages && _api) {
        const res = await _api.get_debug_images();
        if (res.status === 'success') {
          _debugImages = {
            output: res.output ?? null,
            previews: res.previews ?? {},
          };
        }
      }
      const previews = _debugImages?.previews ?? {};
      const output = _debugImages?.output ?? undefined;

      console.log('previews:', previews);

      // Helper: find the best preview for a given step
      function previewForStep(step: number): string | undefined {
        console.log('previewForStep:', step, previews[String(step)]);
        return previews[String(step)];
      }

      const dummyEvents: Record<string, InferenceEvent | null> = {
        model_loading: { type: 'model_loading', data: { message: 'Loading transformer model...' } },
        model_loaded: { type: 'model_loaded', data: {} },
        model_unloaded: { type: 'model_unloaded', data: {} },
        job_queued: { type: 'job_queued', data: { job_id: 'debug-001' } },
        job_started: { type: 'job_started', data: { job_id: 'debug-001' } },
        job_progress: {
          type: 'job_progress',
          data: { job_id: 'debug-001', step: 12, total_steps: 28, width: this.params.width, height: this.params.height, preview_path: previewForStep(10) },
        },
        job_progress_0: {
          type: 'job_progress',
          data: { job_id: 'debug-001', step: 0, total_steps: 28, width: this.params.width, height: this.params.height, preview_path: previewForStep(0) },
        },
        job_progress_5: {
          type: 'job_progress',
          data: { job_id: 'debug-001', step: 5, total_steps: 28, width: this.params.width, height: this.params.height, preview_path: previewForStep(5) },
        },
        job_progress_10: {
          type: 'job_progress',
          data: { job_id: 'debug-001', step: 10, total_steps: 28, width: this.params.width, height: this.params.height, preview_path: previewForStep(10) },
        },
        job_progress_15: {
          type: 'job_progress',
          data: { job_id: 'debug-001', step: 15, total_steps: 28, width: this.params.width, height: this.params.height, preview_path: previewForStep(15) },
        },
        job_progress_20: {
          type: 'job_progress',
          data: { job_id: 'debug-001', step: 20, total_steps: 28, width: this.params.width, height: this.params.height, preview_path: previewForStep(20) },
        },
        job_progress_25: {
          type: 'job_progress',
          data: { job_id: 'debug-001', step: 25, total_steps: 28, width: this.params.width, height: this.params.height, preview_path: previewForStep(25) },
        },
        job_progress_28: {
          type: 'job_progress',
          data: { job_id: 'debug-001', step: 28, total_steps: 28, width: this.params.width, height: this.params.height, preview_path: previewForStep(28) },
        },
        job_completed: {
          type: 'job_completed',
          data: { job_id: 'debug-001', seed: 42, timestamp: Date.now() / 1000, width: this.params.width, height: this.params.height, image_path: output },
        },
        job_failed: { type: 'job_failed', data: { job_id: 'debug-001', error: 'CUDA out of memory. Tried to allocate 2.00 GiB' } },
        job_cancelled: { type: 'job_cancelled', data: { job_id: 'debug-001' } },
      };

      if (['job_queued', 'job_started'].includes(eventType) || eventType.startsWith('job_progress')) {
        this.isGenerating = true;
        this.currentJobId = 'debug-001';
      }

      const event = dummyEvents[eventType];
      if (event) {
        this.processEvents([event]);
      }
    },

    async loadImageDataUrl(imagePath: string) {
      if (!_api) return;
      const res = await _api.get_image_base64({ image_path: imagePath });
      if (res.status === 'success' && res.data_url) {
        this.previewDataUrl = res.data_url;
      }
    },
  },
});
