<template>
  <div class="h-full overflow-y-auto">
    <div v-if="!selectedNodeData" class="flex items-center justify-center h-full text-surface-400 text-sm">
      Select a node to view its properties
    </div>
    <ImageInputProps v-else-if="selectedNodeData.type === 'image_input'" />
    <VisionQAProps v-else-if="selectedNodeData.type === 'vision_qa'" />
    <ImageGenProps v-else-if="selectedNodeData.type === 'image_gen'" />
    <TextProps v-else-if="selectedNodeData.type === 'text'" />
    <OutputProps v-else-if="selectedNodeData.type === 'output'" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useWorkflowStore } from '@/stores/workflow';
import ImageInputProps from './ImageInputProps.vue';
import VisionQAProps from './VisionQAProps.vue';
import ImageGenProps from './ImageGenProps.vue';
import TextProps from './TextProps.vue';
import OutputProps from './OutputProps.vue';

const workflowStore = useWorkflowStore();

const selectedNodeData = computed(() => workflowStore.selectedNodeData);
</script>
