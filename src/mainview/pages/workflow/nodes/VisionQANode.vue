<template>
  <BaseNode :id="id" :data="data" :selected="selected">
    <div class="flex flex-col gap-1">
      <div class="flex items-center gap-1">
        <span
          class="px-1 rounded text-white font-medium"
          :class="data.provider === 'chat_service' ? 'bg-purple-700' : 'bg-emerald-700'"
          style="font-size: 10px">
          {{ providerLabel }}
        </span>
      </div>
      <div v-if="data.question" class="text-surface-400 truncate leading-tight">
        {{ data.question }}
      </div>
      <div v-else class="text-surface-500 italic">No question set</div>
    </div>
  </BaseNode>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import BaseNode from './BaseNode.vue';
import type { VisionQANodeData } from '@/types/workflow';

const props = defineProps<{
  id: string;
  data: VisionQANodeData;
  selected?: boolean;
}>();

const providerLabel = computed(() => {
  return props.data.provider === 'chat_service' ? 'Claude' : 'OpenAI';
});
</script>
