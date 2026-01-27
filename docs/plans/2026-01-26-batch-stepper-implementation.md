# Batch Dialog 3-Step Stepper Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refactor batch generation dialog to a guided 3-step stepper with combinatorial data processing and template history.

**Architecture:** PrimeVue Stepper with 3 panels, SQLite template history, Rust combinatorial algorithm using Cartesian product, reactive Vue state management with navigation guards.

**Tech Stack:** Vue 3 Composition API, PrimeVue Stepper, SQLite, Rust (rusqlite), Tauri IPC

---

## Task 1: Database Schema & Migration

**Files:**
- Modify: `src-tauri/src/gallery/mod.rs:50-80` (initialize_db function)

**Step 1: Add template history table to database initialization**

In `src-tauri/src/gallery/mod.rs`, add the new table creation after existing tables:

```rust
// After existing table creation (around line 70)

// Batch template history table
conn.execute(
    "CREATE TABLE IF NOT EXISTS batch_template_history (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        template TEXT NOT NULL,
        used_at TEXT NOT NULL,
        image_count INTEGER NOT NULL,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP
    )",
    [],
)?;

conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_batch_template_history_used_at
     ON batch_template_history(used_at DESC)",
    [],
)?;
```

**Step 2: Test database initialization**

Run: `cd src-tauri && cargo build`
Expected: Build succeeds without errors

**Step 3: Verify table creation**

Run the app, then check the database:
```bash
sqlite3 ~/.local/share/com.rzem.ai-inference/gallery.db ".schema batch_template_history"
```
Expected: Shows the table schema

**Step 4: Commit database schema**

```bash
git add src-tauri/src/gallery/mod.rs
git commit -m "feat(batch): add template history database table"
```

---

## Task 2: TypeScript Types for New Features

**Files:**
- Modify: `src/components/generation/batch/types.ts:1-50`

**Step 1: Add new TypeScript types**

Add to `types.ts`:

```typescript
export type BatchMode = 'as-is' | 'combinatorial';

export interface BatchConfig {
  mode: BatchMode;
  sourceData: BatchData;
  processedData: BatchData;
  template: string;
}

export interface TemplateHistoryEntry {
  id: number;
  template: string;
  used_at: string;  // ISO 8601 timestamp
  image_count: number;
}
```

**Step 2: Verify types compile**

Run: `npm run type-check`
Expected: No TypeScript errors

**Step 3: Commit TypeScript types**

```bash
git add src/components/generation/batch/types.ts
git commit -m "feat(batch): add types for stepper and template history"
```

---

## Task 3: Rust Combinatorial Algorithm

**Files:**
- Create: `src-tauri/src/batch/combinatorial.rs`
- Modify: `src-tauri/src/batch/mod.rs:1-10` (add module declaration)

**Step 1: Create combinatorial module file**

Create `src-tauri/src/batch/combinatorial.rs`:

```rust
use super::types::BatchData;
use anyhow::Result;
use std::collections::{HashMap, HashSet};

/// Generate all combinations (Cartesian product) from batch data
///
/// Takes rows with potentially duplicate values per column and generates
/// all unique combinations. Example: 2 unique styles × 3 unique subjects = 6 combinations
pub fn generate_combinations(data: BatchData) -> Result<BatchData> {
    if data.rows.is_empty() {
        return Ok(data);
    }

    // Extract unique values per column
    let mut column_values: HashMap<String, Vec<String>> = HashMap::new();

    for col in &data.columns {
        let mut unique_values = HashSet::new();
        for row in &data.rows {
            if let Some(value) = row.get(col) {
                unique_values.insert(value.clone());
            }
        }
        let mut values: Vec<String> = unique_values.into_iter().collect();
        values.sort(); // Deterministic order
        column_values.insert(col.clone(), values);
    }

    // Generate Cartesian product
    let combinations = cartesian_product(&data.columns, &column_values);

    Ok(BatchData {
        columns: data.columns,
        rows: combinations,
    })
}

/// Recursive Cartesian product implementation
fn cartesian_product(
    columns: &[String],
    values: &HashMap<String, Vec<String>>
) -> Vec<HashMap<String, String>> {
    if columns.is_empty() {
        return vec![HashMap::new()];
    }

    let first_col = &columns[0];
    let first_values = values.get(first_col).unwrap();
    let rest_combinations = cartesian_product(&columns[1..], values);

    let mut result = Vec::new();
    for val in first_values {
        for rest in &rest_combinations {
            let mut combo = rest.clone();
            combo.insert(first_col.clone(), val.clone());
            result.push(combo);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_combinations() {
        let data = BatchData {
            columns: vec!["style".to_string(), "subject".to_string()],
            rows: vec![
                [("style", "watercolor"), ("subject", "cat")].iter()
                    .map(|(k, v)| (k.to_string(), v.to_string())).collect(),
                [("style", "oil"), ("subject", "dog")].iter()
                    .map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            ],
        };

        let result = generate_combinations(data).unwrap();
        assert_eq!(result.rows.len(), 4); // 2 × 2 = 4

        // Verify all combinations exist
        let has_watercolor_cat = result.rows.iter().any(|r|
            r.get("style") == Some(&"watercolor".to_string()) &&
            r.get("subject") == Some(&"cat".to_string())
        );
        assert!(has_watercolor_cat);
    }

    #[test]
    fn test_three_columns() {
        let data = BatchData {
            columns: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            rows: vec![
                [("a", "1"), ("b", "x"), ("c", "m")].iter()
                    .map(|(k, v)| (k.to_string(), v.to_string())).collect(),
                [("a", "2"), ("b", "y"), ("c", "n")].iter()
                    .map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            ],
        };

        let result = generate_combinations(data).unwrap();
        assert_eq!(result.rows.len(), 8); // 2 × 2 × 2 = 8
    }

    #[test]
    fn test_empty_data() {
        let data = BatchData {
            columns: vec!["col".to_string()],
            rows: vec![],
        };

        let result = generate_combinations(data).unwrap();
        assert_eq!(result.rows.len(), 0);
    }
}
```

