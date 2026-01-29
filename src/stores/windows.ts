import { defineStore } from 'pinia';

export type ConnectionMode = 'local' | 'server' | 'client';

export const useWindowsStore = defineStore('windows', {
  state: () => ({
    mainHeight: 0,
    navHeight: 0,
    windowsHeight: 0,
  }),

  actions: {
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
