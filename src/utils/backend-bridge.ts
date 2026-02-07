/**
 * Backend Bridge - pywebview backend API
 *
 * This module provides a unified API for communicating with the pywebview backend.
 */

// Promise that resolves when pywebview is ready
let pywebviewReady: Promise<void> | null = null;

// Initialize the ready promise
if (typeof window !== 'undefined') {
  pywebviewReady = new Promise<void>((resolve) => {
    // Check if already ready
    if (window.pywebview?.api) {
      resolve();
      return;
    }

    // Listen for pywebview ready event
    const checkReady = () => {
      if (window.pywebview?.api) {
        resolve();
        window.removeEventListener('pywebviewready', checkReady);
      }
    };

    window.addEventListener('pywebviewready', checkReady);

    // Fallback: poll for pywebview
    const pollInterval = setInterval(() => {
      if (window.pywebview?.api) {
        resolve();
        clearInterval(pollInterval);
        window.removeEventListener('pywebviewready', checkReady);
      }
    }, 50);

    // Timeout after 10 seconds
    setTimeout(() => {
      clearInterval(pollInterval);
      window.removeEventListener('pywebviewready', checkReady);
      if (!window.pywebview?.api) {
        console.error('pywebview failed to initialize within 10 seconds');
      }
      resolve(); // Resolve anyway to prevent hanging
    }, 10000);
  });
}

/**
 * Wait for pywebview to be ready before calling API
 * Also verifies that key methods exist to ensure full initialization
 */
async function ensureReady(): Promise<void> {
  if (pywebviewReady) {
    await pywebviewReady;
  }

  // Additional check: wait for API methods to be available
  // This handles cases where pywebview object exists but methods aren't attached yet
  if (window.pywebview?.api) {
    const maxAttempts = 50; // 5 seconds max (50 * 100ms)
    let attempts = 0;

    while (attempts < maxAttempts) {
      // Check if essential methods exist
      if (typeof window.pywebview.api.health_check === 'function') {
        // API is fully loaded
        return;
      }

      // Wait a bit and try again
      await new Promise(resolve => setTimeout(resolve, 100));
      attempts++;
    }

    console.warn('PyWebView API methods may not be fully loaded after 5 seconds');
  }
}

// Type for pywebview API
interface PywebviewApi {
  // Health
  health_check(): Promise<string>;

  // Database
  init_database(db_path: string): Promise<{ status: string; message: string }>;

  // Queue/Generation
  queue_generation(params: any): Promise<{ status: string; job_id?: string; message?: string }>;
  get_all_jobs(): Promise<any[]>;
  get_job(job_id: string): Promise<any | null>;
  cancel_job(job_id: string): Promise<{ status: string; message?: string }>;
  clear_completed_jobs(): Promise<{ status: string; message?: string }>;
  increment_style_usage(style_id: string): Promise<{ status: string; message?: string }>;

  // Client Mode (aliases)
  client_add_to_queue(params: any): Promise<{ status: string; job_id?: string; message?: string }>;
  client_get_queue_jobs(): Promise<any[]>;
  client_get_queue_job(job_id: string): Promise<any | null>;
  client_cancel_queue_job(job_id: string): Promise<{ status: string; message?: string }>;

  // Gallery
  get_all_images(limit?: number): Promise<any[]>;
  get_gallery_images(limit?: number): Promise<any[]>;
  get_image_by_id(image_id: string): Promise<any | null>;
  delete_image(image_id: string): Promise<{ status: string; message?: string }>;
  delete_gallery_image(image_id: string): Promise<{ status: string; message?: string }>;
  toggle_favorite(image_id: string): Promise<{ status: string; message?: string }>;
  search_gallery_images(query?: string, tags?: string[], folder_id?: string, favorites_only?: boolean, limit?: number): Promise<any[]>;
  add_image_tag(image_id: string, tag: string): Promise<{ status: string; message?: string }>;
  remove_image_tag(image_id: string, tag: string): Promise<{ status: string; message?: string }>;

  // Settings
  get_settings(): Promise<any>;
  save_settings(settings: any): Promise<{ status: string; message?: string }>;

  // API Keys
  get_hf_token(): Promise<{ token: string | null }>;
  save_hf_token(token: string): Promise<{ status: string; message?: string }>;
  get_claude_api_key(): Promise<{ key: string | null }>;
  save_claude_api_key(key: string): Promise<{ status: string; message?: string }>;
  get_fal_key(): Promise<{ key: string | null }>;
  save_fal_key(key: string): Promise<{ status: string; message?: string }>;

  // Cache
  get_cache_stats(): Promise<any>;
  get_cache_config(): Promise<any>;
  save_cache_config(config: any): Promise<{ status: string; message?: string }>;
  clear_cache(): Promise<{ status: string; message?: string }>;

