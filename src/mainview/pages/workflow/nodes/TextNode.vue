<template>
  <BaseNode :id="id" :data="data" :selected="selected">
    <div class="flex flex-col gap-1">
      <div v-if="data.isTemplate" class="flex items-center gap-1">
        <Braces :size="12" class="text-amber-400 shrink-0" />
        <span class="text-amber-400">Template</span>
      </div>
      <div v-if="data.content" class="text-surface-400 line-clamp-2 leading-tight">
        {{ data.content }}
      </div>
      <div v-else class="text-surface-500 italic">Empty</div>
    </div>
  </BaseNode>
</template>

<script setup lang="ts">
import { Braces } from 'lucide-vue-next';
import BaseNode from './BaseNode.vue';
import type { TextNodeData } from '@/types/workflow';

defineProps<{
  id: string;
  data: TextNodeData;
  selected?: boolean;
}>();
</script>
