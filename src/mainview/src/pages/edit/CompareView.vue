<template>
  <div class="flex gap-4 h-full w-full items-center justify-center p-4">
    <!-- Input side -->
    <div class="flex-1 flex flex-col items-center gap-2 h-full min-w-0">
      <span class="text-sm text-surface-400 font-medium">Input</span>
      <div class="flex-1 flex items-center justify-center w-full overflow-hidden">
        <img
          v-if="store.inputImageDataUrl"
          :src="store.inputImageDataUrl"
          alt="Input"
          class="max-w-full max-h-full object-contain rounded-xl" />
        <div
          v-else
          class="border border-surface-200 rounded-xl bg-surface-100 flex flex-col items-center justify-center w-full h-full max-w-md max-h-96 gap-2">
          <ImageIcon :size="48" class="text-slate-500" />
          <div class="text-lg text-slate-500">Select an input image</div>
          <div class="text-base text-slate-400">Use the sidebar to pick or drop an image</div>
        </div>
      </div>
    </div>

    <!-- Output side -->
    <div class="flex-1 flex flex-col items-center gap-2 h-full min-w-0">
      <span class="text-sm text-surface-400 font-medium">Output</span>
      <div class="flex-1 flex items-center justify-center w-full overflow-hidden relative">
        <!-- Preview during generation -->
        <img
          v-if="store.isGenerating && store.previewDataUrl"
          :src="store.previewDataUrl"
          alt="Preview"
          class="max-w-full max-h-full object-contain rounded-xl"
          style="filter: blur(1px)" />

        <!-- Completed output -->
        <template v-else-if="outputDataUrl">
          <img
            :src="outputDataUrl"
            alt="Output"
            class="max-w-full max-h-full object-contain rounded-xl" />
          <Button
            class="absolute bottom-3 right-3"
            severity="secondary"
            size="small"
            raised
            title="Use as input"
            @click="store.useOutputAsInput()">
            <ArrowLeftToLine :size="14" class="mr-1" />
            Use as Input
          </Button>
        </template>

        <!-- Empty state -->
        <div
          v-else
          class="border border-surface-200 rounded-xl bg-surface-100 flex flex-col items-center justify-center w-full h-full max-w-md max-h-96 gap-2">
          <Sparkles :size="48" class="text-slate-500" />
          <div class="text-lg text-slate-500">Output will appear here</div>
          <div class="text-base text-slate-400">Enter a prompt and click Generate</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useEditStore } from '@/stores/edit';

const store = useEditStore();
const outputDataUrl = computed(() => store.selectedImage?.dataUrl ?? null);
</script>
