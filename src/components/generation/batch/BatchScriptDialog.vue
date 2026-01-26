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
      <StepPanel header="Load Data">
        <template #default>
          <div class="flex flex-col gap-4 p-4">
            <FileInputSection @data-loaded="handleDataLoaded" />

            <!-- Mode Selector -->
            <div class="flex flex-col gap-2">
              <h3 class="text-lg font-semibold">Processing Mode</h3>
              <div class="flex gap-4">
                <label class="flex items-center gap-2">
                  <input type="radio" value="as-is" v-model="batchMode" />
                  <span>Use data as-is
                    <span v-if="sourceData">
                      ({{ sourceData.rows.length }} images)
                    </span>
                  </span>
                </label>
                <label class="flex items-center gap-2">
                  <input type="radio" value="combinatorial" v-model="batchMode" />
                  <span>Generate all combinations
                    <span v-if="batchMode === 'combinatorial' && processedData">
                      ({{ processedData.rows.length }} images)
                    </span>
                  </span>
                </label>
              </div>
            </div>

            <!-- Loading Indicator -->
            <div v-if="isProcessing" class="flex items-center gap-2 p-4 bg-surface-ground rounded">
              <ProgressSpinner style="width: 24px; height: 24px" />
              <span>Processing combinations...</span>
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
                @click="nextStep"
                :disabled="!step1Valid" />
            </div>
          </div>
        </template>
      </StepPanel>

      <!-- Step 2: Template -->
      <StepPanel header="Template">
        <template #default>
          <div class="flex flex-col gap-4 p-4">

            <!-- Recent Templates -->
            <div v-if="templateHistory.length > 0" class="flex flex-col gap-2">
              <h3 class="text-lg font-semibold">Recent Templates</h3>
              <div class="flex gap-2 flex-wrap">
                <Button
                  v-for="entry in templateHistory"
                  :key="entry.id"
                  :label="`${entry.template.substring(0, 40)}... (${entry.image_count} images)`"
                  severity="secondary"
                  size="small"
                  @click="loadTemplate(entry.template)"
                  class="max-w-xs" />
              </div>
            </div>

            <!-- Template Editor -->
            <TemplateEditor
              ref="templateEditorRef"
              :available-columns="availableColumns"
              @template-change="handleTemplateChange" />

            <!-- Preview Table -->
            <div v-if="previewRows.length > 0" class="mt-4">
              <PreviewTable :rows="previewRows" :max-display-rows="10" />
            </div>

            <!-- Rendering indicator -->
            <div v-if="isRendering" class="flex items-center gap-2 p-4 bg-surface-ground rounded">
              <ProgressSpinner style="width: 24px; height: 24px" />
              <span>Rendering template...</span>
            </div>

            <!-- Navigation -->
            <div class="flex justify-between mt-4">
              <Button label="Back" icon="pi pi-arrow-left" @click="prevStep" severity="secondary" />
              <Button
                label="Next: Confirm"
                icon="pi pi-arrow-right"
                iconPos="right"
                @click="nextStep"
                :disabled="!step2Valid" />
            </div>
          </div>
        </template>
      </StepPanel>

      <!-- Step 3: Confirm & Submit -->
      <StepPanel header="Confirm & Submit">
        <template #default>
          <div class="flex flex-col gap-6 p-4">

            <!-- Summary Card -->
            <div class="flex flex-col gap-4 p-6 bg-surface-800 rounded-xl border border-surface-700">
              <h3 class="text-xl font-semibold text-primary-400">Batch Generation Summary</h3>

              <div class="grid grid-cols-2 gap-4">
                <!-- Data Source -->
                <div>
                  <p class="text-sm text-gray-400">Data Source</p>
                  <p class="text-lg font-medium">{{ dataSourceName || 'Unknown' }}</p>
                </div>

                <!-- Processing Mode -->
                <div>
                  <p class="text-sm text-gray-400">Processing Mode</p>
                  <p class="text-lg font-medium">
                    {{ batchMode === 'as-is' ? 'As-Is' : 'Combinatorial' }}
                    <span class="text-primary-400">({{ processedData?.rows.length || 0 }} images)</span>
                  </p>
                </div>

                <!-- Template -->
                <div class="col-span-2">
                  <p class="text-sm text-gray-400">Template</p>
                  <p class="text-base font-mono bg-surface-900 p-3 rounded mt-1 overflow-x-auto">
                    {{ templateString }}
                  </p>
                </div>

                <!-- Generation Settings -->
                <div class="col-span-2">
                  <p class="text-sm text-gray-400 mb-2">Generation Settings</p>
                  <div class="grid grid-cols-2 gap-2 text-sm">
                    <div><span class="text-gray-400">Steps:</span> {{ generationStore.currentParams.steps }}</div>
                    <div><span class="text-gray-400">CFG Scale:</span> {{ generationStore.currentParams.cfgScale }}</div>
                    <div><span class="text-gray-400">Size:</span> {{ generationStore.currentParams.width }}×{{ generationStore.currentParams.height }}</div>
                    <div><span class="text-gray-400">Seed:</span> {{ displaySeed }}</div>
                    <div class="col-span-2"><span class="text-gray-400">Model:</span> {{ generationStore.currentParams.model }}</div>
                    <div v-if="activeLorasCount > 0" class="col-span-2">
                      <span class="text-gray-400">LoRAs:</span> {{ activeLorasCount }} active
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Final Preview -->
            <div class="flex flex-col gap-2">
              <h3 class="text-lg font-semibold">Preview ({{ previewRows.length }} images will be generated)</h3>
              <PreviewTable :rows="previewRows" :max-display-rows="5" />
              <p v-if="previewRows.length > 5" class="text-sm text-gray-400">
                Showing 5 of {{ previewRows.length }} prompts
              </p>
            </div>

            <!-- Navigation -->
            <div class="flex justify-between mt-4">
              <Button label="Back" icon="pi pi-arrow-left" @click="prevStep" severity="secondary" :disabled="isGenerating" />
              <Button
                :label="`Generate ${previewRows.length} Images`"
                icon="pi pi-check"
                iconPos="right"
                @click="generateBatch"
                :loading="isGenerating"
                :disabled="!canGenerate" />
            </div>
          </div>
        </template>
      </StepPanel>

    </Stepper>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useToast } from 'primevue/usetoast';
