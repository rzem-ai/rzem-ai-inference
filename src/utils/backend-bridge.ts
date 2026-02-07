/**
 * Backend Bridge - Compatibility layer for pywebview backend
 *
 * This module provides a unified API for communicating with the backend,
 * whether it's Tauri or pywebview. It abstracts the differences between
 * the two systems to minimize changes to the rest of the codebase.
 */

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
    __TAURI__?: any;
  }
}

/**
 * Check if we're running in Tauri or pywebview
 */
export function isBackendAvailable(): boolean {
  return !!(window.__TAURI__ || window.pywebview);
}

export function isTauri(): boolean {
  return !!window.__TAURI__;
}

export function isPywebview(): boolean {
  return !!window.pywebview;
}

/**
 * Invoke a backend command
 *
 * This wraps both Tauri's invoke and pywebview's API calls
 */
export async function invoke<T = any>(command: string, args?: any): Promise<T> {
  if (isTauri()) {
    // Use Tauri invoke
    const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
    return tauriInvoke(command, args);
  } else if (isPywebview() && window.pywebview?.api) {
    // Use pywebview API
    const api = window.pywebview.api as any;
    const method = api[command];

    if (!method) {
      throw new Error(`Command '${command}' not found in pywebview API`);
    }

    // Call the method
    const result = await method.call(api, args);

    // Handle error responses
    if (result && typeof result === "object" && result.status === "error") {
      throw new Error(result.message || "Unknown error");
    }

    return result;
  } else {
    throw new Error("No backend available");
  }
}

/**
 * Event listener management
 *
 * For Tauri: Use native event system
 * For pywebview: Use polling-based system
 */

type EventCallback = (payload: any) => void;

const eventListeners = new Map<string, Set<EventCallback>>();
let pollInterval: number | null = null;

/**
 * Listen to backend events
 */
export async function listen(
  event: string,
  callback: EventCallback
): Promise<() => void> {
  if (isTauri()) {
    // Use Tauri's event system
    const { listen: tauriListen } = await import("@tauri-apps/api/event");
    const unlisten = await tauriListen(event, (e: any) => {
      callback(e.payload);
    });
    return unlisten;
  } else if (isPywebview()) {
    // Use polling-based event system
    if (!eventListeners.has(event)) {
      eventListeners.set(event, new Set());
    }

    eventListeners.get(event)!.add(callback);

    // Start polling if not already started
    if (pollInterval === null) {
      startEventPolling();
    }

    // Return unlisten function
    return () => {
      const listeners = eventListeners.get(event);
      if (listeners) {
        listeners.delete(callback);
        if (listeners.size === 0) {
          eventListeners.delete(event);
        }
      }

      // Stop polling if no more listeners
      if (eventListeners.size === 0 && pollInterval !== null) {
        stopEventPolling();
      }
    };
  } else {
    throw new Error("No backend available");
  }
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
 * Emit an event (Tauri only, not supported in pywebview)
 */
export async function emit(event: string, payload: any): Promise<void> {
  if (isTauri()) {
    const { emit: tauriEmit } = await import("@tauri-apps/api/event");
    await tauriEmit(event, payload);
  } else {
    // Not supported in pywebview
    console.warn("emit() is not supported in pywebview backend");
  }
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
