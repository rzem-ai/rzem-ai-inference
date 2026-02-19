import { defineStore } from 'pinia';
import type { PywebviewAPI } from '@/types/pywebview';

let _api: PywebviewAPI | null = null;

export const useAppStore = defineStore('app', {
  state: () => ({
    counter: 0,
    userName: '',
    greeting: '',
    systemInfo: null as Record<string, string> | null,
    isLoading: false,
    error: null as string | null,
  }),

  actions: {
    setApi(apiRef: PywebviewAPI) {
      _api = apiRef;
    },

    async fetchSystemInfo() {
      if (!_api) return;
      this.isLoading = true;
      this.error = null;
      try {
        this.systemInfo = await _api.get_system_info();
      } catch (e: any) {
        this.error = e.message ?? 'Failed to fetch system info';
      } finally {
        this.isLoading = false;
      }
    },

    async sendGreeting() {
      if (!_api || !this.userName.trim()) return;
      this.isLoading = true;
      this.error = null;
      try {
        this.greeting = await _api.greet(this.userName);
      } catch (e: any) {
        this.error = e.message ?? 'Failed to send greeting';
      } finally {
        this.isLoading = false;
      }
    },

    async incrementCounter() {
      if (!_api) return;
      try {
        this.counter = await _api.increment_counter();
      } catch (e: any) {
        this.error = e.message ?? 'Failed to increment counter';
      }
    },

    async fetchCounter() {
      if (!_api) return;
      try {
        this.counter = await _api.get_counter();
      } catch (e: any) {
        this.error = e.message ?? 'Failed to fetch counter';
      }
    },
  },
});
