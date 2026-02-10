import { defineStore } from 'pinia';
import type { PywebviewAPI } from '@/types/pywebview';

export const useAppStore = defineStore('app', {
  state: () => ({
    counter: 0,
    userName: '',
    greeting: '',
    systemInfo: null as Record<string, string> | null,
    isLoading: false,
  }),

  actions: {
    async fetchSystemInfo(api: PywebviewAPI) {
      this.isLoading = true;
      try {
        this.systemInfo = await api.get_system_info();
      } finally {
        this.isLoading = false;
      }
    },

    async sendGreeting(api: PywebviewAPI) {
      if (!this.userName.trim()) return;
      this.isLoading = true;
      try {
        this.greeting = await api.greet(this.userName);
      } finally {
        this.isLoading = false;
      }
    },

    async incrementCounter(api: PywebviewAPI) {
      this.counter = await api.increment_counter();
    },

    async fetchCounter(api: PywebviewAPI) {
      this.counter = await api.get_counter();
    },
  },
});
