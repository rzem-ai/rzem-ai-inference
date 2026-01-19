import { defineStore } from 'pinia';
import { ref } from 'vue';

export type ConnectionMode = 'local' | 'server' | 'client';

export const useWindowsStore = defineStore('windows', () => {
  // State
  const mainHeight = ref<number>(0);
  const navHeight = ref<number>(0);
  const windowsHeight = ref<number>(0);

  // Actions
  function setMainHeight(height: number) {
    mainHeight.value = height;
  }

  function setNavHeight(height: number) {
    navHeight.value = height;
  }

  function setWindowsHeight(height: number) {
    windowsHeight.value = height;
  }

  return {
    // State
    mainHeight,
    navHeight,
    windowsHeight,
    // Actions
    setMainHeight,
    setNavHeight,
    setWindowsHeight,
  };
});
