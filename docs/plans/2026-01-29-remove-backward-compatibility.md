# Remove Backward Compatibility Code - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove all backward compatibility, legacy fallback, and migration code since this app hasn't been released yet.

**Architecture:** Transform from a migration-friendly codebase with multiple fallback layers to a clean, single-path implementation that assumes the latest schema and bundle system.

**Tech Stack:** Rust, SQLite, Vue 3, TypeScript

---

## Task 1: Update CLAUDE.md with NO BACKWARD COMPATIBILITY Rule

**Files:**
- Modify: `CLAUDE.md:1-10`

**Step 1: Verify rule is added**

Read: `CLAUDE.md` and confirm the critical rule exists at the top

**Step 2: Commit documentation change**

```bash
git add CLAUDE.md
git commit -m "docs: add critical NO BACKWARD COMPATIBILITY rule

App hasn't been released - no need for compatibility code"
```

---

## Task 2: Remove Legacy Path System from ModelPaths

**Files:**
- Modify: `src-tauri/src/models/paths.rs`

**Step 1: Remove legacy path methods**

Delete methods (lines ~254-417):
- `legacy_component_path()`
- `legacy_clip_path()`
- `legacy_vae_path()`
- `legacy_transformer_path()`
- `legacy_tokenizer_path()`
- `legacy_t5_path()`
- `legacy_t5_tokenizer_path()`
- `get_snapshot_hash()`
- `snapshot_path()`

**Step 2: Remove legacy dirs function**

Delete `get_legacy_dirs()` function (lines ~196-227)

**Step 3: Remove fallback from public API methods**

Replace methods that use `unwrap_or_else(|_| self.legacy_*())`:

```rust
// BEFORE
pub fn clip_path(&self) -> PathBuf {
    self.component_path(ComponentRole::Clip)
        .unwrap_or_else(|_| self.legacy_clip_path())
}

// AFTER
pub fn clip_path(&self) -> Result<PathBuf> {
    self.component_path(ComponentRole::Clip)
}
```

Do this for:
- `clip_path()`
- `vae_path()`
- `transformer_path()`
- `tokenizer_path()`
- `t5_path()`
- `t5_tokenizer_path()`

**Step 4: Remove new_with_context fallback logic**

Replace (lines ~59-92):

```rust
// BEFORE: 4-level priority with fallback
pub fn new_with_context(...) -> Result<Self> {
    if let Some(bundle_id) = bundle_id { ... }
    if t5_component_id.is_some() { ... }
    if let Ok(paths) = Self::from_active_bundle() { ... }
    Self::new_legacy() // Fallback
}

// AFTER: Bundle or components required
pub fn new_with_context(
    bundle_id: Option<&str>,
    model_id: &str,
    t5_id: &str,
    clip_id: &str,
    vae_id: &str,
) -> Result<Self> {
    if let Some(bundle_id) = bundle_id {
        let db_path = Self::get_db_path()?;
        let db = crate::gallery::GalleryDb::new(&db_path)?;
        let bundle_info = db.get_bundle(bundle_id)?;
        Self::from_bundle_info(&bundle_info)
    } else {
        Self::from_component_ids(model_id, Some(t5_id), Some(clip_id), Some(vae_id))
    }
}
```

**Step 5: Remove new_legacy() function**

Delete entire `new_legacy()` private function

**Step 6: Remove legacy fields from struct**

```rust
// BEFORE
pub struct ModelPaths {
    bundle_id: Option<String>,
    bundle_components: Option<HashMap<ComponentRole, PathBuf>>,
    pub cache_dir: PathBuf,  // Legacy
    pub schnell_dir: PathBuf, // Legacy
}

// AFTER
pub struct ModelPaths {
    bundle_id: Option<String>,
    bundle_components: HashMap<ComponentRole, PathBuf>,
}
```

**Step 7: Update from_bundle_info()**

Remove legacy field initialization:

```rust
// BEFORE
Ok(Self {
    bundle_id: Some(bundle.id.clone()),
    bundle_components: Some(component_paths),
    cache_dir,  // Legacy
    schnell_dir, // Legacy
})

// AFTER
Ok(Self {
    bundle_id: Some(bundle.id.clone()),
    bundle_components: component_paths,
})
```