**Step 2: Add module declaration**

In `src-tauri/src/batch/mod.rs`, add at the top:

```rust
mod combinatorial;
```

**Step 3: Run tests**

Run: `cd src-tauri && cargo test batch::combinatorial`
Expected: All 3 tests pass

**Step 4: Build to verify compilation**

Run: `cd src-tauri && cargo build`
Expected: Build succeeds

**Step 5: Commit combinatorial algorithm**

```bash
git add src-tauri/src/batch/combinatorial.rs src-tauri/src/batch/mod.rs
git commit -m "feat(batch): implement combinatorial Cartesian product algorithm"
```

---

## Task 4: Rust Template History Types & Commands

**Files:**
- Modify: `src-tauri/src/batch/types.rs:1-50`
- Modify: `src-tauri/src/batch/mod.rs:50-150`

**Step 1: Add TemplateHistoryEntry type**

In `src-tauri/src/batch/types.rs`, add:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateHistoryEntry {
    pub id: i64,
    pub template: String,
    pub used_at: String,
    pub image_count: i64,
}
```

**Step 2: Add batch_generate_combinations command**

In `src-tauri/src/batch/mod.rs`, add:

```rust
#[tauri::command]
pub fn batch_generate_combinations(data: BatchData) -> Result<BatchData, String> {
    combinatorial::generate_combinations(data)
        .map_err(|e| format!("Failed to generate combinations: {}", e))
}
```

**Step 3: Add batch_get_recent_templates command**

In `src-tauri/src/batch/mod.rs`, add:

```rust
use tauri::State;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[tauri::command]
pub fn batch_get_recent_templates(
    db: State<'_, Arc<Mutex<Connection>>>
) -> Result<Vec<TemplateHistoryEntry>, String> {
    let conn = db.lock().map_err(|e| format!("Database lock error: {}", e))?;

    let mut stmt = conn.prepare(
        "SELECT id, template, used_at, image_count
         FROM batch_template_history
         ORDER BY used_at DESC
         LIMIT 5"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let entries = stmt.query_map([], |row| {
        Ok(TemplateHistoryEntry {
            id: row.get(0)?,
            template: row.get(1)?,
            used_at: row.get(2)?,
            image_count: row.get(3)?,
        })
    }).map_err(|e| format!("Query failed: {}", e))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("Failed to collect results: {}", e))?;

    Ok(entries)
}
```

**Step 4: Add batch_save_template command**

In `src-tauri/src/batch/mod.rs`, add:

```rust
use rusqlite::params;

#[tauri::command]
pub fn batch_save_template(
    db: State<'_, Arc<Mutex<Connection>>>,
    template: String,
    image_count: i64
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("Database lock error: {}", e))?;

    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO batch_template_history (template, used_at, image_count)
         VALUES (?1, ?2, ?3)",
        params![template, now, image_count]
    ).map_err(|e| format!("Failed to save template: {}", e))?;

    Ok(())
}
```

**Step 5: Add chrono dependency**

In `src-tauri/Cargo.toml`, add to dependencies:

```toml
chrono = { version = "0.4", features = ["serde"] }
```

**Step 6: Build to verify**

Run: `cd src-tauri && cargo build`
Expected: Build succeeds

**Step 7: Commit backend commands**

```bash
git add src-tauri/src/batch/types.rs src-tauri/src/batch/mod.rs src-tauri/Cargo.toml
git commit -m "feat(batch): add template history and combinations commands"
```

---

## Task 5: Register New Tauri Commands

**Files:**
- Modify: `src-tauri/src/lib.rs:80-120` (invoke_handler section)

**Step 1: Add new commands to invoke_handler**

In `src-tauri/src/lib.rs`, add to the `invoke_handler!` macro:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...

    // Batch commands
    batch::batch_parse_data,
    batch::batch_render_template,
    batch::batch_generate_combinations,     // NEW
    batch::batch_get_recent_templates,      // NEW
    batch::batch_save_template,             // NEW
])
```

