<template>
  <div class="flex flex-col gap-3 p-4">
    <div class="text-sm font-semibold text-surface-300">Output</div>

    <!-- Label -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Label</label>
      <InputText :model-value="nodeData.label" @update:model-value="handleUpdate({ label: $event })" fluid />
    </div>

    <!-- Output preview -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Result</label>

      <div v-if="!nodeState" class="text-xs text-surface-500 italic">
        No output yet. Run the workflow to see results.
      </div>

      <template v-else>
        <!-- Status -->
        <div class="flex items-center gap-2">
          <div class="w-2 h-2 rounded-full" :class="statusColor" />
          <span class="text-xs text-surface-400 capitalize">{{ nodeState.status }}</span>
        </div>

        <!-- Error -->
        <div v-if="nodeState.error" class="text-xs text-red-400 bg-red-950/30 rounded-lg p-2 mt-1">
          {{ nodeState.error }}
        </div>

        <!-- Outputs -->
        <div v-if="nodeState.outputs" class="flex flex-col gap-2 mt-1">
          <!-- Image outputs -->
          <div v-if="outputImageUrl" class="relative">
            <img :src="outputImageUrl" alt="Output" class="w-full rounded-lg object-contain max-h-48" />
          </div>

          <!-- Text outputs -->
          <div v-if="outputText" class="text-xs text-surface-300 bg-surface-800 rounded-lg p-2 whitespace-pre-wrap max-h-48 overflow-y-auto">
            {{ outputText }}
          </div>
        </div>
      </template>
    </div>

    <!-- Execution status -->
    <NodeStatusBadge :node-id="workflowStore.selectedNodeId!" />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useWorkflowStore } from '@/stores/workflow';
import { getApiAsync } from '@/bridge';
import type { OutputNodeData } from '@/types/workflow';
import NodeStatusBadge from './NodeStatusBadge.vue';

const workflowStore = useWorkflowStore();

const nodeData = computed(() => workflowStore.selectedNodeData as OutputNodeData);
const nodeState = computed(() => workflowStore.selectedNodeState);
const outputImageUrl = ref<string | null>(null);

const statusColor = computed(() => {
  switch (nodeState.value?.status) {
    case 'completed': return 'bg-green-400';
    case 'running': return 'bg-blue-400';
    case 'pending': return 'bg-yellow-400';
    case 'failed': return 'bg-red-400';
    default: return 'bg-surface-500';
  }
});

const outputText = computed(() => {
  const outputs = nodeState.value?.outputs;
  if (!outputs) return null;
  if (typeof outputs.text === 'string') return outputs.text;
  if (typeof outputs.answer === 'string') return outputs.answer;
  return null;
});

// Load image output if present
watch(
  () => nodeState.value?.outputs,
  async (outputs) => {
    if (!outputs) {
      outputImageUrl.value = null;
      return;
    }
    const imagePath = typeof outputs.image === 'string' ? outputs.image
      : typeof outputs.image_path === 'string' ? outputs.image_path
      : null;
    if (imagePath) {
      const api = await getApiAsync();
      const res = await api.get_image_base64({ image_path: imagePath });
      if (res.status === 'success' && res.data_url) {
        outputImageUrl.value = res.data_url;
      }
    } else {
      outputImageUrl.value = null;
    }
  },
  { immediate: true, deep: true },
);

function handleUpdate(partial: Partial<OutputNodeData>) {
  if (workflowStore.selectedNodeId) {
    workflowStore.updateNodeData(workflowStore.selectedNodeId, partial);
  }
}
</script>
