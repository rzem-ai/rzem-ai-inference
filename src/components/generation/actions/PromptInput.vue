<template>
  <GenerationAction :collapsed="false" :toggleable="false" icon="pen-to-square" label="Image Description">
    <PromptEditor v-model="prompt" label="Prompt" placeholder="Describe the image you want to generate..." :rows="4" />
    <div class="flex flex-col gap-2">
      <SplitButton :loading="queueStore.hasRunningJobs" fluid size="small" @click="handleGenerate" :model="generationCounts" :disabled="!canGenerate">
        {{ queueStore.queueLength > 0 ? `Generate` : 'Generate' }} ( {{ imageCount }} )
      </SplitButton>
      <Button label="Batch Script" icon="pi pi-list" severity="secondary" size="small" fluid @click="showBatchDialog = true" />
    </div>

    <!-- Batch Script Dialog -->
    <BatchScriptDialog v-model:visible="showBatchDialog" />
  </GenerationAction>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useGenerationStore } from '@/stores/generation';
import { useQueueStore } from '@/stores/queue';
import GenerationAction from './GenerationAction.vue';
import PromptEditor from './PromptEditor.vue';
import SplitButton from 'primevue/splitbutton';
import Button from 'primevue/button';
import BatchScriptDialog from '@/components/generation/batch/BatchScriptDialog.vue';

const emit = defineEmits<{
  generate: [];
}>();

const generationStore = useGenerationStore();
const queueStore = useQueueStore();

// Batch dialog visibility
const showBatchDialog = ref(false);

const prompt = computed({
  get: () => generationStore.currentParams.prompt,
  set: (value: string) => {
    generationStore.currentParams.prompt = value;
  },
});

const canGenerate = computed(() => {
  return generationStore.currentParams.prompt.trim().length > 0;
});

const imageCount = computed({
  get: () => generationStore.currentParams.batchSize,
  set: (value: number) => {
    generationStore.currentParams.batchSize = value;
  },
});

const generationCounts = [
  {
    label: 'Generate 1 Image',
    command: () => {
      imageCount.value = 1;
    },
  },
  {
    label: 'Generate 2 Images',
    command: () => {
      imageCount.value = 2;
    },
  },
  {
    label: 'Generate 3 Images',
    command: () => {
      imageCount.value = 3;
    },
  },
  {
    label: 'Generate 4 Images',
    command: () => {
      imageCount.value = 4;
    },
  },
];

const handleGenerate = () => {
  if (!canGenerate.value) return;
  emit('generate');
};
</script>
