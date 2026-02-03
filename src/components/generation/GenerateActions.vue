<template>
  <div class="flex flex-col h-full gap-2">
    <!-- Scrollable Content -->
    <ul role="list" class="divide-y divide-surface-600/50">
      <PromptInput @generate="handleGenerate" />
      <StyleDropdown v-if="showStyle" />
      <ImageDimensions @generate="handleGenerate" />
      <QualitySelector @generate="handleGenerate" v-if="showQuality" />
      <AdvancedSettings @generate="handleGenerate" v-if="showAdvanced" />
    </ul>
  </div>
</template>

<script setup lang="ts">
import AdvancedSettings from '@/components/generation/actions/AdvancedSettings.vue';
import PromptInput from '@/components/generation/actions/PromptInput.vue';
import ImageDimensions from '@/components/generation/actions/ImageDimensions.vue';
import StyleDropdown from '@/components/generation/actions/StyleDropdown.vue';

import QualitySelector from './actions/QualitySelector.vue';

defineProps(['showQuality', 'showStyle', 'showAdvanced']);

const emit = defineEmits<{
  generate: [];
}>();

const handleGenerate = () => {
  emit('generate');
};
</script>
