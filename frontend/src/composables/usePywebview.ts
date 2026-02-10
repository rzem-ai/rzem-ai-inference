import { ref, onMounted } from "vue";
import type { PywebviewAPI } from "@/types/pywebview";
import { waitForPywebview, getApi, isPywebview, mockApi } from "@/bridge";

export function usePywebview() {
  const api = ref<PywebviewAPI>(mockApi);
  const isReady = ref(false);
  const isNative = ref(false);
  const error = ref<string | null>(null);

  onMounted(async () => {
    try {
      await waitForPywebview();
      const nativeApi = getApi();
      if (nativeApi) {
        api.value = nativeApi;
        isNative.value = true;
      }
    } catch {
      // Not inside pywebview — fall back to mock
      error.value = "Running in browser mode (mock API)";
    }
    isReady.value = true;
  });

  return { api, isReady, isNative, error };
}

export function usePywebviewStatic(): { isPywebview: () => boolean } {
  return { isPywebview };
}
