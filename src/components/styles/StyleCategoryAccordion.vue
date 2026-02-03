<template>
  <div class="flex flex-col border rounded-lg border-surface-700 bg-surface-800/50">
    <!-- Category Header -->
    <div
      class="flex items-center gap-2 p-3 transition-colors cursor-pointer hover:bg-surface-700/50"
      @click="emit('toggle')">
      <!-- Expand/Collapse Icon -->
      <fa
        :icon="['fal', isExpanded ? 'chevron-down' : 'chevron-right']"
        size="sm"
        class="text-surface-400" />

      <!-- Category checkbox for select-all -->
      <Checkbox
        :modelValue="isAllSelected"
        :indeterminate="isSomeSelected && !isAllSelected"
        binary
        @click.stop
        @change="handleSelectAll"
        class="flex-shrink-0" />

      <!-- Category name and count -->
      <div class="flex items-center flex-1 gap-2">
        <fa :icon="['fal', 'folder']" size="sm" class="text-surface-400" />
        <span class="text-sm font-semibold text-surface-100">{{ category }}</span>
        <Tag :value="styles.length" severity="secondary" class="ml-auto text-xs" />
      </div>
    </div>

    <!-- Expandable content -->
    <transition name="expand">
      <div v-if="isExpanded" class="flex flex-col border-t border-surface-700">
        <StyleListItem
          v-for="style in styles"
          :key="style.id"
          :style="style"
          :isSelected="selectedIds.has(style.id)"
          @click="emit('select-style', style.id)"
          @toggle-select="emit('toggle-select', style.id)"
          @edit="emit('edit', style.id)"
          @delete="emit('delete', style.id)"
          @toggle-favorite="emit('toggle-favorite', style.id)"
          class="border-b border-surface-700 last:border-b-0" />
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { StyleInfo } from '@/types';
import StyleListItem from './StyleListItem.vue';
import Checkbox from 'primevue/checkbox';
import Tag from 'primevue/tag';

const props = defineProps<{
  category: string;
  styles: StyleInfo[];
  isExpanded: boolean;
  selectedIds: Set<string>;
}>();

const emit = defineEmits<{
  toggle: [];
  'select-style': [styleId: string];
  'toggle-select': [styleId: string];
  'select-all': [];
  'deselect-all': [];
  edit: [styleId: string];
  delete: [styleId: string];
  'toggle-favorite': [styleId: string];
}>();

const isAllSelected = computed(() => {
  if (props.styles.length === 0) return false;
  return props.styles.every(style => props.selectedIds.has(style.id));
});

const isSomeSelected = computed(() => {
  return props.styles.some(style => props.selectedIds.has(style.id));
});

function handleSelectAll() {
  if (isAllSelected.value) {
    emit('deselect-all');
  } else {
    emit('select-all');
  }
}
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
