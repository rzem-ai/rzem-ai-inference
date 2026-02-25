import { defineStore } from 'pinia';
import { getApiAsync } from '@/bridge';
import type { ModelBundle, SubmitJobParams, LoraParam, InferenceEvent, GeneratedImage, StyleLoRA, GridConfig } from '@/types/inference';
import { useGalleryStore } from '@/stores/gallery';

// Non-reactive module-level state (no reactivity tracking needed)
let _pollTimer: ReturnType<typeof setInterval> | null = null;
let _debugImages: { output: string | null; previews: Record<string, string> } | null = null;

const EXAMPLE_PROMPTS = [
  'Portrait of an elderly woman with deep wrinkles and kind eyes, silver hair pulled back, wearing a hand-knitted shawl, soft window light, shot on medium format film',
  'Misty mountain valley at dawn, layers of fog rolling between pine-covered ridges, golden sunlight breaking through clouds, aerial perspective',
  'A lone astronaut standing on the edge of a massive crater on an alien world, two moons visible in the violet sky, bioluminescent plants growing from the rocky terrain',
  'An ancient library carved into the trunk of a colossal tree, warm candlelight illuminating endless rows of leather-bound books, dust motes floating in golden light beams',
  'Brutalist concrete apartment building overgrown with lush tropical plants, birds nesting in the balconies, morning fog, shot from below looking up',
  'A weathered fishing boat resting on a pebble beach at golden hour, nets draped over the hull, calm turquoise sea in the background, Mediterranean coast',
  'A red fox curled up sleeping in a snow-covered hollow log, soft winter light filtering through bare branches, frost crystals on its fur',
  'A massive steampunk clockwork mechanism filling an entire cathedral, gears and cogs of brass and copper, shafts of light streaming through stained glass windows',
  'Narrow alleyway in Tokyo at night after rain, neon signs reflecting in puddles, a single figure with an umbrella walking away, atmospheric perspective',
  'An abandoned greenhouse reclaimed by nature, shattered glass roof, wild roses and ivy climbing the iron frame, sunbeams cutting through the overgrowth',
  'Close-up of hands shaping wet clay on a potter\'s wheel, motion blur on the spinning clay, warm studio light, shallow depth of field',
  'A cosmic whale swimming through a nebula, its body made of constellations and stardust, trailing aurora-like light, deep space background',
];

function randomExamplePrompt(): string {
  return EXAMPLE_PROMPTS[Math.floor(Math.random() * EXAMPLE_PROMPTS.length)];
}

export interface AspectRatio {
  label: string;
  width: number;
  height: number;
}