  // Events
  poll_events(max_events?: number): Promise<Array<{ event: string; payload: any }>>;

  // System Stats
  get_system_stats(): Promise<any>;

  // Styles
  get_all_styles(): Promise<any[]>;
  get_style_detail(style_id: string): Promise<any | null>;
  create_style(style: any): Promise<{ status: string; id?: string; message?: string }>;
  update_style(style_id: string, style: any): Promise<{ status: string; message?: string }>;
  delete_style(style_id: string): Promise<{ status: string; message?: string }>;
  add_lora_to_style(style_id: string, lora_id: string, strength?: number, priority?: number): Promise<{ status: string; message?: string }>;
  remove_lora_from_style(style_id: string, lora_id: string): Promise<{ status: string; message?: string }>;
  add_style_example(style_id: string, example_type: string, content: string, generation_params?: string): Promise<{ status: string; id?: string; message?: string }>;
  remove_style_example(example_id: string): Promise<{ status: string; message?: string }>;
  render_style_template(template: string, variables: any): Promise<{ status: string; rendered?: string; message?: string }>;
  upload_style_thumbnail(style_id: string, thumbnail_path: string): Promise<{ status: string; message?: string }>;
  delete_style_thumbnail(style_id: string): Promise<{ status: string; message?: string }>;

  // Folders
  get_folder_tree(): Promise<any[]>;
  create_folder(folder: any): Promise<{ status: string; folder?: any; message?: string }>;
  update_folder(folder_id: string, folder: any): Promise<{ status: string; message?: string }>;
  delete_folder(folder_id: string): Promise<{ status: string; message?: string }>;
  move_folder(folder_id: string, new_parent_id: string | null): Promise<{ status: string; message?: string }>;
  reorder_folders(folder_ids: string[]): Promise<{ status: string; message?: string }>;
  add_images_to_folder(image_ids: string[], folder_id: string): Promise<{ status: string; message?: string }>;
  remove_images_from_folder(image_ids: string[], folder_id: string): Promise<{ status: string; message?: string }>;
  get_folder_images(folder_id: string, limit?: number): Promise<any[]>;
  get_uncategorized_images(limit?: number): Promise<any[]>;

  // Tags
  get_all_tags(): Promise<any[]>;
  update_tag(tag_id: number, tag: any): Promise<{ status: string; message?: string }>;
  delete_tag(tag_id: number): Promise<{ status: string; message?: string }>;
  bulk_add_tag(image_ids: string[], tag: string): Promise<{ status: string; message?: string }>;
  bulk_remove_tag(image_ids: string[], tag: string): Promise<{ status: string; message?: string }>;

  // Auto-Tag
  get_auto_tag_settings(): Promise<any>;
  update_auto_tag_settings(settings: any): Promise<{ status: string; message?: string }>;
  check_vision_model_status(): Promise<any>;
  download_vision_model(): Promise<{ status: string; message?: string }>;
  clear_vision_model_locks(): Promise<{ status: string; message?: string }>;
  auto_tag_images(image_ids: string[]): Promise<{ status: string; message?: string }>;

  // Models
  get_all_models(): Promise<any[]>;
  update_model(model_id: string, model: any): Promise<{ status: string; message?: string }>;
  add_model_tag(model_id: string, tag: string): Promise<{ status: string; message?: string }>;
  remove_model_tag(model_id: string, tag: string): Promise<{ status: string; message?: string }>;
  add_example(entity_type: string, entity_id: string, example_type: string, content: string): Promise<{ status: string; id?: string; message?: string }>;
  remove_example(example_id: string): Promise<{ status: string; message?: string }>;
  scan_directory_for_models(directory: string): Promise<{ status: string; found?: number; message?: string }>;
  scan_and_discover_models(): Promise<{ status: string; found?: number; message?: string }>;
  convert_comfyui_model(source_path: string): Promise<{ status: string; message?: string }>;
  get_compatible_models(bundle_id: string): Promise<any[]>;

  // Bundles
  get_all_bundles(): Promise<any[]>;
  create_bundle(bundle: any): Promise<{ status: string; id?: string; message?: string }>;
  update_bundle(bundle_id: string, bundle: any): Promise<{ status: string; message?: string }>;
  delete_bundle(bundle_id: string): Promise<{ status: string; message?: string }>;
  set_active_bundle(bundle_id: string): Promise<{ status: string; message?: string }>;

  // LoRAs
  get_loras(): Promise<any[]>;
  import_lora(file_path: string): Promise<{ status: string; id?: string; message?: string }>;
  remove_lora(lora_id: string): Promise<{ status: string; message?: string }>;
  get_lora_file_info(file_path: string): Promise<any>;

  // Chatbot
  chat_refine_prompt(prompt: string, context?: any): Promise<{ status: string; refined_prompt?: string; message?: string }>;

