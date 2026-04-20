<template>
  <div class="w-full h-full">
    <VueFlow
      :nodes="workflowStore.nodes"
      :edges="workflowStore.edges"
      :node-types="nodeTypes"
      :default-viewport="workflowStore.viewport"
      :connection-mode="ConnectionMode.Loose"
      :is-valid-connection="handleValidateConnection"
      :snap-to-grid="true"
      :snap-grid="[16, 16]"
      class="workflow-canvas"
      @nodes-change="workflowStore.onNodesChange"
      @edges-change="workflowStore.onEdgesChange"
      @connect="workflowStore.onConnect"
      @node-click="handleNodeClick"
      @pane-click="handlePaneClick">
      <Background :gap="16" :size="1" pattern-color="rgba(255,255,255,0.05)" />
      <Controls position="bottom-right" />
      <MiniMap
        position="bottom-left"
        :pannable="true"
        :zoomable="true"
        node-color="rgb(100, 116, 139)"
        mask-color="rgba(0, 0, 0, 0.7)" />
    </VueFlow>
  </div>
</template>

<script setup lang="ts">
import { markRaw } from 'vue';
import { VueFlow, ConnectionMode } from '@vue-flow/core';
import type { Connection, NodeMouseEvent } from '@vue-flow/core';
import { Background } from '@vue-flow/background';
import { Controls } from '@vue-flow/controls';
import { MiniMap } from '@vue-flow/minimap';

import '@vue-flow/core/dist/style.css';
import '@vue-flow/core/dist/theme-default.css';
import '@vue-flow/controls/dist/style.css';
import '@vue-flow/minimap/dist/style.css';

import ImageInputNode from '@/pages/workflow/nodes/ImageInputNode.vue';
import VisionQANode from '@/pages/workflow/nodes/VisionQANode.vue';
import ImageGenNode from '@/pages/workflow/nodes/ImageGenNode.vue';
import TextNode from '@/pages/workflow/nodes/TextNode.vue';
import OutputNode from '@/pages/workflow/nodes/OutputNode.vue';

import { NODE_REGISTRY } from '@/pages/workflow/nodeRegistry';
import { useWorkflowStore } from '@/stores/workflow';

const workflowStore = useWorkflowStore();

// Register custom node types — markRaw to prevent Vue from making components reactive
const nodeTypes: Record<string, any> = {
  image_input: markRaw(ImageInputNode),
  vision_qa: markRaw(VisionQANode),
  image_gen: markRaw(ImageGenNode),
  text: markRaw(TextNode),
  output: markRaw(OutputNode),
};

function handleNodeClick(event: NodeMouseEvent) {
  workflowStore.selectNode(event.node.id);
}

function handlePaneClick() {
  workflowStore.selectNode(null);
}

function handleValidateConnection(connection: Connection): boolean {
  // Find the source and target port data types from the registry
  const sourceNode = workflowStore.nodes.find((n) => n.id === connection.source);
  const targetNode = workflowStore.nodes.find((n) => n.id === connection.target);
  if (!sourceNode || !targetNode) return false;

  const sourceInfo = NODE_REGISTRY[sourceNode.type as keyof typeof NODE_REGISTRY];
  const targetInfo = NODE_REGISTRY[targetNode.type as keyof typeof NODE_REGISTRY];
  if (!sourceInfo || !targetInfo) return false;

  const sourcePort = sourceInfo.outputs.find((p) => p.id === connection.sourceHandle);
  const targetPort = targetInfo.inputs.find((p) => p.id === connection.targetHandle);
  if (!sourcePort || !targetPort) return false;

  return sourcePort.dataType === targetPort.dataType;
}
</script>

<style scoped>
.workflow-canvas {
  background-color: rgb(15, 23, 42);
}

/* Dark theme overrides for Vue Flow */
:deep(.vue-flow__edge-path) {
  stroke: rgb(148, 163, 184);
  stroke-width: 2;
}

:deep(.vue-flow__edge.selected .vue-flow__edge-path),
:deep(.vue-flow__edge:hover .vue-flow__edge-path) {
  stroke: rgb(96, 165, 250);
  stroke-width: 2.5;
}

:deep(.vue-flow__connection-path) {
  stroke: rgb(96, 165, 250);
  stroke-width: 2;
  stroke-dasharray: 5 5;
}

:deep(.vue-flow__controls) {
  background: rgb(30, 41, 59);
  border: 1px solid rgb(71, 85, 105);
  border-radius: 8px;
  box-shadow: none;
}

:deep(.vue-flow__controls-button) {
  background: transparent;
  border: none;
  color: rgb(203, 213, 225);
  fill: rgb(203, 213, 225);
}

:deep(.vue-flow__controls-button:hover) {
  background: rgb(51, 65, 85);
}

:deep(.vue-flow__minimap) {
  background: rgb(30, 41, 59);
  border: 1px solid rgb(71, 85, 105);
  border-radius: 8px;
}

:deep(.vue-flow__attribution) {
  display: none;
}
</style>
