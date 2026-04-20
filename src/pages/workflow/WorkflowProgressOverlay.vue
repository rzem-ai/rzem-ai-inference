<template>
  <div v-if="workflowStore.isRunning" class="flex flex-col gap-1">
    <div class="flex justify-between text-xs text-slate-600">
      <span>{{ statusLabel }}</span>
      <span class="tabular-nums">{{ completedCount }} / {{ totalCount }}</span>
    </div>
    <div class="h-1 overflow-hidden rounded-full bg-slate-200">
      <div class="h-full rounded-full bg-blue-500 transition-[width] duration-300 ease-out" :style="{ width: overallPercent + '%' }" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useWorkflowStore } from '@/stores/workflow';

const workflowStore = useWorkflowStore();

const totalCount = computed(() => workflowStore.nodes.length);

const completedCount = computed(() =>
  Object.values(workflowStore.nodeStates).filter((s) => s.status === 'completed' || s.status === 'failed').length,
);

const overallPercent = computed(() => {
  if (!totalCount.value) return 0;
  return Math.round((completedCount.value / totalCount.value) * 100);
});

const runningNode = computed(() => {
  const runningId = Object.entries(workflowStore.nodeStates).find(([, s]) => s.status === 'running')?.[0];
  if (!runningId) return null;
  return workflowStore.nodes.find((n) => n.id === runningId) ?? null;
});

const statusLabel = computed(() => {
  if (runningNode.value) {
    const label = (runningNode.value.data as { label?: string }).label;
    return label ? `Running: ${label}` : 'Running node...';
  }
  return 'Starting...';
});
</script>