  // Batch Generation
  batch_parse_data(data: string, format?: string): Promise<{ status: string; rows?: any[]; message?: string }>;
  batch_render_template(template: string, data: any): Promise<{ status: string; rendered?: string; message?: string }>;
  batch_save_template(template: string): Promise<{ status: string; message?: string }>;
  batch_get_recent_templates(limit?: number): Promise<any[]>;
  batch_generate_combinations(template: string, variables: any): Promise<{ status: string; combinations?: any[]; message?: string }>;

  // Image Analysis
  analyze_image_for_prompt(image_path: string): Promise<{ status: string; prompt?: string; message?: string }>;

  // Auto-Update
  get_version(): Promise<{ version: string; status: string }>;
  check_for_updates(): Promise<any>;
  download_update(): Promise<{ status: string; message?: string }>;
}

declare global {
  interface Window {
    pywebview?: {
      api: PywebviewApi;
    };
  }
}

/**
 * Check if pywebview backend is available
 */
export function isBackendAvailable(): boolean {
  return !!(window.pywebview && window.pywebview.api);
}

export function isPywebview(): boolean {
  return !!window.pywebview;
}

/**
 * Invoke a backend command via pywebview API
 */
export async function invoke<T = any>(command: string, args?: any): Promise<T> {
  // Wait for pywebview to be ready
  await ensureReady();

  if (!window.pywebview?.api) {
    throw new Error("pywebview backend not available");
  }

  const api = window.pywebview.api as any;
  const method = api[command];

  if (!method) {
    throw new Error(`Command '${command}' not found in pywebview API`);
  }

  // Call the method with or without args
  const result = args !== undefined
    ? await method.call(api, args)
    : await method.call(api);

  // Handle error responses
  if (result && typeof result === "object" && result.status === "error") {
    throw new Error(result.message || "Unknown error");
  }

  return result;
}

/**
 * Event listener management
 *
 * For Tauri: Use native event system
 * For pywebview: Use polling-based system
 */

type EventCallback<T = any> = (payload: T) => void;
export type UnlistenFn = () => void;

const eventListeners = new Map<string, Set<EventCallback<any>>>();
let pollInterval: number | null = null;

/**
 * Listen to backend events via polling
 */
export async function listen<T = any>(
  event: string,
  callback: EventCallback<T>
): Promise<UnlistenFn> {
  // Use polling-based event system
  if (!eventListeners.has(event)) {
    eventListeners.set(event, new Set());
  }

  eventListeners.get(event)!.add(callback as EventCallback<any>);

  // Start polling if not already started
  if (pollInterval === null) {
    startEventPolling();
  }

  // Return unlisten function
  return () => {
    const listeners = eventListeners.get(event);
    if (listeners) {
      listeners.delete(callback as EventCallback<any>);
      if (listeners.size === 0) {
        eventListeners.delete(event);
      }
    }

    // Stop polling if no more listeners
    if (eventListeners.size === 0 && pollInterval !== null) {
      stopEventPolling();
    }
  };
}

/**
 * Start polling for events (pywebview only)
 */
function startEventPolling() {
  if (pollInterval !== null) return;

  pollInterval = window.setInterval(async () => {
    try {
      if (!window.pywebview?.api) return;

      const events = await window.pywebview.api.poll_events(50);

      for (const { event, payload } of events) {
        const listeners = eventListeners.get(event);
        if (listeners) {
          for (const callback of listeners) {
            try {
              callback(payload);
            } catch (error) {
              console.error(`Error in event listener for ${event}:`, error);
            }
          }
        }
      }
    } catch (error) {
      console.error("Error polling events:", error);
    }
  }, 100); // Poll every 100ms
}

/**
 * Stop polling for events (pywebview only)
 */
function stopEventPolling() {
  if (pollInterval !== null) {
    window.clearInterval(pollInterval);
    pollInterval = null;
  }
}

/**
 * Emit an event (not supported in pywebview - events flow backend → frontend only)
 */
export async function emit(_event: string, _payload: any): Promise<void> {
  console.warn("emit() is not supported in pywebview backend - events flow backend → frontend only");
}

/**
 * Convert a file path to a URL that can be loaded by the frontend
 *
 * pywebview serves files, so we can use file:// protocol
 */
export function convertFileSrc(filePath: string): string {
  // On macOS/Linux, paths start with /, on Windows they might not
  if (!filePath.startsWith('/') && !filePath.match(/^[A-Za-z]:/)) {
    filePath = '/' + filePath;
  }
  return `file://${filePath}`;
}

/**
 * Cleanup on page unload
 */
if (typeof window !== "undefined") {
  window.addEventListener("beforeunload", () => {
    stopEventPolling();
    eventListeners.clear();
  });
}
