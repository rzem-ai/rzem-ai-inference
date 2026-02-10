import type { PywebviewAPI } from "@/types/pywebview";

const READY_TIMEOUT = 5000;

/**
 * Wait for the pywebview bridge to become available.
 * Resolves immediately if already ready, otherwise listens for the
 * `pywebviewready` DOM event with a timeout fallback.
 */
export function waitForPywebview(): Promise<void> {
  if (window.pywebview) return Promise.resolve();

  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error("pywebview bridge did not become ready in time"));
    }, READY_TIMEOUT);

    window.addEventListener(
      "pywebviewready",
      () => {
        clearTimeout(timer);
        resolve();
      },
      { once: true },
    );
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
    return ++mockApi._counter;
  },
  async get_counter() {
    return mockApi._counter;
  },
  _counter: 0,
} as PywebviewAPI & { _counter: number };
