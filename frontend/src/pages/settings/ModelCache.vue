<template>
  <div class="flex flex-col gap-4 p-4 overflow-y-auto">
    <!-- Cache Locations -->
    <div>
      <div class="text-xl font-semibold text-slate-900 mb-3">Cache Locations</div>
      <div class="flex gap-2">
        <!-- HuggingFace Cache -->
        <Card class="grow">
          <template #title>
            <div class="flex items-center gap-2">HuggingFace Cache</div>
          </template>
          <template #content>
            <div class="text-base font-mono text-slate-700 truncate" :title="settingsStore.dataPaths?.hf_cache_dir ?? ''">
              {{ settingsStore.dataPaths?.hf_cache_dir ?? 'Unknown' }}
            </div>
            <div v-if="settingsStore.diskUsage" class="text-base text-slate-400 mt-1"> {{ formatBytes(settingsStore.diskUsage.hf_cache_size) }} used </div>
          </template>
        </Card>

        <!-- Model Output -->
        <Card class="grow">
          <template #title>
            <div class="flex items-center gap-2">Model Output</div>
          </template>
          <template #content>
            <div class="text-base font-mono text-slate-700 truncate" :title="settingsStore.dataPaths?.output_dir ?? ''">
              {{ settingsStore.dataPaths?.output_dir ?? 'Unknown' }}
            </div>
            <div v-if="settingsStore.diskUsage" class="text-base text-slate-400 mt-1"> {{ formatBytes(settingsStore.diskUsage.output_size) }} used </div>
          </template>
        </Card>
      </div>
    </div>

    <!-- Cached Models -->
    <div>
      <div class="flex items-center justify-between mb-3">
        <div class="text-lg font-semibold">Cached Models</div>
        <Button severity="help" size="small" @click="refreshCache"> Refresh </Button>
      </div>

      <div v-if="settingsStore.cachedModels.length === 0" class="bg-white rounded-xl border border-slate-200 p-6 text-center">
        <p class="text-base text-slate-500">No cached models found</p>
      </div>

      <div v-else class="flex flex-col gap-2">
        <div v-for="model in settingsStore.cachedModels" :key="model.repo_id" class="bg-white rounded-xl border border-slate-200 p-4 flex items-center gap-4">
          <div class="flex-1 min-w-0">
            <div class="text-sm font-medium text-slate-900 truncate">{{ model.repo_id }}</div>
            <div class="flex items-center gap-3 mt-1">
              <span class="text-base text-slate-500">{{ formatBytes(model.size_on_disk) }}</span>
              <span class="text-base text-slate-400">{{ model.nb_files }} files</span>
              <span class="text-base text-slate-400">{{ model.revisions.length }} revision{{ model.revisions.length !== 1 ? 's' : '' }}</span>
            </div>
          </div>
          <button class="p-1 text-slate-400 hover:text-red-500 transition-colors shrink-0" title="Delete cached model" @click="handleDelete(model.repo_id)">
            <Trash2 :size="14" />
          </button>
        </div>

        <!-- Total usage bar -->
        <div class="bg-white rounded-xl border border-slate-200 p-4 mt-1">
          <div class="flex items-center justify-between mb-2">
            <span class="text-base font-medium text-slate-500">Total Cache Size</span>
            <span class="text-base font-semibold text-slate-900">{{ formatBytes(settingsStore.cacheTotalSize) }}</span>
          </div>
          <div class="h-2 rounded-full bg-slate-100 overflow-hidden">
            <div class="h-full bg-blue-500 rounded-full transition-all" :style="{ width: cacheBarWidth }" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useSettingsStore } from '@/stores/settings';

const settingsStore = useSettingsStore();

const cacheBarWidth = computed(() => {
  // Scale relative to a reasonable max (100 GB)
  const maxBytes = 100 * 1024 ** 3;
  const pct = Math.min((settingsStore.cacheTotalSize / maxBytes) * 100, 100);
  return `${pct}%`;
});

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const gb = bytes / 1024 ** 3;
  if (gb >= 1) return `${gb.toFixed(1)} GB`;
  const mb = bytes / 1024 ** 2;
  if (mb >= 1) return `${mb.toFixed(0)} MB`;
  const kb = bytes / 1024;
  return `${kb.toFixed(0)} KB`;
}

async function handleDelete(repoId: string) {
  await settingsStore.deleteCachedModel(repoId);
}

async function refreshCache() {
  await Promise.all([settingsStore.loadCacheInfo(), settingsStore.loadDiskUsage()]);
}

onMounted(async () => {
  await Promise.all([settingsStore.loadCacheInfo(), settingsStore.loadDiskUsage()]);
});
</script>
