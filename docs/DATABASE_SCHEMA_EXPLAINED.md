# Model Bundle Database Schema Explained

## Overview

The bundle system uses **3 tables** to manage model components and their relationships:

```
model_components  ─────┐
  (physical files)     │
                       ├──→  bundle_components  ←──┐
                       │      (relationships)       │
model_bundles     ─────┘                            │
  (logical groups) ────────────────────────────────┘
```

## Table Purposes

### 1. `model_components` - Physical Files

**What it stores:** Individual model files discovered on disk

**Think of it as:** "A catalog of every model file on your computer"

**Examples:**
- `flux1-dev.safetensors` (23 GB transformer)
- `ae.safetensors` (335 MB VAE)
- `model.safetensors` (500 MB CLIP encoder)
- `t5-encoder-Q5_K_M.gguf` (3 GB T5 encoder, quantized)

**Key fields:**
```sql
id                TEXT PRIMARY KEY         -- UUID for this component
component_type    TEXT NOT NULL            -- "transformer", "vae", "t5_encoder", "clip_encoder"
file_path         TEXT NOT NULL UNIQUE     -- /home/user/.cache/huggingface/.../flux1-dev.safetensors
file_hash         TEXT                     -- SHA256 hash for deduplication
repo_id           TEXT                     -- "black-forest-labs/FLUX.1-dev"
quantization      TEXT                     -- "Q5_K_M", "Q8_0", or NULL for full precision
vram_mb           INTEGER                  -- Estimated VRAM usage
is_available      INTEGER                  -- 1 if file exists, 0 if deleted
```

**One row = One physical file on disk**

---

### 2. `model_bundles` - Logical Groupings

**What it stores:** Named collections of components that work together

**Think of it as:** "Complete model configurations ready to use"

**Examples:**
- "FLUX.1 Schnell" bundle (transformer + T5 + CLIP + VAE from same repo)
- "FLUX.1 Dev" bundle (different transformer, same encoders/VAE)
- "Custom Mix" bundle (user-created: dev transformer + quantized T5)

**Key fields:**
```sql
id                TEXT PRIMARY KEY         -- UUID for this bundle
name              TEXT NOT NULL            -- "FLUX.1 Schnell", "My Custom Setup"
bundle_type       TEXT NOT NULL            -- "auto_discovered", "user_created", "system"
model_family      TEXT NOT NULL            -- "flux", "zindex"
is_complete       INTEGER                  -- 1 if has all 4 required components
is_active         INTEGER                  -- 1 if currently selected for generation
total_vram_mb     INTEGER                  -- Sum of all component VRAM
default_steps     INTEGER                  -- Default generation steps
default_guidance  REAL                     -- Default guidance scale
```

**One row = One complete model configuration**

---

### 3. `bundle_components` - Join Table (Relationships)

**What it stores:** Which components belong to which bundles, and their roles

**Think of it as:** "The mapping that connects physical files to bundles"

**Key fields:**
```sql
id                INTEGER PRIMARY KEY AUTOINCREMENT
bundle_id         TEXT NOT NULL            -- FK → model_bundles.id
component_id      TEXT NOT NULL            -- FK → model_components.id
component_role    TEXT NOT NULL            -- "transformer", "t5", "clip", "vae"
is_required       INTEGER                  -- 1 if this component is mandatory
priority          INTEGER                  -- Loading order (lower = earlier)

FOREIGN KEY (bundle_id) REFERENCES model_bundles(id) ON DELETE CASCADE
FOREIGN KEY (component_id) REFERENCES model_components(id) ON DELETE CASCADE
UNIQUE (bundle_id, component_role, component_id)
```

**One row = One component's role in one bundle**

---

## Relationships Explained

### The Pattern: Many-to-Many with Roles

