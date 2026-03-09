<template>
  <MenuPanel title="Edit" icon="PenToolIcon">
    <template #content>
      <div class="flex-1 overflow-y-auto px-4 pb-2 flex flex-col gap-4">
        <ImageInput />
        <PromptInput @submit="onSubmit" />
        <ModelSelect />
        <QualitySection />
      </div>

      <div v-if="store.error" class="px-4 py-1">
        <span class="text-sm text-red-500">{{ store.error }}</span>
      </div>
    </template>
    <template #footer>
      <ProgressOverlay />
      <div v-if="!store.isGenerating" class="flex gap-2">
        <Button
          class="flex-1 transition-colors flex items-center justify-center gap-2"
          severity="primary"
          raised
          :disabled="!canGenerate"
          @click="onSubmit">
          <Sparkles :size="16" />
          Generate
        </Button>
      </div>
      <Button
        v-else
        class="transition-colors flex items-center justify-center gap-2"
        severity="danger"
        fluid
        raised
        @click="store.cancelJob()">
        <Square :size="14" />
        Cancel
      </Button>
    </template>
  </MenuPanel>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, watch } from 'vue';
import { useEditStore } from '@/stores/edit';
import { useInferenceStore } from '@/stores/inference';
import ImageInput from './ImageInput.vue';
import PromptInput from './PromptInput.vue';
import ModelSelect from './ModelSelect.vue';
import QualitySection from './QualitySection.vue';
import ProgressOverlay from './ProgressOverlay.vue';
import MenuPanel from '@/components/MenuPanel.vue';

const store = useEditStore();
const inferenceStore = useInferenceStore();

const canGenerate = computed(
  () => store.engineReady && store.inputImagePath && store.params.prompt.trim() && !store.isGenerating,
);

function onSubmit() {
  store.submitJob();
}

// Watch inference store events for our job updates
const stopWatch = watch(
  () => inferenceStore.events.length,
  () => store.processNewEvents(),
);

// Clipboard paste handler
function onPaste(e: ClipboardEvent) {
  const items = e.clipboardData?.items;
  if (!items) return;
  for (const item of items) {
    if (item.type.startsWith('image/')) {
      const blob = item.getAsFile();
      if (!blob) continue;
      const reader = new FileReader();
      reader.onload = async () => {
        const dataUrl = reader.result as string;
        const api = await (await import('@/bridge')).getApiAsync();
        const res = await api.save_clipboard_image({ data_url: dataUrl });
        if (res.status === 'success' && res.path) {
          store.setInputImage(res.path);
        }
      };
      reader.readAsDataURL(blob);
      break;
    }
  }
}

onMounted(async () => {
  window.addEventListener('paste', onPaste);

  await inferenceStore.loadGpuInfo();
  await store.loadBundles();
  if (store.bundles.length && !store.selectedBundleId) {
    store.applyBundle(store.bundles[0]);
  }

  // Ensure engine is running
  if (!inferenceStore.engineReady && !inferenceStore.engineStarting) {
    await inferenceStore.startEngine();
  }
});

onUnmounted(() => {
  window.removeEventListener('paste', onPaste);
  stopWatch();
});
</script>
