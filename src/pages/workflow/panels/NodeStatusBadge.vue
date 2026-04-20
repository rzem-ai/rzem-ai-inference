<template>
  <div v-if="nodeState" class="flex flex-col gap-1 pt-2 border-t border-surface-700">
    <div class="flex items-center gap-2">
      <div class="w-2 h-2 rounded-full" :class="statusColor" />
      <span class="text-xs text-surface-400 capitalize">{{ nodeState.status }}</span>
      <span v-if="nodeState.status === 'running' && nodeState.progress > 0" class="text-xs text-surface-500">
        {{ Math.round(nodeState.progress * 100) }}%
      </span>
    </div>
    <div v-if="nodeState.error" class="text-xs text-red-400 bg-red-950/30 rounded-lg p-2">
      {{ nodeState.error }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useWorkflowStore } from '@/stores/workflow';

const props = defineProps<{
  nodeId: string;
}>();

const workflowStore = useWorkflowStore();

const nodeState = computed(() => workflowStore.getNodeState(props.nodeId));

const statusColor = computed(() => {
  switch (nodeState.value?.status) {
    case 'completed': return 'bg-green-400';
    case 'running': return 'bg-blue-400';
    case 'pending': return 'bg-yellow-400';
    case 'failed': return 'bg-red-400';
    default: return 'bg-surface-500';
  }
});
</script>
