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
import type { BatchData, RenderResult, PreviewRow } from './types';
import Divider from 'primevue/divider';
import ProgressSpinner from 'primevue/progressspinner';
import Button from 'primevue/button';
import Dialog from 'primevue/dialog';

// Props
const props = defineProps<{
  visible: boolean;
}>();

// Emits
const emit = defineEmits<{
  'update:visible': [value: boolean];
}>();

// Stores
const queueStore = useQueueStore();
const generationStore = useGenerationStore();
const modelsStore = useModelsStore();
const toast = useToast();

// State
const fileData = ref<BatchData | null>(null);
const templateString = ref('');
const previewData = ref<RenderResult | null>(null);
const isRendering = ref(false);
const isGenerating = ref(false);

// Computed
const availableColumns = computed(() => {
  return fileData.value?.columns || [];
});

const previewRows = computed<PreviewRow[]>(() => {
  if (!previewData.value || !fileData.value) return [];

  const result: PreviewRow[] = [];
  const { rendered, errors } = previewData.value;

  for (let i = 0; i < rendered.length; i++) {
    const errorForRow = errors.find((e) => e.row === i);

    result.push({
      rowNumber: i + 1,
      prompt: rendered[i],
      data: fileData.value.rows[i] || {},
      error: errorForRow?.error,
    });
  }

  return result;
});

const hasErrors = computed(() => {
  return previewData.value?.errors && previewData.value.errors.length > 0;
});

const canGenerate = computed(() => {
  return (
    fileData.value !== null &&
    templateString.value.trim() !== '' &&
    previewData.value !== null &&
    !hasErrors.value &&
    !isGenerating.value
  );
});

// Watch template changes and re-render
watch([templateString, fileData], async () => {
  if (!fileData.value || !templateString.value.trim()) {
    previewData.value = null;
    return;
  }

  await renderTemplate();
});

// Handle data loaded from file
function handleDataLoaded(data: BatchData) {
  fileData.value = data;
}

// Handle template change
function handleTemplateChange(template: string) {
  templateString.value = template;
}

// Render template with current data
async function renderTemplate() {
  if (!fileData.value || !templateString.value.trim()) return;

  isRendering.value = true;

  try {
    const result = await invoke<RenderResult>('batch_render_template', {
      template: templateString.value,
      rows: fileData.value.rows,
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
    const frozenSeed =
      baseParams.seed >= 0 ? baseParams.seed : Math.floor(Math.random() * Number.MAX_SAFE_INTEGER);

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

    toast.add({
      severity: 'success',
      summary: 'Batch Queued',
      detail: `${successCount} images queued with seed ${frozenSeed}`,
      life: 5000,
    });

    // Close dialog
    emit('update:visible', false);

    // Reset state
    fileData.value = null;
    templateString.value = '';
    previewData.value = null;
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

// Close dialog
function handleClose() {
  emit('update:visible', false);
}
</script>

<template>
  <Dialog
    :visible="visible"
    @update:visible="emit('update:visible', $event)"
    modal
    header="Batch Script Generation"
    :style="{ width: '900px', maxWidth: '95vw' }"
    :dismissableMask="true"
  >
    <div class="batch-dialog-content">
      <!-- File Input Section -->
      <FileInputSection @data-loaded="handleDataLoaded" />

      <Divider />

      <!-- Template Editor -->
      <TemplateEditor
        :available-columns="availableColumns"
        @template-change="handleTemplateChange"
      />

      <Divider />

      <!-- Preview Table -->
      <div class="preview-section">
        <PreviewTable :rows="previewRows" :max-display-rows="100" />

        <!-- Rendering indicator -->
        <div v-if="isRendering" class="rendering-indicator">
          <ProgressSpinner style="width: 24px; height: 24px" />
          <span>Rendering template...</span>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <template #footer>
      <div class="dialog-footer">
        <Button label="Cancel" severity="secondary" @click="handleClose" :disabled="isGenerating" />
        <Button
          :label="`Generate ${previewRows.length} Images`"
          icon="pi pi-arrow-right"
          iconPos="right"
          @click="generateBatch"
          :disabled="!canGenerate"
          :loading="isGenerating"
        />
      </div>
    </template>
  </Dialog>
</template>

<style scoped>
.batch-dialog-content {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  padding: 0.5rem 0;
}

.preview-section {
  position: relative;
}

.rendering-indicator {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem;
  background: var(--surface-ground);
  border-radius: var(--border-radius);
  margin-top: 0.5rem;
  font-size: 0.9rem;
  color: var(--text-color-secondary);
}

.dialog-footer {
  display: flex;
  justify-content: space-between;
  width: 100%;
}
</style>
