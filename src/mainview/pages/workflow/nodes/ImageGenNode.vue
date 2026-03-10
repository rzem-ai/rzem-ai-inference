<template>
  <BaseNode :id="id" :data="data" :selected="selected">
    <div class="flex flex-col gap-1">
      <div v-if="data.bundleId" class="flex items-center gap-1">
        <Sparkles :size="12" class="text-green-400 shrink-0" />
        <span class="truncate">{{ data.bundleId }}</span>
      </div>
      <div v-else class="text-surface-500 italic">No model selected</div>
      <div class="text-surface-500">
        {{ data.width }}x{{ data.height }}
      </div>
    </div>
  </BaseNode>
</template>

<script setup lang="ts">
import { Sparkles } from 'lucide-vue-next';
import BaseNode from './BaseNode.vue';
import type { ImageGenNodeData } from '@/types/workflow';

defineProps<{
  id: string;
  data: ImageGenNodeData;
  selected?: boolean;
}>();
</script>