**Step 8: Remove validate_legacy_paths()**

Delete `validate_legacy_paths()` function (lines ~684-698)

**Step 9: Remove get_legacy_status()**

Delete `get_legacy_status()` function (lines ~741-766)

**Step 10: Simplify all_files_exist()**

```rust
// BEFORE
pub fn all_files_exist(&self) -> bool {
    if let Some(ref components) = self.bundle_components {
        self.validate_bundle_components().is_ok()
    } else {
        self.validate_legacy_paths()
    }
}

// AFTER
pub fn all_files_exist(&self) -> bool {
    self.validate_bundle_components().is_ok()
}
```

**Step 11: Simplify get_status()**

```rust
// BEFORE
pub fn get_status(&self) -> Vec<(String, bool, String)> {
    if self.is_bundle_mode() {
        self.get_bundle_status()
    } else {
        self.get_legacy_status()
    }
}

// AFTER
pub fn get_status(&self) -> Vec<(String, bool, String)> {
    self.get_bundle_status()
}
```

**Step 12: Remove is_bundle_mode() (no longer needed)**

Delete `is_bundle_mode()` method - everything is bundle mode now

**Step 13: Update method signatures to return Result**

Change methods from `PathBuf` to `Result<PathBuf>` where needed, removing unwrap_or_else patterns

**Step 14: Remove Dev/Z-Image specific legacy methods**

Delete methods:
- `dev_dir()`
- `get_dev_snapshot_hash()`
- `dev_transformer_path()`
- `quantized_dev_transformer_path()`
- `is_dev_downloaded()`
- `has_quantized_dev()`
- `zimage_dir()`
- `get_zimage_snapshot_hash()`
- `qwen3_path()`
- `qwen3_tokenizer_path()`
- `zimage_transformer_path()`
- `zimage_vae_path()`
- `quantized_zimage_transformer_path()`
- `is_zimage_downloaded()`
- `has_quantized_zimage()`
- `transformer_path_for()`
- `quantized_transformer_path_for()`
- `has_quantized_for()`
- `quantized_transformer_path()`
- `has_quantized_transformer()`
- `quantized_t5_path()`
- `has_quantized_t5()`

All these are legacy methods - components are now selected explicitly.

**Step 15: Run tests**

```bash
cargo test --lib paths::tests
```

Expected: Tests may fail, need updating to use new API

**Step 16: Commit**

```bash
git add src-tauri/src/models/paths.rs
git commit -m "refactor: remove legacy path system

- Remove all legacy_*_path() methods
- Remove fallback priority logic
- Require bundle or component IDs
- Simplify ModelPaths to bundle-only
- Remove legacy struct fields"
```

---

## Task 3: Remove Database Migration Functions

**Files:**
- Modify: `src-tauri/src/gallery/mod.rs`

**Step 1: Remove run_migrations() call**

Find in `init_schema()` (line ~705):

```rust
// BEFORE
// Run migrations for existing databases
self.run_migrations()?;

// AFTER
// Deleted - assume latest schema
```

**Step 2: Delete run_migrations() function**

Delete entire function (lines ~270-309)

**Step 3: Delete all migrate_* functions**

Delete:
- `migrate_models_table()` (lines ~312-348)
- `migrate_models_table_v2()` (lines ~351-383)
- `migrate_models_table_v3()` (lines ~386-400)
- `migrate_source_to_urls()` (lines ~403-426)
- `rebuild_fts_if_needed()` (lines ~429-461)

**Step 4: Run tests**

```bash
cargo test --lib gallery::tests
```

Expected: Tests pass with latest schema

**Step 5: Commit**

```bash
git add src-tauri/src/gallery/mod.rs
git commit -m "refactor: remove database migrations

- Delete run_migrations() and all migrate_* functions
- Assume latest schema always present
- Simpler initialization"
```

---

## Task 4: Remove Settings File Migration

**Files:**
- Modify: `src-tauri/src/settings/mod.rs`

**Step 1: Delete migrate_from_file_to_db() function**

Delete entire function (lines ~134-155)

**Step 2: Remove migration call from load_from_db()**

Find and remove call to `migrate_from_file_to_db()`

**Step 3: Commit**

