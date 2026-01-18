<template>
  <div class="prompt-input">
    <div class="field">
      <label for="prompt">Prompt</label>
      <Textarea id="prompt" v-model="prompt" rows="4" placeholder="Describe the image you want to generate..." class="w-full" />
    </div>

    <div class="field">
      <label for="negative-prompt">Negative Prompt</label>
      <Textarea id="negative-prompt" v-model="negativePrompt" rows="2" placeholder="What to avoid in the image..." class="w-full" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useGenerationStore } from '@/stores/generation';
import Textarea from 'primevue/textarea';

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
