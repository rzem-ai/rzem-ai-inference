# Model & Bundle Manager

## 1) Full problem description (Markdown)

### Goal
Build a desktop application (Tauri + Rust + Vue + PrimeVue + Tailwind) to:
1) Scan local directories for **supported model “bundles”** and **supported model files**.
2) Present the results in two tabs (**Bundles** and **Models**), sorted alphabetically.
3) Allow users to view details, rename (display name), tag, and enrich entries with examples (images + prompts).
4) Enable creation of new bundles by selecting compatible components only.
5) Persist **compatibility metadata** so future workflows do not require re-reading/parsing the files.

### Non-goals
- No migration from prior implementation (greenfield replacement).
- No online downloads or Hugging Face auth required (local filesystem scanning only).
- Exclude non-target model types (e.g., ControlNet, IP-Adapter, etc.).

---

### Core concepts

#### A) “Model files” (individual assets)
The app scans a chosen directory recursively and detects supported model assets. Anything not matching recognized patterns is ignored (including ControlNet/IP-Adapter/etc.).

Each recognized model is categorized into one of these types:
- Base checkpoint (e.g., diffusion transformer / main checkpoint)
- LoRA
- VAE
- Text encoder
- Tokenizer
- Scheduler
- Autoencoder / AE
- Other explicitly-supported core files (only if defined)

Each model entry stores:
- Physical file path, hash, size, modified time
- A user-editable display name (not just filename)
- Tags
- Optional metadata depending on type:
  - Base checkpoint: preferred steps, preferred CFG
  - LoRA: preferred strengths, trigger words
- Example images and example prompts (for base checkpoints and LoRAs; optionally extensible)

#### B) “Bundles” (compatible sets)
A bundle is a curated set of compatible assets used together (e.g., base checkpoint + VAE + text encoders + tokenizers + scheduler + AE).

Users can:
- Scan bundles from existing “well-known” repo structures (e.g., HF-style snapshots with symlinks)
- Create new bundles
- Attach descriptions, tags, example images, example prompts

**Compatibility constraint**: When selecting components for a bundle, only show compatible options (e.g., incompatible VAEs/text encoders must be hidden).

#### C) Symlink-aware scanning (Hugging Face style)
Hugging Face cache and snapshot layouts commonly use symbolic links. The scanner must:
- Detect symlinks and resolve to target blobs
- Record both the symlink path and resolved target path
- Still identify the “logical” file in the snapshot structure

#### D) Important exclusion rule: sharded files
For certain models, weights may be sharded:
- model-00001-of-000NN.safetensors
- diffusion_pytorch_model-00001-of-000NN.safetensors
- model.safetensors.index.json

Requirement: **Do not treat sharded pieces or shard index configs as “important files”** for the simplified “bundle signature” selection. The app should focus on single-file weights (where applicable) or only the explicitly “important” files list for that model family.

---

### UX requirements

#### Tabs
1) **Bundles tab**
- Button: Scan Bundles
- Button: Create Bundle
- List of bundles (alphabetically)
- Selecting a bundle shows details + editable metadata + selected components

2) **Models tab**
- Button: Scan Models (choose a directory; recursively scan)
- List of models grouped by type and sorted alphabetically
- Selecting a model shows details + editable metadata

#### Editing
- Rename display name
- Add/remove tags (all model types + bundles)
- Add multiple example images
- Add multiple example prompts
- Type-specific fields:
  - Base checkpoint: preferred_steps, preferred_cfg
  - LoRA: preferred_strength_min/max/default, trigger_words

#### Persistence
Store everything in SQLite:
- Models and their physical file identity (hash, size, mtime)
- Logical identity (display name, tags, examples)
- Compatibility metadata between model types and/or specific model items
- Bundle composition
- Scan runs/logs (optional but strongly recommended)

---

### Sorting & presentation
- All list views sorted alphabetically by display name (fallback: filename).
- Consistent grouping by model type.
- Duplicate detection by content hash (same blob linked multiple places) should collapse into a single logical model with multiple “locations”.

---

### Future-proofing
- Compatibility data should be stored in DB so later UIs can filter compatible components instantly.
- Model classification rules should be extensible without schema changes (store raw classifier output as JSON).

---

## 2) SQLite schema — `schema.sql`

