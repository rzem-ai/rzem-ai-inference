<script setup lang="ts">
import { computed } from 'vue';
import type { PreviewRow } from './types';
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';
import Button from 'primevue/button';
import Message from 'primevue/message';

// Props
const props = defineProps<{
  rows: PreviewRow[];
  maxDisplayRows?: number;
}>();

// Compute display rows (limit to maxDisplayRows)
const displayRows = computed(() => {
  const max = props.maxDisplayRows || 100;
  return props.rows.slice(0, max);
});

const hasMoreRows = computed(() => {
  const max = props.maxDisplayRows || 100;
  return props.rows.length > max;
});

const hiddenRowCount = computed(() => {
  const max = props.maxDisplayRows || 100;
  return props.rows.length - max;
});

// Check if any errors exist
const hasErrors = computed(() => {
  return props.rows.some((row) => row.error);
});

const errorCount = computed(() => {
  return props.rows.filter((row) => row.error).length;
});

// Get row class
function getRowClass(row: PreviewRow) {
  return row.error ? 'error-row' : '';
}
</script>

<template>
  <div class="flex flex-col gap-3">
    <div class="flex items-center justify-between">
      <h3 class="m-0 text-lg font-semibold">
        Preview ({{ rows.length }} image{{ rows.length !== 1 ? 's' : '' }} will be generated)
      </h3>
    </div>

    <!-- Error warning -->
    <Message v-if="hasErrors" severity="warn" :closable="false">
      <strong>{{ errorCount }}</strong> row{{ errorCount !== 1 ? 's have' : ' has' }} rendering
      errors. Generation will be blocked until these are fixed.
    </Message>

    <!-- Too many rows warning -->
    <Message v-if="hasMoreRows" severity="info" :closable="false">
      Showing first {{ displayRows.length }} rows. {{ hiddenRowCount }} more rows will be
      generated but are not displayed.
    </Message>

    <!-- DataTable -->
    <DataTable
      :value="displayRows"
      :rowClass="getRowClass"
      scrollable
      scrollHeight="400px"
      stripedRows
      class="preview-datatable"
    >
      <Column field="rowNumber" header="Row" :style="{ width: '80px' }" />

      <Column field="prompt" header="Rendered Prompt" :style="{ minWidth: '300px' }">
        <template #body="{ data }">
          <div v-if="data.error" class="flex items-center gap-2 text-red-500">
            <i class="pi pi-exclamation-triangle text-base shrink-0"></i>
            <span class="text-sm italic">{{ data.error }}</span>
          </div>
          <div v-else class="leading-normal">
            {{ data.prompt }}
          </div>
        </template>
      </Column>

      <Column header="Data" :style="{ width: '100px' }">
        <template #body="{ data }">
          <Button
            icon="pi pi-eye"
            text
            rounded
            size="small"
            v-tooltip.top="{
              value: Object.entries(data.data)
                .map(([k, v]) => `${k}: ${v}`)
                .join('\n'),
              escape: true,
            }"
          />
        </template>
      </Column>
    </DataTable>

    <!-- Empty state -->
    <div v-if="rows.length === 0" class="text-center py-12 px-4 text-gray-400">
      <i class="pi pi-inbox text-5xl block mb-4 opacity-50"></i>
      <p class="m-0 text-base">No preview available. Load data and enter a template.</p>
    </div>
  </div>
</template>

<style scoped>
/* Error row highlighting for PrimeVue DataTable */
.preview-datatable :deep(.error-row) {
  background-color: rgba(239, 68, 68, 0.1) !important;
}
</style>
