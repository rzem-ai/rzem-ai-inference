# Batch Dialog 3-Step Stepper - Design Document

**Date:** 2026-01-26
**Status:** Approved for Implementation

## Overview

Refactor the batch generation dialog from a single-page form to a guided 3-step stepper with combinatorial data processing and template history.

## Goals

1. Improve UX with guided step-by-step flow
2. Add combinatorial mode for generating all permutations of data
3. Provide template history for reusing previous templates
4. Maintain same-seed generation strategy for comparison

## User Flow

### Step 1: Load Data
- User loads CSV/JSON file (existing FileInputSection)
- Choose processing mode:
  - **As-is**: Use rows directly (N rows = N images)
  - **Combinatorial**: Generate Cartesian product (col1_values × col2_values × ... = N images)
- Preview data table shows processed rows
- **Validation**: Must load data to proceed
- **Action**: Click "Next: Template"

### Step 2: Template Editor
- Display 5 most recent templates at top (clickable cards)
- Template editor with variable insertion (existing TemplateEditor)
- Live preview of rendered prompts (existing PreviewTable)
- **Validation**: Template must be non-empty and error-free to proceed
- **Actions**: "Back" or "Next: Review"

### Step 3: Confirm & Submit
- Summary card showing:
  - Data source filename
  - Processing mode and image count
  - Template text
  - Generation settings (steps, CFG, size, seed, model, LoRAs)
- Final preview table (read-only)
- **Action**: "Generate N Images" → Queue all jobs → Save template to history

## Navigation Rules

- **Forward**: Only enabled when current step is valid
- **Backward**: Always enabled (preserves state)
- **Step indicators**: Show complete/active/disabled states
- **Cancel**: Available at all steps (with confirmation if generating)

## Data Structures

### TypeScript

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
  used_at: string;  // ISO 8601
  image_count: number;
}
```

### Database Schema

```sql
CREATE TABLE IF NOT EXISTS batch_template_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    template TEXT NOT NULL,
    used_at TEXT NOT NULL,
    image_count INTEGER NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_batch_template_history_used_at
ON batch_template_history(used_at DESC);
```

### Rust Types

```rust
pub enum BatchMode {
    AsIs,
    Combinatorial,
}

pub struct TemplateHistoryEntry {
    pub id: i64,
    pub template: String,
    pub used_at: String,
    pub image_count: i64,
}
```

## Backend Logic

### Combinatorial Generation Algorithm

**Input**: BatchData with rows
```
columns: ["style", "subject"]
rows: [
  {"style": "watercolor", "subject": "cat"},
  {"style": "oil", "subject": "dog"}
]
```

**Process**:
1. Extract unique values per column: `{style: [watercolor, oil], subject: [cat, dog]}`
2. Compute Cartesian product: 2 × 2 = 4 combinations
3. Generate all combinations:
   ```
   {"style": "watercolor", "subject": "cat"}
   {"style": "watercolor", "subject": "dog"}
   {"style": "oil", "subject": "cat"}
   {"style": "oil", "subject": "dog"}
   ```

**Output**: BatchData with 4 rows

### Template History

**On batch generation complete:**
- Save template text + timestamp + image count to database
- Keep most recent 100 entries (auto-cleanup on startup)

**On Step 2 load:**
- Fetch 5 most recent templates
- Display as clickable cards with metadata

## Component Architecture

### BatchScriptDialog.vue (Refactored)

**Main orchestrator with stepper:**
- PrimeVue Stepper component
- 3 StepperPanel components
- State management for all steps
- Navigation guards
- Auto-processing on mode/data changes

### Reused Components

- `FileInputSection.vue` - Step 1 file loading (unchanged)
- `TemplateEditor.vue` - Step 2 template input (unchanged)
- `PreviewTable.vue` - Steps 2 & 3 preview (unchanged)

### New Components

None - all functionality integrated into refactored BatchScriptDialog.

## Tauri Commands

### New Commands

```rust
#[tauri::command]
pub fn batch_generate_combinations(data: BatchData) -> Result<BatchData, String>

#[tauri::command]
pub async fn batch_get_recent_templates(
    db: State<'_, Arc<Mutex<Connection>>>
) -> Result<Vec<TemplateHistoryEntry>, String>