```sql
-- schema.sql (SQLite)
-- Greenfield schema for Model & Bundle Manager

PRAGMA foreign_keys = ON;

-- ------------------------------------------------------------
-- ENUM-LIKE FIELDS (documented):
-- model_type: 'base_checkpoint' | 'lora' | 'vae' | 'text_encoder' | 'tokenizer' | 'scheduler' | 'ae' | 'other'
-- location_kind: 'real_file' | 'symlink'
-- entity_kind: 'model' | 'bundle'
-- example_kind: 'image' | 'prompt'
-- compat_scope: 'type_to_type' | 'model_to_model' | 'bundle_rule'
-- ------------------------------------------------------------

-- Scan runs (Models or Bundles), useful for audit/debug and progress UI.
CREATE TABLE scan_run (
  id                TEXT PRIMARY KEY,          -- UUID
  scan_kind         TEXT NOT NULL,             -- 'models' or 'bundles'
  root_path         TEXT NOT NULL,
  started_at        TEXT NOT NULL,             -- ISO8601
  finished_at       TEXT,                      -- ISO8601
  status            TEXT NOT NULL,             -- 'running'|'success'|'error'
  error_message     TEXT
);

-- Physical file identity (dedupe by hash if desired)
CREATE TABLE file_blob (
  id                TEXT PRIMARY KEY,          -- UUID
  sha256            TEXT NOT NULL UNIQUE,      -- 64 hex
  size_bytes        INTEGER NOT NULL,
  created_at        TEXT NOT NULL              -- ISO8601
);

-- A "location" is a path on disk that points to a blob.
-- Supports Hugging Face-style symlinks by recording both link path and resolved target.
CREATE TABLE file_location (
  id                TEXT PRIMARY KEY,          -- UUID
  blob_id           TEXT NOT NULL REFERENCES file_blob(id) ON DELETE CASCADE,
  scan_run_id       TEXT REFERENCES scan_run(id) ON DELETE SET NULL,

  path              TEXT NOT NULL,             -- path user sees (may be symlink path)
  location_kind     TEXT NOT NULL,             -- 'real_file'|'symlink'
  symlink_target    TEXT,                      -- if symlink, the target path
  resolved_path     TEXT NOT NULL,             -- absolute resolved path to actual bytes
  modified_at       TEXT,                      -- ISO8601 from FS
  UNIQUE(path)
);

-- Logical model entity (the user edits this)
CREATE TABLE model (
  id                TEXT PRIMARY KEY,          -- UUID
  model_type        TEXT NOT NULL,
  display_name      TEXT NOT NULL,             -- user editable
  description       TEXT,
  created_at        TEXT NOT NULL,             -- ISO8601
  updated_at        TEXT NOT NULL              -- ISO8601
);

-- Many-to-many: model can have multiple file locations (same blob linked in multiple places)
-- One model may also map to multiple blobs in edge cases, but we keep the "primary blob"
-- as the canonical identity. Additional blobs can be attached if needed later.
CREATE TABLE model_file (
  id                TEXT PRIMARY KEY,          -- UUID
  model_id          TEXT NOT NULL REFERENCES model(id) ON DELETE CASCADE,
  location_id       TEXT NOT NULL REFERENCES file_location(id) ON DELETE CASCADE,
  blob_id           TEXT NOT NULL REFERENCES file_blob(id) ON DELETE CASCADE,

  filename          TEXT NOT NULL,
  extension         TEXT,                      -- e.g. 'safetensors','json','md'
  is_important       INTEGER NOT NULL DEFAULT 1, -- 1=true. For bundles: only important files matter.
  is_shard           INTEGER NOT NULL DEFAULT 0, -- 1=true; MUST be excluded from "important files" views
  shard_group_key    TEXT,                     -- e.g. 'model.safetensors' or 'diffusion_pytorch_model'
  created_at        TEXT NOT NULL,
  UNIQUE(model_id, location_id)
);

-- Flexible classifier output & technical properties captured once (avoid re-reading later)
CREATE TABLE model_tech (
  model_id          TEXT PRIMARY KEY REFERENCES model(id) ON DELETE CASCADE,
  format_family     TEXT,                      -- e.g. 'diffusers', 'single_safetensors', 'gguf', etc.
  architecture      TEXT,                      -- e.g. 'flux', 'sd15', etc.
  raw_metadata_json TEXT                       -- JSON string
);

-- Type-specific fields (base checkpoints)
CREATE TABLE model_base_prefs (
  model_id          TEXT PRIMARY KEY REFERENCES model(id) ON DELETE CASCADE,
  preferred_steps   INTEGER,
  preferred_cfg     REAL
);

-- Type-specific fields (LoRAs)
CREATE TABLE model_lora_prefs (
  model_id                  TEXT PRIMARY KEY REFERENCES model(id) ON DELETE CASCADE,
  preferred_strength_min    REAL,
  preferred_strength_max    REAL,
  preferred_strength_default REAL,
  trigger_words_json        TEXT              -- JSON array of strings
);

-- Tags (shared between models and bundles)
CREATE TABLE tag (
  id                TEXT PRIMARY KEY,          -- UUID
  name              TEXT NOT NULL UNIQUE       -- lowercase recommended
);

CREATE TABLE entity_tag (
  id                TEXT PRIMARY KEY,          -- UUID
  entity_kind       TEXT NOT NULL,             -- 'model'|'bundle'
  entity_id         TEXT NOT NULL,
  tag_id            TEXT NOT NULL REFERENCES tag(id) ON DELETE CASCADE,
  created_at        TEXT NOT NULL,

  UNIQUE(entity_kind, entity_id, tag_id)
);

-- Examples: images or prompts for models and bundles
CREATE TABLE example (
  id                TEXT PRIMARY KEY,          -- UUID
  entity_kind       TEXT NOT NULL,             -- 'model'|'bundle'
  entity_id         TEXT NOT NULL,
  example_kind      TEXT NOT NULL,             -- 'image'|'prompt'
  title             TEXT,
  content_text      TEXT,                      -- prompt text when example_kind='prompt'
  image_path        TEXT,                      -- path to image when example_kind='image' (stored local)
  created_at        TEXT NOT NULL
);

-- Bundles
CREATE TABLE bundle (
  id                TEXT PRIMARY KEY,          -- UUID
  display_name      TEXT NOT NULL,
  description       TEXT,
  created_at        TEXT NOT NULL,
  updated_at        TEXT NOT NULL
);

-- Bundle items: which models belong to a bundle and what role they play.
CREATE TABLE bundle_item (
  id                TEXT PRIMARY KEY,          -- UUID
  bundle_id         TEXT NOT NULL REFERENCES bundle(id) ON DELETE CASCADE,
  model_id          TEXT NOT NULL REFERENCES model(id) ON DELETE RESTRICT,
  role              TEXT NOT NULL,             -- e.g. 'base_checkpoint','vae','text_encoder','text_encoder_2','tokenizer','tokenizer_2','scheduler','ae'
  is_required       INTEGER NOT NULL DEFAULT 1,
  created_at        TEXT NOT NULL,
  UNIQUE(bundle_id, model_id, role)
);

-- Compatibility rules: persisted so we can filter without re-reading files.
-- You can use either type-to-type compatibility or specific model-to-model.
CREATE TABLE compatibility_rule (
  id                TEXT PRIMARY KEY,          -- UUID
  compat_scope      TEXT NOT NULL,             -- 'type_to_type'|'model_to_model'|'bundle_rule'

  -- type-to-type
  left_model_type   TEXT,
  right_model_type  TEXT,

  -- model-to-model
  left_model_id     TEXT REFERENCES model(id) ON DELETE CASCADE,
  right_model_id    TEXT REFERENCES model(id) ON DELETE CASCADE,

  -- rule semantics
  is_compatible     INTEGER NOT NULL,          -- 1=true, 0=false
  reason            TEXT,
  data_json         TEXT,                      -- JSON: any computed constraints (dim, vocab, etc.)
  created_at        TEXT NOT NULL,

  CHECK (
    (compat_scope='type_to_type' AND left_model_type IS NOT NULL AND right_model_type IS NOT NULL)
    OR
    (compat_scope='model_to_model' AND left_model_id IS NOT NULL AND right_model_id IS NOT NULL)
    OR
    (compat_scope='bundle_rule')
  )
);

-- Helpful indexes
CREATE INDEX idx_model_type ON model(model_type);
CREATE INDEX idx_model_display_name ON model(display_name);
CREATE INDEX idx_bundle_display_name ON bundle(display_name);
CREATE INDEX idx_model_file_model ON model_file(model_id);
CREATE INDEX idx_entity_tag_entity ON entity_tag(entity_kind, entity_id);
CREATE INDEX idx_example_entity ON example(entity_kind, entity_id);
CREATE INDEX idx_file_location_blob ON file_location(blob_id);
CREATE INDEX idx_bundle_item_bundle ON bundle_item(bundle_id);
```

---

## 3) Data dictionary tables (columns, type/size, usage)

### 3.1 `scan_run`

