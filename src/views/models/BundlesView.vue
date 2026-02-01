<template>
  <div class="flex h-full">
    <!-- Bundles List -->
    <div class="w-80 flex flex-col border-r border-surface-700 bg-surface-900">
      <div class="p-3 border-b border-surface-700 flex items-center justify-between">
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

      <div v-if="bundlesStore.bundles.length === 0" class="flex-1 flex items-center justify-center">
        <p class="text-sm text-surface-500 text-center px-4">No bundles yet.<br />Scan to discover bundles.</p>
      </div>

      <div v-else class="flex-1 overflow-y-auto">
        <div
          v-for="bundle in bundlesStore.bundles"
          :key="bundle.id"
          class="p-3 border-b border-surface-800 cursor-pointer hover:bg-surface-800 transition-colors"
          :class="{ 'bg-surface-800': selectedBundleId === bundle.id }"
          @click="selectedBundleId = bundle.id"
        >
          <div class="flex items-center justify-between">
            <span class="text-sm font-medium text-surface-200 truncate">{{ bundle.displayName }}</span>
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

    <!-- Details Panel -->
    <div class="flex-1 p-4 overflow-y-auto">
      <div v-if="selectedBundle">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-semibold text-surface-100">{{ selectedBundle.displayName }}</h2>
          <div class="flex gap-2">
            <Button v-if="!selectedBundle.isActive" severity="secondary" size="small" @click="bundlesStore.setActiveBundle(selectedBundle.id)">
              Set Active
            </Button>
            <Button severity="danger" size="small" @click="deleteBundle(selectedBundle.id)">
              <template #icon><fa :icon="['fal', 'trash-can']" /></template>
            </Button>
          </div>
        </div>

        <p v-if="selectedBundle.description" class="text-sm text-surface-400 mb-4">{{ selectedBundle.description }}</p>

        <!-- Components -->
        <div class="mb-4">
          <h3 class="text-sm font-semibold text-surface-300 mb-2">Components</h3>
          <div class="space-y-2">
            <div v-for="item in selectedBundle.items" :key="item.id" class="flex items-center justify-between p-2 rounded bg-surface-800">
              <div class="flex items-center gap-2">
                <span class="text-xs text-surface-500 w-20 uppercase">{{ item.role }}</span>
                <span class="text-sm text-surface-200">{{ item.modelDisplayName }}</span>
              </div>
              <div class="flex items-center gap-2">
                <Tag v-if="item.modelQuantization" :value="item.modelQuantization" severity="info" class="text-xs" />
                <span v-if="item.modelVramMb" class="text-xs text-surface-500">{{ bundlesStore.formatVram(item.modelVramMb) }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Tags -->
        <div class="mb-4">
          <h3 class="text-sm font-semibold text-surface-300 mb-2">Tags</h3>
          <div class="flex flex-wrap gap-1">
            <span v-for="tag in selectedBundle.tags" :key="tag" class="text-xs bg-surface-700 text-surface-300 px-2 py-1 rounded flex items-center gap-1">
              {{ tag }}
              <fa :icon="['fal', 'xmark']" size="xs" class="cursor-pointer hover:text-red-400" @click="bundlesStore.deleteBundle" />
            </span>
          </div>
        </div>

        <!-- Examples -->
        <div>
          <h3 class="text-sm font-semibold text-surface-300 mb-2">Examples</h3>
          <div v-if="selectedBundle.examples.length === 0" class="text-sm text-surface-500">No examples added.</div>
          <div v-else class="space-y-1">
            <div v-for="ex in selectedBundle.examples" :key="ex.id" class="text-sm text-surface-400 p-2 rounded bg-surface-800">
              <span class="text-xs text-surface-500 uppercase mr-2">{{ ex.exampleType }}</span>
              {{ ex.content }}
            </div>
          </div>
        </div>
      </div>

      <div v-else class="flex items-center justify-center h-full">
        <p class="text-sm text-surface-500">Select a bundle to view details.</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useBundlesStore } from '@/stores/bundles';
import { Button, Tag } from 'primevue';

const bundlesStore = useBundlesStore();

const selectedBundleId = ref<string | null>(null);
const scanning = ref(false);

const selectedBundle = computed(() => {
  if (!selectedBundleId.value) return null;
  return bundlesStore.bundles.find((b) => b.id === selectedBundleId.value) ?? null;
});

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

async function deleteBundle(id: string) {
  await bundlesStore.deleteBundle(id);
  if (selectedBundleId.value === id) selectedBundleId.value = null;
}
</script>
