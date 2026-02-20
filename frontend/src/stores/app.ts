import { defineStore } from 'pinia';
import { getApiAsync } from '@/bridge';

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
    async fetchSystemInfo() {
      this.isLoading = true;
      this.error = null;
      try {
        const api = await getApiAsync();
        this.systemInfo = await api.get_system_info();
      } catch (e: any) {
        this.error = e.message ?? 'Failed to fetch system info';
      } finally {
        this.isLoading = false;
      }
    },

    async sendGreeting() {
      if (!this.userName.trim()) return;
      this.isLoading = true;
      this.error = null;
      try {
        const api = await getApiAsync();
        this.greeting = await api.greet(this.userName);
      } catch (e: any) {
        this.error = e.message ?? 'Failed to send greeting';
      } finally {
        this.isLoading = false;
      }
    },

    async incrementCounter() {
      try {
        const api = await getApiAsync();
        this.counter = await api.increment_counter();
      } catch (e: any) {
        this.error = e.message ?? 'Failed to increment counter';
      }
    },

    async fetchCounter() {
      try {
        const api = await getApiAsync();
        this.counter = await api.get_counter();
      } catch (e: any) {
        this.error = e.message ?? 'Failed to fetch counter';
      }
    },
  },
});