| Column        | Type | Size | Usage                        |
| ------------- | ---: | ---: | ---------------------------- |
| id            | TEXT |  ~36 | UUID primary key             |
| scan_kind     | TEXT |   16 | `models` or `bundles`        |
| root_path     | TEXT |  var | User-selected root directory |
| started_at    | TEXT |   25 | ISO8601 timestamp            |
| finished_at   | TEXT |   25 | ISO8601 timestamp            |
| status        | TEXT |   16 | `running/success/error`      |
| error_message | TEXT |  var | Error text if failed         |

### 3.2 `file_blob`

| Column     |    Type |    Size | Usage                   |
| ---------- | ------: | ------: | ----------------------- |
| id         |    TEXT |     ~36 | UUID PK                 |
| sha256     |    TEXT |      64 | Content hash for dedupe |
| size_bytes | INTEGER | 8 bytes | File size               |
| created_at |    TEXT |      25 | When first recorded     |

### 3.3 `file_location`

| Column         | Type | Size | Usage                                          |
| -------------- | ---: | ---: | ---------------------------------------------- |
| id             | TEXT |  ~36 | UUID PK                                        |
| blob_id        | TEXT |  ~36 | FK → `file_blob`                               |
| scan_run_id    | TEXT |  ~36 | FK → `scan_run` (optional)                     |
| path           | TEXT |  var | Path discovered in scan (symlink path allowed) |
| location_kind  | TEXT |   16 | `real_file` or `symlink`                       |
| symlink_target | TEXT |  var | Target path if symlink                         |
| resolved_path  | TEXT |  var | Resolved path to actual bytes                  |
| modified_at    | TEXT |   25 | FS modified time                               |

### 3.4 `model`

| Column       | Type | Size | Usage                                      |
| ------------ | ---: | ---: | ------------------------------------------ |
| id           | TEXT |  ~36 | UUID PK                                    |
| model_type   | TEXT |   32 | Category (`base_checkpoint`, `lora`, etc.) |
| display_name | TEXT |  var | User-friendly name (editable)              |
| description  | TEXT |  var | Free text                                  |
| created_at   | TEXT |   25 | ISO8601                                    |
| updated_at   | TEXT |   25 | ISO8601                                    |

### 3.5 `model_file`

| Column          |    Type | Size | Usage                                                                       |
| --------------- | ------: | ---: | --------------------------------------------------------------------------- |
| id              |    TEXT |  ~36 | UUID PK                                                                     |
| model_id        |    TEXT |  ~36 | FK → `model`                                                                |
| location_id     |    TEXT |  ~36 | FK → `file_location`                                                        |
| blob_id         |    TEXT |  ~36 | FK → `file_blob`                                                            |
| filename        |    TEXT |  var | Basename                                                                    |
| extension       |    TEXT |   16 | `safetensors`, `json`, etc.                                                 |
| is_important    | INTEGER |    1 | 1=used for bundle signature + UI “important files”                          |
| is_shard        | INTEGER |    1 | 1=sharded piece or shard index; must be excluded from “important” selection |
| shard_group_key |    TEXT |  var | Groups shards under a common key                                            |
| created_at      |    TEXT |   25 | ISO8601                                                                     |

### 3.6 `model_tech`

| Column            | Type | Size | Usage                                                        |
| ----------------- | ---: | ---: | ------------------------------------------------------------ |
| model_id          | TEXT |  ~36 | PK & FK → `model`                                            |
| format_family     | TEXT |   32 | e.g., diffusers / gguf / single_safetensors                  |
| architecture      | TEXT |   32 | e.g., flux / sd15 / etc.                                     |
| raw_metadata_json | TEXT |  var | Stored classifier outputs and derived metadata (JSON string) |

### 3.7 `model_base_prefs`

| Column          |    Type | Size | Usage                     |
| --------------- | ------: | ---: | ------------------------- |
| model_id        |    TEXT |  ~36 | PK & FK → `model`         |
| preferred_steps | INTEGER |    8 | Preferred inference steps |
| preferred_cfg   |    REAL |    8 | Preferred CFG             |

### 3.8 `model_lora_prefs`

| Column                     | Type | Size | Usage                       |
| -------------------------- | ---: | ---: | --------------------------- |
| model_id                   | TEXT |  ~36 | PK & FK → `model`           |
| preferred_strength_min     | REAL |    8 | Suggested min strength      |
| preferred_strength_max     | REAL |    8 | Suggested max strength      |
| preferred_strength_default | REAL |    8 | Suggested default strength  |
| trigger_words_json         | TEXT |  var | JSON array of trigger words |

### 3.9 `tag` and `entity_tag`

`tag`

| Column | Type | Size | Usage           |
| ------ | ---: | ---: | --------------- |
| id     | TEXT |  ~36 | UUID PK         |
| name   | TEXT |  var | Unique tag name |

`entity_tag`

| Column      | Type | Size | Usage               |
| ----------- | ---: | ---: | ------------------- |
| id          | TEXT |  ~36 | UUID PK             |
| entity_kind | TEXT |   16 | `model` or `bundle` |
| entity_id   | TEXT |  ~36 | ID of model/bundle  |
| tag_id      | TEXT |  ~36 | FK → `tag`          |
| created_at  | TEXT |   25 | ISO8601             |

### 3.10 `example`

| Column       | Type | Size | Usage               |
| ------------ | ---: | ---: | ------------------- |
| id           | TEXT |  ~36 | UUID PK             |
| entity_kind  | TEXT |   16 | `model` or `bundle` |
| entity_id    | TEXT |  ~36 | ID of model/bundle  |
| example_kind | TEXT |   16 | `image` or `prompt` |
| title        | TEXT |  var | Optional label      |
| content_text | TEXT |  var | Prompt text         |
| image_path   | TEXT |  var | Local image path    |
| created_at   | TEXT |   25 | ISO8601             |

### 3.11 `bundle` and `bundle_item`

`bundle`

| Column       | Type | Size | Usage              |
| ------------ | ---: | ---: | ------------------ |
| id           | TEXT |  ~36 | UUID PK            |
| display_name | TEXT |  var | User-friendly name |
| description  | TEXT |  var | Free text          |
| created_at   | TEXT |   25 | ISO8601            |
| updated_at   | TEXT |   25 | ISO8601            |

`bundle_item`

| Column      |    Type | Size | Usage                               |
| ----------- | ------: | ---: | ----------------------------------- |
| id          |    TEXT |  ~36 | UUID PK                             |
| bundle_id   |    TEXT |  ~36 | FK → `bundle`                       |
| model_id    |    TEXT |  ~36 | FK → `model`                        |
| role        |    TEXT |   32 | What the model is inside the bundle |
| is_required | INTEGER |    1 | 1=required                          |
| created_at  |    TEXT |   25 | ISO8601                             |

### 3.12 `compatibility_rule`