#[tauri::command]
pub async fn batch_save_template(
    db: State<'_, Arc<Mutex<Connection>>>,
    template: String,
    image_count: i64
) -> Result<(), String>
```

### Existing Commands (Unchanged)

- `batch_parse_data` - Parse CSV/JSON
- `batch_render_template` - Render template with data

## Error Handling

### Critical Errors (Block User)
- Data parsing failure → Show error in Step 1
- Template rendering errors → Show in Step 2 preview, block Next
- Generation queueing failure → Show toast, keep dialog open

### Non-Critical Errors (Graceful Degradation)
- Template history load failure → Continue without history
- Template history save failure → Log warning, continue
- Combination count > 1000 → Show warning toast, allow proceed

## UI/UX Details

### Step 1: Data Mode Selector

```
Processing Mode:
○ Use data as-is (3 images)
○ Generate all combinations (6 images)
  style:2 × subject:3
```

### Step 2: Recent Templates

```
┌────────────────────────────────────┐
│ A {{ style }} painting of...      │
│ Jan 26, 2024 • 18 images          │
└────────────────────────────────────┘
```

### Step 3: Summary Card

```
📊 Data Source: batch_data.csv
🔢 Mode: Combinatorial (6 images)
📝 Template: A {{ style }} painting...

Generation Settings:
• Steps: 20
• CFG Scale: 7.0
• Size: 1024×1024
• Seed: 12345 (same for all)
```

## Implementation Phases

### Phase 1: Backend (Combinatorial + History)
1. Add database schema migration
2. Implement combinatorial algorithm
3. Add template history CRUD commands
4. Add unit tests

### Phase 2: Frontend (Stepper Refactor)
1. Refactor BatchScriptDialog to use Stepper
2. Add Step 1 mode selector and preview
3. Add Step 2 template history cards
4. Add Step 3 summary card
5. Implement navigation guards

### Phase 3: Integration & Polish
1. Wire up auto-processing watches
2. Template history save on generation
3. Error handling and validation
4. Manual testing of all flows

## Testing Strategy

### Manual Test Cases

**Step 1 - Data Loading:**
- [ ] Load CSV file, see data preview
- [ ] Load JSON file (array format), see data preview
- [ ] Load JSON file (object format), see data preview
- [ ] Switch from as-is to combinatorial, see count update
- [ ] Combinatorial shows correct math breakdown
- [ ] Next button disabled without data
- [ ] Next button enabled with data

**Step 2 - Template:**
- [ ] Recent templates display (if any exist)
- [ ] Click recent template, loads into editor
- [ ] Template editor works as before
- [ ] Preview updates when template changes
- [ ] Preview updates when going back and changing mode
- [ ] Next button disabled with empty template
- [ ] Next button disabled with template errors
- [ ] Back button returns to Step 1 (preserves data)

**Step 3 - Confirm:**
- [ ] Summary shows correct data source
- [ ] Summary shows correct mode and count
- [ ] Summary shows generation settings
- [ ] Preview table shows final prompts
- [ ] Generate button queues all jobs
- [ ] Template saved to history after generation
- [ ] Dialog closes after successful generation
- [ ] Back button returns to Step 2

**Combinatorial Mode:**
- [ ] 2 columns × 2 unique values each = 4 combinations
- [ ] 3 columns with varying unique counts = correct product
- [ ] Large batch (>1000) shows warning
- [ ] As-is mode with same data shows original row count

**Template History:**
- [ ] History loads on dialog open
- [ ] History shows most recent 5 templates
- [ ] History shows date and image count
- [ ] Clicking history loads template
- [ ] New template saved after generation
- [ ] History persists across app restarts

### Edge Cases

- [ ] Empty CSV file → Error in Step 1
- [ ] CSV with 1 column → Combinatorial = same as as-is
- [ ] Template with missing variables → Shows error rows
- [ ] Cancel during generation → (disabled, tested by checking button state)
- [ ] Reopen dialog after cancel → Resets to Step 1
- [ ] Very long template → Truncated in history cards

## Success Criteria

✅ User can load data and choose as-is or combinatorial
✅ Combinatorial mode generates correct Cartesian product
✅ Stepper guides user through 3 clear steps
✅ Template history provides quick reuse
✅ Navigation guards prevent invalid states
✅ All batch jobs use same seed
✅ Template saved to database after generation
✅ Error handling provides clear feedback

## Future Enhancements (Out of Scope)

- Template library with folders/categories
- Combinatorial with exclusion rules
- Seed strategy options (increment per row, random per row)
- Export combinations as CSV without generating
- Batch progress tracking in queue panel
