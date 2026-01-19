<template>
  <div class="" :style="'height: ' + windowsStore.mainHeight + 'px'">
    <div class="image-canvas">
      <div v-if="!imageSrc" class="canvas-empty">
        <p>Generated images will appear here [{{ windowsStore.mainHeight }}]  [{{ windowsStore.navHeight }}]   [{{ windowsStore.windowsHeight }}] </p>
      </div>

      <div v-else class="canvas-content">
        <img :src="imageSrc" alt="Generated image" class="generated-image" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/core';
import { useWindowsStore } from '../../stores/windows';

const windowsStore = useWindowsStore();

// For now, we'll store the image path in the job
// In a future task, this will come from gallery
const imageSrc = ref<string | null>(null);

defineExpose({
  setImage: (path: string) => {
    // Convert filesystem path to asset URL
    imageSrc.value = convertFileSrc(path);
  },
});
</script>

<style scoped>
@reference "tailwindcss";

.image-canvas-wrapper {
  @apply w-full h-full flex rounded-lg items-center justify-center pb-2 pr-2 pl-1;
}

.image-canvas {
  @apply w-full h-full flex items-center justify-center border border-dashed rounded-xl overflow-hidden;
  border-color: var(--color-slate-700);
  background-color: var(--color-slate-900);
}

.canvas-empty {
  @apply text-sm;
  color: var(--color-slate-500);
}

.canvas-content {
  @apply w-full h-full flex items-center justify-center p-4;
}

.generated-image {
  @apply max-w-full max-h-full object-contain rounded-lg;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
}
</style>
