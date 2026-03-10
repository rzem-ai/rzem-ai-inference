<template>
  <BaseNode :id="id" :data="data" :selected="selected">
    <div class="flex items-center gap-1">
      <Monitor :size="12" class="text-red-400 shrink-0" />
      <span v-if="hasOutputs" class="text-surface-400">Results ready</span>
      <span v-else class="text-surface-500 italic">Awaiting results</span>
    </div>
  </BaseNode>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Monitor } from 'lucide-vue-next';
import BaseNode from './BaseNode.vue';
import type { OutputNodeData } from '@/types/workflow';
import { useWorkflowStore } from '@/stores/workflow';

const props = defineProps<{
  id: string;
  data: OutputNodeData;
  selected?: boolean;
}>();

const workflowStore = useWorkflowStore();

const hasOutputs = computed(() => {
  const state = workflowStore.nodeState(props.id);
  return state.status === 'completed' && state.outputs !== null;
});
</script>