**Step 2: Build to verify**

Run: `cd src-tauri && cargo build`
Expected: Build succeeds

**Step 3: Commit command registration**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(batch): register new Tauri commands"
```

---

## Task 6: Refactor BatchScriptDialog to Stepper - Step 1 Structure

**Files:**
- Modify: `src/components/generation/batch/BatchScriptDialog.vue:1-270`

**Step 1: Replace Dialog template with Stepper**

Replace the entire `<template>` section in `BatchScriptDialog.vue`:

```vue
<template>
  <Dialog
    :visible="visible"
    @update:visible="emit('update:visible', $event)"
    modal
    header="Batch Generation"
    :style="{ width: '1000px', maxWidth: '95vw' }"
    :dismissableMask="true">

    <Stepper v-model:activeStep="activeStep" linear>
      <!-- Step 1: Load Data -->
      <StepperPanel header="Load Data">
        <template #content="{ nextCallback }">
          <div class="flex flex-col gap-4">
            <FileInputSection @data-loaded="handleDataLoaded" />

            <!-- Mode selector (shown after data loaded) -->
            <div v-if="sourceData" class="flex flex-col gap-3 p-4 bg-surface-700 rounded-lg">
              <label class="font-semibold text-base">Processing Mode:</label>

              <div class="flex flex-col gap-3">
                <div class="flex items-center gap-2">
                  <RadioButton v-model="batchMode" inputId="mode-asis" value="as-is" />
                  <label for="mode-asis" class="cursor-pointer">
                    Use data as-is ({{ sourceData.rows.length }} images)
                  </label>
                </div>

                <div class="flex items-center gap-2">
                  <RadioButton v-model="batchMode" inputId="mode-combo" value="combinatorial" />
                  <label for="mode-combo" class="cursor-pointer">
                    Generate all combinations ({{ combinationCount }} images)
                  </label>
                </div>

                <div v-if="batchMode === 'combinatorial'" class="text-sm text-gray-400 ml-7">
                  {{ combinationBreakdown }}
                </div>
              </div>
            </div>

            <!-- Data preview table -->
            <div v-if="previewDataRows.length > 0" class="mt-2">
              <DataTable
                :value="previewDataRows"
                scrollable
                scrollHeight="300px"
                :pt="{ root: { class: 'text-sm' } }">
                <Column
                  v-for="col in displayColumns"
                  :key="col"
                  :field="col"
                  :header="col"
                  :style="{ minWidth: '150px' }" />
              </DataTable>
            </div>
          </div>
        </template>

        <template #footer>
          <div class="flex justify-between w-full pt-4">
            <Button
              label="Cancel"
              severity="secondary"
              @click="handleClose"
              :disabled="isGenerating" />
            <Button
              label="Next: Template"
              icon="pi pi-arrow-right"
              iconPos="right"
              @click="nextStep"
              :disabled="!sourceData" />
          </div>
        </template>
      </StepperPanel>

      <!-- Step 2 and 3 placeholders (will implement in next tasks) -->
      <StepperPanel header="Template">
        <template #content>
          <div>Step 2 - Coming soon</div>
        </template>
      </StepperPanel>

      <StepperPanel header="Confirm">
        <template #content>
          <div>Step 3 - Coming soon</div>
        </template>
      </StepperPanel>
    </Stepper>
  </Dialog>
</template>
```

**Step 2: Update script imports**

At the top of `<script setup>`, add new imports:

```typescript
import Stepper from 'primevue/stepper';
import StepperPanel from 'primevue/stepperpanel';
import RadioButton from 'primevue/radiobutton';
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';
```

**Step 3: Add new state variables**

In the script section, add after existing refs:

```typescript
// Stepper state
const activeStep = ref(0);
const batchMode = ref<BatchMode>('as-is');
const processedData = ref<BatchData | null>(null);
const dataSourceName = ref('');