```
┌─────────────────────┐
│  model_components   │  Physical files (can be reused)
│  ─────────────────  │
│  • flux1-dev.st     │────┐
│  • flux1-schnell.st │────┤
│  • ae.safetensors   │────┼────┐
│  • clip/model.st    │────┼────┤
│  • t5-encoder.gguf  │────┼────┼────┐
└─────────────────────┘    │    │    │
                           │    │    │
                           ▼    ▼    ▼
                    ┌──────────────────────────┐
                    │  bundle_components       │  Relationships
                    │  ────────────────────────│
                    │  bundle_id | component_id | role        │
                    │  ──────────────────────────────────────│
                    │  bundle_1  | flux1-dev    | transformer │
                    │  bundle_1  | ae.st        | vae         │
                    │  bundle_1  | clip/model   | clip        │
                    │  bundle_1  | t5-encoder   | t5          │
                    │  ──────────────────────────────────────│
                    │  bundle_2  | flux-schnell | transformer │
                    │  bundle_2  | ae.st        | vae         │  ← SAME VAE
                    │  bundle_2  | clip/model   | clip        │  ← SAME CLIP
                    │  bundle_2  | t5-encoder   | t5          │  ← SAME T5
                    └──────────────────────────┘
                           │    │
                           ▼    ▼
                    ┌──────────────────────┐
                    │  model_bundles       │  Logical groups
                    │  ────────────────────│
                    │  • bundle_1: "FLUX.1 Dev"     (is_active=1) │
                    │  • bundle_2: "FLUX.1 Schnell" (is_active=0) │
                    └──────────────────────┘
```

### Key Insight: Component Reuse

**The same component can be used in multiple bundles:**

```
Example: ae.safetensors (FLUX VAE)
  ├─ Used in "FLUX.1 Dev" bundle (role: vae)
  ├─ Used in "FLUX.1 Schnell" bundle (role: vae)
  └─ Used in "Custom Mix" bundle (role: vae)

This is why they're separate tables!
```

## Concrete Example

### Scenario: Two FLUX bundles sharing components

**model_components table:**
```
┌────────────┬───────────────┬──────────────────────────────────────────────┬──────────┐
│ id         │ component_type│ file_path                                    │ vram_mb  │
├────────────┼───────────────┼──────────────────────────────────────────────┼──────────┤
│ comp-001   │ transformer   │ .../FLUX.1-dev/flux1-dev.safetensors        │ 23000    │
│ comp-002   │ transformer   │ .../FLUX.1-schnell/flux1-schnell.safetensors│ 23000    │
│ comp-003   │ vae           │ .../FLUX.1-dev/ae.safetensors               │ 335      │
│ comp-004   │ clip_encoder  │ .../FLUX.1-dev/text_encoder/model.st        │ 500      │
│ comp-005   │ t5_encoder    │ .../FLUX.1-dev/text_encoder_2/model-*.st    │ 9000     │
└────────────┴───────────────┴──────────────────────────────────────────────┴──────────┘
```

**model_bundles table:**
```
┌──────────┬──────────────────┬─────────────────┬──────────────┬───────────┬───────────┐
│ id       │ name             │ bundle_type     │ model_family │ is_complete│ is_active │
├──────────┼──────────────────┼─────────────────┼──────────────┼───────────┼───────────┤
│ bundle-A │ FLUX.1 Dev       │ auto_discovered │ flux         │ 1          │ 1         │
│ bundle-B │ FLUX.1 Schnell   │ auto_discovered │ flux         │ 1          │ 0         │
└──────────┴──────────────────┴─────────────────┴──────────────┴───────────┴───────────┘
```

**bundle_components table (the mapping):**
```
┌───────────┬──────────────┬─────────────┬───────────┬────────┐
│ bundle_id │ component_id │ role        │ is_required│ priority│
├───────────┼──────────────┼─────────────┼───────────┼────────┤
│ bundle-A  │ comp-001     │ transformer │ 1         │ 0      │  ← FLUX.1 Dev transformer
│ bundle-A  │ comp-003     │ vae         │ 1         │ 3      │  ← Shared VAE
│ bundle-A  │ comp-004     │ clip        │ 1         │ 2      │  ← Shared CLIP
│ bundle-A  │ comp-005     │ t5          │ 1         │ 1      │  ← Shared T5
├───────────┼──────────────┼─────────────┼───────────┼────────┤
│ bundle-B  │ comp-002     │ transformer │ 1         │ 0      │  ← FLUX.1 Schnell transformer
│ bundle-B  │ comp-003     │ vae         │ 1         │ 3      │  ← SAME VAE as bundle-A!
│ bundle-B  │ comp-004     │ clip        │ 1         │ 2      │  ← SAME CLIP as bundle-A!
│ bundle-B  │ comp-005     │ t5          │ 1         │ 1      │  ← SAME T5 as bundle-A!
└───────────┴──────────────┴─────────────┴───────────┴────────┘
```

### Querying Example

