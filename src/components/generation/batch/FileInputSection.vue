<script setup lang="ts">
import { ref } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { readTextFile } from '@tauri-apps/plugin-fs';
import { invoke } from '@tauri-apps/api/core';
import type { BatchData, DataLoadedEvent } from './types';
import ProgressSpinner from 'primevue/progressspinner';
import Message from 'primevue/message';
import Button from 'primevue/button';
import Textarea from 'primevue/textarea';

// Emits
const emit = defineEmits<{
  dataLoaded: [payload: DataLoadedEvent];
}>();

// State
const fileName = ref<string | null>(null);
const rowCount = ref<number>(0);
const columnCount = ref<number>(0);
const parseError = ref<string | null>(null);
const isLoading = ref(false);
const pasteText = ref('');
const isDragging = ref(false);

// Parse data (CSV or JSON)
async function parseData(content: string, format: string, filename: string) {
  isLoading.value = true;
  parseError.value = null;

  try {
    const data = await invoke<BatchData>('batch_parse_data', { content, format });
    rowCount.value = data.rows.length;
    columnCount.value = data.columns.length;
    emit('dataLoaded', { data, filename });
  } catch (error) {
    parseError.value = String(error);
    console.error('Parse error:', error);
  } finally {
    isLoading.value = false;
  }
}

// File picker
async function handleFilePicker() {
  const selected = await open({
    multiple: false,
    filters: [
      { name: 'Data Files', extensions: ['csv', 'json'] },
      { name: 'CSV', extensions: ['csv'] },
      { name: 'JSON', extensions: ['json'] },
    ],
  });

  if (selected && typeof selected === 'string') {
    await loadFile(selected);
  }
}

// Load file from path
async function loadFile(filePath: string) {
  try {
    const content = await readTextFile(filePath);
    const fileNameFromPath = filePath.split('/').pop() || filePath;
    fileName.value = fileNameFromPath;

    // Detect format by extension
    const format = filePath.endsWith('.json') ? 'json' : 'csv';
    await parseData(content, format, fileNameFromPath);
  } catch (error) {
    parseError.value = `Failed to read file: ${error}`;
    console.error('File read error:', error);
  }
}

// Paste handler
async function handlePaste() {
  if (!pasteText.value.trim()) {
    parseError.value = 'Please paste CSV or JSON data';
    return;
  }

  const pastedDataName = 'Pasted Data';
  fileName.value = pastedDataName;

  // Try to detect format (JSON starts with [ or {)
  const trimmed = pasteText.value.trim();
  const format = trimmed.startsWith('[') || trimmed.startsWith('{') ? 'json' : 'csv';

  await parseData(pasteText.value, format, pastedDataName);
}

// Drag-drop handlers
function handleDragEnter(e: DragEvent) {
  e.preventDefault();
  isDragging.value = true;
}

function handleDragLeave(e: DragEvent) {
  e.preventDefault();
  isDragging.value = false;
}

function handleDragOver(e: DragEvent) {
  e.preventDefault();
}

async function handleDrop(e: DragEvent) {
  e.preventDefault();
  isDragging.value = false;

  const files = e.dataTransfer?.files;
  if (files && files.length > 0) {
    const file = files[0];
    const reader = new FileReader();

    reader.onload = async (event) => {
      const content = event.target?.result as string;
      fileName.value = file.name;

      // Detect format by extension
      const format = file.name.endsWith('.json') ? 'json' : 'csv';
      await parseData(content, format, file.name);
    };

    reader.readAsText(file);
  }
}

// Clear data
function clearData() {
  fileName.value = null;
  rowCount.value = 0;
  columnCount.value = 0;
  parseError.value = null;
  pasteText.value = '';
}
</script>

<template>
  <div class="file-input-section">
    <h3 class="section-title">File Input</h3>

    <!-- File picker button -->
    <div class="file-picker-row">
      <Button
        icon="pi pi-folder-open"
        label="Choose File"
        @click="handleFilePicker"
        :disabled="isLoading"
      />
      <span v-if="fileName" class="file-info">
        {{ fileName }}
      </span>
    </div>

    <!-- Drag-drop zone -->
    <div
      class="drop-zone"
      :class="{ 'drop-zone--dragging': isDragging }"
      @dragenter="handleDragEnter"
      @dragleave="handleDragLeave"
      @dragover="handleDragOver"
      @drop="handleDrop"
    >
      <i class="pi pi-cloud-upload drop-zone__icon"></i>
      <p class="drop-zone__text">Drop CSV or JSON file here</p>
    </div>

    <!-- Paste textarea -->
    <div class="paste-section">
      <label for="paste-data">Or paste CSV/JSON data:</label>
      <Textarea
        id="paste-data"
        v-model="pasteText"
        rows="5"
        placeholder="Paste your CSV or JSON data here..."
        :disabled="isLoading"
      />
      <Button
        label="Parse Pasted Data"
        icon="pi pi-check"
        @click="handlePaste"
        :disabled="isLoading || !pasteText.trim()"
        class="mt-2"
      />
    </div>

    <!-- Loading indicator -->
    <div v-if="isLoading" class="loading-indicator">
      <ProgressSpinner class="w-8 h-8" />
      <span>Parsing data...</span>
    </div>

    <!-- Success message -->
    <Message v-if="rowCount > 0 && !parseError" severity="success" :closable="false">
      ✓ Loaded: {{ rowCount }} rows, {{ columnCount }} columns
      <Button
        icon="pi pi-times"
        text
        rounded
        size="small"
        @click="clearData"
        class="ml-2"
      />
    </Message>

    <!-- Error message -->
    <Message v-if="parseError" severity="error" :closable="true" @close="parseError = null">
      {{ parseError }}
    </Message>
  </div>
</template>

<style scoped>
.file-input-section {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.section-title {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--text-color);
}

.file-picker-row {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.file-info {
  font-size: 0.9rem;
  color: var(--text-color-secondary);
}

.drop-zone {
  border: 2px dashed var(--surface-border);
  border-radius: var(--border-radius);
  padding: 2rem;
  text-align: center;
  background: var(--surface-ground);
  transition: all 0.2s;
  cursor: pointer;
}

.drop-zone--dragging {
  border-color: var(--primary-color);
  background: var(--surface-hover);
}

.drop-zone__icon {
  font-size: 2rem;
  color: var(--text-color-secondary);
  display: block;
  margin-bottom: 0.5rem;
}

.drop-zone__text {
  margin: 0;
  color: var(--text-color-secondary);
  font-size: 0.95rem;
}

.paste-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.paste-section label {
  font-size: 0.9rem;
  font-weight: 500;
  color: var(--text-color);
}

.loading-indicator {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem;
  background: var(--surface-ground);
  border-radius: var(--border-radius);
}
</style>
