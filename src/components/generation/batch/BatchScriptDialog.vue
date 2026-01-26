<template>
  <Dialog
    :visible="visible"
    @update:visible="emit('update:visible', $event)"
    modal
    header="Batch Script Generation"
    :style="{ width: '900px', maxWidth: '95vw' }"
    :dismissableMask="true">

    <!-- Stepper Component -->
    <Stepper v-model:activeStep="currentStep" linear>

      <!-- Step 1: Load Data -->
      <StepperPanel header="Load Data">
        <template #content="{ nextCallback }">
          <div class="flex flex-col gap-4 p-4">
            <FileInputSection @data-loaded="handleDataLoaded" />

            <!-- Mode Selector (placeholder for Task 7) -->
            <div class="flex flex-col gap-2">
              <h3 class="text-lg font-semibold">Processing Mode</h3>
              <div class="flex gap-4">
                <label class="flex items-center gap-2">
                  <input type="radio" value="as-is" v-model="batchMode" />
                  <span>Use data as-is ({{ sourceData?.rows.length || 0 }} images)</span>
                </label>
                <label class="flex items-center gap-2">
                  <input type="radio" value="combinatorial" v-model="batchMode" />
                  <span>Generate all combinations</span>
                </label>
              </div>
            </div>

            <!-- Preview table -->
            <div v-if="processedData" class="mt-4">
              <h3 class="text-lg font-semibold mb-2">Data Preview</h3>
              <DataTable :value="processedData.rows.slice(0, 10)" scrollable scrollHeight="200px">
                <Column v-for="col in processedData.columns" :key="col" :field="col" :header="col" />
              </DataTable>
              <p v-if="processedData.rows.length > 10" class="text-sm text-gray-400 mt-2">
                Showing 10 of {{ processedData.rows.length }} rows
              </p>
            </div>

            <!-- Navigation -->
            <div class="flex justify-end mt-4">
              <Button
                label="Next: Template"
                icon="pi pi-arrow-right"
                iconPos="right"
                @click="nextCallback"
                :disabled="!step1Valid" />
            </div>
          </div>
        </template>
      </StepperPanel>

      <!-- Step 2: Template (placeholder) -->
      <StepperPanel header="Template">
        <template #content="{ prevCallback, nextCallback }">
          <div class="flex flex-col gap-4 p-4">
            <p class="text-gray-400">Step 2: Template editor (Task 8)</p>

            <div class="flex justify-between mt-4">
              <Button label="Back" icon="pi pi-arrow-left" @click="prevCallback" severity="secondary" />
              <Button label="Next: Confirm" icon="pi pi-arrow-right" iconPos="right" @click="nextCallback" />
            </div>
          </div>
        </template>
      </StepperPanel>

      <!-- Step 3: Confirm (placeholder) -->
      <StepperPanel header="Confirm & Submit">
        <template #content="{ prevCallback }">
          <div class="flex flex-col gap-4 p-4">
            <p class="text-gray-400">Step 3: Summary and confirmation (Task 9)</p>

            <div class="flex justify-between mt-4">
              <Button label="Back" icon="pi pi-arrow-left" @click="prevCallback" severity="secondary" />
              <Button label="Generate" icon="pi pi-check" iconPos="right" />
            </div>
          </div>
        </template>
      </StepperPanel>

    </Stepper>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useToast } from 'primevue/usetoast';
import FileInputSection from './FileInputSection.vue';
import type { BatchData, BatchMode } from './types';
import Stepper from 'primevue/stepper';
import StepperPanel from 'primevue/stepperpanel';
import Button from 'primevue/button';
import Dialog from 'primevue/dialog';
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';

// Props
const props = defineProps<{
  visible: boolean;
}>();

// Emits
const emit = defineEmits<{
  'update:visible': [value: boolean];
}>();

// Stepper state
const currentStep = ref(0);

// Data state
const sourceData = ref<BatchData | null>(null);
const processedData = ref<BatchData | null>(null);
const batchMode = ref<BatchMode>('as-is');

// Toast
const toast = useToast();

// Computed validations
const step1Valid = computed(() => {
  return sourceData.value !== null && processedData.value !== null;
});

// Handlers
function handleDataLoaded(data: BatchData) {
  sourceData.value = data;
  // For now, just copy as-is (Task 7 will add combinatorial processing)
  processedData.value = data;
}

// Reset on dialog close
watch(() => props.visible, (newVal) => {
  if (!newVal) {
    currentStep.value = 0;
    sourceData.value = null;
    processedData.value = null;
    batchMode.value = 'as-is';
  }
});
</script>

<style scoped>
/* Minimal styles - most styling via TailwindCSS */
</style>