```bash
git add src-tauri/src/settings/mod.rs
git commit -m "refactor: remove settings file migration

- Delete migrate_from_file_to_db()
- Assume database storage only"
```

---

## Task 5: Remove LoRA JSON Migration

**Files:**
- Modify: `src-tauri/src/models/lora_manager.rs`

**Step 1: Remove index_path field from struct**

```rust
// BEFORE
pub struct LoraManager {
    loras: HashMap<String, Arc<LoraAdapter>>,
    index_path: PathBuf,  // For migration
    db: Arc<tokio::sync::Mutex<Option<GalleryDb>>>,
}

// AFTER
pub struct LoraManager {
    loras: HashMap<String, Arc<LoraAdapter>>,
    db: Arc<tokio::sync::Mutex<Option<GalleryDb>>>,
}
```

**Step 2: Delete migrate_from_file_to_db() function**

Delete entire function (lines ~329-359)

**Step 3: Update constructor**

Remove index_path initialization from `new()`:

```rust
// BEFORE
pub fn new(db: ...) -> Result<Self> {
    let index_path = ...;
    Ok(Self { loras, index_path, db })
}

// AFTER
pub fn new(db: ...) -> Result<Self> {
    Ok(Self { loras, db })
}
```

**Step 4: Remove migration call**

Find and remove any calls to `migrate_from_file_to_db()`

**Step 5: Commit**

```bash
git add src-tauri/src/models/lora_manager.rs
git commit -m "refactor: remove LoRA JSON file migration

- Delete migrate_from_file_to_db()
- Remove index_path field
- Assume database storage only"
```

---

## Task 6: Remove Model Config Fallbacks

**Files:**
- Modify: `src-tauri/src/models/model_config.rs`
- Modify: `src-tauri/src/models/model_config_cache.rs`

**Step 1: Delete fallback() and default_flux() methods**

In `model_config.rs`, delete:
- `fallback()` function (lines ~69-125)
- `default_flux()` function (lines ~127-129)

**Step 2: Simplify from_record()**

Remove all `.unwrap_or()` fallbacks, make fields required:

```rust
// BEFORE
pub fn from_record(record: &ModelRecord) -> Self {
    Self {
        steps: (record.step_min.unwrap_or(1), record.step_max.unwrap_or(50)),
        default_steps: record.default_settings.unwrap_or(4),
        // ... many unwrap_or fallbacks
    }
}

// AFTER
pub fn from_record(record: &ModelRecord) -> Result<Self> {
    Ok(Self {
        steps: (
            record.step_min.ok_or_else(|| anyhow!("step_min missing"))?,
            record.step_max.ok_or_else(|| anyhow!("step_max missing"))?
        ),
        default_steps: record.default_settings.ok_or_else(|| anyhow!("default_steps missing"))?,
        // ... require all fields or return error
    })
}
```

**Step 3: Remove fallback from model_config_cache.rs**

```rust
// BEFORE
pub fn get(&self, model_id: &str) -> ModelConfig {
    // Try cache... try DB... fallback to hard-coded
    ModelConfig::fallback(model_id)
}

// AFTER
pub fn get(&self, model_id: &str) -> Result<ModelConfig> {
    // Try cache
    // Try DB
    // Return error if not found
    anyhow::bail!("Model config not found for {}", model_id)
}
```

**Step 4: Commit**

```bash
git add src-tauri/src/models/model_config.rs src-tauri/src/models/model_config_cache.rs
git commit -m "refactor: remove model config fallbacks

- Delete fallback() and default_flux()
- Make from_record() return Result
- Require all config fields in database
- Remove hard-coded model configs"
```

---

## Task 7: Remove Legacy Scanner Format

**Files:**
- Modify: `src-tauri/src/models/scanner.rs`
- Modify: `src-tauri/src/models/mod.rs`

**Step 1: Delete DiscoveredModel struct**

Delete old struct (lines after component structs):

```rust
// DELETE THIS
pub struct DiscoveredModel {
    pub id: String,
    pub display_name: String,
    pub repo_id: String,
    pub format: ModelFormat,
    pub path: PathBuf,
    pub supports_loras: bool,
    pub vram_mb: usize,
}
```

**Step 2: Delete scan_cache_for_models() function**