**To get all components in a bundle:**
```sql
SELECT 
    mc.id,
    mc.component_type,
    mc.file_path,
    bc.component_role,
    mc.vram_mb
FROM model_bundles mb
JOIN bundle_components bc ON bc.bundle_id = mb.id
JOIN model_components mc ON mc.id = bc.component_id
WHERE mb.id = 'bundle-A'
ORDER BY bc.priority;
```

**Result:**
```
comp-001 | transformer  | .../flux1-dev.safetensors        | transformer | 23000
comp-005 | t5_encoder   | .../text_encoder_2/model-*.st    | t5          | 9000
comp-004 | clip_encoder | .../text_encoder/model.st        | clip        | 500
comp-003 | vae          | .../ae.safetensors               | vae         | 335
```

## Why This Design?

### Problem It Solves

**Without bundle system (old approach):**
```
❌ Had to store duplicate encoder/VAE data for each model
❌ Couldn't mix and match components
❌ Downloading FLUX.1-dev AND FLUX.1-schnell = duplicate VAE/CLIP/T5
```

**With bundle system:**
```
✅ Component files stored once, referenced many times
✅ Can create custom bundles (e.g., Dev transformer + quantized T5)
✅ Deduplication via file_hash (same file in multiple repos = one entry)
✅ Easy to swap components (just update bundle_components)
```

### Real-World Benefit

**Disk space savings:**
```
Without deduplication:
  FLUX.1-dev full set:     32 GB (transformer + VAE + CLIP + T5)
  FLUX.1-schnell full set: 32 GB (different transformer, duplicate encoders)
  Total:                   64 GB

With bundle system:
  Transformer (dev):       23 GB
  Transformer (schnell):   23 GB
  VAE (shared):            0.3 GB  ← Stored once
  CLIP (shared):           0.5 GB  ← Stored once
  T5 (shared):             9 GB    ← Stored once
  Total:                   ~56 GB  (8 GB saved from deduplication)
```

## Table Relationships (SQL)

```sql
-- One-to-Many: Bundle HAS MANY component relationships
model_bundles.id ──< bundle_components.bundle_id

-- One-to-Many: Component CAN BE USED IN MANY bundles
model_components.id ──< bundle_components.component_id

-- Many-to-Many: Bundles and Components (via bundle_components)
model_bundles ──< bundle_components >── model_components
```

## Component Roles

The `component_role` field in `bundle_components` defines how a component is used:

```
Required roles for a complete FLUX bundle:
├─ "transformer"  - Main diffusion model (FLUX transformer)
├─ "t5"           - T5-XXL text encoder (long prompts)
├─ "clip"         - CLIP text encoder (short prompts)
└─ "vae"          - VAE decoder (latent → RGB image)

Optional roles:
├─ "t5_tokenizer"   - T5 tokenizer (usually auto-loaded)
└─ "clip_tokenizer" - CLIP tokenizer (usually auto-loaded)
```

**A bundle is `is_complete=1` when it has all 4 required roles.**

## Practical Examples

### Example 1: Auto-Discovered Bundle

When you download `black-forest-labs/FLUX.1-dev`:

**1. Scanner finds components:**
```
🔍 Scanning: black-forest-labs/FLUX.1-dev
Found: flux1-dev.safetensors (transformer)
Found: ae.safetensors (vae)
Found: text_encoder/model.safetensors (clip)
Found: text_encoder_2/model-*.safetensors (t5, sharded)
```

**2. Inserts into `model_components`:**
```
INSERT INTO model_components VALUES
  ('comp-dev-tf',  'transformer',  '/path/to/flux1-dev.safetensors', ...),
  ('comp-dev-vae', 'vae',          '/path/to/ae.safetensors', ...),
  ('comp-dev-clip','clip_encoder', '/path/to/text_encoder/model.st', ...),
  ('comp-dev-t5',  't5_encoder',   '/path/to/text_encoder_2/', ...);
```

**3. Creates bundle in `model_bundles`:**
```
INSERT INTO model_bundles VALUES
  ('bundle-flux-dev', 'FLUX.1 Dev', 'auto_discovered', 'flux', 1, 0, ...);
```

