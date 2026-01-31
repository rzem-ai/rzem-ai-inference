<template>
  <div class="flex flex-col">
    <div class="">
      <Button size="small" class="w-full" :loading="bundlesStore.isScanning" @click="handleScanBundles">
        <div class="gap-2"> <fa :icon="['fal', 'barcode-read']" size="sm" /> Scan Bundles </div>
      </Button>
    </div>

    <Divider />

    <div v-if="bundlesStore.activeBundle" class="hidden p-3 mx-4 mb-3 border rounded-lg bg-green-900/20 border-green-800/50">
      <div class="mb-1 text-xs font-medium text-green-300">Active Bundle</div>
      <div class="text-sm text-surface-100">{{ bundlesStore.activeBundle.name }}</div>
    </div>

    <!-- Model Count -->
    <div class="flex items-center justify-between px-4 py-2">
      <span class="text-xs text-surface-400">{{ bundlesStore.bundles.length }} bundles</span>
    </div>

    <Divider />

    <div class="px-2 space-y-2 overflow-y-auto">
      <div
        v-for="bundle in bundlesStore.bundles"
        :key="bundle.id"
        class="flex flex-row gap-2 p-2 transition-colors border rounded-lg cursor-pointer bg-surface-950"
        :class="selectedBundle?.id === bundle.id ? 'border-primary bg-primary-900/20' : 'border-surface-700 hover:border-surface-600'"
        @click="selectBundle(bundle)">
        <div class="content-center"><fa :icon="['fal', 'box-taped']" size="xl" /></div>
        <div>
          <div class="mb-1 text-sm font-medium text-surface-100">{{ bundle.name }}</div>
          <div class="flex items-center gap-2 text-xs text-surface-500">
            <Tag v-if="bundle.isActive" value="Active" severity="success" class="text-xs" />
            <span>{{ bundle.components.length }} components</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import Button from 'primevue/button';
import Divider from 'primevue/divider';
import Tag from 'primevue/tag';
import { useBundlesStore } from '@/stores/bundles';
import type { BundleInfo } from '@/stores/bundles';
import { useToast } from 'primevue/usetoast';

const bundlesStore = useBundlesStore();
const toast = useToast();

const selectedBundle = ref<BundleInfo | null>(null);

// Expose selectedBundle for parent to access
defineExpose({ selectedBundle });

function selectBundle(bundle: BundleInfo) {
  selectedBundle.value = bundle;
  bundlesStore.activeBundle = bundle;
  // Notify the right panel by updating the route or emitting an event
  // For now, we'll rely on the parent to sync this
}

async function handleScanBundles() {
  try {
    const result = await bundlesStore.scanBundles();

    toast.add({
      severity: 'success',
      summary: 'Scan Complete',
      detail: `Found ${result.componentsFound} components, created ${result.bundlesCreated} bundles`,
      life: 3000,
    });

    // Auto-select first bundle
    if (bundlesStore.bundles.length > 0) {
      selectBundle(bundlesStore.bundles[0]);
    }
  } catch (err) {
    toast.add({
      severity: 'error',
      summary: 'Scan Failed',
      detail: String(err),
      life: 5000,
    });
  }
}
</script>
