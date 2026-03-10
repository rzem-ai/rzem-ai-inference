import type { PywebviewAPI } from "@/types/pywebview";

// ── Environment detection ────────────────────────────────────────

/** Check if running inside an Electrobun webview. */
export function isElectrobun(): boolean {
  return typeof window !== "undefined" && "__electrobunWebviewId" in window;
}

/** Check if running inside a pywebview window (legacy). */
export function isPywebview(): boolean {
  return !!window.pywebview;
}

// ── Key conversion utilities ─────────────────────────────────────

function snakeToCamel(str: string): string {
  return str.replace(/_([a-z])/g, (_, c) => c.toUpperCase());
}

function camelToSnake(str: string): string {
  return str.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`);
}

/** Deep-convert object keys from snake_case to camelCase. */
function keysToCamel(obj: unknown): unknown {
  if (Array.isArray(obj)) return obj.map(keysToCamel);
  if (obj && typeof obj === "object" && !(obj instanceof Date)) {
    const result: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(obj as Record<string, unknown>)) {
      result[snakeToCamel(key)] = keysToCamel(value);
    }
    return result;
  }
  return obj;
}

/** Deep-convert object keys from camelCase to snake_case. */
function keysToSnake(obj: unknown): unknown {
  if (Array.isArray(obj)) return obj.map(keysToSnake);
  if (obj && typeof obj === "object" && !(obj instanceof Date)) {
    const result: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(obj as Record<string, unknown>)) {
      result[camelToSnake(key)] = keysToSnake(value);
    }
    return result;
  }
  return obj;
}

// ── Electrobun RPC bridge ────────────────────────────────────────

let _rpc: any = null;
let _rpcReady: Promise<any> | null = null;

async function initRPC(): Promise<any> {
  const { Electroview } = await import("electrobun/view");
  const rpc = Electroview.defineRPC({
    maxRequestTime: 30_000,
    handlers: {
      requests: {},
      messages: {},
    },
  });
  // Activate the WebSocket transport
  new Electroview({ rpc });
  return rpc;
}

function getRPC(): Promise<any> {
  if (!_rpcReady) {
    _rpcReady = initRPC().then((rpc) => {
      _rpc = rpc;
      return rpc;
    });
  }
  return _rpcReady;
}

/**
 * Create a PywebviewAPI-compatible wrapper around Electrobun RPC.
 *
 * The frontend uses snake_case method names and param keys (inherited
 * from the Python backend). The Electrobun RPC uses camelCase. This
 * Proxy transparently converts between the two conventions.
 */
function createElectrobunApi(): PywebviewAPI {
  return new Proxy({} as PywebviewAPI, {
    get(_target, prop: string) {
      if (typeof prop !== "string") return undefined;
      const camelMethod = snakeToCamel(prop);

      return async (args?: unknown) => {
        const rpc = await getRPC();
        // Convert snake_case param keys to camelCase
        const camelArgs =
          args && typeof args === "object" && !Array.isArray(args)
            ? keysToCamel(args)
            : args ?? {};
        try {
          const result = await rpc.request[camelMethod](camelArgs);
          // Convert camelCase response keys back to snake_case
          return keysToSnake(result);
        } catch (err) {
          console.error(`[bridge] RPC ${prop} → ${camelMethod} failed:`, err);
          throw err;
        }
      };
    },
  });
}

// ── Push event listeners ─────────────────────────────────────────

export interface InferenceEventPayload {
  event: string;
  jobId: string;
  data: Record<string, unknown>;
}

export interface ChatEventPayload {
  event: string;
  data: Record<string, unknown>;
}

export interface SidecarStatusPayload {
  ready: boolean;
  pid: number | null;
}

/**
 * Register a listener for inference events pushed from the bun process.
 * Returns an unsubscribe function.
 */
export function onInferenceEvent(
  callback: (payload: InferenceEventPayload) => void,
): () => void {
  if (!isElectrobun()) return () => {};
  getRPC().then((rpc) => rpc.addMessageListener("inferenceEvent", callback));
  return () => {
    getRPC().then((rpc) =>
      rpc.removeMessageListener("inferenceEvent", callback),
    );
  };
}

/**
 * Register a listener for chat events pushed from the bun process.
 * Returns an unsubscribe function.
 */
export function onChatEvent(
  callback: (payload: ChatEventPayload) => void,
): () => void {
  if (!isElectrobun()) return () => {};
  getRPC().then((rpc) => rpc.addMessageListener("chatEvent", callback));
  return () => {
    getRPC().then((rpc) => rpc.removeMessageListener("chatEvent", callback));
  };
}

/**
 * Register a listener for sidecar status updates.
 * Returns an unsubscribe function.
 */
export function onSidecarStatus(
  callback: (payload: SidecarStatusPayload) => void,
): () => void {
  if (!isElectrobun()) return () => {};
  getRPC().then((rpc) => rpc.addMessageListener("sidecarStatus", callback));
  return () => {
    getRPC().then((rpc) =>
      rpc.removeMessageListener("sidecarStatus", callback),
    );
  };
}

// ── Legacy pywebview support ─────────────────────────────────────

const READY_TIMEOUT = 5000;

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

/** Get the typed pywebview API, or null when running in a browser. */
export function getApi(): PywebviewAPI | null {
  return window.pywebview?.api ?? null;
}

// ── Public API ───────────────────────────────────────────────────

let _apiPromise: Promise<PywebviewAPI> | null = null;

/**
 * Get the API instance. Resolves to:
 * - Electrobun RPC wrapper (when in Electrobun webview)
 * - Native pywebview API (when in pywebview window, legacy)
 * - Mock API (when in plain browser for development)
 */
export function getApiAsync(): Promise<PywebviewAPI> {
  if (!_apiPromise) {
    const env = isElectrobun() ? "electrobun" : isPywebview() ? "pywebview" : "browser";
    console.log(`[bridge] Environment: ${env}`);
    if (isElectrobun()) {
      _apiPromise = getRPC().then(() => {
        console.log("[bridge] RPC initialized, creating Electrobun API proxy");
        return createElectrobunApi();
      });
    } else if (isPywebview()) {
      _apiPromise = waitForPywebview()
        .then(() => getApi() ?? mockApi)
        .catch(() => mockApi);
    } else {
      console.log("[bridge] Using mock API");
      _apiPromise = Promise.resolve(mockApi);
    }
  }
  return _apiPromise;
}

// ── Mock API ─────────────────────────────────────────────────────

let _counter = 0;
const _mockSettings: Record<string, string> = {};

/** Mock API for browser-based development without a native shell. */
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
    return `Hello, ${_name}! (mock response)`;
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
        { id: "flux1_dev", label: "FLUX.1 Dev", icon: "gpu", guide: "FLUX.1 Dev", sort_order: 0 },
        { id: "flux1_kontext", label: "FLUX.1 Kontext", icon: "gpu", guide: "FLUX.1 Kontext", sort_order: 4 },
        { id: "fal_cloud", label: "FAL.ai Cloud", icon: "cloud", guide: "FAL.ai Cloud", sort_order: 10 },
      ],
    };
  },

  // ── Bundles ──
  async get_bundles(_args?) {
    return { status: "success", bundles: [] };
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

  // ── Setup ──
  async complete_setup(_args) {
    return { status: "success" };
  },

  // ── Settings (key-value) ──
  async get_setting(_args) {
    const key = typeof _args === "object" ? _args.key : _args;
    return { status: "success", value: _mockSettings[key] ?? null };
  },
  async set_setting(_args) {
    const { key, value } = _args as { key: string; value: string };
    _mockSettings[key] = value;
    return { status: "success" };
  },

  // ── Settings pages ──
  async get_vram_usage() {
    return { status: "success", available: true, allocated: 0, reserved: 0, free: 0, total: 0 };
  },
  async clear_vram_cache() {
    return { status: "success" };
  },
  async get_engine_status() {
    return { status: "success", ready: false, completed_count: 0 };
  },
  async get_cuda_version() {
    return { status: "success", cuda_version: null };
  },
  async reset_engine() {
    return { status: "success" };
  },
  async get_cache_info() {
    return { status: "success", models: [], total_size: 0 };
  },
  async delete_cached_model(_args) {
    return { status: "success" };
  },
  async get_data_paths() {
    return { status: "success", data_dir: "/mock", output_dir: "/mock/output", hf_cache_dir: "/mock/cache" };
  },
  async get_disk_usage() {
    return { status: "success", output_size: 0, hf_cache_size: 0 };
  },
  async browse_output_directory() {
    return { status: "success", changed: false };
  },

  // ── SSL Verification ──
  async get_ssl_verification_disabled() {
    return { status: "success", disabled: false };
  },
  async set_ssl_verification_disabled(_args: { disabled: boolean }) {
    return { status: "success", disabled: _args.disabled };
  },

  // ── Styles ──
  async get_styles(_args?) {
    return { status: "success", styles: [] };
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
    return { status: "success", categories: [] };
  },
  async get_style_bundle_type_counts() {
    return { status: "success", counts: [] };
  },
  async get_styles_dir() {
    return { status: "success" as const, path: "/mock/styles" };
  },
  async list_filter_thumbnails() {
    return { status: "success" as const, thumbnails: [] };
  },
  async get_filter_thumbnail(_args: { image_name: string }) {
    return { status: "error" as const, message: "Mock mode" };
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
    return { status: "success" as const, path: null };
  },
  async save_clipboard_image() {
    return { status: "success" as const, path: "/tmp/mock_clipboard.png" };
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
    return { status: "error", message: "Not available in mock mode" };
  },

  // ── Style from Image ──
  async create_style_from_image(_args) {
    return { status: "error", message: "Not available in mock mode" };
  },

  // ── Style AI Review ──
  async review_styles(_args) {
    return { status: "error", message: "Not available in mock mode" };
  },
  async apply_style_review(_args) {
    return { status: "error", message: "Not available in mock mode" };
  },

  // ── Style Builder ──
  async generate_style_from_filters(_args) {
    return { status: "error", message: "Not available in mock mode" };
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
        { id: "claude-haiku-4-5-20251001", label: "Claude Haiku 4.5 — Fast, low cost" },
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

  // ── Workflows ──
  async get_workflows(_args?: Record<string, unknown>) {
    return { status: "success" as const, workflows: [] };
  },
  async get_workflow(_args: { workflow_id: string }) {
    return { status: "error" as const, workflow: { id: "", name: "", description: "", graph_json: "{}", created_at: 0, updated_at: 0 } };
  },
  async save_workflow(_args: { workflow_id: string; name: string; description?: string; graph_json?: string }) {
    return { status: "success" as const };
  },
  async delete_workflow(_args: { workflow_id: string }) {
    return { status: "success" as const };
  },
  async run_workflow(_args: { graph_json: string }) {
    return { status: "success" as const, run_id: `mock-run-${++_counter}` };
  },
  async cancel_workflow(_args: { run_id: string }) {
    return { status: "success" as const };
  },
  async poll_workflow_events(_args?: Record<string, unknown>) {
    return { status: "success" as const, events: [] };
  },
  async browse_workflow_image(_args?: Record<string, unknown>) {
    return { status: "success" as const, path: "/mock/workflow_image.png" };
  },
};
