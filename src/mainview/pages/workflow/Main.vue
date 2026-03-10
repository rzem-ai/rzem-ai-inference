<template>
  <div class="flex flex-col h-full">
    <!-- Canvas area -->
    <div class="flex-1 min-h-0">
      <WorkflowCanvas />
    </div>

    <!-- Bottom panel: properties + output -->
    <div class="h-80 shrink-0 border-t border-surface-600 bg-surface-800 flex">
      <div class="flex-1 overflow-y-auto border-r border-surface-600">
        <NodePropertiesPanel />
      </div>
      <div class="w-80 shrink-0 overflow-y-auto">
        <OutputDisplay />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import WorkflowCanvas from './canvas/WorkflowCanvas.vue';
import NodePropertiesPanel from './panels/NodePropertiesPanel.vue';
import OutputDisplay from './panels/OutputDisplay.vue';
</script>
