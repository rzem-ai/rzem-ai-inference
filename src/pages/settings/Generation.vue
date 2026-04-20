<template>
  <div class="flex flex-col gap-4 p-4 overflow-y-auto">
    <!-- Preview Settings -->
    <div>
      <div class="text-xl font-semibold text-slate-900 mb-3">Image Generation</div>
      <div class="text-base"> Control the intermediate preview images shown during generation. Changes take effect the next time the engine starts. </div>

      <div class="flex gap-2">
        <!-- Preview Interval -->
        <Card class="grow">
          <template #title>
            <div class="flex items-center gap-2">Preview Interval</div>
          </template>
          <template #content>
            <div class="flex items-center justify-between mb-2">
              <div>
                <div class="text-sm">Number of denoising steps between each preview</div>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-sm font-semibold text-right">
                  {{ localInterval }}
                </span>
                <span class="text-sm">steps</span>
              </div>
            </div>
            <input v-model.number="localInterval" type="range" :min="1" :max="20" :step="1" class="w-full accent-blue-600" @change="saveInterval" />
            <div class="flex justify-between text-sm">
              <span>1 (frequent)</span>
              <span>20 (infrequent)</span>
            </div>
          </template>
        </Card>

        <!-- Preview Max Size -->
        <Card class="grow">
          <template #title>
            <div class="flex items-center gap-2">Preview Size</div>
          </template>
          <template #content>
            <div class="flex items-center justify-between mb-2">
              <div>
                <div class="text-sm">Maximum dimension (width or height) of preview images in pixels</div>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-sm font-semibold text-right">
                  {{ localMaxSize }}
                </span>
                <span class="text-sm">px</span>
              </div>
            </div>
            <input v-model.number="localMaxSize" type="range" :min="64" :max="1024" :step="64" class="w-full accent-blue-600" @change="saveMaxSize" />
            <div class="flex justify-between text-xs text-slate-400 mt-1">
              <span>64px (fast)</span>
              <span>1024px (detailed)</span>
            </div>
          </template>
        </Card>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useSettingsStore } from '@/stores/settings';

const settingsStore = useSettingsStore();

const localInterval = ref(settingsStore.previewInterval);
const localMaxSize = ref(settingsStore.previewMaxSize);

async function saveInterval() {
  await settingsStore.savePreviewInterval(localInterval.value);
}

async function saveMaxSize() {
  await settingsStore.savePreviewMaxSize(localMaxSize.value);
}

onMounted(async () => {
  await settingsStore.loadPreviewSettings();
  localInterval.value = settingsStore.previewInterval;
  localMaxSize.value = settingsStore.previewMaxSize;
});
</script>