| Column           |    Type | Size | Usage                                             |
| ---------------- | ------: | ---: | ------------------------------------------------- |
| id               |    TEXT |  ~36 | UUID PK                                           |
| compat_scope     |    TEXT |   16 | `type_to_type` / `model_to_model` / `bundle_rule` |
| left_model_type  |    TEXT |   32 | For type rules                                    |
| right_model_type |    TEXT |   32 | For type rules                                    |
| left_model_id    |    TEXT |  ~36 | For model rules                                   |
| right_model_id   |    TEXT |  ~36 | For model rules                                   |
| is_compatible    | INTEGER |    1 | 1/0                                               |
| reason           |    TEXT |  var | Human-readable                                    |
| data_json        |    TEXT |  var | JSON constraints                                  |
| created_at       |    TEXT |   25 | ISO8601                                           |

---

## 4) Links between tables (relationship map)

| From               | To                                                | Type     | Meaning                               |
| ------------------ | ------------------------------------------------- | -------- | ------------------------------------- |
| `scan_run.id`      | `file_location.scan_run_id`                       | 1 → many | A scan discovers many locations       |
| `file_blob.id`     | `file_location.blob_id`                           | 1 → many | Same content may appear at many paths |
| `file_location.id` | `model_file.location_id`                          | 1 → many | Location attached to a logical model  |
| `file_blob.id`     | `model_file.blob_id`                              | 1 → many | Models reference blob identity        |
| `model.id`         | `model_file.model_id`                             | 1 → many | A model can have many files/locations |
| `model.id`         | `model_tech.model_id`                             | 1 → 1    | Captured compatibility metadata       |
| `model.id`         | `model_base_prefs.model_id`                       | 1 → 0/1  | Base checkpoint prefs                 |
| `model.id`         | `model_lora_prefs.model_id`                       | 1 → 0/1  | LoRA prefs                            |
| `tag.id`           | `entity_tag.tag_id`                               | 1 → many | Tags applied to models/bundles        |
| `bundle.id`        | `bundle_item.bundle_id`                           | 1 → many | Bundle composition                    |
| `model.id`         | `bundle_item.model_id`                            | 1 → many | A model may be used in many bundles   |
| `model.id`         | `compatibility_rule.left_model_id/right_model_id` | many     | Specific model compatibility          |
| `model`/`bundle`   | `example.entity_id`                               | 1 → many | Examples attached                     |

---

## 5) Technical architecture diagram (Mermaid)

```mermaid
flowchart LR
  subgraph UI[Vue + PrimeVue + Tailwind (Tauri WebView)]
    A[Bundles Tab] -->|select| D[Details Panel]
    B[Models Tab] -->|select| D
    A -->|Scan Bundles| C1[Scan Dialog]
    B -->|Scan Models| C2[Scan Dialog]
    A -->|Create Bundle| W[Bundle Builder Wizard]
  end

  subgraph TAURI[Tauri Host]
    R[Rust Core]
    DB[(SQLite)]
    FS[(Filesystem)]
  end

  UI -->|invoke Tauri commands (JSON)| R
  R -->|read/write| DB
  R -->|scan + resolve symlinks + hash| FS
  R -->|returns DTOs| UI

  subgraph Rules[Classifier & Rules Engine (Rust)]
    CL[Model classifier]
    SYM[Symlink resolver]
    SH[Shard detector/excluder]
    COMP[Compatibility builder]
  end

  R --> Rules
  Rules --> R
```

---

## 6) User stories (prioritized)

### Epic: Scanning

1. **Scan Models (recursive)**

   * As a user, I can select a directory and scan recursively to find supported model files only.
   * Acceptance: unsupported types (ControlNet/IP-Adapter/etc.) are excluded; results sorted A→Z.

2. **Scan Bundles (symlink aware)**

   * As a user, I can scan bundle-like structures (including HF snapshot symlinks) and see the “important files” that define a bundle.
   * Acceptance: symlinks are shown with resolved targets; sharded pieces + shard index configs are excluded from “important files”.

3. **Deduplicate identical blobs**

   * As a user, I should not see duplicates if the same blob is referenced in multiple places.
   * Acceptance: one logical model with multiple locations.

### Epic: Model Management

4. **View model details**

   * As a user, I can click a model and see file paths, hash, size, type, and derived compatibility metadata.

5. **Rename model**

   * As a user, I can change the display name independent of the filename.

6. **Tag any model**

   * As a user, I can add/remove tags to any model type.

7. **LoRA preferences**

   * As a user, for LoRAs I can store preferred strength range/default and trigger words.

8. **Base checkpoint preferences**

   * As a user, for base checkpoints I can store preferred steps and preferred CFG.

9. **Examples**

   * As a user, I can add multiple example images and multiple example prompts to base checkpoints and LoRAs.

### Epic: Bundle Management

10. **Create bundle**

* As a user, I can create a bundle, name it, describe it, tag it, and attach examples.

11. **Select compatible components only**

* As a user, when I pick a base model, I can only choose compatible VAEs/text encoders/etc.
* Acceptance: incompatible options are hidden/disabled; compatibility comes from DB metadata.

12. **Edit bundle composition**

* As a user, I can add/remove components, and set roles (e.g., tokenizer vs tokenizer_2).

### Epic: Persistence & Performance

13. **Compatibility persistence**

* As a user, compatibility decisions are stored so future filtering is instant and does not require re-reading model files.

14. **Scan audit**

* As a user, I can see whether a scan succeeded and view errors if it failed.

---

## 7) API contracts (Tauri commands)

These are the “backend API” contracts between Vue and Rust (Tauri `invoke`). All payloads are JSON.

### 7.1 Common DTOs

```ts
// Shared
type UUID = string; // e.g. "550e8400-e29b-41d4-a716-446655440000"

type TagDTO = { id: UUID; name: string };

type FileLocationDTO = {
  id: UUID;
  path: string;
  location_kind: "real_file" | "symlink";
  symlink_target?: string | null;
  resolved_path: string;
  sha256: string;
  size_bytes: number;
  modified_at?: string | null; // ISO8601
};

type ModelType =
  | "base_checkpoint"
  | "lora"
  | "vae"
  | "text_encoder"
  | "tokenizer"
  | "scheduler"
  | "ae"
  | "other";

type ModelDTO = {
  id: UUID;
  model_type: ModelType;
  display_name: string;
  description?: string | null;
  tags: TagDTO[];
  locations: FileLocationDTO[];
  tech?: {
    format_family?: string | null;
    architecture?: string | null;
    raw_metadata_json?: string | null;
  } | null;
  base_prefs?: { preferred_steps?: number | null; preferred_cfg?: number | null } | null;
  lora_prefs?: {
    preferred_strength_min?: number | null;
    preferred_strength_max?: number | null;
    preferred_strength_default?: number | null;
    trigger_words?: string[] | null;
  } | null;
};

type ExampleDTO = {
  id: UUID;
  example_kind: "image" | "prompt";
  title?: string | null;
  content_text?: string | null;
  image_path?: string | null;
  created_at: string;
};

type BundleItemDTO = {
  id: UUID;
  model_id: UUID;
  role: string;
  is_required: boolean;
  model_summary: { id: UUID; model_type: ModelType; display_name: string };
};

type BundleDTO = {
  id: UUID;
  display_name: string;
  description?: string | null;
  tags: TagDTO[];
  items: BundleItemDTO[];
  examples: ExampleDTO[];
  created_at: string;
  updated_at: string;
};
```

