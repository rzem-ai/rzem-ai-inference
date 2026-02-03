<template>
  <div class="flex items-center justify-between px-4 py-2 border rounded-lg border-primary-600 bg-primary-900/20">
    <!-- Selection count -->
    <div class="flex items-center gap-2 text-sm text-surface-100">
      <fa :icon="['fal', 'check-square']" size="sm" class="text-primary-400" />
      <span class="font-semibold">{{ selectedCount }}</span>
      <span class="text-surface-400">selected</span>
    </div>

    <!-- Actions -->
    <div class="flex items-center gap-2">
      <Button
        label="Clear"
        @click="emit('clear-selection')"
        severity="secondary"
        variant="outlined"
        size="small" />

      <SplitButton
        label="Bulk Actions"
        :model="bulkActionsMenu"
        severity="primary"
        size="small"
        @click="handleMainAction" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import Button from 'primevue/button';
import SplitButton from 'primevue/splitbutton';
import type { MenuItem } from 'primevue/menuitem';

defineProps<{
  selectedCount: number;
}>();

const emit = defineEmits<{
  'bulk-delete': [];
  'bulk-favorite': [];
  'bulk-unfavorite': [];
  'bulk-categorize': [];
  'clear-selection': [];
}>();

const bulkActionsMenu = computed<MenuItem[]>(() => [
  {
    label: 'Add to Favorites',
    icon: 'fa fa-star',
    command: () => emit('bulk-favorite'),
  },
  {
    label: 'Remove from Favorites',
    icon: 'fa fa-star-o',
    command: () => emit('bulk-unfavorite'),
  },
  {
    separator: true,
  },
  {
    label: 'Change Category',
    icon: 'fa fa-folder',
    command: () => emit('bulk-categorize'),
  },
  {
    separator: true,
  },
  {
    label: 'Delete Selected',
    icon: 'fa fa-trash',
    class: 'text-red-500',
    command: () => emit('bulk-delete'),
  },
]);

function handleMainAction() {
  // Main button action - default to categorize
  emit('bulk-categorize');
}
</script>
