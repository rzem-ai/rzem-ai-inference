<template>
  <div class="flex flex-col gap-1">
    <div class="text-base font-medium text-slate-600">Model</div>

    <Select
      v-model="store.selectedBundleId"
      :options="store.bundlesByType"
      option-group-label="label"
      option-group-children="items"
      option-label="label"
      option-value="id"
      placeholder="Select model bundle"
      fluid
      @change="onBundleChange">
      <template #option="{ option }">
        <div class="flex items-center justify-between w-full gap-2">
          <div>{{ option.label }}</div>
          <div class="flex items-center gap-1">
            <Tag :severity="tierClass(option.tier)" class="uppercase">{{ option.tier }}</Tag>
            <Tag v-if="option.source === 'cloud'" severity="info">Cloud</Tag>
            <Tag v-else :severity="vramClass(option.vram_estimate_gb)">~{{ option.vram_estimate_gb }} GB</Tag>
          </div>
        </div>
      </template>
      <template #value="{ value, placeholder }">
        <div v-if="store.selectedBundle" class="flex items-center gap-2 w-full">
          <Cloud v-if="store.selectedBundle.source === 'cloud'" :size="14" class="text-cyan-500 shrink-0" />
          <Brain v-else :size="14" class="text-blue-500 shrink-0" />
          <div class="truncate grow">{{ store.selectedBundle.label }}</div>
          <Tag v-if="store.selectedBundle.source === 'cloud'" severity="info">Cloud</Tag>
          <Tag v-else :severity="vramClass(store.selectedBundle.vram_estimate_gb)">~{{ store.selectedBundle.vram_estimate_gb }} GB</Tag>
        </div>
        <div v-else class="text-slate-400 text-xs">{{ placeholder }}</div>
      </template>
    </Select>
  </div>
</template>

<script setup lang="ts">
import { Brain, Cloud } from 'lucide-vue-next';
import { Select, Tag } from 'primevue';
import { useInferenceStore } from '@/stores/inference';

const store = useInferenceStore();

function onBundleChange(e: { value: string }) {
  const bundle = store.bundles.find((b) => b.id === e.value);
  if (bundle) store.applyBundle(bundle);
}

function tierClass(tier: string): string {
  switch (tier) {
    case 'performance':
      return 'success';
    case 'balanced':
      return 'info';
    case 'quality':
      return 'danger';
    default:
      return 'secondary';
  }
}

function vramClass(gb: number): string {
  const total = store.gpuTotalVramGb;
  if (total <= 0) {
    // GPU info not loaded yet — neutral styling
    return 'secondary';
  }

  const ratio = gb / total;
  if (ratio <= 0.75) {
    return 'success';
  }
  if (ratio <= 0.95) {
    return 'info';
  }

  return 'danger';
}
</script>
