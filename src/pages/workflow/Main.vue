<template>
  <div class="flex flex-col h-full">
    <!-- Canvas area -->
    <div class="flex-1 min-h-0">
      <WorkflowCanvas />
    </div>

    <!-- Bottom panel: node properties -->
    <div class="h-80 shrink-0 border-t border-surface-600 bg-surface-800 overflow-y-auto">
      <NodePropertiesPanel />
    </div>
  </div>
</template>

<script setup lang="ts">
import WorkflowCanvas from './canvas/WorkflowCanvas.vue';
import NodePropertiesPanel from './panels/NodePropertiesPanel.vue';
</script>
