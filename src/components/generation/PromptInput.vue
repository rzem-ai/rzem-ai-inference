<template>
  <div class="flex flex-col gap-4">
    <PromptEditor v-model="prompt" label="Prompt" placeholder="Describe the image you want to generate..." :rows="4" />
    <Button
      :loading="queueStore.hasRunningJobs"
      fluid
      size="small"
      @click="handleGenerate"
      :disabled="!canGenerate">
      {{ queueStore.queueLength > 0 ? `Generate` : 'Generate' }}
    </Button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useGenerationStore } from '@/stores/generation';
import { useQueueStore } from '@/stores/queue';
import PromptEditor from './PromptEditor.vue';
import Button from 'primevue/button';

const emit = defineEmits<{
  generate: [];
}>();

const store = useGenerationStore();
const queueStore = useQueueStore();

const prompt = computed({
  get: () => store.currentParams.prompt,
  set: (value: string) => {
    store.currentParams.prompt = value;
  },
});

const canGenerate = computed(() => {
  return store.currentParams.prompt.trim().length > 0;
});

const handleGenerate = () => {
  if (!canGenerate.value) return;
  emit('generate');
};
</script>