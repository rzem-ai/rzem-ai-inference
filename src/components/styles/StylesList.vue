<template>
  <StyleListItem v-for="style in styles" :key="style.id" :style="style" class="border-b border-surface-700 last:border-b-0" />
</template>

<script setup lang="ts">
import type { StyleInfo } from '@/types';
import StyleListItem from './StyleListItem.vue';

defineProps<{
  category: string;
  styles: StyleInfo[];
}>();
</script>

<style scoped>
.expand-enter-active,
.expand-leave-active {
  transition: all 0.3s ease;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  max-height: 0;
  opacity: 0;
}

.expand-enter-to,
.expand-leave-from {
  max-height: 2000px;
  opacity: 1;
}
</style>
