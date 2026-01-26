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
  <div class="preview-table">
    <div class="preview-header">
      <h3 class="section-title">
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
          <div v-if="data.error" class="error-cell">
            <i class="pi pi-exclamation-triangle error-icon"></i>
            <span class="error-message">{{ data.error }}</span>
          </div>
          <div v-else class="prompt-cell">
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
    <div v-if="rows.length === 0" class="empty-state">
      <i class="pi pi-inbox empty-state__icon"></i>
      <p class="empty-state__text">No preview available. Load data and enter a template.</p>
    </div>
  </div>
</template>

<style scoped>
.preview-table {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.section-title {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--text-color);
}

.preview-datatable :deep(.error-row) {
  background-color: rgba(239, 68, 68, 0.1) !important;
}

.error-cell {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--red-500);
}

.error-icon {
  font-size: 1rem;
  flex-shrink: 0;
}

.error-message {
  font-size: 0.9rem;
  font-style: italic;
}

.prompt-cell {
  line-height: 1.4;
}

.data-overlay {
  max-width: 400px;
  max-height: 300px;
  overflow-y: auto;
}

.data-row {
  padding: 0.5rem 0;
  border-bottom: 1px solid var(--surface-border);
  font-size: 0.9rem;
}

.data-row:last-child {
  border-bottom: none;
}

.data-row strong {
  color: var(--text-color);
  margin-right: 0.5rem;
}

.empty-state {
  text-align: center;
  padding: 3rem 1rem;
  color: var(--text-color-secondary);
}

.empty-state__icon {
  font-size: 3rem;
  display: block;
  margin-bottom: 1rem;
  opacity: 0.5;
}

.empty-state__text {
  margin: 0;
  font-size: 0.95rem;
}
</style>
