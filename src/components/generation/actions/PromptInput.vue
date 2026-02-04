<template>
  <GenerationAction icon="pen-to-square" label="Image Description">
    <div class="relative">
      <PromptEditor v-model="prompt" label="Prompt" placeholder="Describe the image you want to generate..." :rows="4" />
    </div>

    <!-- Configuration validation message -->
    <Message v-if="!generationStore.isValidConfiguration && prompt.trim().length > 0" severity="warn" size="small">
      <template #icon>
        <fa :icon="['fal', 'triangle-exclamation']" size="lg" />
      </template>
      Please select a model bundle or configure all components (Model, T5, CLIP, VAE) in the Quality section
    </Message>

    <!-- Batch Script Dialog -->
    <BatchScriptDialog v-model:visible="showBatchDialog" />
  </GenerationAction>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useGenerationStore } from '@/stores/generation';

import GenerationAction from './GenerationAction.vue';
import PromptEditor from './PromptEditor.vue';

import BatchScriptDialog from '@/components/generation/batch/BatchScriptDialog.vue';
import Message from 'primevue/message';

defineEmits<{
  generate: [];
}>();

const generationStore = useGenerationStore();

// Batch dialog visibility
const showBatchDialog = ref(false);

const prompt = computed({
  get: () => generationStore.currentParams.prompt,
  set: (value: string) => {
    generationStore.currentParams.prompt = value;
  },
});
</script>
