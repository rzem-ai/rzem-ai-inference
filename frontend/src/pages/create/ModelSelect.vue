<template>
  <div class="flex flex-col gap-1">
    <div class="text-base font-medium text-slate-600">Model</div>
    <ModelSelector
      v-model="store.selectedBundleId"
      :bundles="store.bundles"
      :gpu-vram-gb="store.gpuTotalVramGb"
      types="create"
      placeholder="Select model bundle"
      @change="(b) => store.applyBundle(b)" />
  </div>
</template>

<script setup lang="ts">
import ModelSelector from '@/components/ModelSelector.vue';
import { useInferenceStore } from '@/stores/inference';

const store = useInferenceStore();
</script>