### 7.2 Commands

#### Scanning

```ts
// Scan models under a root path (recursive). Must resolve symlinks.
// Must exclude unsupported model families (ControlNet/IP-Adapter/etc).
type ScanModelsRequest = {
  root_path: string;
};

type ScanModelsResponse = {
  scan_run_id: UUID;
  models_found: number;
  models: ModelDTO[]; // optionally return only summaries and fetch details separately
};

// Scan bundles (repo-like structures). Must respect "important files" rules and exclude sharded files.
type ScanBundlesRequest = {
  root_path: string;
};

type ScanBundlesResponse = {
  scan_run_id: UUID;
  bundles_found: number;
  bundles: BundleDTO[]; // could be summaries
};
```

#### Query lists (sorted A→Z in backend for consistency)

```ts
type ListModelsRequest = {
  model_type?: ModelType | null;
  q?: string | null; // search by display_name/filename/tag
};

type ListModelsResponse = { models: Array<Pick<ModelDTO, "id" | "model_type" | "display_name" | "tags">> };

type ListBundlesRequest = { q?: string | null };

type ListBundlesResponse = { bundles: Array<Pick<BundleDTO, "id" | "display_name" | "tags" | "updated_at">> };
```

#### Read details

```ts
type GetModelRequest = { model_id: UUID };
type GetModelResponse = { model: ModelDTO; examples: ExampleDTO[] };

type GetBundleRequest = { bundle_id: UUID };
type GetBundleResponse = { bundle: BundleDTO };
```

#### Updates (rename, description, prefs, tags, examples)

```ts
type UpdateModelRequest = {
  model_id: UUID;
  display_name?: string;
  description?: string | null;
  base_prefs?: { preferred_steps?: number | null; preferred_cfg?: number | null } | null;
  lora_prefs?: {
    preferred_strength_min?: number | null;
    preferred_strength_max?: number | null;
    preferred_strength_default?: number | null;
    trigger_words?: string[] | null;
  } | null;
};
type UpdateModelResponse = { model: ModelDTO };

type SetEntityTagsRequest = {
  entity_kind: "model" | "bundle";
  entity_id: UUID;
  tag_names: string[]; // backend upserts tag records
};
type SetEntityTagsResponse = { tags: TagDTO[] };

type AddExampleRequest =
  | {
      entity_kind: "model" | "bundle";
      entity_id: UUID;
      example_kind: "prompt";
      title?: string | null;
      content_text: string;
    }
  | {
      entity_kind: "model" | "bundle";
      entity_id: UUID;
      example_kind: "image";
      title?: string | null;
      image_path: string; // local path chosen via file picker; backend may copy to app storage if desired
    };
type AddExampleResponse = { example: ExampleDTO };

type DeleteExampleRequest = { example_id: UUID };
type DeleteExampleResponse = { ok: true };
```

#### Bundle creation / editing with compatibility filtering

```ts
type CreateBundleRequest = { display_name: string; description?: string | null };
type CreateBundleResponse = { bundle: BundleDTO };

type UpdateBundleRequest = { bundle_id: UUID; display_name?: string; description?: string | null };
type UpdateBundleResponse = { bundle: BundleDTO };

type AddBundleItemRequest = {
  bundle_id: UUID;
  model_id: UUID;
  role: string;
  is_required?: boolean;
};
type AddBundleItemResponse = { bundle: BundleDTO };

type RemoveBundleItemRequest = { bundle_item_id: UUID };
type RemoveBundleItemResponse = { bundle: BundleDTO };
```

#### Compatibility queries

```ts
// Given currently selected items (or base model), return allowed candidates for a role.
type GetCompatibleModelsRequest = {
  bundle_id?: UUID | null; // optional context
  selected_model_ids: UUID[]; // current selection
  target_role: string;        // e.g. 'vae' or 'text_encoder'
  target_model_type: ModelType;
};

type GetCompatibleModelsResponse = {
  candidates: Array<Pick<ModelDTO, "id" | "model_type" | "display_name" | "tags">>;
};
```

---

## 8) UI flow design

### Navigation structure

* Left nav (optional): **Bundles** / **Models** / **Settings**
* Main content: tab page with list + details split view

### Bundles tab

1. **Header actions**

   * “Scan Bundles” (folder picker)
   * “Create Bundle”
2. **Bundle list**

   * Sorted A→Z
   * Shows: display name + tag chips + updated date
3. **Bundle details panel**

   * Editable: display name, description, tags
   * Sections:

     * Components (group by role)
     * Examples (images gallery + prompts list)
   * “Edit Components” opens **Bundle Builder Wizard**

### Bundle Builder Wizard (compatibility enforced)

* Step 1: Choose Base Checkpoint (required)
* Step 2: Choose VAE (filtered by compatibility)
* Step 3: Choose Text Encoders (filtered)
* Step 4: Choose Tokenizers (filtered)
* Step 5: Choose Scheduler + AE (filtered)
* Step 6: Review + Save

Filtering behavior:

* As soon as base model selected, all component pickers query `getCompatibleModels(...)`
* If the user changes the base model, downstream selections that are no longer compatible are cleared with a warning toast.

### Models tab

1. **Header actions**

   * “Scan Models” (folder picker)
2. **Model list**

   * Grouped by type (accordion or grouped list)
   * Within each group sorted A→Z
3. **Model details panel**

   * File locations (symlink + resolved path shown)
   * Hash, size, modified time
   * Editable:

     * Display name
     * Description
     * Tags
     * Type-specific prefs
     * Examples (base + lora)

---

## 9) Implementation plan (high-level, pragmatic)

### Phase 1 — Foundations

* Set up Rust crates/modules:

  * `db` (SQLite connection, migrations, repositories)
  * `scanner` (filesystem walk + symlink resolver + hashing)
  * `classifier` (rules for inclusion/exclusion + shard detection)
  * `compat` (compatibility builder + query)
  * `tauri_api` (commands + DTO mapping)
* Set up Vue:

  * PrimeVue layout + Tailwind
  * State management (Pinia recommended)

### Phase 2 — Scanning + persistence

* Implement:

  * Recursive scan with symlink awareness
  * Hashing (sha256) + blob & location tables
  * Classification rules:

    * recognize “important files” per known bundle families
    * detect shards and set `is_shard=1` and `is_important=0`
    * exclude ControlNet/IP-Adapter/etc.
  * Deduplicate by blob hash
