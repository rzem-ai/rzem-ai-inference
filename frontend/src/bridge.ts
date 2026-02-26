import type { PywebviewAPI } from "@/types/pywebview";

const READY_TIMEOUT = 5000;

/**
 * Wait for the pywebview bridge to become fully available.
 *
 * pywebview attaches `window.pywebview` before its API methods are ready,
 * so we poll for `health_check` (our canary method) to confirm the API
 * surface is fully populated.
 */
export function waitForPywebview(): Promise<void> {
  return new Promise((resolve, reject) => {
    const deadline = Date.now() + READY_TIMEOUT;

    function check() {
      if (
        window.pywebview?.api &&
        typeof window.pywebview.api.health_check === "function"
      ) {
        resolve();
        return;
      }
      if (Date.now() > deadline) {
        reject(new Error("pywebview bridge did not become ready in time"));
        return;
      }
      setTimeout(check, 50);
    }

    check();
  });
}

/** Check whether we are running inside a pywebview window. */
export function isPywebview(): boolean {
  return !!window.pywebview;
}

/** Get the typed pywebview API, or null when running in a browser. */
export function getApi(): PywebviewAPI | null {
  return window.pywebview?.api ?? null;
}

// Singleton promise: first call triggers init, subsequent calls return the same (resolved) promise
let _apiPromise: Promise<PywebviewAPI> | null = null;

/** Lazily-initialized, cached promise that resolves to the PywebviewAPI once ready. */
export function getApiAsync(): Promise<PywebviewAPI> {
  if (!_apiPromise) {
    _apiPromise = waitForPywebview()
      .then(() => getApi() ?? mockApi)
      .catch(() => mockApi);
  }
  return _apiPromise;
}

let _counter = 0;

