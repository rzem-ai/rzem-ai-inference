<template>
  <div class="prompt-input">
    <PromptEditor
      v-model="prompt"
      label="Prompt"
      placeholder="Describe the image you want to generate..."
      :rows="4"
    />

    <PromptEditor
      v-model="negativePrompt"
      label="Negative Prompt"
      placeholder="What to avoid in the image..."
      :rows="2"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useGenerationStore } from '@/stores/generation';
import PromptEditor from './PromptEditor.vue';

const store = useGenerationStore();

const prompt = computed({
  get: () => store.currentParams.prompt,
  set: (value: string) => {
    store.currentParams.prompt = value;
  },
});

const negativePrompt = computed({
  get: () => store.currentParams.negativePrompt || '',
  set: (value: string) => {
    store.currentParams.negativePrompt = value || undefined;
  },
});
</script>

<style scoped>
@reference "tailwindcss";

.prompt-input {
  @apply flex flex-col gap-4;
}
</style>
