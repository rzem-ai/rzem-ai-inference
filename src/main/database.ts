/**
 * Database initialization using better-sqlite3 (Node.js compatible).
 * Replaces bun:sqlite from the Electrobun/Bun version.
 */

import Database from "better-sqlite3";
import { DEFAULT_BUNDLES, DEFAULT_BUNDLE_TYPES } from "./services/bundles";

const SCHEMA = `
-- rzem-ai-inference database schema

-- ── Images ──────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS images (
    id              TEXT PRIMARY KEY,
    file_path       TEXT NOT NULL,
    thumbnail_path  TEXT,
    prompt          TEXT NOT NULL,
    raw_prompt      TEXT,
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
    bundle_id           TEXT,
    is_favorite         INTEGER NOT NULL DEFAULT 0,
    usage_count         INTEGER NOT NULL DEFAULT 0,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_styles_category ON styles(category);
CREATE INDEX IF NOT EXISTS idx_styles_favorite ON styles(is_favorite) WHERE is_favorite = 1;
CREATE INDEX IF NOT EXISTS idx_styles_bundle   ON styles(bundle_id);

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
    bundle_id       TEXT,
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

-- ── Bundle Types (model families) ─────────────────────────────────
CREATE TABLE IF NOT EXISTS bundle_types (
    id          TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    icon        TEXT,
    guide       TEXT,
    sort_order  INTEGER NOT NULL DEFAULT 0
);

-- ── Bundles (model configurations) ────────────────────────────────
CREATE TABLE IF NOT EXISTS bundles (
    id                  TEXT PRIMARY KEY,
    label               TEXT NOT NULL,
    description         TEXT NOT NULL DEFAULT '',
    transformer_type    TEXT NOT NULL,
    tier                TEXT NOT NULL DEFAULT 'balanced',
    transformer_model   TEXT NOT NULL,
    vae_model           TEXT NOT NULL,
    clip_tokenizer      TEXT,
    clip_encoder        TEXT,
    t5_tokenizer        TEXT,
    t5_encoder          TEXT,
    t5_encoder_config   TEXT,
    qwen3_tokenizer     TEXT,
    qwen3_encoder       TEXT,
    steps               INTEGER NOT NULL DEFAULT 20,
    cfg_scale           REAL NOT NULL DEFAULT 1.0,
    sampler             TEXT NOT NULL DEFAULT 'euler',
    scheduler           TEXT NOT NULL DEFAULT 'normal',
    vram_estimate_gb    REAL NOT NULL DEFAULT 0.0,
    is_default          INTEGER NOT NULL DEFAULT 1,
    source              TEXT NOT NULL DEFAULT 'local',
    fal_endpoint        TEXT,
    fal_aspectratio     TEXT,
    favorite            INTEGER NOT NULL DEFAULT 0,
    hidden              INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (transformer_type) REFERENCES bundle_types(id)
);

CREATE INDEX IF NOT EXISTS idx_bundles_transformer_type ON bundles(transformer_type);
CREATE INDEX IF NOT EXISTS idx_bundles_favorite ON bundles(favorite) WHERE favorite = 1;
CREATE INDEX IF NOT EXISTS idx_bundles_hidden ON bundles(hidden);

-- ── Workflows ─────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS workflows (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    graph_json  TEXT NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_workflows_updated ON workflows(updated_at DESC);

-- ── Conversations (AI Chat) ───────────────────────────────────────
CREATE TABLE IF NOT EXISTS conversations (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS conversation_messages (
    id                  TEXT PRIMARY KEY,
    conversation_id     TEXT NOT NULL,
    role                TEXT NOT NULL,
    content             TEXT NOT NULL,
    display_text        TEXT,
    image_paths         TEXT,
    tool_calls          TEXT,
    created_at          INTEGER NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);
`;

/**
 * Initialize the database with WAL mode and the full schema.
 * Returns the better-sqlite3 Database instance.
 */
export function initDatabase(dbPath: string): Database.Database {
	const db = new Database(dbPath);

	// Match the Python app's PRAGMA settings
	db.pragma("journal_mode = WAL");
	db.pragma("foreign_keys = ON");
	db.pragma("busy_timeout = 5000");

	// Apply schema (all CREATE IF NOT EXISTS — safe to re-run)
	db.exec(SCHEMA);

	// Seed default bundle types and bundles if empty
	const typeCount = db.prepare("SELECT COUNT(*) as n FROM bundle_types").get() as { n: number };
	if (typeCount.n === 0) {
		const insertType = db.prepare(
			"INSERT INTO bundle_types (id, label, icon, sort_order, guide) VALUES (?, ?, ?, ?, ?)",
		);
		for (const t of DEFAULT_BUNDLE_TYPES) {
			insertType.run(t.id, t.label, t.icon, t.sort_order, t.guide);
		}
	}
	const bundleCount = db.prepare("SELECT COUNT(*) as n FROM bundles").get() as { n: number };
	if (bundleCount.n === 0) {
		const insertBundle = db.prepare(
			`INSERT INTO bundles (id, label, description, transformer_type, tier, transformer_model, vae_model, clip_tokenizer, clip_encoder, t5_tokenizer, t5_encoder, t5_encoder_config, qwen3_tokenizer, qwen3_encoder, steps, cfg_scale, sampler, scheduler, vram_estimate_gb, is_default, source, fal_endpoint, fal_aspectratio) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
		);
		for (const b of DEFAULT_BUNDLES) {
			insertBundle.run(
				b.id, b.label, b.description, b.transformer_type, b.tier,
				b.transformer_model, b.vae_model, b.clip_tokenizer, b.clip_encoder,
				b.t5_tokenizer, b.t5_encoder, b.t5_encoder_config, b.qwen3_tokenizer,
				b.qwen3_encoder, b.steps, b.cfg_scale, b.sampler, b.scheduler,
				b.vram_estimate_gb, b.is_default, b.source, b.fal_endpoint, b.fal_aspectratio,
			);
		}
	}

	return db;
}