// Rename fileData to sourceData for clarity
const sourceData = ref<BatchData | null>(null);
```

**Step 4: Update handleDataLoaded**

```typescript
function handleDataLoaded(data: BatchData) {
  sourceData.value = data;
  dataSourceName.value = 'Loaded data'; // Will be improved later
  batchMode.value = 'as-is';
}
```

**Step 5: Add combination count computed properties**

```typescript
const combinationCount = computed(() => {
  if (!sourceData.value || batchMode.value !== 'combinatorial') {
    return sourceData.value?.rows.length || 0;
  }

  // Calculate product of unique values per column
  const counts = sourceData.value.columns.map(col => {
    const uniqueValues = new Set(sourceData.value!.rows.map(r => r[col]));
    return uniqueValues.size;
  });

  return counts.reduce((a, b) => a * b, 1);
});

const combinationBreakdown = computed(() => {
  if (!sourceData.value || batchMode.value !== 'combinatorial') return '';

  const counts = sourceData.value.columns.map(col => {
    const uniqueValues = new Set(sourceData.value!.rows.map(r => r[col]));
    return `${col}:${uniqueValues.size}`;
  });

  return counts.join(' × ');
});
```

**Step 6: Add preview data computed properties**

```typescript
const displayColumns = computed(() => {
  return sourceData.value?.columns || [];
});

const previewDataRows = computed(() => {
  if (!sourceData.value) return [];

  // Show first 10 rows for preview
  const dataToShow = processedData.value || sourceData.value;
  return dataToShow.rows.slice(0, 10);
});
```

**Step 7: Add navigation functions**

```typescript
function nextStep() {
  if (activeStep.value === 0 && !sourceData.value) {
    toast.add({
      severity: 'warn',
      summary: 'No Data',
      detail: 'Please load data before proceeding',
      life: 3000
    });
    return;
  }

  activeStep.value++;
}
```

**Step 8: Test Step 1 UI**

Run: `npm run dev`
- Open app, click "Batch Script"
- Load a CSV file
- Verify mode selector appears
- Verify data preview table shows
- Verify Next button works
- Verify combinatorial count calculates correctly

**Step 9: Commit Step 1 refactor**

```bash
git add src/components/generation/batch/BatchScriptDialog.vue
git commit -m "feat(batch): refactor to stepper structure with Step 1"
```

---

## Task 7: Add Auto-Processing for Combinatorial Mode

**Files:**
- Modify: `src/components/generation/batch/BatchScriptDialog.vue:80-120` (script section)

**Step 1: Add watch for mode changes**

In the script section, add a watcher:

```typescript
// Watch mode and source data to regenerate processed data
watch([batchMode, sourceData], async () => {
  if (!sourceData.value) {
    processedData.value = null;
    return;
  }

  try {
    if (batchMode.value === 'combinatorial') {
      // Call backend to generate combinations
      const result = await invoke<BatchData>('batch_generate_combinations', {
        data: sourceData.value
      });
      processedData.value = result;
    } else {
      // Use data as-is
      processedData.value = sourceData.value;
    }
  } catch (error) {
    toast.add({
      severity: 'error',
      summary: 'Processing Error',
      detail: String(error),
      life: 5000
    });
    console.error('Processing error:', error);
  }
}, { deep: true });
```

**Step 2: Test combinatorial processing**

Run: `npm run dev`
- Load CSV with duplicate values (e.g., style appears twice)
- Switch to combinatorial mode
- Verify preview updates to show combinations
- Verify count matches expected product

**Step 3: Commit auto-processing**

```bash
git add src/components/generation/batch/BatchScriptDialog.vue
git commit -m "feat(batch): add auto-processing for combinatorial mode"
```

---

## Task 8: Implement Step 2 - Template with History

**Files:**
- Modify: `src/components/generation/batch/BatchScriptDialog.vue:1-300`

**Step 1: Replace Step 2 placeholder**

Replace the Step 2 StepperPanel content:

```vue
<StepperPanel header="Template">
  <template #content>
    <div class="flex flex-col gap-4">
      <!-- Recent templates -->
      <div v-if="recentTemplates.length > 0" class="flex flex-col gap-2">
        <label class="font-semibold text-sm">Recent Templates:</label>
        <div class="flex flex-col gap-2">
          <Card
            v-for="entry in recentTemplates"
            :key="entry.id"
            class="cursor-pointer hover:bg-surface-700 transition-colors"
            @click="loadTemplate(entry.template)"
            :pt="{
              root: { class: 'p-3' },
              body: { class: 'p-0' },
              content: { class: 'p-0' }
            }">
            <template #content>
              <div class="text-sm font-mono">{{ truncateTemplate(entry.template) }}</div>
              <div class="text-xs text-gray-400 mt-1">
                {{ formatDate(entry.used_at) }} • {{ entry.image_count }} images
              </div>
            </template>
          </Card>
        </div>
      </div>

      <Divider v-if="recentTemplates.length > 0" />

      <!-- Template editor -->
      <TemplateEditor
        :available-columns="availableColumns"
        @template-change="handleTemplateChange"
        ref="templateEditorRef" />

      <!-- Preview -->
      <div v-if="templateString && processedData" class="mt-2">
        <PreviewTable :rows="previewRows" :max-display-rows="50" />

        <!-- Rendering indicator -->
        <div v-if="isRendering" class="flex items-center gap-2 mt-2 text-sm text-gray-400">
          <ProgressSpinner style="width: 20px; height: 20px" />
          <span>Rendering template...</span>
        </div>
      </div>
    </div>
  </template>

  <template #footer>
    <div class="flex justify-between w-full pt-4">
      <Button
        label="Back"
        icon="pi pi-arrow-left"
        @click="prevStep" />
      <div class="flex gap-2">
        <Button
          label="Cancel"
          severity="secondary"
          @click="handleClose"
          :disabled="isGenerating" />
        <Button
          label="Next: Review"
          icon="pi pi-arrow-right"
          iconPos="right"
          @click="nextStep"
          :disabled="!canProceedToReview" />
      </div>
    </div>
  </template>
