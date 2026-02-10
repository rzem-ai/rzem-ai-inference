import { ref } from "vue";
import { defineStore } from "pinia";
import type { PywebviewAPI } from "@/types/pywebview";

export const useAppStore = defineStore("app", () => {
  const counter = ref(0);
  const userName = ref("");
  const greeting = ref("");
  const systemInfo = ref<Record<string, string> | null>(null);
  const isLoading = ref(false);

  async function fetchSystemInfo(api: PywebviewAPI) {
    isLoading.value = true;
    try {
      systemInfo.value = await api.get_system_info();
    } finally {
      isLoading.value = false;
    }
  }

  async function sendGreeting(api: PywebviewAPI) {
    if (!userName.value.trim()) return;
    isLoading.value = true;
    try {
      greeting.value = await api.greet(userName.value);
    } finally {
      isLoading.value = false;
    }
  }

  async function incrementCounter(api: PywebviewAPI) {
    counter.value = await api.increment_counter();
  }

  async function fetchCounter(api: PywebviewAPI) {
    counter.value = await api.get_counter();
  }

  return {
    counter,
    userName,
    greeting,
    systemInfo,
    isLoading,
    fetchSystemInfo,
    sendGreeting,
    incrementCounter,
    fetchCounter,
  };
});