import { useQueueStore } from '@/stores/queue';
import { useGenerationStore } from '@/stores/generation';
import { useModelsStore } from '@/stores/models';
import FileInputSection from './FileInputSection.vue';
import TemplateEditor from './TemplateEditor.vue';
import PreviewTable from './PreviewTable.vue';
import type { BatchData, BatchMode, RenderResult, TemplateHistoryEntry, PreviewRow } from './types';
import Stepper from 'primevue/stepper';
import StepPanel from 'primevue/steppanel';
import Button from 'primevue/button';
import Dialog from 'primevue/dialog';
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';
import ProgressSpinner from 'primevue/progressspinner';

// Props
const props = defineProps<{
  visible: boolean;
}>();

// Emits
const emit = defineEmits<{
  'update:visible': [value: boolean];
}>();

// Initialize stores
const queueStore = useQueueStore();
const generationStore = useGenerationStore();
const modelsStore = useModelsStore();

// Stepper state
const currentStep = ref(0);

// Navigation functions
function nextStep() {
  currentStep.value++;
}

function prevStep() {
  currentStep.value--;
}

// Data state
const sourceData = ref<BatchData | null>(null);
const processedData = ref<BatchData | null>(null);
const batchMode = ref<BatchMode>('as-is');
const isProcessing = ref(false);

// Template state
const templateString = ref('');
const previewData = ref<RenderResult | null>(null);
const isRendering = ref(false);
const isGenerating = ref(false);
const templateHistory = ref<TemplateHistoryEntry[]>([]);

// Toast
const toast = useToast();

// Template editor ref
const templateEditorRef = ref<InstanceType<typeof TemplateEditor> | null>(null);

// Computed validations
const step1Valid = computed(() => {
  return sourceData.value !== null && processedData.value !== null;
});

const availableColumns = computed(() => {
  return processedData.value?.columns || [];
});

const previewRows = computed<PreviewRow[]>(() => {
  if (!previewData.value || !processedData.value) return [];

  const result: PreviewRow[] = [];
  const { rendered, errors } = previewData.value;

  for (let i = 0; i < rendered.length; i++) {
    const errorForRow = errors.find((e) => e.row === i);

    result.push({
      rowNumber: i + 1,
      prompt: rendered[i],
      data: processedData.value.rows[i] || {},
      error: errorForRow?.error,
    });
  }

  return result;
});

const hasErrors = computed(() => {
  return previewData.value?.errors && previewData.value.errors.length > 0;
});

const step2Valid = computed(() => {
  return templateString.value.trim() !== '' &&
         previewData.value !== null &&
         !hasErrors.value;
});

const displaySeed = computed(() => {
  const seed = generationStore.currentParams.seed;
  return seed >= 0 ? seed : 'Random (will freeze for batch)';
});

const activeLorasCount = computed(() => {
  return modelsStore.getActiveLoraConfigs().length;
});

const canGenerate = computed(() => {
  return previewData.value !== null &&
         !hasErrors.value &&
         !isGenerating.value &&
         previewRows.value.length > 0;
});