</StepperPanel>
```

**Step 2: Add Card import**

```typescript
import Card from 'primevue/card';
```

**Step 3: Add template history state**

```typescript
const recentTemplates = ref<TemplateHistoryEntry[]>([]);
const templateEditorRef = ref();
```

**Step 4: Add template history functions**

```typescript
async function loadRecentTemplates() {
  try {
    recentTemplates.value = await invoke<TemplateHistoryEntry[]>(
      'batch_get_recent_templates'
    );
  } catch (error) {
    console.warn('Failed to load template history:', error);
    recentTemplates.value = [];
  }
}

function loadTemplate(template: string) {
  templateString.value = template;
  // Template editor should update via v-model or we need to force it
  if (templateEditorRef.value) {
    templateEditorRef.value.setTemplate(template);
  }
}

function truncateTemplate(template: string, maxLength = 80): string {
  if (template.length <= maxLength) return template;
  return template.substring(0, maxLength) + '...';
}

function formatDate(isoString: string): string {
  const date = new Date(isoString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins} min ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toLocaleDateString();
}
```

**Step 5: Add canProceedToReview computed**

```typescript
const canProceedToReview = computed(() => {
  return templateString.value.trim() !== '' &&
         previewData.value !== null &&
         !hasErrors.value &&
         processedData.value !== null;
});
```

**Step 6: Add prevStep function**

```typescript
function prevStep() {
  if (activeStep.value > 0) {
    activeStep.value--;
  }
}
```

**Step 7: Load templates when dialog opens**

```typescript
watch(() => props.visible, async (isVisible) => {
  if (isVisible) {
    await loadRecentTemplates();
    activeStep.value = 0;
  }
});
```

**Step 8: Update TemplateEditor to support external loading**

In `src/components/generation/batch/TemplateEditor.vue`, add a method:

```typescript
// Expose method to parent
defineExpose({
  setTemplate: (newTemplate: string) => {
    template.value = newTemplate;
  }
});
```

**Step 9: Test Step 2**

Run: `npm run dev`
- Complete Step 1 with data
- Click Next to Step 2
- Verify recent templates show (if any)
- Click a recent template, verify it loads
- Enter a new template
- Verify preview updates
- Verify Back button works

**Step 10: Commit Step 2**

```bash
git add src/components/generation/batch/BatchScriptDialog.vue src/components/generation/batch/TemplateEditor.vue
git commit -m "feat(batch): implement Step 2 with template history"
```

---

## Task 9: Implement Step 3 - Confirm & Submit

**Files:**
- Modify: `src/components/generation/batch/BatchScriptDialog.vue:200-300`

**Step 1: Replace Step 3 placeholder**

Replace the Step 3 StepperPanel content:

```vue
<StepperPanel header="Confirm">
  <template #content>
    <div class="flex flex-col gap-4">
      <!-- Summary card -->
      <Card class="bg-surface-700">
        <template #content>
          <div class="flex flex-col gap-3">
            <div class="flex items-start gap-2">
              <i class="pi pi-file text-primary-400 mt-1"></i>
              <div>
                <div class="text-xs text-gray-400">Data Source</div>
                <div class="text-sm font-medium">{{ dataSourceName }}</div>
              </div>
            </div>

            <div class="flex items-start gap-2">
              <i class="pi pi-cog text-primary-400 mt-1"></i>
              <div>
                <div class="text-xs text-gray-400">Processing Mode</div>
                <div class="text-sm font-medium">
                  {{ batchMode === 'as-is' ? 'As-is' : 'Combinatorial' }}
                  ({{ finalImageCount }} images)
                </div>
              </div>
            </div>

            <div class="flex items-start gap-2">
              <i class="pi pi-file-edit text-primary-400 mt-1"></i>
              <div>
                <div class="text-xs text-gray-400">Template</div>
                <div class="text-sm font-mono">{{ truncateTemplate(templateString, 100) }}</div>
              </div>
            </div>

            <Divider class="my-2" />

            <div>
              <div class="text-sm font-semibold mb-2">Generation Settings:</div>
              <ul class="text-sm space-y-1 list-disc pl-5">
                <li>Steps: {{ generationParams.steps }}</li>
                <li>CFG Scale: {{ generationParams.cfgScale }}</li>
                <li>Size: {{ generationParams.width }}×{{ generationParams.height }}</li>
                <li>Seed: {{ frozenSeed }} <span class="text-gray-400">(same for all)</span></li>
                <li>Model: {{ generationParams.model }}</li>
                <li v-if="activeLoRAs.length > 0">
                  LoRAs: {{ activeLoRAs.map(l => `${l.name} (${l.strength})`).join(', ') }}
                </li>
              </ul>
            </div>
          </div>
        </template>
      </Card>

      <!-- Final preview -->
      <div>
        <h3 class="text-sm font-semibold mb-2">Final Preview:</h3>
        <PreviewTable :rows="previewRows" :max-display-rows="100" />
      </div>
    </div>
  </template>

  <template #footer>
    <div class="flex justify-between w-full pt-4">
      <Button
        label="Back"
        icon="pi pi-arrow-left"
        @click="prevStep" />
      <div class="flex gap-2">
        <Button
          label="Cancel"
          severity="secondary"
          @click="handleClose"
          :disabled="isGenerating" />
        <Button
          :label="`Generate ${finalImageCount} Images`"
          icon="pi pi-bolt"
          iconPos="right"
          @click="generateBatch"
          :loading="isGenerating"
          severity="success" />
      </div>
    </div>
  </template>
