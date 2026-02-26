import { defineStore } from 'pinia';
import { getApiAsync } from '@/bridge';
import { useInferenceStore } from '@/stores/inference';
import { useModelsStore } from '@/stores/models';
import { useGalleryStore } from '@/stores/gallery';
import type { ModelBundle, SubmitJobParams, GeneratedImage, InferenceEvent } from '@/types/inference';

export const useEditStore = defineStore('edit', {
  state: () => ({
    // Input image
    inputImagePath: null as string | null,
    inputImageDataUrl: null as string | null,

    // Job state
    currentJobId: null as string | null,
    isGenerating: false,
    progress: null as { step: number; totalSteps: number; width: number; height: number } | null,
    error: null as string | null,

    // Results
    generatedImages: [] as GeneratedImage[],
    selectedImageIndex: 0,
    previewDataUrl: null as string | null,

    // Bundles & params
    bundles: [] as ModelBundle[],
    selectedBundleId: null as string | null,

    params: {
      prompt: '',
      transformer_model: 'black-forest-labs/FLUX.1-Kontext-dev',
      transformer_type: 'flux1_kontext',
      vae_model: 'black-forest-labs/FLUX.1-dev',
      clip_tokenizer: 'openai/clip-vit-large-patch14',
      clip_encoder: 'openai/clip-vit-large-patch14',
      t5_tokenizer: 'google/t5-v1_1-xxl',
      t5_encoder: 'google/t5-v1_1-xxl',
      steps: 28,
      cfg_scale: 2.5,
      width: 1024,
      height: 1024,
      seed: -1,
      sampler: 'euler',
      scheduler: 'simple',
      loras: [],
    } as SubmitJobParams,

    // Track last processed event index from the inference store
    _lastEventIndex: 0,
  }),

  getters: {
    selectedImage(state): GeneratedImage | null {
      return state.generatedImages[state.selectedImageIndex] ?? null;
    },

    engineReady(): boolean {
      return useInferenceStore().engineReady;
    },

    engineStarting(): boolean {
      return useInferenceStore().engineStarting;
    },

    modelStatus(): string | null {
      return useInferenceStore().modelStatus;
    },
  },

  actions: {
    // ── Input image ──

    async setInputImage(path: string) {
      this.inputImagePath = path;
      const api = await getApiAsync();
      const res = await api.get_image_base64({ image_path: path });
      if (res.status === 'success' && res.data_url) {
        this.inputImageDataUrl = res.data_url;
      }
    },

    clearInputImage() {
      this.inputImagePath = null;
      this.inputImageDataUrl = null;
    },

    useOutputAsInput() {
      const img = this.selectedImage;
      if (img?.imagePath) {
        this.setInputImage(img.imagePath);
      }
    },

    // ── Bundles ──

    async loadBundles() {
      const modelsStore = useModelsStore();
      if (!modelsStore.bundleTypes.length) {
        await modelsStore.loadBundleTypes();
      }
      const api = await getApiAsync();
      const res = await api.get_bundles();
      if (res.status === 'success' && res.bundles) {
        this.bundles = res.bundles.filter(
          (b: ModelBundle) => b.transformer_type === 'flux1_kontext',
        );
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
      this.params.t5_encoder_config = bundle.t5_encoder_config ?? undefined;
      this.params.steps = bundle.steps;
      this.params.cfg_scale = bundle.cfg_scale;
      this.params.sampler = bundle.sampler;
      this.params.scheduler = bundle.scheduler;
    },

    // ── Job lifecycle ──

    async submitJob() {
      if (!this.engineReady || this.isGenerating) return;
      if (!this.params.prompt.trim()) {
        this.error = 'Prompt is required';
        return;
      }
      if (!this.inputImagePath) {
        this.error = 'Input image is required';
        return;
      }

      this.error = null;
      this.isGenerating = true;
      this.progress = null;

      const api = await getApiAsync();
      const jobParams: Record<string, any> = {
        ...this.params,
        input_image_path: this.inputImagePath,
      };
      if (this.selectedBundleId) {
        jobParams.bundle_id = this.selectedBundleId;
      }

      // Clean out undefined keys
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

    async cancelJob() {
      if (!this.currentJobId) return;
      const api = await getApiAsync();
      await api.cancel_job({ job_id: this.currentJobId });
      this.isGenerating = false;
      this.currentJobId = null;
      this.progress = null;
      this.previewDataUrl = null;
    },

    selectImage(index: number) {
      this.selectedImageIndex = index;
      const img = this.generatedImages[index];
      if (img?.params) {
        this.params.prompt = img.params.prompt ?? this.params.prompt;
        this.params.steps = img.params.steps ?? this.params.steps;
        this.params.cfg_scale = img.params.cfg_scale ?? this.params.cfg_scale;
        this.params.seed = img.params.seed ?? this.params.seed;
        this.params.sampler = img.params.sampler ?? this.params.sampler;
        this.params.scheduler = img.params.scheduler ?? this.params.scheduler;
      }
    },

    // ── Event processing ──
    // Called by a watcher in Menu.vue that watches inferenceStore.events.length

    processNewEvents() {
      const inferenceStore = useInferenceStore();
      const events = inferenceStore.events;

      // Reset if events were cleared (e.g. engine restart)
      if (this._lastEventIndex > events.length) {
        this._lastEventIndex = 0;
      }

      while (this._lastEventIndex < events.length) {
        const event: InferenceEvent = events[this._lastEventIndex++];
        const jobId = event.data?.job_id;

        // Skip global events (handled by inference store, read via getters)
        if (!jobId) continue;

        // Only process our own job events
        if (jobId !== this.currentJobId) continue;

        switch (event.type) {
          case 'job_started':
            this.isGenerating = true;
            this.progress = {
              step: 0,
              totalSteps: this.params.steps,
              width: event.data.width ?? this.params.width,
              height: event.data.height ?? this.params.height,
            };
            break;

          case 'job_progress':
            this.progress = {
              step: event.data.step ?? 0,
              totalSteps: event.data.total_steps ?? this.params.steps,
              width: event.data.width ?? this.params.width,
              height: event.data.height ?? this.params.height,
            };
            if (event.data.preview_path) {
              this.loadPreview(event.data.preview_path);
            }
            break;

          case 'job_completed':
            this.handleJobCompleted(event);
            break;

          case 'job_failed':
            this.progress = null;
            this.previewDataUrl = null;
            this.error = event.data.error ?? 'Generation failed';
            this.isGenerating = false;
            this.currentJobId = null;
            break;

          case 'job_cancelled':
            this.progress = null;
            this.previewDataUrl = null;
            this.isGenerating = false;
            this.currentJobId = null;
            break;
        }
      }
    },

    async handleJobCompleted(event: InferenceEvent) {
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

      useGalleryStore().loadImages();

      this.isGenerating = false;
      this.currentJobId = null;
    },

    async loadCompletedImage(img: GeneratedImage) {
      if (!img.imagePath) return;
      const api = await getApiAsync();
      const res = await api.get_image_base64({ image_path: img.imagePath });
      if (res.status === 'success' && res.data_url) {
        img.dataUrl = res.data_url;
      }
    },

    async loadPreview(imagePath: string) {
      const api = await getApiAsync();
      const res = await api.get_image_base64({ image_path: imagePath });
      if (res.status === 'success' && res.data_url) {
        this.previewDataUrl = res.data_url;
      }
    },
  },
});
