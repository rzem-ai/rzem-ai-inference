<template>
  <div class="flex flex-col gap-1">
    <div class="text-base font-medium text-slate-600">Model</div>

    <Select
      v-model="store.selectedBundleId"
      :options="store.bundles"
      option-label="label"
      option-value="id"
      placeholder="Select model bundle"
      fluid
      @change="onBundleChange">
      <template #option="{ option }">
        <div class="flex w-full items-center justify-between gap-2">
          <div class="flex items-center gap-1">
            <Gpu :size="14" stroke-width="2" class="text-primary" />
            <div class="text-base font-medium">{{ option.label }}</div>
          </div>
          <Tag :severity="vramClass(option.vram_estimate_gb)">~{{ option.vram_estimate_gb }} GB</Tag>
        </div>
      </template>
      <template #value="{ value, placeholder }">
        <div v-if="selectedBundle" class="flex w-full items-center gap-2">
          <Gpu :size="14" stroke-width="2" class="text-primary" />
          <div class="grow truncate text-base font-medium">{{ selectedBundle.label }}</div>
          <Tag :severity="vramClass(selectedBundle.vram_estimate_gb)">~{{ selectedBundle.vram_estimate_gb }} GB</Tag>
        </div>
        <div v-else class="text-base font-medium text-slate-400">{{ placeholder }}</div>
      </template>
    </Select>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useEditStore } from '@/stores/edit';
import { useInferenceStore } from '@/stores/inference';

const store = useEditStore();
const inferenceStore = useInferenceStore();

const selectedBundle = computed(() =>
  store.bundles.find((b) => b.id === store.selectedBundleId) ?? null,
);

function onBundleChange(e: { value: string }) {
  const bundle = store.bundles.find((b) => b.id === e.value);
  if (bundle) store.applyBundle(bundle);
}

function vramClass(gb: number): string {
  const total = inferenceStore.gpuTotalVramGb;
  if (total <= 0) return 'secondary';
  const ratio = gb / total;
  if (ratio <= 0.75) return 'success';
  if (ratio <= 0.95) return 'info';
  return 'danger';
}
</script>
