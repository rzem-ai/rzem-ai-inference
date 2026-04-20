<template>
  <div class="flex flex-col gap-3 p-4">
    <div class="text-sm font-semibold text-surface-300">Vision Q&A</div>

    <!-- Label -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Label</label>
      <InputText :model-value="nodeData.label" @update:model-value="handleUpdate({ label: $event })" fluid />
    </div>

    <!-- Provider -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Provider</label>
      <SelectButton
        :model-value="nodeData.provider"
        :options="providerOptions"
        option-label="label"
        option-value="value"
        @update:model-value="handleUpdate({ provider: $event })" />
    </div>

    <!-- Question -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Question</label>
      <Textarea
        :model-value="nodeData.question"
        @update:model-value="handleUpdate({ question: $event })"
        rows="3"
        auto-resize
        fluid
        placeholder="What do you see in this image?" />
    </div>

    <!-- OpenAI-compatible fields -->
    <template v-if="nodeData.provider === 'openai_compatible'">
      <div class="flex flex-col gap-1">
        <label class="text-sm text-surface-400">Endpoint URL</label>
        <InputText
          :model-value="nodeData.endpointUrl"
          @update:model-value="handleUpdate({ endpointUrl: $event })"
          placeholder="http://localhost:1234"
          fluid />
      </div>

      <div class="flex flex-col gap-1">
        <label class="text-sm text-surface-400">Model Name</label>
        <InputText
          :model-value="nodeData.modelName"
          @update:model-value="handleUpdate({ modelName: $event })"
          placeholder="qwen3.5-9b"
          fluid />
      </div>

      <div class="flex flex-col gap-1">
        <label class="text-sm text-surface-400">API Key <span class="text-surface-500">(optional)</span></label>
        <InputText
          :model-value="nodeData.apiKey"
          @update:model-value="handleUpdate({ apiKey: $event })"
          type="password"
          placeholder="sk-..."
          fluid />
      </div>
    </template>

    <!-- Max Tokens -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Max Tokens</label>
      <InputNumber
        :model-value="nodeData.maxTokens"
        @update:model-value="handleUpdate({ maxTokens: $event ?? 512 })"
        :min="1"
        :max="16384"
        fluid />
    </div>

    <!-- Execution status -->
    <NodeStatusBadge :node-id="workflowStore.selectedNodeId!" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useWorkflowStore } from '@/stores/workflow';
import type { VisionQANodeData } from '@/types/workflow';
import NodeStatusBadge from './NodeStatusBadge.vue';

const workflowStore = useWorkflowStore();

const nodeData = computed(() => workflowStore.selectedNodeData as VisionQANodeData);

const providerOptions = [
  { label: 'Chat Service', value: 'chat_service' },
  { label: 'OpenAI Compatible', value: 'openai_compatible' },
];

function handleUpdate(partial: Partial<VisionQANodeData>) {
  if (workflowStore.selectedNodeId) {
    workflowStore.updateNodeData(workflowStore.selectedNodeId, partial);
  }
}
</script>