Delete entire function (lines ~690-714) that converts to legacy format

**Step 3: Update exports in mod.rs**

```rust
// BEFORE
pub use scanner::{ComponentType, DiscoveredComponent, DiscoveredModel, ModelFormat, scan_all_components, scan_cache_for_models};

// AFTER
pub use scanner::{ComponentType, DiscoveredComponent, ModelFormat, scan_all_components};
```

**Step 4: Find and update all callers**

Search for `scan_cache_for_models` and replace with `scan_all_components`

**Step 5: Commit**

```bash
git add src-tauri/src/models/scanner.rs src-tauri/src/models/mod.rs
git commit -m "refactor: remove legacy DiscoveredModel format

- Delete DiscoveredModel struct
- Delete scan_cache_for_models() conversion
- Use DiscoveredComponent directly everywhere"
```

---

## Task 8: Simplify Pipeline Loader

**Files:**
- Modify: `src-tauri/src/inference/flux_pipeline/loader.rs`

**Step 1: Remove conditional error messages**

Replace lines ~79-91:

```rust
// BEFORE
if !paths.all_files_exist() {
    if paths.is_bundle_mode() {
        return Err(anyhow::anyhow!("Bundle has missing components..."));
    } else {
        return Err(anyhow::anyhow!("Legacy mode error..."));
    }
}

// AFTER
if !paths.all_files_exist() {
    return Err(anyhow::anyhow!(
        "Required model components not found. Ensure bundle/components are selected and available."
    ));
}
```

**Step 2: Remove legacy mode checks**

Delete lines referencing `is_bundle_mode()`, `bundle_id()` for logging

**Step 3: Remove legacy-only Dev model check**

Delete lines ~93-98:

```rust
// DELETE THIS
if !paths.is_bundle_mode() && self.model_type.id() == "dev" && !paths.is_dev_downloaded() {
    return Err(...);
}
```

**Step 4: Commit**

```bash
git add src-tauri/src/inference/flux_pipeline/loader.rs
git commit -m "refactor: simplify pipeline loader

- Remove bundle vs legacy conditional logic
- Assume bundle/component mode always
- Single error message for missing files"
```

---

## Task 9: Remove Old Models Table and Related Code

**Files:**
- Modify: `src-tauri/src/gallery/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Delete ModelRecord struct**

Delete `ModelRecord` struct (lines ~119-169) - replaced by ComponentRecord/BundleInfo

**Step 2: Delete models table from schema**

Remove from `init_schema()`:

```sql
-- DELETE THIS TABLE
CREATE TABLE IF NOT EXISTS models (...)
```

**Step 3: Delete model-related database methods**

Delete:
- `insert_model()`
- `get_all_models()`
- `update_model()`
- Any other model table operations

**Step 4: Remove get_all_models Tauri command**

Delete from `lib.rs`:

```rust
#[command]
async fn get_all_models(...) -> Result<Vec<gallery::ModelRecord>, String> {
    ...
}
```

**Step 5: Remove from command registration**

Delete `get_all_models` from `generate_handler![]` list

**Step 6: Update ModelsView.vue**

Remove calls to `invoke('get_all_models')` - use bundles and components instead

**Step 7: Commit**

```bash
git add src-tauri/src/gallery/mod.rs src-tauri/src/lib.rs src/views/ModelsView.vue
git commit -m "refactor: remove old models table

- Delete ModelRecord struct
- Delete models table schema
- Remove get_all_models() method and command
- Use bundle/component system exclusively"
```

---

## Task 10: Remove Legacy UI Warnings

**Files:**
- Modify: `src/components/models/BundleManagement.vue`

**Step 1: Remove "using legacy paths" warning**

Delete lines ~54-56:

```vue
<!-- DELETE THIS -->
<Message v-else severity="warn" :closable="false">
  No bundle active. Using legacy hardcoded paths. Activate a bundle for better control.
</Message>
```

Replace with:

```vue
<Message v-else severity="error" :closable="false">
  No bundle active. Select a bundle or scan for models to enable generation.
</Message>
```

**Step 2: Commit**

```bash
git add src/components/models/BundleManagement.vue
git commit -m "refactor: remove legacy paths UI warning

