<template>
  <Select
    :model-value="modelValue"
    :options="groupedBundles"
    option-group-label="label"
    option-group-children="items"
    option-label="label"
    option-value="id"
    :placeholder="placeholder"
    :show-clear="showClear"
    fluid
    @change="onChange">
    <template #optiongroup="slotProps">
      <div class="flex items-center gap-2">
        <Star v-if="slotProps.option.type === '_favorites'" :size="14" class="text-amber-400 fill-amber-400" />
        <Cloud v-else-if="slotProps.option.type === 'fal_cloud'" :size="14" stroke-width="2" class="text-primary" />
        <Gpu v-else :size="14" stroke-width="2" class="text-primary" />
        <div class="text-base font-semibold">{{ slotProps.option.label }}</div>
      </div>
    </template>
    <template #option="{ option }">
      <div class="flex w-full items-center justify-between gap-2">
        <div class="flex items-center gap-1">
          <Star v-if="option.favorite" :size="12" class="text-amber-400 fill-amber-400 shrink-0" />
          <div class="text-base font-medium">{{ option.label }}</div>
        </div>
        <div class="flex items-center gap-1">
          <Tag :severity="tierClass(option.tier)" class="uppercase">{{ option.tier }}</Tag>
          <Tag v-if="option.source === 'cloud'" severity="info">Cloud</Tag>
          <Tag v-else :severity="vramClass(option.vram_estimate_gb)">~{{ option.vram_estimate_gb }} GB</Tag>
        </div>
      </div>
    </template>
    <template #value="{ placeholder: ph }">
      <div v-if="selectedBundle" class="flex w-full items-center gap-2">
        <Cloud v-if="selectedBundle.source === 'cloud'" :size="14" stroke-width="2" class="text-primary" />
        <Gpu v-else :size="14" stroke-width="2" class="text-primary" />
        <div class="grow truncate text-base font-medium">{{ selectedBundle.label }}</div>
        <Tag v-if="selectedBundle.source === 'cloud'" severity="info">Cloud</Tag>
        <Tag v-else :severity="vramClass(selectedBundle.vram_estimate_gb)">~{{ selectedBundle.vram_estimate_gb }} GB</Tag>
      </div>
      <div v-else class="text-base font-medium text-slate-400">{{ ph }}</div>
    </template>
  </Select>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useModelsStore } from '@/stores/models';
import type { ModelBundle } from '@/types/inference';

interface Props {
  modelValue: string | null;
  bundles: ModelBundle[];
  types?: 'create' | 'edit' | 'all';
  gpuVramGb?: number;
  placeholder?: string;
  showClear?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  types: 'all',
  gpuVramGb: 0,
  placeholder: 'Select model',
  showClear: false,
});

const emit = defineEmits<{
  'update:modelValue': [value: string | null];
  change: [bundle: ModelBundle];
}>();

const modelsStore = useModelsStore();

const filteredBundles = computed(() => {
  switch (props.types) {
    case 'create':
      return props.bundles.filter((b) => !b.hidden && b.transformer_type !== 'flux1_kontext');
    case 'edit':
      return props.bundles.filter((b) => b.transformer_type === 'flux1_kontext');
    default:
      return props.bundles.filter((b) => !b.hidden);
  }
});

const groupedBundles = computed(() => {
  const result: { label: string; type: string; items: ModelBundle[] }[] = [];

  // Favorites group
  const favorites = filteredBundles.value.filter((b) => b.favorite);
  if (favorites.length) {
    result.push({ label: 'Favorites', type: '_favorites', items: favorites });
  }

  // Type groups sorted by sort_order, local before cloud
  const groups = new Map<string, ModelBundle[]>();
  for (const b of filteredBundles.value) {
    const list = groups.get(b.transformer_type) ?? [];
    list.push(b);
    groups.set(b.transformer_type, list);
  }

  const typeGroups = Array.from(groups.entries())
    .map(([type, items]) => ({
      label: modelsStore.getTypeLabel(type),
      type,
      icon: modelsStore.typeMap[type]?.icon ?? null,
      sortOrder: modelsStore.typeMap[type]?.sort_order ?? 0,
      items,
    }))
    .sort((a, b) => a.sortOrder - b.sortOrder);

  const local = typeGroups.filter((g) => g.icon !== 'cloud');
  const cloud = typeGroups.filter((g) => g.icon === 'cloud');
  result.push(...local, ...cloud);

  return result;
});

const selectedBundle = computed(() => {
  if (!props.modelValue) return null;
  return props.bundles.find((b) => b.id === props.modelValue) ?? null;
});

function onChange(e: { value: string | null }) {
  emit('update:modelValue', e.value);
  if (e.value) {
    const bundle = props.bundles.find((b) => b.id === e.value);
    if (bundle) emit('change', bundle);
  }
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
  if (props.gpuVramGb <= 0) return 'secondary';
  const ratio = gb / props.gpuVramGb;
  if (ratio <= 0.75) return 'success';
  if (ratio <= 0.95) return 'info';
  return 'danger';
}
</script>
