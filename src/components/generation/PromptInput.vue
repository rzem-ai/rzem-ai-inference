<template>
  <Panel :collapsed="props.collapsed" :toggleable="props.toggleable">
    <template #header>
      <div class="flex gap-2 px-0 py-2 text-xs font-semibold tracking-wider text-gray-300 uppercase">
        <component :is="props.icon" class="w-4 h-4" />
        {{ props.label }}
      </div>
    </template>

    <div class="flex flex-col w-full gap-2 px-2">
      <PromptEditor v-model="prompt" label="Prompt" placeholder="Describe the image you want to generate..." :rows="4" />
      <Button :loading="queueStore.hasRunningJobs" fluid size="small" @click="handleGenerate" :disabled="!canGenerate">
        {{ queueStore.queueLength > 0 ? `Generate` : 'Generate' }}
      </Button>
    </div>
  </Panel>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useGenerationStore } from '@/stores/generation';
import { useQueueStore } from '@/stores/queue';
import Panel from 'primevue/panel';
import PromptEditor from './PromptEditor.vue';
import Button from 'primevue/button';

const props = defineProps(['collapsed', 'icon', 'label', 'toggleable']);

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
