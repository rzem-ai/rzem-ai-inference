<template>
  <div class="generate-button-container">
    <div class="button-wrapper">
      <Button :label="buttonLabel" @click="handleClick" :disabled="!canGenerate" severity="primary" size="large" class="w-full" />
      <span v-if="props.queueCount > 0" class="queue-badge">
        {{ props.queueCount }}
      </span>
    </div>

    <div v-if="props.queueCount > 0" class="queue-info"> {{ props.queueCount }} job{{ props.queueCount !== 1 ? 's' : '' }} in queue </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useGenerationStore } from '@/stores/generation';
import { useQueueStore } from '@/stores/queue';
import Button from 'primevue/button';

interface Props {
  queueCount?: number;
}

// Add default value for queueCount
const props = withDefaults(defineProps<Props>(), {
  queueCount: 0,
});

const emit = defineEmits<{
  generate: [];
}>();

const store = useGenerationStore();
const queueStore = useQueueStore();

const canGenerate = computed(() => {
  return store.currentParams.prompt.trim().length > 0;
});

const buttonLabel = computed(() => {
  if (queueStore.hasRunningJobs) {
    return 'Generating...';
  }
  return 'Generate';
});

const handleClick = () => {
  if (!canGenerate.value) return;
  emit('generate');
};
</script>

<style scoped>
@reference "tailwindcss";

.generate-button-container {
  @apply flex flex-col gap-2;
}

.button-wrapper {
  @apply relative flex;
}

.queue-badge {
  @apply absolute -top-2 -right-2 rounded-full w-6 h-6;
  @apply flex items-center justify-center text-xs font-semibold shadow z-10;
  background-color: #ef4444;
  color: white;
}

.queue-info {
  @apply text-center text-sm;
  color: var(--color-slate-500);
}
</style>
