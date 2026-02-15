<template>
  <div class="flex flex-col gap-4 p-4 overflow-y-auto">
    <!-- Preview Settings -->
    <div>
      <h2 class="text-lg font-semibold text-slate-900 mb-1">Preview Images</h2>
      <p class="text-sm text-slate-500 mb-3">
        Control the intermediate preview images shown during generation.
        Changes take effect the next time the engine starts.
      </p>

      <div class="flex flex-col gap-3">
        <!-- Preview Interval -->
        <div class="bg-white rounded-xl border border-slate-200 p-4">
          <div class="flex items-center justify-between mb-2">
            <div>
              <div class="text-sm font-medium text-slate-900">Preview Interval</div>
              <div class="text-xs text-slate-500">Number of denoising steps between each preview</div>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-sm font-semibold text-slate-700 w-12 text-right">
                {{ localInterval }}
              </span>
              <span class="text-xs text-slate-400">steps</span>
            </div>
          </div>
          <input
            v-model.number="localInterval"
            type="range"
            :min="1"
            :max="20"
            :step="1"
            class="w-full accent-blue-600"
            @change="saveInterval" />
          <div class="flex justify-between text-xs text-slate-400 mt-1">
            <span>1 (frequent)</span>
            <span>20 (infrequent)</span>
          </div>
        </div>

        <!-- Preview Max Size -->
        <div class="bg-white rounded-xl border border-slate-200 p-4">
          <div class="flex items-center justify-between mb-2">
            <div>
              <div class="text-sm font-medium text-slate-900">Preview Size</div>
              <div class="text-xs text-slate-500">Maximum dimension (width or height) of preview images in pixels</div>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-sm font-semibold text-slate-700 w-12 text-right">
                {{ localMaxSize }}
              </span>
              <span class="text-xs text-slate-400">px</span>
            </div>
          </div>
          <input
            v-model.number="localMaxSize"
            type="range"
            :min="64"
            :max="1024"
            :step="64"
            class="w-full accent-blue-600"
            @change="saveMaxSize" />
          <div class="flex justify-between text-xs text-slate-400 mt-1">
            <span>64px (fast)</span>
            <span>1024px (detailed)</span>
          </div>
        </div>
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
