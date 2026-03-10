<template>
  <BaseNode :id="id" :data="data" :selected="selected">
    <div v-if="data.imagePath" class="flex items-center gap-2">
      <ImageIcon :size="12" class="text-blue-400 shrink-0" />
      <span class="truncate">{{ fileName }}</span>
    </div>
    <div v-else class="text-surface-500 italic">No image selected</div>
  </BaseNode>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { ImageIcon } from 'lucide-vue-next';
import BaseNode from './BaseNode.vue';
import type { ImageInputNodeData } from '@/types/workflow';

const props = defineProps<{
  id: string;
  data: ImageInputNodeData;
  selected?: boolean;
}>();

const fileName = computed(() => {
  if (!props.data.imagePath) return '';
  const parts = props.data.imagePath.split('/');
  return parts[parts.length - 1];
});
</script>