* Provide list + details endpoints

### Phase 3 — Editing + examples + tags

* Implement:

  * update model/bundle metadata
  * tag upsert + linking
  * add/remove examples

### Phase 4 — Bundles + compatibility

* Implement:

  * bundle CRUD
  * compatibility_rule persistence
  * candidate filtering endpoint
  * bundle builder wizard UI

### Phase 5 — Polish

* Progress UI for scans (optional incremental events)
* Error surfaces + scan logs
* Performance: incremental hashing, skip unchanged files by (size, mtime) quick checks

Below is a **greenfield Rust module skeleton** for your Tauri backend that matches the API contracts you defined, plus **Mermaid ER diagrams** for an SQLite schema that supports models, bundles, tags, examples, scanning runs, and persisted compatibility.

---

# Rust module skeleton (Tauri backend)

## Suggested file tree

```text
src-tauri/
  src/
    lib.rs
    main.rs
    error.rs
    state.rs

    tauri_api/
      mod.rs
      scan.rs
      models.rs
      bundles.rs
      tags_examples.rs

    db/
      mod.rs
      migrations.rs
      connection.rs
      types.rs
      repositories/
        mod.rs
        scan_run_repo.rs
        model_repo.rs
        bundle_repo.rs
        tag_repo.rs
        example_repo.rs
        compat_repo.rs

    domain/
      mod.rs
      models.rs
      bundles.rs
      compatibility.rs
      scanning.rs

    services/
      mod.rs
      scan_service.rs
      model_service.rs
      bundle_service.rs
      compat_service.rs

    scanner/
      mod.rs
      fs_walk.rs
      symlink.rs
      filters.rs
      hashing.rs
      discovery.rs

    classifier/
      mod.rs
      flux.rs
      rules.rs
```

> Notes:
>
> * This is designed to be “clean architecture-ish”: Tauri commands call services; services call repos; scanner/classifier are pure-ish helpers.
> * Uses `rusqlite` for SQLite (lightweight, common for Tauri). If you prefer `sqlx`, say so and I’ll refit.

---

## `src-tauri/src/error.rs`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("db error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;
```

---

## `src-tauri/src/state.rs`

```rust
use std::sync::{Arc, Mutex};

use crate::db::connection::Db;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Db>>,
}
```

---

## `src-tauri/src/db/connection.rs`

```rust
use rusqlite::Connection;

pub struct Db {
    pub conn: Connection,
}

impl Db {
    pub fn open(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        // You probably want these:
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        Ok(Self { conn })
    }
}
```

---

## `src-tauri/src/db/types.rs` (DB-facing structs)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbModel {
    pub id: String,
    pub model_type: String,
    pub family: String,
    pub display_name: String,
    pub description: Option<String>,
    pub primary_file_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbModelFile {
    pub id: String,
    pub model_id: String,
    pub path: String,              // symlink path (as found)
    pub resolved_path: String,     // realpath
    pub sha256: Option<String>,
    pub size_bytes: Option<i64>,
    pub mtime_unix: Option<i64>,
    pub is_symlink: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbBundle {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbBundleItem {
    pub id: String,
    pub bundle_id: String,
    pub model_id: String,
    pub role: String, // e.g. "base_checkpoint", "vae", "text_encoder"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbTag {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbExample {
    pub id: String,
    pub entity_type: String, // "model" | "bundle"
    pub entity_id: String,
    pub example_type: String, // "image" | "prompt"
    pub content: String, // file path (image) or prompt text
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbCompatibilityEdge {
    pub id: String,
    pub model_a_id: String,
    pub model_b_id: String,
    pub relation: String, // e.g. "compatible", "incompatible"
    pub reason: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbScanRun {
    pub id: String,
    pub scan_type: String,   // "models" | "bundles"
    pub root_path: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String,      // "running" | "completed" | "failed"
    pub error_message: Option<String>,
}
```

---

## `src-tauri/src/db/repositories/mod.rs`

```rust
pub mod scan_run_repo;
pub mod model_repo;
pub mod bundle_repo;
pub mod tag_repo;
pub mod example_repo;
pub mod compat_repo;
```

---

## `src-tauri/src/db/repositories/model_repo.rs` (skeleton)

```rust
use crate::{db::types::DbModel, error::AppResult};
use rusqlite::{params, Connection};

pub struct ModelRepo<'a> {
    conn: &'a Connection,
}

impl<'a> ModelRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn list_models(&self, model_type: Option<&str>, query: Option<&str>) -> AppResult<Vec<DbModel>> {
        // Skeleton: build query dynamically
        let mut sql = String::from(
            "SELECT id, model_type, family, display_name, description, primary_file_id, created_at, updated_at
             FROM models WHERE 1=1"
        );

        let mut args: Vec<Box<dyn rusqlite::ToSql>> = vec![];

        if let Some(mt) = model_type {
            sql.push_str(" AND model_type = ?");
            args.push(Box::new(mt.to_string()));
        }

        if let Some(q) = query {
            sql.push_str(" AND (display_name LIKE ? OR description LIKE ?)");
            let like = format!("%{}%", q);
            args.push(Box::new(like.clone()));
            args.push(Box::new(like));
        }

        sql.push_str(" ORDER BY display_name COLLATE NOCASE ASC");

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(args.iter()), |r| {
            Ok(DbModel {
                id: r.get(0)?,
                model_type: r.get(1)?,
                family: r.get(2)?,
                display_name: r.get(3)?,
                description: r.get(4)?,
                primary_file_id: r.get(5)?,
                created_at: r.get(6)?,
                updated_at: r.get(7)?,
            })
        })?;

        let mut out = vec![];
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn get_model(&self, model_id: &str) -> AppResult<DbModel> {
        let mut stmt = self.conn.prepare(
            "SELECT id, model_type, family, display_name, description, primary_file_id, created_at, updated_at
             FROM models WHERE id = ?1"
        )?;
        let m = stmt.query_row(params![model_id], |r| {
            Ok(DbModel {
                id: r.get(0)?,
                model_type: r.get(1)?,
                family: r.get(2)?,
                display_name: r.get(3)?,
                description: r.get(4)?,
                primary_file_id: r.get(5)?,
                created_at: r.get(6)?,
                updated_at: r.get(7)?,
            })
        })?;
        Ok(m)
    }

    pub fn update_model(
        &self,
        model_id: &str,
        display_name: Option<&str>,
        description: Option<Option<&str>>,
    ) -> AppResult<()> {
        // Skeleton: you’ll likely want a proper patch builder
        if let Some(name) = display_name {
            self.conn.execute(
                "UPDATE models SET display_name = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![name, model_id],
            )?;
        }

        if let Some(desc_patch) = description {
            self.conn.execute(
                "UPDATE models SET description = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![desc_patch, model_id],
            )?;
        }

        Ok(())
    }
}
```

