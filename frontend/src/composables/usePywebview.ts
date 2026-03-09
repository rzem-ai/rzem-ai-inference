import { ref, onMounted } from "vue";
import type { PywebviewAPI } from "@/types/pywebview";
import { getApiAsync, isElectrobun, isPywebview, mockApi } from "@/bridge";

export function usePywebview() {
  const api = ref<PywebviewAPI>(mockApi);
  const isReady = ref(false);
  const isNative = ref(false);
  const error = ref<string | null>(null);

  onMounted(async () => {
    try {
      api.value = await getApiAsync();
      isNative.value = isElectrobun() || isPywebview();
    } catch {
      error.value = "Running in browser mode (mock API)";
    }
    isReady.value = true;
  });

  return { api, isReady, isNative, error };
}

export function usePywebviewStatic(): { isPywebview: () => boolean } {
  return { isPywebview };
}
