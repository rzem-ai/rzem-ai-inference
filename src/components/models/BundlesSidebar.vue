<template>
  <!-- Bundles List -->
  <div class="flex flex-col w-full h-full px-2">
    <div class="flex items-center justify-between p-3 border-b border-surface-700">
      <span class="text-sm font-semibold text-surface-300">Bundles</span>
      <div class="flex gap-1">
        <Button severity="secondary" size="small" @click="scanBundles" :loading="scanning">
          <template #icon><fa :icon="['fal', 'magnifying-glass']" /></template>
        </Button>
        <Button severity="secondary" size="small" @click="createNewBundle">
          <template #icon><fa :icon="['fal', 'plus']" /></template>
        </Button>
      </div>
    </div>

    <div v-if="bundlesStore.bundles.length === 0" class="flex items-center justify-center flex-1">
      <p class="px-4 text-sm text-center text-surface-500">No bundles yet.<br />Scan to discover bundles.</p>
    </div>

    <div v-else class="flex-1 overflow-y-auto">
      <div
        v-for="bundle in bundlesStore.bundles"
        :key="bundle.id"
        class="p-3 transition-colors border-b cursor-pointer border-surface-800 hover:bg-surface-800"
        :class="{ 'bg-surface-800': selectedBundleId === bundle.id }"
        @click="selectedBundleId = bundle.id">
        <div class="flex items-center justify-between">
          <span class="text-sm font-medium truncate text-surface-200">{{ bundle.displayName }}</span>
          <div class="flex items-center gap-1 ml-2 shrink-0">
            <Tag v-if="bundle.isActive" value="Active" severity="success" class="text-xs" />
            <Tag v-if="!bundle.isComplete" value="Incomplete" severity="warn" class="text-xs" />
          </div>
        </div>
        <div class="flex items-center gap-2 mt-1">
          <span class="text-xs text-surface-500">{{ bundle.items.length }} components</span>
          <span v-if="bundle.totalVramMb" class="text-xs text-surface-500">• {{ bundlesStore.formatVram(bundle.totalVramMb) }}</span>
        </div>
        <div v-if="bundle.tags.length" class="flex flex-wrap gap-1 mt-1">
          <span v-for="tag in bundle.tags" :key="tag" class="text-xs bg-surface-700 text-surface-300 px-1.5 py-0.5 rounded">{{ tag }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useBundlesStore } from '@/stores/bundles';
import { Button, Tag } from 'primevue';

const bundlesStore = useBundlesStore();

const selectedBundleId = ref<string | null>(null);
const scanning = ref(false);

async function scanBundles() {
  scanning.value = true;
  try {
    await bundlesStore.scanHfCache();
  } finally {
    scanning.value = false;
  }
}

function createNewBundle() {
  // TODO: open bundle builder wizard
}
</script>