- Change message to require bundle
- No longer support legacy fallback mode"
```

---

## Task 11: Remove Database Fallback in ModelsView

**Files:**
- Modify: `src/views/ModelsView.vue`

**Step 1: Remove try/catch fallback**

Replace lines ~650-657:

```typescript
// BEFORE
const loadModelsFromDatabase = async () => {
  try {
    const models = await invoke<ModelInfo[]>('get_all_models')
    dbModels.value = models
  } catch (error) {
    console.error('Failed to load models from database:', error)
    // Fallback to legacy models if database fails
    dbModels.value = []
  }
}

// AFTER
const loadModelsFromDatabase = async () => {
  dbModels.value = await invoke<ModelInfo[]>('get_all_models')
}
```

Let errors propagate - don't silently fail.

**Step 2: Commit**

```bash
git add src/views/ModelsView.vue
git commit -m "refactor: remove database fallback in ModelsView

- Let database errors propagate
- Don't silently fail to empty array"
```

---

## Task 12: Clean Up Model Type Enum

**Files:**
- Modify: `src-tauri/src/models/model_type.rs`

**Step 1: Examine ModelType usage**

Check if ModelType enum is still needed or if component IDs replace it entirely.

**Step 2: Decision point**

If ModelType still needed:
- Keep as-is for now
- Mark for future removal

If ModelType not needed:
- Delete entire file
- Replace with component-based inference

**Step 3: Document decision**

Add comment explaining why kept or deleted

**Step 4: Commit**

```bash
git add src-tauri/src/models/model_type.rs
git commit -m "refactor: [keep/remove] ModelType enum

[Reason for decision]"
```

---

## Task 13: Update Tests to Remove Legacy Expectations

**Files:**
- Modify: `src-tauri/tests/bundle_integration_test.rs`
- Modify: `src-tauri/src/models/paths.rs` (tests section)

**Step 1: Update test_model_paths_legacy_fallback**

```rust
// BEFORE
#[test]
fn test_model_paths_legacy_fallback() {
    let paths = ModelPaths::new().unwrap();
    assert!(paths.cache_dir.to_string_lossy().contains("huggingface"));
}

// AFTER - delete or replace with bundle test
#[test]
fn test_model_paths_requires_bundle() {
    // Should fail without bundle
    let result = ModelPaths::new();
    assert!(result.is_err());
}
```

**Step 2: Remove tests for legacy methods**

Delete tests for deleted methods like:
- `test_paths_creation()` if it tests legacy mode
- Any tests checking fallback behavior

**Step 3: Add tests for new behavior**

```rust
#[test]
fn test_model_paths_from_bundle_required() {
    // Test that bundle is required
    let result = ModelPaths::new();
    assert!(result.is_err() || result.unwrap().bundle_id.is_some());
}

#[test]
fn test_model_paths_from_components_required() {
    // Test that all component IDs are required
    let result = ModelPaths::from_component_ids("trans-1", None, None, None);
    assert!(result.is_err()); // Should fail - missing T5, CLIP, VAE
}
```

**Step 4: Run all tests**

```bash
cargo test --test bundle_integration_test
cargo test --lib
```

**Step 5: Commit**

```bash
git add src-tauri/tests/bundle_integration_test.rs src-tauri/src/models/paths.rs
git commit -m "test: update tests for bundle-only mode

- Remove legacy fallback tests
- Add bundle/component requirement tests
- Expect errors when bundle not provided"
```

---

## Task 14: Update Frontend to Require Bundle/Component Selection

**Files:**
- Modify: `src/components/generation/actions/EnhancedModelSelector.vue`
- Modify: `src/views/GenerateView.vue`

**Step 1: Add validation in EnhancedModelSelector**

```typescript
// Add computed property
const isValidConfiguration = computed(() => {
  // Bundle mode: bundleId must be set
  if (generationStore.currentParams.bundleId) {
    return true
  }

  // Individual mode: all component IDs must be set
  return !!(
    generationStore.currentParams.modelComponentId &&
    generationStore.currentParams.t5ComponentId &&
    generationStore.currentParams.clipComponentId &&
    generationStore.currentParams.vaeComponentId
  )
})