---

## Domain + API DTOs (matches your API contracts)

### `src-tauri/src/domain/models.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSummary {
    pub model_id: String,
    pub model_type: String,
    pub family: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDetail {
    pub model_id: String,
    pub model_type: String,
    pub family: String,
    pub display_name: String,
    pub description: Option<String>,
    pub files: Vec<ModelFileDetail>,
    pub tags: Vec<String>,
    pub base_prefs: Option<BasePrefs>,
    pub lora_prefs: Option<LoraPrefs>,
    pub examples: Vec<ExampleItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFileDetail {
    pub file_id: String,
    pub path: String,
    pub resolved_path: String,
    pub sha256: Option<String>,
    pub size_bytes: Option<i64>,
    pub mtime_unix: Option<i64>,
    pub is_symlink: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasePrefs {
    pub preferred_steps: Option<i32>,
    pub preferred_cfg: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraPrefs {
    pub strength_min: Option<f32>,
    pub strength_max: Option<f32>,
    pub strength_default: Option<f32>,
    pub trigger_words: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleItem {
    pub example_id: String,
    pub example_type: String, // "image" | "prompt"
    pub content: String,
}
```

---

## Tauri command layer (API surface)

### `src-tauri/src/tauri_api/mod.rs`

```rust
pub mod scan;
pub mod models;
pub mod bundles;
pub mod tags_examples;
```

### `src-tauri/src/tauri_api/scan.rs`

```rust
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    domain::{bundles::BundleSummary, models::ModelSummary},
    error::AppResult,
    services::scan_service::ScanService,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub root_path: String,
}

#[derive(Debug, Serialize)]
pub struct ScanModelsResponse {
    pub scan_run_id: String,
    pub models: Vec<ModelSummary>,
}

#[derive(Debug, Serialize)]
pub struct ScanBundlesResponse {
    pub scan_run_id: String,
    pub bundles: Vec<BundleSummary>,
}

#[tauri::command]
pub fn scan_models(state: State<'_, AppState>, req: ScanRequest) -> Result<ScanModelsResponse, String> {
    let mut db = state.db.lock().unwrap();
    let svc = ScanService::new(&mut db.conn);

    svc.scan_models(&req.root_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn scan_bundles(state: State<'_, AppState>, req: ScanRequest) -> Result<ScanBundlesResponse, String> {
    let mut db = state.db.lock().unwrap();
    let svc = ScanService::new(&mut db.conn);

    svc.scan_bundles(&req.root_path)
        .map_err(|e| e.to_string())
}
```

### `src-tauri/src/tauri_api/models.rs`

```rust
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    domain::models::{ModelDetail, ModelSummary, BasePrefs, LoraPrefs},
    services::model_service::ModelService,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListModelsRequest {
    pub model_type: Option<String>,
    pub query: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetModelRequest {
    pub model_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateModelRequest {
    pub model_id: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub base_prefs: Option<BasePrefs>,
    pub lora_prefs: Option<LoraPrefs>,
}

#[tauri::command]
pub fn list_models(state: State<'_, AppState>, req: ListModelsRequest) -> Result<Vec<ModelSummary>, String> {
    let mut db = state.db.lock().unwrap();
    let svc = ModelService::new(&mut db.conn);

    svc.list_models(req.model_type.as_deref(), req.query.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_model(state: State<'_, AppState>, req: GetModelRequest) -> Result<ModelDetail, String> {
    let mut db = state.db.lock().unwrap();
    let svc = ModelService::new(&mut db.conn);

    svc.get_model(&req.model_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_model(state: State<'_, AppState>, req: UpdateModelRequest) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    let svc = ModelService::new(&mut db.conn);

    svc.update_model(req).map_err(|e| e.to_string())
}
```

### `src-tauri/src/tauri_api/bundles.rs` (skeleton)

```rust
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    domain::bundles::{BundleDetail, BundleSummary},
    services::bundle_service::BundleService,
    services::compat_service::CompatService,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateBundleRequest {
    pub display_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateBundleResponse {
    pub bundle_id: String,
}

#[derive(Debug, Deserialize)]
pub struct AddBundleItemRequest {
    pub bundle_id: String,
    pub model_id: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct GetBundleRequest {
    pub bundle_id: String,
}

#[derive(Debug, Deserialize)]
pub struct GetCompatibleModelsRequest {
    pub selected_model_ids: Vec<String>,
    pub target_model_type: String,
}

#[tauri::command]
pub fn create_bundle(state: State<'_, AppState>, req: CreateBundleRequest) -> Result<CreateBundleResponse, String> {
    let mut db = state.db.lock().unwrap();
    let svc = BundleService::new(&mut db.conn);

    let id = svc.create_bundle(&req.display_name, req.description.as_deref())
        .map_err(|e| e.to_string())?;

    Ok(CreateBundleResponse { bundle_id: id })
}

#[tauri::command]
pub fn add_bundle_item(state: State<'_, AppState>, req: AddBundleItemRequest) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    let svc = BundleService::new(&mut db.conn);

    svc.add_bundle_item(&req.bundle_id, &req.model_id, &req.role)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_bundle(state: State<'_, AppState>, req: GetBundleRequest) -> Result<BundleDetail, String> {
    let mut db = state.db.lock().unwrap();
    let svc = BundleService::new(&mut db.conn);

    svc.get_bundle(&req.bundle_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_compatible_models(state: State<'_, AppState>, req: GetCompatibleModelsRequest) -> Result<Vec<crate::domain::models::ModelSummary>, String> {
    let mut db = state.db.lock().unwrap();
    let svc = CompatService::new(&mut db.conn);

    svc.get_compatible_models(&req.selected_model_ids, &req.target_model_type)
        .map_err(|e| e.to_string())
}
```

---

## `src-tauri/src/services/scan_service.rs` (skeleton with symlink-safe scanning hooks)

```rust
use rusqlite::Connection;
use uuid::Uuid;

use crate::{
    domain::{models::ModelSummary, bundles::BundleSummary},
    error::{AppResult, AppError},
    scanner::discovery,
};

pub struct ScanService<'a> {
    conn: &'a Connection,
}

impl<'a> ScanService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn scan_models(&self, root_path: &str) -> AppResult<crate::tauri_api::scan::ScanModelsResponse> {
        if root_path.trim().is_empty() {
            return Err(AppError::InvalidInput("root_path is empty".into()));
        }

        let scan_run_id = Uuid::new_v4().to_string();

        // 1) Record scan_run started (repo call)
        // 2) Walk filesystem, resolve symlinks, filter, classify, persist
        let discovered = discovery::scan_models(root_path)?;

        // Persist discovered -> models/files/tags compatibility etc.
        // Return sorted summaries
        let models: Vec<ModelSummary> = discovered
            .into_iter()
            .map(|d| ModelSummary {
                model_id: d.model_id,
                model_type: d.model_type,
                family: d.family,
                display_name: d.display_name,
            })
            .collect();

        Ok(crate::tauri_api::scan::ScanModelsResponse { scan_run_id, models })
    }

    pub fn scan_bundles(&self, root_path: &str) -> AppResult<crate::tauri_api::scan::ScanBundlesResponse> {
        let scan_run_id = Uuid::new_v4().to_string();

        // Similar: use a bundle-aware discovery routine that:
        // - resolves symlinks
        // - excludes sharded files and shard configs
        // - groups required files into “bundle candidates”
        let bundles_found = discovery::scan_bundles(root_path)?;

        let bundles: Vec<BundleSummary> = bundles_found
            .into_iter()
            .map(|b| BundleSummary {
                bundle_id: b.bundle_id,
                display_name: b.display_name,
            })
            .collect();

        Ok(crate::tauri_api::scan::ScanBundlesResponse { scan_run_id, bundles })
    }
}
```

---

## `src-tauri/src/scanner/discovery.rs` (key: symlink resolution + filters)

```rust
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct DiscoveredModel {
    pub model_id: String,
    pub model_type: String,
    pub family: String,
    pub display_name: String,
    // plus discovered files, hashes, etc.
}

#[derive(Debug, Clone)]
pub struct DiscoveredBundle {
    pub bundle_id: String,
    pub display_name: String,
    // plus components
}

pub fn scan_models(root: &str) -> AppResult<Vec<DiscoveredModel>> {
    // TODO:
    // 1) walk recursively (follow symlinks)
    // 2) filter out ControlNet/IPAdapter/etc.
    // 3) ignore shards and shard configs
    // 4) classify by filename + folder conventions + lightweight probing
    // 5) build DiscoveredModel(s)
    // 6) sort alphabetically by display_name (case-insensitive)

    if root.trim().is_empty() {
        return Err(AppError::InvalidInput("root path empty".into()));
    }

    Ok(vec![]) // placeholder
}

pub fn scan_bundles(root: &str) -> AppResult<Vec<DiscoveredBundle>> {
    // TODO:
    // - detect “bundle repo” patterns (HF snapshots with symlinks)
    // - identify required important files (exclude shards / shard configs)
    // - group into DiscoveredBundle(s)
    // - sort alphabetically
    Ok(vec![])
}
```

---

# Mermaid ER Diagrams for SQLite

You asked for ER diagrams; these match the capabilities you described (models, bundles, compatibility, tags, examples, scan runs, and file/symlink tracking).

## 1) `mermaid-er-diagram.md`

````markdown
# SQLite ER Diagram

```mermaid
erDiagram
  SCAN_RUNS {
    TEXT id PK
    TEXT scan_type "models|bundles"
    TEXT root_path
    TEXT started_at
    TEXT finished_at
    TEXT status "running|completed|failed"
    TEXT error_message
  }

  MODELS {
    TEXT id PK
    TEXT model_type "checkpoint|lora|vae|text_encoder|tokenizer|scheduler|ae"
    TEXT family "flux|sd15|sdxl|other"
    TEXT display_name
    TEXT description
    TEXT primary_file_id FK
    TEXT created_at
    TEXT updated_at
  }

  MODEL_FILES {
    TEXT id PK
    TEXT model_id FK
    TEXT path "as found (may be symlink)"
    TEXT resolved_path "realpath"
    TEXT sha256
    INTEGER size_bytes
    INTEGER mtime_unix
    INTEGER is_symlink "0|1"
  }

  MODEL_PREFS_BASE {
    TEXT model_id PK, FK
    INTEGER preferred_steps
    REAL preferred_cfg
  }

  MODEL_PREFS_LORA {
    TEXT model_id PK, FK
    REAL strength_min
    REAL strength_max
    REAL strength_default
  }

  LORA_TRIGGER_WORDS {
    TEXT id PK
    TEXT model_id FK
    TEXT trigger_word
  }

  TAGS {
    TEXT id PK
    TEXT name UNIQUE
  }

  MODEL_TAGS {
    TEXT model_id FK
    TEXT tag_id FK
    TEXT created_at
  }

  BUNDLES {
    TEXT id PK
    TEXT display_name
    TEXT description
    TEXT created_at
    TEXT updated_at
  }

  BUNDLE_ITEMS {
    TEXT id PK
    TEXT bundle_id FK
    TEXT model_id FK
    TEXT role "base|lora|vae|encoder|tokenizer|scheduler|ae"
    INTEGER sort_order
  }

  BUNDLE_TAGS {
    TEXT bundle_id FK
    TEXT tag_id FK
    TEXT created_at
  }

  EXAMPLES {
    TEXT id PK
    TEXT entity_type "model|bundle"
    TEXT entity_id
    TEXT example_type "image|prompt"
    TEXT content "filepath or prompt text"
    TEXT created_at
  }

  COMPATIBILITY_EDGES {
    TEXT id PK
    TEXT model_a_id FK
    TEXT model_b_id FK
    TEXT relation "compatible|incompatible"
    TEXT reason
    TEXT created_at
  }

  %% Relationships
  MODELS ||--o{ MODEL_FILES : "has"
  MODELS ||--o| MODEL_PREFS_BASE : "base prefs"
  MODELS ||--o| MODEL_PREFS_LORA : "lora prefs"
  MODELS ||--o{ LORA_TRIGGER_WORDS : "triggers"

  MODELS ||--o{ MODEL_TAGS : "tagged with"
  TAGS ||--o{ MODEL_TAGS : "applies to"

  BUNDLES ||--o{ BUNDLE_ITEMS : "contains"
  MODELS ||--o{ BUNDLE_ITEMS : "included in"

  BUNDLES ||--o{ BUNDLE_TAGS : "tagged with"
  TAGS ||--o{ BUNDLE_TAGS : "applies to"

  MODELS ||--o{ COMPATIBILITY_EDGES : "compat a"
  MODELS ||--o{ COMPATIBILITY_EDGES : "compat b"
````

### Why this ER design fits your requirements

- **Symlink-aware**: `MODEL_FILES.path` vs `resolved_path`
- **No re-parsing**: `COMPATIBILITY_EDGES` persists decisions
- **Type-specific preferences**: separate `MODEL_PREFS_BASE` + `MODEL_PREFS_LORA` + triggers table
- **Tags everywhere**: models + bundles
- **Examples everywhere**: models + bundles, prompts or images
- **Sorted display**: `BUNDLE_ITEMS.sort_order` supports deterministic ordering
