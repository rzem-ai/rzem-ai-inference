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
 */
async function ensureReady(): Promise<void> {
  if (pywebviewReady) {
    await pywebviewReady;
  }
}

// Type for pywebview API
interface PywebviewApi {
  health_check(): Promise<string>;
  init_database(db_path: string): Promise<{ status: string; message: string }>;
  queue_generation(params: any): Promise<{ status: string; job_id?: string; message?: string }>;
  get_all_jobs(): Promise<any[]>;
  get_job(job_id: string): Promise<any | null>;
  cancel_job(job_id: string): Promise<{ status: string; message?: string }>;
  clear_completed_jobs(): Promise<{ status: string; message?: string }>;
  get_all_images(limit?: number): Promise<any[]>;
  get_image_by_id(image_id: string): Promise<any | null>;
  delete_image(image_id: string): Promise<{ status: string; message?: string }>;
  toggle_favorite(image_id: string): Promise<{ status: string; message?: string }>;
  get_settings(): Promise<any>;
  save_settings(settings: any): Promise<{ status: string; message?: string }>;
  poll_events(max_events?: number): Promise<Array<{ event: string; payload: any }>>;
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
