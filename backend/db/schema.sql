-- rzem-ai-inference database schema
-- 11 tables: images, folders, image_folders, tags, image_tags,
--            styles, style_loras, style_tags, loras, examples, settings
--
-- PRAGMAs (WAL, foreign_keys) are set in Database.connect() before this runs.

-- ── Images ──────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS images (
    id              TEXT PRIMARY KEY,
    file_path       TEXT NOT NULL,
    thumbnail_path  TEXT,
    prompt          TEXT NOT NULL,
    negative_prompt TEXT,
    width           INTEGER NOT NULL,
    height          INTEGER NOT NULL,
    file_size       INTEGER,
    steps           INTEGER NOT NULL,
    cfg_scale       REAL NOT NULL,
    seed            INTEGER NOT NULL,
    bundle_id       TEXT,
    model_config    TEXT,
    loras           TEXT,
    favorite        INTEGER NOT NULL DEFAULT 0,
    generation_time_ms INTEGER,
    status          TEXT NOT NULL DEFAULT 'completed',
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_images_created_at ON images(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_images_favorite   ON images(favorite) WHERE favorite = 1;
CREATE INDEX IF NOT EXISTS idx_images_bundle     ON images(bundle_id);
CREATE INDEX IF NOT EXISTS idx_images_status     ON images(status);

-- ── Folders ─────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS folders (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    parent_id   TEXT,
    color       TEXT,
    icon        TEXT,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE CASCADE,
    UNIQUE(parent_id, name)
);

CREATE INDEX IF NOT EXISTS idx_folders_parent ON folders(parent_id);

-- ── Image ↔ Folder (many-to-many) ──────────────────────────────

CREATE TABLE IF NOT EXISTS image_folders (
    image_id    TEXT NOT NULL,
    folder_id   TEXT NOT NULL,
    added_at    INTEGER NOT NULL,
    PRIMARY KEY (image_id, folder_id),
    FOREIGN KEY (image_id)  REFERENCES images(id)  ON DELETE CASCADE,
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_image_folders_folder ON image_folders(folder_id);
CREATE INDEX IF NOT EXISTS idx_image_folders_image  ON image_folders(image_id);

-- ── Tags ────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS tags (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,
    color       TEXT,
    category    TEXT
);

-- ── Image ↔ Tag (many-to-many) ─────────────────────────────────

CREATE TABLE IF NOT EXISTS image_tags (
    image_id    TEXT NOT NULL,
    tag_id      INTEGER NOT NULL,
    PRIMARY KEY (image_id, tag_id),
    FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id)   REFERENCES tags(id)   ON DELETE CASCADE
);

-- ── Styles ──────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS styles (
    id                  TEXT PRIMARY KEY,
    name                TEXT NOT NULL,
    description         TEXT,
    prompt_template     TEXT NOT NULL,
    negative_prompt     TEXT,
    default_strength    REAL NOT NULL DEFAULT 1.0,
    strength_min        REAL NOT NULL DEFAULT 0.5,
    strength_max        REAL NOT NULL DEFAULT 1.5,
    category            TEXT,
    thumbnail_path      TEXT,
    is_favorite         INTEGER NOT NULL DEFAULT 0,
    usage_count         INTEGER NOT NULL DEFAULT 0,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_styles_category ON styles(category);
CREATE INDEX IF NOT EXISTS idx_styles_favorite ON styles(is_favorite) WHERE is_favorite = 1;

-- ── Style ↔ LoRA (many-to-many) ────────────────────────────────

CREATE TABLE IF NOT EXISTS style_loras (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    style_id    TEXT NOT NULL,
    lora_id     TEXT NOT NULL,
    strength    REAL NOT NULL DEFAULT 1.0,
    priority    INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (style_id) REFERENCES styles(id) ON DELETE CASCADE,
    FOREIGN KEY (lora_id)  REFERENCES loras(id)  ON DELETE CASCADE,
    UNIQUE (style_id, lora_id)
);

CREATE INDEX IF NOT EXISTS idx_style_loras_style ON style_loras(style_id);
CREATE INDEX IF NOT EXISTS idx_style_loras_lora  ON style_loras(lora_id);

-- ── Style ↔ Tag (many-to-many) ───────────────────────────────

CREATE TABLE IF NOT EXISTS style_tags (
    style_id    TEXT NOT NULL,
    tag_id      INTEGER NOT NULL,
    PRIMARY KEY (style_id, tag_id),
    FOREIGN KEY (style_id) REFERENCES styles(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id)   REFERENCES tags(id)   ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_style_tags_style ON style_tags(style_id);
CREATE INDEX IF NOT EXISTS idx_style_tags_tag   ON style_tags(tag_id);

-- ── LoRAs ───────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS loras (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    path            TEXT NOT NULL,
    trigger_words   TEXT,
    base_model      TEXT,
    size_bytes      INTEGER,
    strength        REAL NOT NULL DEFAULT 1.0,
    is_active       INTEGER NOT NULL DEFAULT 0,
    created_at      INTEGER NOT NULL,
    metadata        TEXT
);

-- ── Examples (generic, polymorphic) ─────────────────────────────

CREATE TABLE IF NOT EXISTS examples (
    id              TEXT PRIMARY KEY,
    entity_type     TEXT NOT NULL,
    entity_id       TEXT NOT NULL,
    example_type    TEXT NOT NULL,
    content         TEXT NOT NULL,
    created_at      INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_examples_entity ON examples(entity_type, entity_id);

-- ── Settings (key-value) ────────────────────────────────────────

CREATE TABLE IF NOT EXISTS settings (
    key     TEXT PRIMARY KEY,
    value   TEXT NOT NULL
);
