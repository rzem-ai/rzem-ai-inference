<template>
  <div class="flex flex-col gap-1">
    <div class="text-base font-medium text-slate-600">Style</div>

    <Select
      :model-value="inferenceStore.selectedStyleId"
      :options="styleOptions"
      option-label="name"
      option-value="id"
      placeholder="No style"
      show-clear
      fluid
      @change="onStyleChange">
      <template #option="{ option }">
        <div class="flex items-center gap-2">
          <Paintbrush :size="14" class="text-slate-400 shrink-0" />
          <div class="flex-1 truncate">{{ option.name }}</div>
          <span v-if="option.category" class="text-[10px] text-slate-400">{{ option.category }}</span>
        </div>
      </template>
      <template #value="{ value, placeholder }">
        <div v-if="selectedStyle" class="flex items-center gap-2 w-full">
          <Paintbrush :size="14" class="text-blue-500 shrink-0" />
          <div class="truncate grow">{{ selectedStyle.name }}</div>
          <span v-if="selectedStyle.category" class="text-[10px] text-slate-400">{{ selectedStyle.category }}</span>
        </div>
        <div v-else class="text-slate-400 text-xs">{{ placeholder }}</div>
      </template>
    </Select>

    <!-- Active style info -->
    <p v-if="selectedStyle?.description" class="text-[10px] text-slate-400 px-1">
      {{ selectedStyle.description }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Paintbrush } from 'lucide-vue-next';
import { Select } from 'primevue';
import { useInferenceStore } from '@/stores/inference';
import { useStylesStore } from '@/stores/styles';
import { usePywebview } from '@/composables/usePywebview';

const { api } = usePywebview();
const inferenceStore = useInferenceStore();
const stylesStore = useStylesStore();

const styleOptions = computed(() => stylesStore.styles);

const selectedStyle = computed(() =>
  stylesStore.styles.find(s => s.id === inferenceStore.selectedStyleId) ?? null
);

async function onStyleChange(e: any) {
  const styleId = e.value;
  if (!styleId) {
    inferenceStore.clearStyle();
    return;
  }

  const res = await api.value.get_style({ style_id: styleId });
  if (res.status === 'success' && res.style) {
    inferenceStore.applyStyle(
      res.style.id,
      res.style.prompt_template,
      res.style.negative_prompt,
      res.loras ?? [],
    );
  }
}
</script>