/** Mock API for browser-based development without pywebview. */
export const mockApi: PywebviewAPI = {
  // ── System ──
  async get_system_info() {
    return {
      platform: "Browser (mock)",
      platform_version: navigator.userAgent,
      python_version: "N/A",
      machine: "N/A",
      processor: "N/A",
    };
  },
  async greet(_name) {
    return `Hello, ${_name}! (mock response — not connected to Python)`;
  },
  async increment_counter() {
    return ++_counter;
  },
  async get_counter() {
    return _counter;
  },
  async health_check() {
    return { status: "ok" };
  },

  // ── Inference engine ──
  async get_gpu_info() {
    return { status: "success", device_type: "cuda", device_name: "NVIDIA RTX 4090 (mock)", total_vram_gb: 24 };
  },
  async start_engine(_args?) {
    return { status: "success" };
  },
  async stop_engine() {
    return { status: "success" };
  },
  async engine_ready() {
    return { status: "success", ready: false };
  },

  // ── Jobs ──
  async submit_job(_args) {
    return { status: "success", job_id: "mock-job-001" };
  },
  async cancel_job(_args) {
    return { status: "success" };
  },
  async poll_events() {
    return { status: "success", events: [] };
  },

  // ── Images ──
  async get_image_base64(_args) {
    return { status: "error", message: "Mock mode — no images available" };
  },
  async get_debug_images() {
    return { status: "success", output: null, previews: {} };
  },

  // ── Bundle Types ──
  async get_bundle_types() {
    return {
      status: "success",
      bundle_types: [
        { id: "flux1_dev", label: "FLUX.1 Dev", icon: "gpu", guide: "## Prompting\n\nFLUX.1 Dev responds best to **descriptive, natural language prompts**.", sort_order: 0 },
        { id: "flux2_dev", label: "FLUX.2 Dev", icon: "gpu", guide: "## Prompting\n\nFLUX.2 Dev uses a **Qwen3 text encoder**.", sort_order: 2 },
        { id: "z_image", label: "Z-Image", icon: "gpu", guide: "## Prompting\n\nZ-Image uses a bundled **Qwen3-4B text encoder**.", sort_order: 3 },
        { id: "flux1_kontext", label: "FLUX.1 Kontext", icon: "gpu", guide: "## Editing\n\nFLUX.1 Kontext [dev] edits images from a **text prompt + input image**.", sort_order: 4 },
        { id: "qwen_image", label: "Qwen-Image", icon: "gpu", guide: "## Prompting\n\nQwen-Image excels at **text rendering in images**.", sort_order: 5 },
        { id: "fal_cloud", label: "FAL.ai Cloud", icon: "cloud", guide: "## Overview\n\nFAL.ai Cloud models run on remote servers.", sort_order: 10 },
      ],
    };
  },

  // ── Bundles ──
  async get_bundles(_args?) {
    return {
      status: "success",
      bundles: [
        {
          id: "flux1_dev_performance",
          label: "FLUX.1 Dev — Fast",
          description: "FLUX.1-dev Q4 quantized (mock)",
          transformer_type: "flux1_dev",
          tier: "performance",
          transformer_model: "city96/FLUX.1-dev-gguf/flux1-dev-Q4_K_S.gguf",
          vae_model: "black-forest-labs/FLUX.1-dev",
          clip_tokenizer: "openai/clip-vit-large-patch14",
          clip_encoder: "openai/clip-vit-large-patch14",
          t5_tokenizer: "google/t5-v1_1-xxl",
          t5_encoder: "google/t5-v1_1-xxl",
          steps: 15,
          cfg_scale: 1.0,
          sampler: "euler",
          scheduler: "normal",
          vram_estimate_gb: 17.7,
          is_default: 1,
          favorite: 1,
          hidden: 0,
        },
        {
          id: "flux1_dev_quality",
          label: "FLUX.1 Dev — Quality",
          description: "FLUX.1-dev BF16 full precision (mock)",
          transformer_type: "flux1_dev",
          tier: "quality",
          transformer_model: "black-forest-labs/FLUX.1-dev",
          vae_model: "black-forest-labs/FLUX.1-dev",
          clip_tokenizer: "openai/clip-vit-large-patch14",
          clip_encoder: "openai/clip-vit-large-patch14",
          t5_tokenizer: "google/t5-v1_1-xxl",
          t5_encoder: "google/t5-v1_1-xxl",
          steps: 28,
          cfg_scale: 1.0,
          sampler: "euler",
          scheduler: "normal",
          vram_estimate_gb: 33.7,
          is_default: 1,
          favorite: 0,
          hidden: 0,
        },
        {
          id: "flux2_dev_quality",
          label: "FLUX.2 Dev",
          description: "FLUX.2-dev BF16 with Qwen3 (mock)",
          transformer_type: "flux2_dev",
          tier: "quality",
          transformer_model: "black-forest-labs/FLUX.2-dev",
          vae_model: "black-forest-labs/FLUX.2-dev",
          qwen3_tokenizer: "Qwen/Qwen3-0.6B",
          qwen3_encoder: "Qwen/Qwen3-0.6B",
          steps: 28,
          cfg_scale: 1.0,
          sampler: "euler",
          scheduler: "normal",
          vram_estimate_gb: 23.2,
          is_default: 1,
          favorite: 0,
          hidden: 0,
        },
        {
          id: "flux1_kontext_performance",
          label: "FLUX.1 Kontext - Fast",
          description: "FLUX.1-Kontext Q4 quantized (mock)",
          transformer_type: "flux1_kontext",
          tier: "performance",
          transformer_model: "bullerwins/FLUX.1-Kontext-dev-GGUF/flux1-kontext-dev-Q4_K_S.gguf",
          vae_model: "black-forest-labs/FLUX.1-Kontext-dev",
          clip_tokenizer: "openai/clip-vit-large-patch14",
          clip_encoder: "openai/clip-vit-large-patch14",
          t5_tokenizer: "google/t5-v1_1-xxl",
          t5_encoder: "google/t5-v1_1-xxl",
          t5_encoder_config: { load_in_4bit: true, bnb_4bit_quant_type: "nf4" },
          steps: 28,
          cfg_scale: 2.5,
          sampler: "euler",
          scheduler: "normal",
          vram_estimate_gb: 11.0,
          is_default: 1,
          favorite: 1,
          hidden: 0,
        },
        {
          id: "flux1_kontext_quality",
          label: "FLUX.1 Kontext - Quality",
          description: "FLUX.1-Kontext BF16 full precision (mock)",
          transformer_type: "flux1_kontext",
          tier: "quality",
          transformer_model: "black-forest-labs/FLUX.1-Kontext-dev",
          vae_model: "black-forest-labs/FLUX.1-Kontext-dev",
          clip_tokenizer: "openai/clip-vit-large-patch14",
          clip_encoder: "openai/clip-vit-large-patch14",
          t5_tokenizer: "google/t5-v1_1-xxl",
          t5_encoder: "google/t5-v1_1-xxl",
          steps: 40,
          cfg_scale: 2.5,
          sampler: "euler",
          scheduler: "normal",
          vram_estimate_gb: 33.7,
          is_default: 1,
          favorite: 0,
          hidden: 0,
        },
        {
          id: "fal_flux_schnell",
          label: "FLUX.1 Schnell",
          description: "FAL.ai cloud — FLUX.1 Schnell (mock)",
          transformer_type: "fal_cloud",
          tier: "performance",
          transformer_model: "fal-ai/flux/schnell",
          vae_model: "cloud",
          steps: 4,
          cfg_scale: 1.0,
          sampler: "euler",
          scheduler: "normal",
          vram_estimate_gb: 0,
          is_default: 1,
          source: "cloud",
          fal_endpoint: "fal-ai/flux/schnell",
          favorite: 0,
          hidden: 0,
        },
        {
          id: "fal_flux_dev",
          label: "FLUX.1 Dev",
          description: "FAL.ai cloud — FLUX.1 Dev (mock)",
          transformer_type: "fal_cloud",
          tier: "balanced",
          transformer_model: "fal-ai/flux/dev",
          vae_model: "cloud",
          steps: 28,
          cfg_scale: 3.5,
          sampler: "euler",
          scheduler: "normal",
          vram_estimate_gb: 0,
          is_default: 1,
          source: "cloud",
          fal_endpoint: "fal-ai/flux/dev",
          favorite: 0,
          hidden: 0,
        },
      ],
    };
  },
  async get_bundle(_args) {
    return { status: "success", bundle: undefined };
  },
  async get_bundles_for_type(_args) {
    return { status: "success", bundles: [] };
  },
  async create_bundle(_args) {
    return { status: "success", bundle: undefined };
  },
  async update_bundle(_args) {
    return { status: "success", bundle: undefined };
  },
  async delete_bundle(_args) {
    return { status: "success" };
  },
  async toggle_bundle_favorite(_args) {
    return { status: "success", bundle: undefined };
  },
  async toggle_bundle_hidden(_args) {
    return { status: "success", bundle: undefined };
  },
  async reset_default_bundles() {
    return { status: "success", bundles: [] };
  },

  // ── Gallery ──
  async get_gallery_images(_args?) {
    return { status: "success", images: [], total: 0 };
  },
  async get_image(_args) {
    return { status: "error", message: "Mock mode" };
  },
  async toggle_favorite(_args) {
    return { status: "error", message: "Mock mode" };
  },
  async save_image_as(_args) {
    return { status: "success", saved: false };
  },
  async batch_save_images(_args) {
    return { status: "success", saved_count: 0 };
  },
  async delete_image(_args) {
    return { status: "success" };
  },

  // ── Folders ──
  async get_folders() {
    return { status: "success", folders: [] };
  },
  async create_folder(_args) {
    return { status: "error", message: "Mock mode" };
  },
  async update_folder(_args) {
    return { status: "error", message: "Mock mode" };
  },
  async delete_folder(_args) {
    return { status: "success" };
  },
  async add_image_to_folder(_args) {
    return { status: "success" };
  },
  async remove_image_from_folder(_args) {
    return { status: "success" };
  },

  // ── Tags ──
  async get_tags() {
    return { status: "success", tags: [] };
  },
  async create_tag(_args) {
    return { status: "error", message: "Mock mode" };
  },
  async update_tag(_args) {
    return { status: "error", message: "Mock mode" };
  },
  async delete_tag(_args) {
    return { status: "success" };
  },
  async add_tag_to_image(_args) {
    return { status: "success" };
  },
  async remove_tag_from_image(_args) {
    return { status: "success" };
  },
  async get_image_tags(_args) {
    return { status: "success", tags: [] };
  },

  // ── Settings (key-value) ──
  async get_setting(_args) {
    return { status: "success", value: null };
  },
  async set_setting(_args) {
    return { status: "success" };
  },

  // ── Settings pages ──
  async get_vram_usage() {
    return {
      status: "success",
      available: true,
      allocated: 4.2 * 1024 ** 3,
      reserved: 6.8 * 1024 ** 3,
      free: 17.2 * 1024 ** 3,
      total: 24 * 1024 ** 3,
    };
  },
  async clear_vram_cache() {
    return { status: "success" };
  },
  async get_engine_status() {
    return { status: "success", ready: true, uptime_seconds: 3742, completed_count: 14 };
  },
  async get_cuda_version() {
    return { status: "success", cuda_version: "12.4" };
  },
  async reset_engine() {
    return { status: "success" };
  },
  async get_cache_info() {
    return {
      status: "success",
      models: [
        {
          repo_id: "black-forest-labs/FLUX.1-dev",
          repo_type: "model",
          size_on_disk: 23_800_000_000,
          nb_files: 12,
          revisions: [{ commit_hash: "abc123", size_on_disk: 23_800_000_000, last_modified: Date.now() / 1000 }],
          last_accessed: Date.now() / 1000,
          last_modified: Date.now() / 1000,
        },
        {
          repo_id: "openai/clip-vit-large-patch14",
          repo_type: "model",
          size_on_disk: 1_700_000_000,
          nb_files: 6,
          revisions: [{ commit_hash: "def456", size_on_disk: 1_700_000_000, last_modified: Date.now() / 1000 }],
          last_accessed: Date.now() / 1000,
          last_modified: Date.now() / 1000,
        },
        {
          repo_id: "google/t5-v1_1-xxl",
          repo_type: "model",
          size_on_disk: 11_500_000_000,
          nb_files: 8,
          revisions: [{ commit_hash: "ghi789", size_on_disk: 11_500_000_000, last_modified: Date.now() / 1000 }],
          last_accessed: Date.now() / 1000,
          last_modified: Date.now() / 1000,
        },
      ],
      total_size: 37_000_000_000,
    };
  },
  async delete_cached_model(_args) {
    return { status: "success" };
  },
  async get_data_paths() {
    return {
      status: "success",
      data_dir: "/home/user/.rzem-ai",
      output_dir: "/home/user/.rzem-ai/output",
      hf_cache_dir: "/home/user/.cache/huggingface/hub",
    };
  },
  async get_disk_usage() {
    return {
      status: "success",
      output_size: 2_400_000_000,
      hf_cache_size: 37_000_000_000,
    };
  },
  async browse_output_directory() {
    return { status: "success", changed: false };
  },

  // ── Styles ──
  async get_styles(_args?) {
    return {
      status: "success",
      styles: [
        {
          id: "mock-style-1",
          name: "Cinematic Film",
          description: "Hollywood cinematic look with dramatic lighting",
          prompt_template: "cinematic film still, dramatic lighting, {prompt}, 35mm photograph, film grain",
          negative_prompt: "cartoon, anime, drawing, painting",
          default_strength: 1.0,
          strength_min: 0.5,
          strength_max: 1.5,
          category: "Photography",
          thumbnail_path: null,
          bundle_id: null,
          is_favorite: 1,
          usage_count: 12,
          created_at: 1700000000,
          updated_at: 1700000000,
        },
        {
          id: "mock-style-2",
          name: "Anime Girl",
          description: "High quality anime illustration style",
          prompt_template: "anime illustration, {prompt}, detailed, vibrant colors, studio ghibli",
          negative_prompt: "photo, realistic, 3d render",
          default_strength: 1.0,
          strength_min: 0.5,
          strength_max: 1.5,
          category: "Illustration",
          thumbnail_path: null,
          bundle_id: null,
          is_favorite: 0,
          usage_count: 8,
          created_at: 1700100000,
          updated_at: 1700100000,
        },
        {
          id: "mock-style-3",
          name: "Watercolor Dream",
          description: "Soft watercolor painting effect",
          prompt_template: "watercolor painting, soft edges, {prompt}, paper texture, artistic",
          negative_prompt: "photo, sharp, digital",
          default_strength: 1.0,
          strength_min: 0.5,
          strength_max: 1.5,
          category: "Painting",
          thumbnail_path: null,
          bundle_id: null,
          is_favorite: 0,
          usage_count: 5,
          created_at: 1700200000,
          updated_at: 1700200000,
        },
      ],
    };
  },
  async get_style(_args) {
    return { status: "error", message: "Mock mode" };
  },
  async create_style(_args) {
    return { status: "success", style: undefined };
  },
  async update_style(_args) {
    return { status: "success", style: undefined };
  },
  async delete_style(_args) {
    return { status: "success" };
  },
  async toggle_style_favorite(_args) {
    return { status: "error", message: "Mock mode" };
  },
  async get_style_categories() {
    return { status: "success", categories: ["Photography", "Illustration", "Painting"] };
  },
  async get_styles_dir() {
    return { status: "success" as const, path: "/mock/styles" };
  },
  async list_filter_thumbnails() {
    return { status: "success" as const, thumbnails: [] };
  },
  async get_filter_thumbnail(_args: { image_name: string }) {
    return { status: "error" as const, message: "Mock mode — no thumbnails" };
  },
  async get_style_examples(_args) {
    return { status: "success", examples: [] };
  },
  async create_style_example(_args) {
    return { status: "success", example: undefined };
  },
  async delete_style_example(_args) {
    return { status: "success" };
  },
  async get_style_loras(_args) {
    return { status: "success", loras: [] };
  },
  async set_style_loras(_args) {
    return { status: "success", loras: [] };
  },
  async get_style_tags(_args) {
    return { status: "success", tags: [] };
  },
  async set_style_tags(_args) {
    return { status: "success", tags: [] };
  },
  async get_loras() {
    return { status: "success", loras: [] };
  },
  async create_lora(_args) {
    return { status: "success", lora: undefined };
  },
  async browse_lora_files() {
    return { status: "success", loras: [] };
  },
  async browse_image_file() {
    return { status: "success", path: null, thumbnail_path: null };
  },
  async browse_input_image() {
    return { status: 'success' as const, path: null };
  },
  async save_clipboard_image() {
    return { status: 'success' as const, path: '/tmp/mock_clipboard.png' };
  },
  async import_style_image() {
    return { status: "error", message: "Not available in mock mode" };
  },
  async browse_metadata_files() {
    return { status: "success", paths: [] };
  },
  async browse_and_import_metadata() {
    return { status: "success", styles: [], errors: [] };
  },
  async import_civitai_metadata(_args: { file_path?: string; json_content?: string }) {
    return {
      status: "success",
      style: {
        id: "mock-imported-style",
        name: "Imported Style (mock)",
        description: "Imported from CivitAI metadata",
        prompt_template: "trigger_word {prompt}",
        negative_prompt: null,
        default_strength: 1.0,
        strength_min: 0.5,
        strength_max: 1.5,
        category: null,
        thumbnail_path: null,
        is_favorite: 0,
        usage_count: 0,
        created_at: Date.now() / 1000,
        updated_at: Date.now() / 1000,
      },
    };
  },

  // ── Style Builder ──
  async generate_style_from_filters(_args) {
    await new Promise((r) => setTimeout(r, 1500));
    return {
      status: "success" as const,
      style_data: {
        name: "Mock Generated Style",
        description: "A rich visual style combining selected filters into a cohesive aesthetic.",
        prompt_template: "artistic composition, {prompt}, detailed, high quality",
        negative_prompt: "low quality, blurry, deformed",
        category: "Mixed Media",
      },
    };
  },

  // ── Batch ──
  async batch_parse_data(_args) {
    return { status: "success", columns: [], rows: [] };
  },
  async batch_render_template(_args) {
    return { status: "success", rendered: [], errors: [] };
  },

  // ── Discovery ──
  async get_discovered_servers() {
    return { status: "success", servers: [] };
  },
  async connect_to_server(_args: { host: string; port: number }) {
    return { status: "success", mode: "remote" };
  },
  async disconnect_from_server() {
    return { status: "success", mode: "local" };
  },
  async get_connection_mode() {
    return { status: "success", mode: "local", connected_server: null };
  },

  // ── Chat ──
  async chat_is_configured() {
    return { status: "success", configured: false };
  },
  async chat_set_api_key(_args) {
    return { status: "success" };
  },
  async chat_get_provider_info() {
    return {
      status: "success",
      provider: "claude",
      models: [
        { id: "claude-haiku-4-5-20251001", label: "Claude Haiku 4.6 — Fast, low cost" },
        { id: "claude-sonnet-4-6", label: "Claude Sonnet 4.6 — Balanced (default)" },
        { id: "claude-opus-4-6", label: "Claude Opus 4.6 — Most capable" },
      ],
    };
  },
  async chat_create_conversation(_args?) {
    return { status: "success", conversation: { id: "mock-conv-1", title: "New Chat", created_at: Date.now() / 1000, updated_at: Date.now() / 1000 } };
  },
  async chat_get_conversations() {
    return { status: "success", conversations: [] };
  },
  async chat_get_messages(_args) {
    return { status: "success", messages: [] };
  },
  async chat_delete_conversation(_args) {
    return { status: "success" };
  },
  async chat_send_message(_args) {
    return { status: "success" };
  },
  async poll_chat_events() {
    return { status: "success", events: [] };
  },
};
