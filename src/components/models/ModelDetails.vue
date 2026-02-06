<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-semibold text-surface-100">{{ model.displayName }}</h2>
      <div class="flex items-center gap-2">
        <Tag :value="model.model_type" severity="secondary" />
        <Tag :value="model.family" severity="info" />
      </div>
    </div>

    <!-- File Info -->
    <div class="p-3 mb-4 rounded bg-surface-800">
      <h3 class="mb-2 text-xs font-semibold uppercase text-surface-400">File Information</h3>
      <div v-for="file in model.files" :key="file.id" class="space-y-1 text-sm">
        <div><span class="text-surface-500">Path:</span> <span class="break-all text-surface-300">{{ file.path }}</span></div>
        <div v-if="file.isSymlink"><span class="text-surface-500">Resolved:</span> <span class="break-all text-surface-300">{{ file.resolvedPath }}</span></div>
        <div><span class="text-surface-500">Size:</span> <span class="text-surface-300">{{ formatSize(file.sizeBytes) }}</span></div>
        <div v-if="file.sha256"><span class="text-surface-500">SHA256:</span> <span class="font-mono text-xs text-surface-300">{{ file.sha256.slice(0, 16) }}...</span></div>
        <span v-if="file.isSymlink" class="text-xs text-yellow-400">[symlink]</span>
      </div>
    </div>

    <!-- Metadata -->
    <div class="p-3 mb-4 rounded bg-surface-800">
      <h3 class="mb-2 text-xs font-semibold uppercase text-surface-400">Metadata</h3>
      <div class="grid grid-cols-2 text-sm gap-x-4 gap-y-1">
        <div v-if="model.architecture">
          <span class="text-surface-500">Architecture:</span>
          <span class="ml-1 text-surface-300">{{ model.architecture }}</span>
        </div>
        <div v-if="model.quantization">
          <span class="text-surface-500">Quantization:</span>
          <span class="ml-1 text-surface-300">{{ model.quantization }}</span>
        </div>
        <div v-if="model.vramMb">
          <span class="text-surface-500">Est. VRAM:</span>
          <span class="ml-1 text-surface-300">{{ model.vramMb >= 1000 ? `${(model.vramMb / 1000).toFixed(1)} GB` : `${model.vramMb} MB` }}</span>
        </div>
      </div>
    </div>

    <!-- Tags -->
    <div class="mb-4">
      <h3 class="mb-2 text-xs font-semibold uppercase text-surface-400">Tags</h3>
      <div class="flex flex-wrap gap-1">
        <span v-for="tag in model.tags" :key="tag" class="px-2 py-1 text-xs rounded bg-surface-700 text-surface-300">{{ tag }}</span>
        <span v-if="model.tags.length === 0" class="text-xs text-surface-600">No tags</span>
      </div>
    </div>

    <!-- LoRA-specific prefs -->
    <div v-if="model.model_type === 'lora'" class="p-3 mb-4 rounded bg-surface-800">
      <h3 class="mb-2 text-xs font-semibold uppercase text-surface-400">LoRA Settings</h3>
      <div class="text-sm text-surface-400">
        <div v-if="model.prefsLora">
          <span>Strength: {{ model.prefsLora.strengthMin ?? 0 }} – {{ model.prefsLora.strengthMax ?? 1 }} (default: {{ model.prefsLora.strengthDefault ?? 1 }})</span>
        </div>
        <div v-if="model.triggerWords.length">
          <span class="text-surface-500">Trigger words:</span>
          <span v-for="w in model.triggerWords" :key="w" class="ml-1 text-xs bg-surface-700 px-1.5 py-0.5 rounded">{{ w }}</span>
        </div>
      </div>
    </div>

    <!-- Base checkpoint prefs -->
    <div v-if="model.model_type === 'checkpoint' && model.prefsBase" class="p-3 mb-4 rounded bg-surface-800">
      <h3 class="mb-2 text-xs font-semibold uppercase text-surface-400">Checkpoint Preferences</h3>
      <div class="text-sm text-surface-400">
        <div v-if="model.prefsBase.preferredSteps">Preferred steps: {{ model.prefsBase.preferredSteps }}</div>
        <div v-if="model.prefsBase.preferredCfg">Preferred CFG: {{ model.prefsBase.preferredCfg }}</div>
      </div>
    </div>

    <!-- Examples -->
    <div v-if="model.model_type === 'checkpoint' || model.model_type === 'lora'">
      <h3 class="mb-2 text-xs font-semibold uppercase text-surface-400">Examples</h3>
      <div v-if="model.examples.length === 0" class="text-sm text-surface-600">No examples.</div>
      <div v-else class="space-y-2">
        <div v-for="ex in model.examples" :key="ex.id" class="p-2 rounded bg-surface-800">
          <span class="text-xs uppercase text-surface-500">{{ ex.exampleType }}</span>
          <p class="text-sm text-surface-300 mt-0.5">{{ ex.content }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ModelInfo } from '@/types';
import { Tag } from 'primevue';

defineProps<{
  model: ModelInfo;
}>();

function formatSize(bytes: number): string {
  if (bytes >= 1e9) return `${(bytes / 1e9).toFixed(2)} GB`;
  if (bytes >= 1e6) return `${(bytes / 1e6).toFixed(1)} MB`;
  return `${(bytes / 1e3).toFixed(1)} KB`;
}
</script>