**4. Links components in `bundle_components`:**
```
INSERT INTO bundle_components VALUES
  (NULL, 'bundle-flux-dev', 'comp-dev-tf',   'transformer', 1, 0),
  (NULL, 'bundle-flux-dev', 'comp-dev-t5',   't5',          1, 1),
  (NULL, 'bundle-flux-dev', 'comp-dev-clip', 'clip',        1, 2),
  (NULL, 'bundle-flux-dev', 'comp-dev-vae',  'vae',         1, 3);
```

### Example 2: User-Created Custom Bundle

User wants to create a custom bundle mixing components:

**User action:** "Create bundle with dev transformer but quantized T5"

**Database operations:**

**1. Query available components:**
```sql
SELECT * FROM model_components 
WHERE component_type = 'transformer' AND architecture = 'flux-dev';
-- Returns: comp-dev-tf

SELECT * FROM model_components 
WHERE component_type = 't5_encoder' AND quantization = 'Q5_K_M';
-- Returns: comp-t5-quant
```

**2. Create new bundle:**
```sql
INSERT INTO model_bundles VALUES
  ('bundle-custom', 'Dev + Quantized T5', 'user_created', 'flux', 1, 0, ...);
```

**3. Link components with roles:**
```sql
INSERT INTO bundle_components VALUES
  (NULL, 'bundle-custom', 'comp-dev-tf',    'transformer', 1, 0),
  (NULL, 'bundle-custom', 'comp-t5-quant',  't5',          1, 1),
  (NULL, 'bundle-custom', 'comp-dev-clip',  'clip',        1, 2),
  (NULL, 'bundle-custom', 'comp-dev-vae',   'vae',         1, 3);
```

**Result:** Custom bundle reuses 3 existing components, only T5 is different!

### Example 3: Deduplication in Action

**Scenario:** User downloads same model from two repos

```
Repos:
  black-forest-labs/FLUX.1-dev  (contains ae.safetensors)
  comfyanonymous/FLUX.1-dev     (mirror, contains same ae.safetensors)
```

**Scanner behavior:**

**1. Scan repo 1:**
```
Find ae.safetensors → Compute SHA256: abc123def...
INSERT INTO model_components (id='comp-vae-1', file_hash='abc123def', ...)
```

**2. Scan repo 2:**
```
Find ae.safetensors → Compute SHA256: abc123def...
Try INSERT → FAILS (file_hash already exists)
Use existing comp-vae-1 instead
```

**3. Both bundles reference the same component:**
```
bundle_components:
  bundle-repo1 → comp-vae-1 (role: vae)
  bundle-repo2 → comp-vae-1 (role: vae)  ← Same component ID!
```

**Result:** One VAE file, two bundles, no duplication!

## Query Patterns

### Get all components in active bundle
```sql
SELECT 
    bc.component_role,
    mc.component_type,
    mc.file_path,
    mc.vram_mb
FROM model_bundles mb
JOIN bundle_components bc ON bc.bundle_id = mb.id
JOIN model_components mc ON mc.id = bc.component_id
WHERE mb.is_active = 1
ORDER BY bc.priority;
```

### Find bundles using a specific component
```sql
SELECT 
    mb.name,
    mb.bundle_type,
    bc.component_role
FROM model_components mc
JOIN bundle_components bc ON bc.component_id = mc.id
JOIN model_bundles mb ON mb.id = bc.bundle_id
WHERE mc.file_path LIKE '%flux1-dev.safetensors';
```

### Check if bundle is complete
```sql
SELECT 
    mb.name,
    COUNT(DISTINCT bc.component_role) as role_count,
    CASE 
        WHEN COUNT(DISTINCT bc.component_role) >= 4 THEN 'complete'
        ELSE 'incomplete'
    END as status
FROM model_bundles mb
LEFT JOIN bundle_components bc ON bc.bundle_id = mb.id
WHERE bc.is_required = 1
GROUP BY mb.id, mb.name;
```

## Summary

**Three-table design:**
1. **`model_components`** = Inventory of physical files
2. **`model_bundles`** = Named configurations
3. **`bundle_components`** = Mapping with roles

**Key benefits:**
- ✅ Component reuse across bundles
- ✅ Flexible: Mix and match any components
- ✅ Efficient: Deduplication via file_hash
- ✅ Organized: Bundles group related components
- ✅ User-friendly: Create custom configurations

**Analogy:**
- `model_components` = LEGO pieces
- `model_bundles` = Instruction manuals
- `bundle_components` = Step-by-step mapping ("place piece A in slot B")