// Add for data source tracking (Task 12 will improve this)
const dataSourceName = computed(() => {
  return 'batch_data.csv'; // Placeholder - Task 12 will add real filename tracking
});

// Handlers
function handleDataLoaded(data: BatchData) {
  sourceData.value = data;
  // processedData will be set by the watch
}

function handleTemplateChange(template: string) {
  templateString.value = template;
}

function loadTemplate(template: string) {
  templateEditorRef.value?.setTemplate(template);
}

// Load template history
async function loadTemplateHistory() {
  try {
    const history = await invoke<TemplateHistoryEntry[]>('batch_get_recent_templates');
    templateHistory.value = history;
  } catch (error) {
    console.error('Failed to load template history:', error);
    // Non-critical error - continue without history
  }
}

// Render template with current data
async function renderTemplate() {
  if (!processedData.value || !templateString.value.trim()) {
    previewData.value = null;
    return;
  }

  isRendering.value = true;

  try {
    const result = await invoke<RenderResult>('batch_render_template', {
      template: templateString.value,
      rows: processedData.value.rows,
    });

    previewData.value = result;
  } catch (error) {
    toast.add({
      severity: 'error',
      summary: 'Render Error',
      detail: String(error),
      life: 5000,
    });
    console.error('Render error:', error);
  } finally {
    isRendering.value = false;
  }
}

// Generate batch
async function generateBatch() {
  if (!canGenerate.value || !previewData.value) return;

  isGenerating.value = true;

  try {
    const baseParams = generationStore.currentParams;

    // Freeze seed (use current seed if set, otherwise generate random)
    const frozenSeed = baseParams.seed >= 0
      ? baseParams.seed
      : Math.floor(Math.random() * Number.MAX_SAFE_INTEGER);

    let successCount = 0;

    // Queue each prompt
    for (const prompt of previewData.value.rendered) {
      if (!prompt) continue; // Skip empty prompts (errors)

      try {
        await queueStore.addToQueue({
          prompt,
          negative_prompt: baseParams.negativePrompt,
          steps: baseParams.steps,
          cfg_scale: baseParams.cfgScale,
          width: baseParams.width,
          height: baseParams.height,
          seed: frozenSeed, // SAME SEED FOR ALL ROWS
          model: baseParams.model,
          sampler: baseParams.sampler,
          scheduler: baseParams.scheduler,
          loras: modelsStore.getActiveLoraConfigs(),
        });

        successCount++;
      } catch (error) {
        console.error('Failed to queue job:', error);
      }
    }

    // Save template to history
    try {
      await invoke('batch_save_template', {
        template: templateString.value,
        imageCount: successCount,
      });
    } catch (error) {
      console.warn('Failed to save template to history:', error);
      // Non-critical error - don't block success
    }

    toast.add({
      severity: 'success',
      summary: 'Batch Queued',
      detail: `${successCount} images queued with seed ${frozenSeed}`,
      life: 5000,
    });

    // Close dialog
    emit('update:visible', false);
  } catch (error) {
    toast.add({
      severity: 'error',
      summary: 'Batch Generation Failed',
      detail: String(error),
      life: 5000,
    });
    console.error('Batch generation error:', error);
  } finally {
    isGenerating.value = false;
  }
}

// Auto-process data when mode or source changes
watch([batchMode, sourceData], async () => {
  if (!sourceData.value) {
    processedData.value = null;
    return;
  }

  isProcessing.value = true;

  if (batchMode.value === 'as-is') {
    // As-is mode: use source data directly
    processedData.value = sourceData.value;
  } else {
    // Combinatorial mode: generate all combinations
    try {
      const result = await invoke<BatchData>('batch_generate_combinations', {
        data: sourceData.value,
      });
      processedData.value = result;
    } catch (error) {
      toast.add({
        severity: 'error',
        summary: 'Combinatorial Generation Failed',
        detail: String(error),
        life: 5000,
      });
      console.error('Combinatorial error:', error);
      // Fallback to as-is on error
      processedData.value = sourceData.value;
    }
  }

  isProcessing.value = false;
});

// Watch template changes and re-render
watch([templateString, processedData], async () => {
  if (!processedData.value || !templateString.value.trim()) {
    previewData.value = null;
    return;
  }

  await renderTemplate();
});

// Load history when dialog opens
watch(() => props.visible, (newVal) => {
  if (newVal) {
    loadTemplateHistory();
  } else {
    // Reset state
    currentStep.value = 0;
    sourceData.value = null;
    processedData.value = null;
    batchMode.value = 'as-is';
    templateString.value = '';
    previewData.value = null;
    templateHistory.value = [];
    isProcessing.value = false;
    isRendering.value = false;
  }
});
</script>

<style scoped>
/* Minimal styles - most styling via TailwindCSS */
</style>
