<template>
  <div class="flex flex-col h-full p-4 gap-4">
    <!-- Progress area (during generation) -->
    <div v-if="store.isGenerating" class="flex flex-col gap-3 p-4 bg-surface-100 rounded-lg">
      <div v-if="store.modelStatus" class="flex items-center gap-2 text-sm text-surface-600">
        <ProgressSpinner style="width: 1.2rem; height: 1.2rem" />
        <span>{{ store.modelStatus }}</span>
      </div>

      <div v-if="store.progress" class="flex flex-col gap-1">
        <div class="flex justify-between text-xs text-surface-500">
          <span>Step {{ store.progress.step }} / {{ store.progress.totalSteps }}</span>
          <span>{{ Math.round((store.progress.step / store.progress.totalSteps) * 100) }}%</span>
        </div>
        <ProgressBar :value="(store.progress.step / store.progress.totalSteps) * 100" :show-value="false" style="height: 6px" />
      </div>

      <div v-if="!store.progress && !store.modelStatus" class="flex items-center gap-2 text-sm text-surface-600">
        <ProgressSpinner style="width: 1.2rem; height: 1.2rem" />
        <span>Preparing...</span>
      </div>

      <!-- Live preview -->
      <div v-if="store.previewDataUrl" class="flex items-center justify-center bg-surface-50 rounded overflow-hidden">
        <img :src="store.previewDataUrl" alt="Generation preview" class="max-w-full max-h-[400px] object-contain" />
      </div>

      <Button label="Cancel" icon="pi pi-stop" size="small" severity="secondary" @click="store.cancelJob()" />
    </div>

    <!-- Error display -->
    <div v-if="store.error && !store.isGenerating" class="p-3 bg-red-50 text-red-600 rounded-lg text-sm">
      {{ store.error }}
    </div>

    <!-- Result area (latest generated image) -->
    <div v-if="displayedImage?.dataUrl" class="flex flex-col gap-3 flex-1 min-h-0">
      <div class="flex-1 min-h-0 flex items-center justify-center bg-surface-100 rounded-lg overflow-hidden">
        <img :src="displayedImage.dataUrl" alt="Generated image" class="max-w-full max-h-full object-contain" />
      </div>

      <!-- Metadata -->
      <div class="flex items-center gap-4 text-xs text-surface-500">
        <span>Seed: {{ displayedImage.seed }}</span>
        <span>{{ formatTimestamp(displayedImage.timestamp) }}</span>
        <div class="flex-1" />
        <Button label="Copy Seed" icon="pi pi-copy" size="small" severity="secondary" text @click="copySeed(displayedImage!.seed)" />
      </div>
    </div>

    <!-- Empty state -->
    <div v-if="!store.isGenerating && !displayedImage?.dataUrl && !store.error" class="flex-1 flex items-center justify-center text-surface-400">
      <div class="text-center">
        <i class="pi pi-image text-4xl mb-2 block" />
        <p class="text-sm">Generated images will appear here</p>
        <p class="text-xs mt-1">Start the engine, choose a model, and enter a prompt to begin</p>
      </div>
    </div>

    <!-- History strip -->
    <div v-if="store.generatedImages.length > 1" class="shrink-0">
      <div class="flex gap-2 overflow-x-auto pb-2">
        <div
          v-for="(img, index) in store.generatedImages"
          :key="img.jobId"
          class="shrink-0 w-16 h-16 rounded cursor-pointer overflow-hidden border-2 transition-colors"
          :class="index === selectedIndex ? 'border-primary' : 'border-transparent hover:border-surface-300'"
          @click="selectedIndex = index">
          <img v-if="img.dataUrl" :src="img.dataUrl" alt="" class="w-full h-full object-cover" />
          <div v-else class="w-full h-full bg-surface-200 flex items-center justify-center">
            <i class="pi pi-image text-surface-400" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Button, ProgressBar, ProgressSpinner } from 'primevue';
import { useInferenceStore } from '@/stores/inference';

const store = useInferenceStore();
const selectedIndex = ref(0);
const displayedImage = computed(() => store.generatedImages[selectedIndex.value] ?? null);

// When a new image is added, select it
watch(
  () => store.generatedImages.length,
  () => {
    selectedIndex.value = 0;
  },
);

watch(
  () => store.generatedImages.length,
  () => {
    selectedIndex.value = 0;
  },
);

function formatTimestamp(ts: number): string {
  return new Date(ts * 1000).toLocaleTimeString();
}

async function copySeed(seed: number) {
  try {
    await navigator.clipboard.writeText(String(seed));
  } catch {
    // Clipboard might not be available in pywebview
  }
}
</script>
