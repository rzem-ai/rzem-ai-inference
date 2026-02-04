<template>
  <GenerationAction icon="sliders" label="Style">
    <!-- Style Selector -->
    <Select
      v-model="selectedStyleId"
      :options="styleOptions"
      option-label="label"
      option-value="value"
      placeholder="Select a style..."
      show-clear
      size="small"
      fluid
      @change="handleStyleChange">
      <template #value="slotProps">
        <div v-if="slotProps.value" class="flex items-center gap-2">
          <fa v-if="currentStyle?.isFavorite" :icon="['fas', 'star']" class="text-yellow-400" size="sm" />
          <span>{{ currentStyle?.name }}</span>
        </div>
        <span v-else>{{ slotProps.placeholder }}</span>
      </template>
      <template #option="slotProps">
        <div class="flex items-center justify-between w-full">
          <div class="flex items-center gap-2">
            <fa v-if="slotProps.option.isFavorite" :icon="['fas', 'star']" class="text-yellow-400" size="sm" />
            <span>{{ slotProps.option.label }}</span>
          </div>
          <span v-if="slotProps.option.category" class="text-xs text-surface-500">
            {{ slotProps.option.category }}
          </span>
        </div>
      </template>
    </Select>

    <!-- Style Preview -->
    <div v-if="currentStyle" class="p-3 rounded bg-surface-800">
      <div class="flex items-start justify-between mb-2">
        <div class="flex-1 min-w-0">
          <p class="text-xs font-medium text-surface-400">Template</p>
          <p class="font-mono text-sm break-words text-surface-200">{{ currentStyle.promptTemplate }}</p>
        </div>
      </div>

      <!-- LoRAs in style -->
      <div v-if="currentStyle.loras && currentStyle.loras.length > 0" class="pt-2 mt-2 border-t border-surface-700">
        <p class="mb-1 text-xs font-medium text-surface-400">LoRAs ({{ currentStyle.loras.length }})</p>
        <div class="flex flex-col gap-1">
          <div v-for="lora in currentStyle.loras" :key="lora.loraId" class="flex items-center justify-between text-xs">
            <span class="truncate text-surface-300">{{ lora.loraName }}</span>
            <span class="text-surface-500">{{ lora.strength.toFixed(2) }}</span>
          </div>
        </div>
      </div>
    </div>
  </GenerationAction>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';

import { useStylesStore } from '@/stores/styles';
import { useGenerationStore } from '@/stores/generation';

import Select from 'primevue/select';
import GenerationAction from './GenerationAction.vue';

const stylesStore = useStylesStore();
const generationStore = useGenerationStore();

const selectedStyleId = ref<string | null>(null);

const styleOptions = computed(() => {
  return stylesStore.styles.map((style) => ({
    label: style.name,
    value: style.id,
    isFavorite: style.isFavorite,
    category: style.category,
  }));
});

const currentStyle = computed(() => {
  if (!selectedStyleId.value) return null;
  return stylesStore.selectedStyle;
});

async function handleStyleChange(event: any) {
  const styleId = event.value;
  if (styleId) {
    // Load style detail for preview
    await stylesStore.loadStyleDetail(styleId);
  }
}

// async function handleApplyStyle() {
//   if (!selectedStyleId.value) return;

//   isApplying.value = true;
//   try {
//     await generationStore.applyStyle(selectedStyleId.value);
//   } catch (error) {
//     console.error('Failed to apply style:', error);
//   } finally {
//     isApplying.value = false;
//   }
// }

// function handleClearStyle() {
//   selectedStyleId.value = null;
//   generationStore.clearStyle();
//   stylesStore.clearSelection();
// }

// function navigateToStyles() {
//   router.push('/styles');
// }

// Watch for external style selection changes
watch(
  () => generationStore.selectedStyleId,
  (newStyleId) => {
    if (newStyleId !== selectedStyleId.value) {
      selectedStyleId.value = newStyleId;
      if (newStyleId) {
        stylesStore.loadStyleDetail(newStyleId);
      }
    }
  },
);

onMounted(async () => {
  // Load styles list
  if (stylesStore.styles.length === 0) {
    await stylesStore.loadStyles();
  }

  // Restore previously selected style if any
  if (generationStore.selectedStyleId) {
    selectedStyleId.value = generationStore.selectedStyleId;
    await stylesStore.loadStyleDetail(generationStore.selectedStyleId);
  }
});
</script>

<style scoped>
.list-item-component {
  padding-left: 1rem;
  padding-right: 1rem;
}
</style>