</StepperPanel>
```

**Step 2: Add computed properties for Step 3**

```typescript
const generationParams = computed(() => generationStore.currentParams);

const frozenSeed = computed(() => {
  const params = generationParams.value;
  return params.seed >= 0
    ? params.seed
    : Math.floor(Math.random() * Number.MAX_SAFE_INTEGER);
});

const activeLoRAs = computed(() => {
  return modelsStore.getActiveLoraConfigs().map(config => ({
    name: config.id,
    strength: config.strength
  }));
});

const finalImageCount = computed(() => {
  return previewData.value?.rendered.length || 0;
});
```

**Step 3: Update generateBatch to save template history**

```typescript
async function generateBatch() {
  if (!canGenerate.value || !previewData.value || !processedData.value) return;

  isGenerating.value = true;

  try {
    const baseParams = generationStore.currentParams;
    const seed = frozenSeed.value;

    let successCount = 0;

    // Queue each prompt
    for (const prompt of previewData.value.rendered) {
      if (!prompt) continue;

      try {
        await queueStore.addToQueue({
          prompt,
          negative_prompt: baseParams.negativePrompt,
          steps: baseParams.steps,
          cfg_scale: baseParams.cfgScale,
          width: baseParams.width,
          height: baseParams.height,
          seed: seed,
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
        imageCount: successCount
      });
    } catch (error) {
      console.warn('Failed to save template history:', error);
    }

    toast.add({
      severity: 'success',
      summary: 'Batch Queued',
      detail: `${successCount} images queued with seed ${seed}`,
      life: 5000,
    });

    // Close and reset
    emit('update:visible', false);
    resetState();

  } catch (error) {
    toast.add({
      severity: 'error',
      summary: 'Batch Generation Failed',
      detail: String(error),
      life: 5000,
    });
  } finally {
    isGenerating.value = false;
  }
}
```

**Step 4: Add resetState function**

```typescript
function resetState() {
  activeStep.value = 0;
  sourceData.value = null;
  batchMode.value = 'as-is';
  processedData.value = null;
  templateString.value = '';
  previewData.value = null;
  dataSourceName.value = '';
  isRendering.value = false;
  isGenerating.value = false;
  recentTemplates.value = [];
}
```

**Step 5: Update handleClose**

```typescript
function handleClose() {
  if (isGenerating.value) return;

  emit('update:visible', false);
  setTimeout(resetState, 300);
}
```

**Step 6: Test Step 3 and full flow**

Run: `npm run dev`
- Load data → Choose mode → Click Next
- Enter template → Click Next
- Verify summary shows all correct info
- Verify generation settings display
- Click Generate
- Verify jobs queue
- Verify dialog closes
- Reopen dialog, verify template in history

**Step 7: Commit Step 3**

```bash
git add src/components/generation/batch/BatchScriptDialog.vue
git commit -m "feat(batch): implement Step 3 with summary and generation"
```

---

## Task 10: Add Auto-Render on Template/Data Changes

**Files:**
- Modify: `src/components/generation/batch/BatchScriptDialog.vue:150-180`

**Step 1: Add watch for template and processed data**

```typescript
// Auto-render when template or processed data changes
watch([templateString, processedData], async () => {
  if (!processedData.value || !templateString.value.trim()) {
    previewData.value = null;
    return;
  }

  await renderTemplate();
});
```

**Step 2: Update renderTemplate to use processedData**

```typescript
async function renderTemplate() {
  if (!processedData.value || !templateString.value.trim()) return;

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
```

**Step 3: Test auto-rendering**

Run: `npm run dev`
- Load data
- Switch between as-is and combinatorial modes
- Enter template in Step 2
- Verify preview updates automatically
- Go back to Step 1, change mode
- Return to Step 2, verify preview updated

**Step 4: Commit auto-render**

```bash
git add src/components/generation/batch/BatchScriptDialog.vue
git commit -m "feat(batch): add auto-render on template/data changes"
```

---

## Task 11: Add Navigation Guards & Validation

**Files:**
- Modify: `src/components/generation/batch/BatchScriptDialog.vue:200-250`

**Step 1: Enhance nextStep validation**

```typescript
function nextStep() {
  // Step 1 → Step 2: require data
  if (activeStep.value === 0) {
    if (!sourceData.value) {
      toast.add({
        severity: 'warn',
        summary: 'No Data Loaded',
        detail: 'Please load a CSV or JSON file',
        life: 3000
      });
      return;
    }

    // Warn about large batches
    if (batchMode.value === 'combinatorial' && combinationCount.value > 1000) {
      toast.add({
        severity: 'info',
        summary: 'Large Batch',
        detail: `This will generate ${combinationCount.value} images`,
        life: 5000
      });
    }
  }

  // Step 2 → Step 3: require valid template
  if (activeStep.value === 1) {
    if (!templateString.value.trim()) {
      toast.add({
        severity: 'warn',
        summary: 'No Template',
        detail: 'Please enter a template',
        life: 3000
      });
      return;
    }

    if (hasErrors.value) {
      toast.add({
        severity: 'error',
        summary: 'Template Errors',
        detail: 'Fix template errors before proceeding',
        life: 3000
      });
      return;
    }

    if (!processedData.value) {
      toast.add({
        severity: 'error',
        summary: 'Processing Error',
        detail: 'Data processing failed',
        life: 3000
      });
      return;
    }
  }

  activeStep.value++;
}
```

**Step 2: Test validation**

Run: `npm run dev`
- Try clicking Next without data → Should show warning
- Load data with 100+ combinations → Should show info toast
- Go to Step 2 without template → Try Next → Should warn
- Add template with errors → Try Next → Should block
- Fix template → Next should work

**Step 3: Commit navigation guards**

```bash
git add src/components/generation/batch/BatchScriptDialog.vue
git commit -m "feat(batch): add navigation guards and validation"
```

---

## Task 12: Improve Data Source Name Tracking

**Files:**
- Modify: `src/components/generation/batch/FileInputSection.vue:60-80`
- Modify: `src/components/generation/batch/BatchScriptDialog.vue:130-140`

**Step 1: Update FileInputSection to emit filename**

In `FileInputSection.vue`, modify the emit to include filename:

```typescript
// Update the emit event to include filename
const emit = defineEmits<{
  dataLoaded: [data: BatchData, filename: string];
}>();

// In handleFilePicker
async function handleFilePicker() {
  const selected = await open({
    multiple: false,
    filters: [
      { name: 'Data Files', extensions: ['csv', 'json'] },
    ],
  });

  if (selected && typeof selected === 'string') {
    await loadFile(selected);
  }
}

async function loadFile(filePath: string) {
  try {
    const content = await readTextFile(filePath);
    const filename = filePath.split('/').pop() || filePath;
    fileName.value = filename;

    const format = filePath.endsWith('.json') ? 'json' : 'csv';
    const data = await invoke<BatchData>('batch_parse_data', { content, format });

    rowCount.value = data.rows.length;
    columnCount.value = data.columns.length;

    emit('dataLoaded', data, filename);
  } catch (error) {
    parseError.value = `Failed to read file: ${error}`;
  }
}

// In handlePaste
async function handlePaste() {
  if (!pasteText.value.trim()) {
    parseError.value = 'Please paste CSV or JSON data';
    return;
  }

  fileName.value = 'Pasted data';
  const trimmed = pasteText.value.trim();
  const format = trimmed.startsWith('[') || trimmed.startsWith('{') ? 'json' : 'csv';

  try {
    const data = await invoke<BatchData>('batch_parse_data', {
      content: pasteText.value,
      format
    });

    rowCount.value = data.rows.length;
    columnCount.value = data.columns.length;

    emit('dataLoaded', data, 'Pasted data');
  } catch (error) {
    parseError.value = String(error);
  }
}

// In handleDrop (similar update)
```

**Step 2: Update BatchScriptDialog to receive filename**

```typescript
function handleDataLoaded(data: BatchData, filename: string) {
  sourceData.value = data;
  dataSourceName.value = filename;
  batchMode.value = 'as-is';
}
```

**Step 3: Test filename tracking**

Run: `npm run dev`
- Load CSV file, go to Step 3, verify filename shows
- Paste data, go to Step 3, verify "Pasted data" shows
- Drag-drop file, verify filename shows

**Step 4: Commit filename tracking**

```bash
git add src/components/generation/batch/FileInputSection.vue src/components/generation/batch/BatchScriptDialog.vue
git commit -m "feat(batch): track and display data source filename"
```

---

## Task 13: Manual Testing & Bug Fixes

**Step 1: Test complete flow**

Run through all test cases from design doc:
- [ ] Load CSV, JSON (array), JSON (object)
- [ ] As-is mode works
- [ ] Combinatorial mode calculates correctly
- [ ] Template history loads and works
- [ ] Navigation works (forward/back)
- [ ] Validation blocks invalid states
- [ ] Generation queues correctly
- [ ] Template saves to history
- [ ] Dialog resets properly

**Step 2: Fix any bugs found**

Document bugs and fix them one at a time with commits.

**Step 3: Test edge cases**

- Empty files
- Very large combinations (>1000)
- Templates with errors
- Cancel and reopen

**Step 4: Commit bug fixes**

```bash
git add <fixed-files>
git commit -m "fix(batch): <description of fix>"
```

---

## Task 14: Update Documentation & Examples

**Files:**
- Modify: `examples/batch-scripting/README.md:1-100`

**Step 1: Update README with new stepper flow**

Add section about the 3-step process and combinatorial mode:

```markdown
## Using the Batch Generator

### Step 1: Load Data

1. Click "Batch Script" button
2. Load your CSV or JSON file
3. Choose processing mode:
   - **As-is**: Each row = one image
   - **Combinatorial**: Generate all combinations

#### Combinatorial Example

Input CSV:
```csv
style,subject
watercolor,cat
oil,dog
```

Combinatorial mode generates:
- watercolor + cat
- watercolor + dog
- oil + cat
- oil + dog

Total: 2 × 2 = 4 images

### Step 2: Template

1. (Optional) Click a recent template to load it
2. Enter your template using `{{ variable }}` syntax
3. Preview updates automatically

### Step 3: Confirm & Generate

1. Review summary and settings
2. Click "Generate N Images"
3. All jobs queued with same seed
```

**Step 2: Add example for combinatorial**

Create `examples/batch-scripting/example-combinatorial.csv`:

```csv
style,subject,mood
watercolor,cat,peaceful
oil painting,dog,dramatic
digital art,bird,energetic
```

**Step 3: Commit documentation**

```bash
git add examples/batch-scripting/
git commit -m "docs(batch): update examples for stepper and combinatorial"
```

---

## Task 15: Final Cleanup & Styling Polish

**Files:**
- Modify: `src/components/generation/batch/BatchScriptDialog.vue:250-270` (styles)

**Step 1: Review and improve Tailwind usage**

Replace any remaining manual CSS with Tailwind classes. Check for:
- Custom padding/margins → Tailwind spacing
- Custom colors → Tailwind color utilities
- Custom flex layouts → Tailwind flex classes

**Step 2: Remove unused imports and code**

Remove any old single-page dialog code that's no longer needed.

**Step 3: Add loading states polish**

Ensure all loading indicators are visible and clear.

**Step 4: Test in different screen sizes**

- Large desktop (1920px)
- Laptop (1366px)
- Tablet (768px)

**Step 5: Commit final polish**

```bash
git add src/components/generation/batch/
git commit -m "style(batch): polish stepper UI and cleanup"
```

---

## Verification Checklist

After completing all tasks, verify:

- [ ] All Rust tests pass: `cd src-tauri && cargo test`
- [ ] App builds: `npm run build`
- [ ] Database table exists and works
- [ ] Combinatorial algorithm generates correct count
- [ ] Template history persists across restarts
- [ ] All 3 steps work correctly
- [ ] Navigation guards prevent invalid states
- [ ] Generation uses same seed for all
- [ ] Template saves after generation
- [ ] No console errors during normal use
- [ ] UI is responsive and polished

---

## Success Criteria

✅ 3-step stepper guides user through flow
✅ Combinatorial mode generates Cartesian product
✅ Template history saves and loads
✅ Navigation validation prevents errors
✅ All generation params inherited correctly
✅ Same seed used for comparison
✅ Clean, polished UI with good UX
✅ No regressions in existing batch features
