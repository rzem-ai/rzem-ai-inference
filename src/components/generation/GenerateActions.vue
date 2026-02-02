<template>
  <div class="flex flex-col h-full gap-2">
    <!-- Scrollable Content -->

    <ul role="list" class="divide-y divide-surface-600/50">
      <PromptInput @generate="handleGenerate" />
      <ImageDimensions @generate="handleGenerate" />
      <QualitySelector @generate="handleGenerate" v-if="showQuality" />
      <LoraPanel @generate="handleGenerate" v-if="showStyle" />
      <AdvancedSettings @generate="handleGenerate" v-if="showAdvanced" />
    </ul>

    <!-- Preset Modal -->
    <Dialog v-model:visible="showPresetModal" modal header="Presets" :style="{ width: '450px' }">
      <PresetSelector />
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

import Dialog from 'primevue/dialog';

import AdvancedSettings from '@/components/generation/actions/AdvancedSettings.vue';
import PromptInput from '@/components/generation/actions/PromptInput.vue';
import ImageDimensions from '@/components/generation/actions/ImageDimensions.vue';
import LoraPanel from '@/components/generation/actions/LoraPanel.vue';
import PresetSelector from '@/components/generation/actions/PresetSelector.vue';

import QualitySelector from './actions/QualitySelector.vue';

defineProps(['showQuality', 'showStyle', 'showAdvanced']);

const emit = defineEmits<{
  generate: [];
}>();

const handleGenerate = () => {
  emit('generate');
};

const showPresetModal = ref(false);

// Model defaults are now applied via bundle selection in EnhancedModelSelector
</script>