export const ASPECT_RATIOS: AspectRatio[] = [
  { label: '9:21', width: 672, height: 1568 },
  { label: '9:16', width: 768, height: 1376 },
  { label: '3:4', width: 896, height: 1184 },
  { label: '1:1', width: 1024, height: 1024 },
  { label: '4:3', width: 1184, height: 896 },
  { label: '16:9', width: 1376, height: 768 },
  { label: '21:9', width: 1568, height: 672 },
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
    selectedImageIndex: 0,
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
    styleLoras: [] as StyleLoRA[],

    // UI state
    chatbotOpen: false,
    activeAspectRatio: '1:1' as string | null,
    qualityOpen: false,
    advancedOpen: false,
    devMode: false,

    params: {
      prompt: randomExamplePrompt(),
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
      fal_endpoint: undefined,
    } as SubmitJobParams,

    // Batch state
    batchActive: false,
    batchTotal: 0,
    batchCompleted: 0,
    batchJobIds: [] as string[],

    // Grid state
    gridActive: false,
    gridConfig: null as GridConfig | null,
    gridResults: new Map<string, GeneratedImage>(),
    gridJobMap: new Map<string, { x: number; y: number }>(),
    gridTotal: 0,
    gridCompleted: 0,

    // Event log (for debugging)
    events: [] as InferenceEvent[],
  }),

  getters: {
    selectedImage(state): GeneratedImage | null {
      return state.generatedImages[state.selectedImageIndex] ?? null;
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
        fal_cloud: 'FAL.ai Cloud',
      };
      const groups = new Map<string, ModelBundle[]>();
      for (const b of state.bundles) {
        const list = groups.get(b.transformer_type) ?? [];
        list.push(b);
        groups.set(b.transformer_type, list);
      }
      return Array.from(groups.entries()).map(([type, items]) => ({
        label: typeLabels[type] ?? type,
        type: type,
        items,
      }));
    },
  },

  actions: {
    async loadGpuInfo() {
      const api = await getApiAsync();
      try {
        const res = await api.get_gpu_info();
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
      if (this.engineStarting) return;
      this.engineStarting = true;
      this.error = null;
      const api = await getApiAsync();
      try {
        const res = await api.start_engine();
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
      const api = await getApiAsync();
      this.stopPolling();
      try {
        await api.stop_engine();
      } finally {
        this.engineReady = false;
        this.modelStatus = null;
      }
    },

    async loadBundles() {
      const api = await getApiAsync();
      const res = await api.get_bundles();
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
      this.params.fal_endpoint = bundle.fal_endpoint;
      this.params.fal_aspectratio = bundle.fal_aspectratio;
    },

    async submitJob() {
      if (!this.engineReady || this.isGenerating) return;
      if (!this.params.prompt.trim()) {
        this.error = 'Prompt is required';
        return;
      }
      this.error = null;
      this.isGenerating = true;
      this.progress = null;

      const api = await getApiAsync();
      const jobParams: Record<string, any> = { ...this.params };
      if (this.selectedBundleId) {
        jobParams.bundle_id = this.selectedBundleId;
      }
      if (this.selectedStyleId) {
        jobParams.style_id = this.selectedStyleId;
      }
      if (this.stylePromptTemplate) {
        jobParams.raw_prompt = this.params.prompt;
        jobParams.prompt = this.stylePromptTemplate.replace('{prompt}', this.params.prompt);
      }
      if (this.styleNegativePrompt) {
        jobParams.negative_prompt = this.styleNegativePrompt;
      }

      // Clean out undefined keys so Python doesn't get "undefined" strings
      for (const key of Object.keys(jobParams)) {
        if (jobParams[key] === undefined || jobParams[key] === '') {
          delete jobParams[key];
        }
      }

      const res = await api.submit_job(jobParams);
      if (res.status === 'error') {
        this.error = res.message ?? 'Failed to submit job';
        this.isGenerating = false;
        return;
      }
      this.currentJobId = res.job_id ?? null;
    },

    selectImage(index: number) {
      this.selectedImageIndex = index;
      const img = this.generatedImages[index];
      if (img?.params) {
        Object.assign(this.params, img.params);
      }
    },

    async cancelJob() {
      if (!this.currentJobId) return;
      if (this.gridActive) {
        return this.cancelGrid();
      }
      if (this.batchActive) {
        return this.cancelBatch();
      }
      const api = await getApiAsync();
      await api.cancel_job({ job_id: this.currentJobId });
      this.isGenerating = false;
      this.currentJobId = null;
      this.progress = null;
    },

    // ── Batch ──

    async submitBatch(prompts: string[]) {
      if (!this.engineReady || this.isGenerating) return;
      if (!prompts.length) return;

      this.error = null;
      this.batchActive = true;
      this.batchTotal = prompts.length;
      this.batchCompleted = 0;
      this.batchJobIds = [];
      this.isGenerating = true;
      const api = await getApiAsync();

      // Freeze seed: use current seed or generate a random one
      const frozenSeed = this.params.seed >= 0 ? this.params.seed : Math.floor(Math.random() * 2147483647);

      for (const prompt of prompts) {
        const jobParams: Record<string, any> = { ...this.params, prompt, seed: frozenSeed };
        if (this.selectedBundleId) {
          jobParams.bundle_id = this.selectedBundleId;
        }
        if (this.selectedStyleId) {
          jobParams.style_id = this.selectedStyleId;
        }
        if (this.stylePromptTemplate) {
          jobParams.raw_prompt = prompt;
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

        const res = await api.submit_job(jobParams);
        if (res.status === 'success' && res.job_id) {
          this.batchJobIds.push(res.job_id);
        }
      }
    },

    async cancelBatch() {
      const api = await getApiAsync();
      for (const jobId of this.batchJobIds) {
        await api.cancel_job({ job_id: jobId });
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

    // ── Grid ──

    async submitGrid(config: GridConfig) {
      if (!this.engineReady || this.isGenerating) return;

      const xValues = config.xAxis.values;
      const yValues = config.yAxis.values;
      if (!xValues.length || !yValues.length) return;

      this.error = null;
      this.gridActive = true;
      this.gridConfig = config;
      this.gridResults = new Map();
      this.gridJobMap = new Map();
      this.gridTotal = xValues.length * yValues.length;
      this.gridCompleted = 0;
      this.isGenerating = true;

      const api = await getApiAsync();
      const frozenSeed = this.params.seed >= 0 ? this.params.seed : Math.floor(Math.random() * 2147483647);

      for (let y = 0; y < yValues.length; y++) {
        for (let x = 0; x < xValues.length; x++) {
          if (!this.gridActive) return; // cancelled

          const jobParams: Record<string, any> = { ...this.params, seed: frozenSeed };
          jobParams[config.xAxis.param] = xValues[x];
          jobParams[config.yAxis.param] = yValues[y];

          if (this.selectedBundleId) jobParams.bundle_id = this.selectedBundleId;
          if (this.selectedStyleId) jobParams.style_id = this.selectedStyleId;
          if (this.stylePromptTemplate) {
            jobParams.raw_prompt = this.params.prompt;
            jobParams.prompt = this.stylePromptTemplate.replace('{prompt}', this.params.prompt);
          }
          if (this.styleNegativePrompt) jobParams.negative_prompt = this.styleNegativePrompt;

          for (const key of Object.keys(jobParams)) {
            if (jobParams[key] === undefined || jobParams[key] === '') delete jobParams[key];
          }

          const res = await api.submit_job(jobParams);
          if (res.status === 'success' && res.job_id) {
            this.gridJobMap.set(res.job_id, { x, y });
          }
        }
      }
    },

    cancelGrid() {
      this.gridActive = false;
      this.gridConfig = null;
      this.gridJobMap = new Map();
      this.gridTotal = 0;
      this.gridCompleted = 0;
      this.isGenerating = false;
      this.currentJobId = null;
      this.progress = null;
      this.previewDataUrl = null;
    },

    clearGrid() {
      this.gridActive = false;
      this.gridConfig = null;
      this.gridResults = new Map();
      this.gridJobMap = new Map();
      this.gridTotal = 0;
      this.gridCompleted = 0;
    },

    applyStyle(styleId: string, promptTemplate: string, negativePrompt: string | null, loras: StyleLoRA[], bundleId?: string | null) {
      this.selectedStyleId = styleId;
      this.stylePromptTemplate = promptTemplate;
      this.styleNegativePrompt = negativePrompt;
      this.styleLoras = loras;
      this.params.loras = loras.map((l) => ({
        model_file: l.lora_path,
        strength: l.strength,
      }));
      if (bundleId) {
        const bundle = this.bundles.find((b) => b.id === bundleId);
        if (bundle) this.applyBundle(bundle);
      }
    },

    clearStyle() {
      this.selectedStyleId = null;
      this.stylePromptTemplate = null;
      this.styleNegativePrompt = null;
      this.styleLoras = [];
      this.params.loras = [];
    },

    updateStyleLoraStrength(index: number, strength: number) {
      if (index < 0 || index >= this.styleLoras.length) return;
      this.styleLoras[index].strength = strength;
      if (this.params.loras[index]) {
        this.params.loras[index].strength = strength;
      }
    },

    addLora() {
      this.params.loras.push({ model_file: '', strength: 1.0 });
    },

    removeLora(index: number) {
      this.params.loras.splice(index, 1);
    },

    // ── State setters ──

    applyParams(partial: Partial<SubmitJobParams>) {
      Object.assign(this.params, partial);
    },

    setError(error: string | null) {
      this.error = error;
    },

    setActiveAspectRatio(ratio: string | null) {
      this.activeAspectRatio = ratio;
    },

    setSelectedBundleId(id: string | null) {
      this.selectedBundleId = id;
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

    toggleQuality() {
      this.qualityOpen = !this.qualityOpen;
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
      const api = await getApiAsync();
      try {
        const res = await api.poll_events();
        if (res.status === 'success' && res.events?.length) {
          await this.processEvents(res.events);
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
            this.modelStatus = event.data.message ?? 'Loading model...';
            break;

          case 'model_loaded':
            this.modelStatus = 'Model loaded';
            break;

          case 'model_unloaded':
            this.modelStatus = null;
            break;

          case 'server_connected':
            this.engineReady = true;
            break;

          case 'server_disconnected':
            this.engineReady = false;
            this.error = event.data.message ?? 'Disconnected from remote server';
            break;

          case 'job_queued':
            break;

          case 'job_started':
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
              params: { ...this.params, seed: event.data.seed ?? -1 },
            };

            await this.loadCompletedImage(img);
            this.generatedImages.unshift(img);
            this.selectedImageIndex = 0;

            // Track grid cell results
            const gridCell = this.gridJobMap.get(event.data.job_id);
            if (gridCell) {
              this.gridResults.set(`${gridCell.x},${gridCell.y}`, img);
            }

            useGalleryStore().loadImages();

            if (this.gridActive) {
              this.gridCompleted++;
              if (this.gridCompleted >= this.gridTotal) {
                this.gridActive = false;
                this.isGenerating = false;
                this.currentJobId = null;
              }
            } else if (this.batchActive) {
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

            if (this.gridActive) {
              this.gridCompleted++;
              if (this.gridCompleted >= this.gridTotal) {
                this.gridActive = false;
                this.isGenerating = false;
                this.currentJobId = null;
              }
            } else if (this.batchActive) {
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

            if (this.gridActive) {
              this.gridCompleted++;
              if (this.gridCompleted >= this.gridTotal) {
                this.gridActive = false;
                this.isGenerating = false;
                this.currentJobId = null;
              }
            } else if (this.batchActive) {
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
      }
    },

    async loadCompletedImage(img: GeneratedImage) {
      if (!img.imagePath) return;
      const api = await getApiAsync();
      const res = await api.get_image_base64({ image_path: img.imagePath });
      if (res.status === 'success' && res.data_url) {
        img.dataUrl = res.data_url;
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
      if (!_debugImages) {
        const api = await getApiAsync();
        const res = await api.get_debug_images();
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
      const api = await getApiAsync();
      const res = await api.get_image_base64({ image_path: imagePath });
      if (res.status === 'success' && res.data_url) {
        this.previewDataUrl = res.data_url;
      }
    },
  },
});
