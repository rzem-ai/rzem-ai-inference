<template>
  <div class="flex flex-col gap-3 p-4">
    <div class="text-sm font-semibold text-surface-300">Text</div>

    <!-- Label -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Label</label>
      <InputText :model-value="nodeData.label" @update:model-value="handleUpdate({ label: $event })" fluid />
    </div>

    <!-- Template toggle -->
    <div class="flex items-center gap-2">
      <ToggleSwitch
        :model-value="nodeData.isTemplate"
        @update:model-value="handleUpdate({ isTemplate: $event })" />
      <label class="text-sm text-surface-400">Template mode</label>
    </div>

    <!-- Template hint -->
    <div v-if="nodeData.isTemplate" class="text-xs text-surface-500 bg-surface-800 rounded-lg p-2">
      Use <code class="text-primary-400">{input}</code> as a placeholder for the connected input value.
    </div>

    <!-- Content -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Content</label>
      <Textarea
        :model-value="nodeData.content"
        @update:model-value="handleUpdate({ content: $event })"
        rows="6"
        auto-resize
        fluid
        :placeholder="nodeData.isTemplate ? 'Describe this image: {input}' : 'Enter text content...'" />
    </div>

    <!-- Execution status -->
    <NodeStatusBadge :node-id="workflowStore.selectedNodeId!" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useWorkflowStore } from '@/stores/workflow';
import type { TextNodeData } from '@/types/workflow';
import NodeStatusBadge from './NodeStatusBadge.vue';

const workflowStore = useWorkflowStore();

const nodeData = computed(() => workflowStore.selectedNodeData as TextNodeData);

function handleUpdate(partial: Partial<TextNodeData>) {
  if (workflowStore.selectedNodeId) {
    workflowStore.updateNodeData(workflowStore.selectedNodeId, partial);
  }
}
</script>
