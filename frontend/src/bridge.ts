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

/** Mock API for browser-based development without pywebview. */
export const mockApi: PywebviewAPI = {
  async get_system_info() {
    return {
      platform: "Browser (mock)",
      platform_version: navigator.userAgent,
      python_version: "N/A",
      machine: "N/A",
      processor: "N/A",
    };
  },
  async greet(name: string) {
    return `Hello, ${name}! (mock response — not connected to Python)`;
  },
  async increment_counter() {
    return ++(mockApi as any)._counter;
  },
  async get_counter() {
    return (mockApi as any)._counter;
  },
  async health_check() {
    return { status: "ok" };
  },

  // ── Inference mocks ──
  async start_engine() {
    return { status: "success" as const };
  },
  async stop_engine() {
    return { status: "success" as const };
  },
  async engine_ready() {
    return { status: "success" as const, ready: false };
  },
  async submit_job() {
    return { status: "success" as const, job_id: "mock-job-001" };
  },
  async cancel_job() {
    return { status: "success" as const };
  },
  async poll_events() {
    return { status: "success" as const, events: [] };
  },
  async get_image_base64() {
    return { status: "error" as const, message: "Mock mode — no images available" };
  },
  async get_model_presets() {
    return {
      status: "success" as const,
      presets: [
        {
          id: "flux1_dev",
          label: "FLUX.1 Dev",
          description: "Black Forest Labs FLUX.1-dev (mock)",
          transformer_type: "flux1_dev" as const,
          transformer_model: "black-forest-labs/FLUX.1-dev",
          vae_model: "black-forest-labs/FLUX.1-dev",
          text_encoders: {
            clip_tokenizer: "openai/clip-vit-large-patch14",
            clip_encoder: "openai/clip-vit-large-patch14",
            t5_tokenizer: "google/t5-v1_1-xxl",
            t5_encoder: "google/t5-v1_1-xxl",
          },
        },
      ],
    };
  },
} as PywebviewAPI & { _counter: number };

(mockApi as any)._counter = 0;
