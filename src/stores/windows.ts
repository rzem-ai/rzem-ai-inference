import { defineStore } from 'pinia';

export type ConnectionMode = 'local' | 'server' | 'client';

export const useWindowsStore = defineStore('windows', {
  state: () => ({
    mainHeight: 0,
    navHeight: 0,
    windowsHeight: 0,
    isInitialized: false,
  }),

  actions: {
    // Minimal initialization (no backend calls needed)
    async initialize(): Promise<void> {
      if (this.isInitialized) return
      // No-op - window dimensions managed by App.vue
      this.isInitialized = true
    },

    setMainHeight(height: number) {
      this.mainHeight = height;
    },

    setNavHeight(height: number) {
      this.navHeight = height;
    },

    setWindowsHeight(height: number) {
      this.windowsHeight = height;
    },
  },
});
