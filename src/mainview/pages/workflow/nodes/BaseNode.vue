<template>
  <div
    class="rounded-lg overflow-hidden shadow-lg min-w-40 border transition-colors"
    :class="selected ? 'border-blue-400 shadow-blue-400/20' : 'border-surface-600'">
    <!-- Header -->
    <div class="flex items-center gap-2 px-3 py-2 text-white text-xs font-medium" :class="nodeInfo.color">
      <component :is="iconComponent" :size="14" />
      <span class="flex-1 truncate">{{ data.label }}</span>
      <span v-if="executionState.status !== 'idle'" class="flex items-center gap-1">
        <Loader2 v-if="executionState.status === 'running'" :size="12" class="animate-spin" />
        <CheckCircle2 v-else-if="executionState.status === 'completed'" :size="12" class="text-green-200" />
        <XCircle v-else-if="executionState.status === 'failed'" :size="12" class="text-red-200" />
        <Clock v-else-if="executionState.status === 'pending'" :size="12" class="text-white/60" />
      </span>
    </div>

    <!-- Content slot -->
    <div class="bg-surface-800 px-3 py-2 text-xs text-surface-300">
      <slot />
    </div>

    <!-- Input handles (left side) -->
    <Handle
      v-for="port in nodeInfo.inputs"
      :key="'in-' + port.id"
      type="target"
      :id="port.id"
      :position="Position.Left"
      :style="getHandleStyle(port, 'input')"
      :class="getHandleClass(port)" />

    <!-- Output handles (right side) -->
    <Handle
      v-for="port in nodeInfo.outputs"
      :key="'out-' + port.id"
      type="source"
      :id="port.id"
      :position="Position.Right"
      :style="getHandleStyle(port, 'output')"
      :class="getHandleClass(port)" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Handle, Position } from '@vue-flow/core';
import {
  ImagePlus,
  Eye,
  Sparkles,
  Type,
  Monitor,
  Loader2,
  CheckCircle2,
  XCircle,
  Clock,
} from 'lucide-vue-next';
import { NODE_REGISTRY } from '@/pages/workflow/nodeRegistry';
import type { PortDefinition } from '@/pages/workflow/nodeRegistry';
import type { WorkflowNodeData } from '@/types/workflow';
import { useWorkflowStore } from '@/stores/workflow';

const props = defineProps<{
  id: string;
  data: WorkflowNodeData;
  selected?: boolean;
}>();

const workflowStore = useWorkflowStore();

const nodeInfo = computed(() => NODE_REGISTRY[props.data.type]);

const iconMap: Record<string, typeof ImagePlus> = {
  ImagePlus,
  Eye,
  Sparkles,
  Type,
  Monitor,
};

const iconComponent = computed(() => iconMap[nodeInfo.value.icon] ?? ImagePlus);

const executionState = computed(() => workflowStore.nodeState(props.id));

function getHandleStyle(port: PortDefinition, direction: 'input' | 'output') {
  const ports = direction === 'input' ? nodeInfo.value.inputs : nodeInfo.value.outputs;
  const index = ports.findIndex((p) => p.id === port.id);
  const total = ports.length;
  // Distribute handles vertically across the node
  const headerHeight = 32;
  const contentHeight = 40;
  const totalHeight = headerHeight + contentHeight;
  const spacing = totalHeight / (total + 1);
  const top = spacing * (index + 1);

  return { top: `${top}px` };
}

function getHandleClass(port: PortDefinition): string {
  return port.dataType === 'image'
    ? 'workflow-handle workflow-handle-image'
    : 'workflow-handle workflow-handle-text';
}
</script>

<style scoped>
:deep(.workflow-handle) {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  border: 2px solid;
}

:deep(.workflow-handle-image) {
  background: rgb(59, 130, 246);
  border-color: rgb(96, 165, 250);
}

:deep(.workflow-handle-text) {
  background: rgb(217, 119, 6);
  border-color: rgb(251, 191, 36);
}
</style>
