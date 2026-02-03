<template>
  <Dialog
    :visible="visible"
    modal
    header="Change Category"
    :style="{ width: '450px' }"
    @update:visible="emit('update:visible', $event)">
    <div class="flex flex-col gap-4">
      <!-- Selection info -->
      <div class="flex items-center gap-2 p-3 rounded bg-surface-800">
        <fa :icon="['fal', 'info-circle']" class="text-primary-400" />
        <span class="text-sm text-surface-200">
          Changing category for <strong>{{ selectedCount }}</strong> style{{ selectedCount !== 1 ? 's' : '' }}
        </span>
      </div>

      <!-- Category selector -->
      <div class="flex flex-col gap-2">
        <label for="category-select" class="text-sm font-semibold text-surface-200">
          Select Category
        </label>
        <Dropdown
          id="category-select"
          v-model="selectedCategory"
          :options="categoryOptions"
          optionLabel="label"
          optionValue="value"
          placeholder="Choose a category"
          showClear
          editable
          class="w-full" />
        <span class="text-xs text-surface-400">
          You can select an existing category or type a new one
        </span>
      </div>
    </div>

    <!-- Footer -->
    <template #footer>
      <div class="flex justify-end gap-2">
        <Button
          label="Cancel"
          severity="secondary"
          variant="outlined"
          @click="emit('update:visible', false)" />
        <Button
          label="Apply"
          severity="primary"
          :disabled="!selectedCategory"
          @click="handleApply" />
      </div>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import Dialog from 'primevue/dialog';
import Dropdown from 'primevue/dropdown';
import Button from 'primevue/button';

const props = defineProps<{
  visible: boolean;
  selectedCount: number;
  existingCategories: string[];
}>();

const emit = defineEmits<{
  'update:visible': [value: boolean];
  apply: [category: string];
}>();

const selectedCategory = ref<string | null>(null);

const categoryOptions = computed(() => {
  return props.existingCategories.map(cat => ({
    label: cat,
    value: cat,
  }));
});

function handleApply() {
  if (selectedCategory.value) {
    emit('apply', selectedCategory.value);
    emit('update:visible', false);
  }
}

// Reset selection when dialog opens
watch(() => props.visible, (isVisible) => {
  if (isVisible) {
    selectedCategory.value = null;
  }
});
</script>