// Expose to parent
defineExpose({ isValidConfiguration })
```

**Step 2: Disable generate button if invalid**

In GenerateView.vue:

```vue
<Button
  label="Generate"
  :disabled="!isValidConfiguration"
  @click="handleGenerate"
/>
```

**Step 3: Show error message if incomplete**

```vue
<Message v-if="!isValidConfiguration" severity="error">
  Please select a bundle or configure all components (Transformer, T5, CLIP, VAE)
</Message>
```

**Step 4: Commit**

```bash
git add src/components/generation/actions/EnhancedModelSelector.vue src/views/GenerateView.vue
git commit -m "feat: require bundle or full component selection

- Add validation for complete configuration
- Disable generate button when invalid
- Show error message for incomplete setup"
```

---

## Task 15: Clean Up Documentation

**Files:**
- Modify: `MODEL_BUNDLE_IMPLEMENTATION_STATUS.md`
- Modify: `STEP_6_PIPELINE_INTEGRATION_SUMMARY.md`
- Modify: `STEP_7_FRONTEND_INTEGRATION_SUMMARY.md`

**Step 1: Remove backward compatibility sections**

Search docs for mentions of:
- "backward compatible"
- "legacy mode"
- "fallback"
- "migration"

Update sections to reflect bundle-only approach.

**Step 2: Update architecture diagrams**

Remove legacy path branches from flow diagrams.

**Step 3: Commit**

```bash
git add docs/*.md *.md
git commit -m "docs: update to reflect bundle-only architecture

- Remove backward compatibility mentions
- Update flow diagrams
- Clarify bundle requirement"
```

---

## Task 16: Final Verification

**Step 1: Full test suite**

```bash
cargo test --all
npm run build
```

Expected: All tests pass, no compilation errors

**Step 2: Start app and verify**

```bash
npm run tauri:dev
```

Test flow:
1. App starts
2. Models → Bundles → Scan Models
3. Activate bundle
4. Generate → Select bundle
5. Generate image
6. Success!

**Step 3: Check for remaining "fallback" code**

```bash
rg -i "fallback|legacy|backward|compat|migration" src-tauri/src --type rust
rg -i "fallback|legacy|backward|compat" src --type ts --type vue
```

Expected: Only comments or docs, no actual fallback logic

**Step 4: Final commit**

```bash
git add -A
git commit -m "refactor: complete removal of backward compatibility

- All legacy fallback code removed
- Bundle/component system is required
- Clean, single-path implementation

BREAKING CHANGE: Requires bundle or component selection"
```

---

## Summary of Deletions

### Code Removed
- **~600 lines** from `paths.rs` (legacy methods)
- **~250 lines** from `gallery/mod.rs` (migrations)
- **~80 lines** from model config files (fallbacks)
- **~60 lines** from settings/LoRA (file migrations)
- **~50 lines** from scanner (legacy format)
- **~30 lines** from loader (conditional logic)
- **~20 lines** from UI (fallback messages)

**Total:** ~1,090 lines deleted

### Tables Removed
- `models` table (replaced by `model_components` + `model_bundles`)

### Complexity Reduced
- Single path through code (no if/else for mode detection)
- No priority-based fallback chains
- No migration checks on startup
- Required parameters (no optional with defaults)

### Result
Cleaner, simpler codebase focused on one approach: bundle/component system.

---

## Risks & Mitigation

### Risk: Users with existing databases
**Mitigation:** Database will auto-create latest schema on first run

### Risk: Missing components
**Mitigation:** Validation requires scan + bundle activation upfront

### Risk: Breaking existing development work
**Mitigation:** This IS a breaking change, but app hasn't been released so acceptable

---

## Testing Strategy

1. **Unit Tests:** Update to expect bundle/component requirements
2. **Integration Tests:** Verify bundle-only flow works end-to-end
3. **Manual Testing:** Full user flow from scan to generation
4. **Regression Prevention:** Tests should fail if fallback code is added back

---

## Success Criteria

✅ **Done when:**
- [ ] No code references "legacy", "fallback", "backward", "migration" in logic (only docs/comments OK)
- [ ] All tests pass
- [ ] App requires bundle or component selection
- [ ] Generation works with bundle selection
- [ ] Codebase ~1,000 lines lighter
- [ ] Single path through all code
