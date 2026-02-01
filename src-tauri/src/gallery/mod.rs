//! Gallery database and metadata management

use rusqlite::{Connection, params};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::inference::GenerationStats;

// ========== Folder Types ==========

/// Folder for organizing gallery images
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Folder with computed hierarchy information for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderNode {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub created_at: i64,
    pub updated_at: i64,
    /// Direct children of this folder
    pub children: Vec<FolderNode>,
    /// Count of images directly in this folder
    pub image_count: i32,
    /// Count of images in this folder and all descendants
    pub total_image_count: i32,
    /// Breadcrumb path from root to this folder
    pub path: Vec<String>,
}

/// Enhanced tag with color and category support
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
    pub category: Option<String>,
    pub usage_count: i32,
}

/// Image metadata for internal use (snake_case)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub id: String,
    pub file_path: Option<String>,  // Nullable: None for pending images
    pub thumbnail_path: Option<String>,
    pub prompt: String,
    pub created_at: i64,
    pub width: Option<i32>,  // Nullable: None for pending images
    pub height: Option<i32>,  // Nullable: None for pending images
    pub file_size: Option<i64>,  // Nullable: None for pending images
    pub model_name: String,
    pub negative_prompt: Option<String>,
    pub steps: Option<i32>,
    pub cfg_scale: Option<f64>,
    pub seed: Option<i64>,
    pub sampler: Option<String>,
    pub generation_time_ms: Option<i64>,
    pub status: String,  // "pending", "processing", "completed", "failed"
    pub session_id: Option<String>,  // UUID for session tracking
    pub updated_at: i64,  // Unix timestamp of last update
}

/// Image metadata for frontend API (camelCase)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryImage {
    pub id: String,
    pub file_path: Option<String>,  // Nullable: None for pending images
    pub thumbnail_path: Option<String>,
    pub created_at: i64,
    pub width: Option<i32>,  // Nullable: None for pending images
    pub height: Option<i32>,  // Nullable: None for pending images
    pub file_size: Option<i64>,  // Nullable: None for pending images
    pub is_favorite: bool,
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub model_name: String,
    pub steps: Option<i32>,
    pub cfg_scale: Option<f64>,
    pub seed: Option<i64>,
    pub sampler: Option<String>,
    pub tags: Vec<String>,
    /// IDs of folders this image belongs to
    pub folder_ids: Vec<String>,
    pub status: String,  // "pending", "processing", "completed", "failed"
    pub session_id: Option<String>,  // UUID for session tracking
    pub updated_at: i64,  // Unix timestamp of last update
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationPreset {
    pub id: String,
    pub name: String,
    pub mode: String,
    pub prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub steps: i32,
    pub cfg_scale: f64,
    pub width: i32,
    pub height: i32,
    pub seed: Option<i64>,
    pub model_id: Option<String>,
    pub lora_ids: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

// ========== Model Bundle System Types ==========

/// Physical model component file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentRecord {
    pub id: String,
    pub component_type: String,
    pub format: String,
    pub file_path: String,
    pub file_size: i64,
    pub file_hash: Option<String>,
    pub name: String,
    pub repo_id: Option<String>,
    pub repo_snapshot: Option<String>,
    pub architecture: Option<String>,
    pub quantization: Option<String>,
    pub supports_loras: bool,
    pub is_sharded: bool,
    pub shard_count: Option<i32>,
    pub vram_mb: Option<i32>,
    pub discovered_at: i64,
    pub last_verified_at: Option<i64>,
    pub is_available: bool,
    pub metadata: Option<serde_json::Value>,
}

/// Logical model bundle grouping components
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub bundle_type: String,
    pub model_family: String,
    pub default_steps: Option<i32>,
    pub default_guidance: Option<f64>,
    pub step_min: Option<i32>,
    pub step_max: Option<i32>,
    pub total_vram_mb: Option<i32>,
    pub is_complete: bool,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub validation_errors: Option<String>,
}

/// Component role in a bundle with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentInfo {
    pub id: String,
    pub role: String,
    pub name: String,
    pub component_type: String,
    pub format: String,
    pub file_path: String,
    pub file_size: i64,
    pub quantization: Option<String>,
    pub vram_mb: Option<i32>,
    pub is_available: bool,
    pub is_required: bool,
    pub priority: i32,
}

/// Bundle with full component details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub bundle_type: String,
    pub model_family: String,
    pub default_steps: Option<i32>,
    pub default_guidance: Option<f64>,
    pub step_min: Option<i32>,
    pub step_max: Option<i32>,
    pub total_vram_mb: Option<i32>,
    pub is_complete: bool,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub validation_errors: Option<String>,
    pub components: Vec<ComponentInfo>,
}

// ========== Response DTOs (match frontend TypeScript types) ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelFileInfoResponse {
    pub id: String,
    pub model_id: String,
    pub path: String,
    pub resolved_path: String,
    pub sha256: Option<String>,
    pub size_bytes: i64,
    pub is_symlink: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExampleResponse {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub example_type: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPrefsBaseResponse {
    pub model_id: String,
    pub preferred_steps: Option<i32>,
    pub preferred_cfg: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPrefsLoraResponse {
    pub model_id: String,
    pub strength_min: Option<f64>,
    pub strength_max: Option<f64>,
    pub strength_default: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfoResponse {
    pub id: String,
    pub model_type: String,
    pub family: String,
    pub display_name: String,
    pub description: Option<String>,
    pub files: Vec<ModelFileInfoResponse>,
    pub tags: Vec<String>,
    pub examples: Vec<ExampleResponse>,
    pub prefs_base: Option<ModelPrefsBaseResponse>,
    pub prefs_lora: Option<ModelPrefsLoraResponse>,
    pub trigger_words: Vec<String>,
    pub architecture: Option<String>,
    pub quantization: Option<String>,
    pub vram_mb: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleItemInfoResponse {
    pub id: String,
    pub model_id: String,
    pub role: String,
    pub model_display_name: String,
    pub model_family: String,
    pub model_type: String,
    pub model_vram_mb: Option<i32>,
    pub model_quantization: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleInfoResponse {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub is_complete: bool,
    pub total_vram_mb: i32,
    pub tags: Vec<String>,
    pub items: Vec<BundleItemInfoResponse>,
    pub examples: Vec<ExampleResponse>,
    pub created_at: String,
    pub updated_at: String,
}

/// Map scanner component_type to frontend model_type taxonomy
fn component_type_to_model_type(comp_type: &str) -> &'static str {
    match comp_type {
        "transformer" => "checkpoint",
        "t5_encoder" => "text_encoder",
        "clip_encoder" => "text_encoder",
        "vae" => "vae",
        "t5_tokenizer" => "tokenizer",
        "clip_tokenizer" => "tokenizer",
        _ => "other",
    }
}

/// Infer model family from architecture string
fn infer_family(architecture: Option<&str>) -> &'static str {
    match architecture {
        Some(a) if a.contains("z-image") => "zindex",
        Some(a) if a.contains("flux") || a.contains("schnell") || a.contains("dev") => "flux",
        _ => "other",
    }
}

pub struct GalleryDb {
    conn: Connection,
}

impl GalleryDb {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        // Always initialize schema on new connection
        db.init_schema()?;
        Ok(db)
    }

    pub fn init_schema(&self) -> Result<()> {
        tracing::info!("🔧 Initializing database schema...");
        // Create images table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS images (
                id TEXT PRIMARY KEY,
                file_path TEXT UNIQUE,
                thumbnail_path TEXT,
                created_at INTEGER NOT NULL,
                width INTEGER,
                height INTEGER,
                file_size INTEGER,
                is_favorite INTEGER DEFAULT 0,
                prompt TEXT NOT NULL,
                negative_prompt TEXT,
                model_name TEXT NOT NULL,
                steps INTEGER,
                cfg_scale REAL,
                seed INTEGER,
                sampler TEXT,
                server_id TEXT,
                generation_time_ms INTEGER DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'completed',
                session_id TEXT,
                updated_at INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        // Create index for efficient session queries
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_images_session_status
             ON images(session_id, status, created_at DESC)",
            [],
        )?;

        // Migrate existing data: set updated_at to created_at where it's 0
        self.conn.execute(
            "UPDATE images SET updated_at = created_at WHERE updated_at = 0",
            [],
        )?;

        // Create tags table with color and category support
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                color TEXT,
                category TEXT
            )",
            [],
        )?;

        // Create image_tags junction table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS image_tags (
                image_id TEXT NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (image_id, tag_id),
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create folders table with hierarchy support
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS folders (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                parent_id TEXT,
                color TEXT,
                icon TEXT,
                sort_order INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE CASCADE,
                UNIQUE(parent_id, name)
            )",
            [],
        )?;

        // Create image_folders junction table (many-to-many)
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS image_folders (
                image_id TEXT NOT NULL,
                folder_id TEXT NOT NULL,
                added_at INTEGER NOT NULL,
                PRIMARY KEY (image_id, folder_id),
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
                FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create indexes for folder queries
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_folders_parent ON folders(parent_id)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_image_folders_folder ON image_folders(folder_id)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_image_folders_image ON image_folders(image_id)",
            [],
        )?;

        // Create FTS5 virtual table for full-text search
        self.conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS images_fts USING fts5(
                image_id UNINDEXED,
                prompt,
                negative_prompt
            )",
            [],
        )?;


        // Create loras table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS loras (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                trigger_words TEXT,
                base_model TEXT,
                size_bytes INTEGER,
                strength REAL DEFAULT 1.0,
                is_active INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                metadata TEXT
            )",
            [],
        )?;

        // Create presets table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS presets (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                mode TEXT NOT NULL,
                prompt TEXT,
                negative_prompt TEXT,
                steps INTEGER NOT NULL,
                cfg_scale REAL NOT NULL,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                seed INTEGER,
                model_id TEXT,
                lora_ids TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Create generation_stats table for detailed timing statistics
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS generation_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                image_id TEXT NOT NULL UNIQUE,
                model_load_ms INTEGER,
                t5_load_ms INTEGER,
                clip_load_ms INTEGER,
                vae_load_ms INTEGER,
                flux_load_ms INTEGER,
                t5_encode_ms INTEGER NOT NULL,
                clip_encode_ms INTEGER NOT NULL,
                denoise_ms INTEGER NOT NULL,
                vae_decode_ms INTEGER NOT NULL,
                png_encode_ms INTEGER NOT NULL,
                total_ms INTEGER NOT NULL,
                steps INTEGER NOT NULL,
                model_type TEXT NOT NULL,
                t5_embedding_shape TEXT,
                clip_embedding_shape TEXT,
                latent_shape TEXT,
                image_shape TEXT,
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create app_settings table for feature settings (JSON storage)
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Create batch_template_history table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS batch_template_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                template TEXT NOT NULL,
                used_at TEXT NOT NULL,
                image_count INTEGER NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create index for batch template history
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_batch_template_history_used_at
             ON batch_template_history(used_at DESC)",
            [],
        )?;

        // ===== Model Bundle System Tables =====

        // Create model_components table - physical model files
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS model_components (
                id TEXT PRIMARY KEY,
                component_type TEXT NOT NULL,
                format TEXT NOT NULL,
                file_path TEXT NOT NULL UNIQUE,
                file_size INTEGER NOT NULL,
                file_hash TEXT,
                name TEXT NOT NULL,
                repo_id TEXT,
                repo_snapshot TEXT,
                architecture TEXT,
                quantization TEXT,
                supports_loras INTEGER DEFAULT 0,
                is_sharded INTEGER DEFAULT 0,
                shard_count INTEGER,
                vram_mb INTEGER,
                discovered_at INTEGER NOT NULL,
                last_verified_at INTEGER,
                is_available INTEGER DEFAULT 1,
                metadata TEXT
            )",
            [],
        )?;

        // Create model_bundles table - logical groupings
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS model_bundles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                bundle_type TEXT NOT NULL,
                model_family TEXT NOT NULL,
                default_steps INTEGER,
                default_guidance REAL,
                step_min INTEGER,
                step_max INTEGER,
                total_vram_mb INTEGER,
                is_complete INTEGER DEFAULT 0,
                is_active INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                validation_errors TEXT
            )",
            [],
        )?;

        // Create bundle_components table - many-to-many relationships
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS bundle_components (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                bundle_id TEXT NOT NULL,
                component_id TEXT NOT NULL,
                component_role TEXT NOT NULL,
                is_required INTEGER DEFAULT 1,
                priority INTEGER DEFAULT 0,
                FOREIGN KEY (bundle_id) REFERENCES model_bundles(id) ON DELETE CASCADE,
                FOREIGN KEY (component_id) REFERENCES model_components(id) ON DELETE CASCADE,
                UNIQUE (bundle_id, component_role, component_id)
            )",
            [],
        )?;

        // Create model_tags table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS model_tags (
                model_id TEXT NOT NULL,
                tag TEXT NOT NULL,
                PRIMARY KEY (model_id, tag),
                FOREIGN KEY (model_id) REFERENCES model_components(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create examples table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS examples (
                id TEXT PRIMARY KEY,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                example_type TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_examples_entity ON examples(entity_type, entity_id)",
            [],
        )?;

        tracing::info!("✅ Model bundle tables created");

        // Create indexes for model bundle system
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_components_type ON model_components(component_type)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_components_repo ON model_components(repo_id)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_components_available ON model_components(is_available)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_components_hash ON model_components(file_hash)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_bundles_family ON model_bundles(model_family)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_bundles_active ON model_bundles(is_active)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_bundle_components_bundle ON bundle_components(bundle_id)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_bundle_components_component ON bundle_components(component_id)",
            [],
        )?;

        tracing::info!("✅ Database schema initialization complete");
        Ok(())
    }

    pub fn insert_image(&self, metadata: &ImageMetadata) -> Result<()> {
        // Insert into images table with all metadata
        self.conn.execute(
            "INSERT INTO images (
                id, file_path, thumbnail_path, prompt, created_at,
                width, height, file_size, model_name, negative_prompt,
                steps, cfg_scale, seed, sampler, generation_time_ms,
                status, session_id, updated_at
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            params![
                metadata.id,
                metadata.file_path,
                metadata.thumbnail_path,
                metadata.prompt,
                metadata.created_at,
                metadata.width,
                metadata.height,
                metadata.file_size,
                metadata.model_name,
                metadata.negative_prompt,
                metadata.steps,
                metadata.cfg_scale,
                metadata.seed,
                metadata.sampler,
                metadata.generation_time_ms,
                metadata.status,
                metadata.session_id,
                metadata.updated_at,
            ],
        )?;

        // Insert into FTS table
        self.conn.execute(
            "INSERT INTO images_fts (image_id, prompt, negative_prompt)
             VALUES (?1, ?2, ?3)",
            params![
                metadata.id,
                metadata.prompt,
                metadata.negative_prompt.as_deref().unwrap_or("")
            ],
        )?;

        Ok(())
    }

    pub fn get_recent_images(&self, limit: usize) -> Result<Vec<ImageMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_path, thumbnail_path, prompt, created_at,
                    width, height, file_size, model_name, negative_prompt,
                    steps, cfg_scale, seed, sampler, generation_time_ms,
                    status, session_id, updated_at
             FROM images
             ORDER BY created_at DESC
             LIMIT ?1"
        )?;

        let images = stmt.query_map(params![limit], |row| {
            Ok(ImageMetadata {
                id: row.get(0)?,
                file_path: row.get(1)?,
                thumbnail_path: row.get(2)?,
                prompt: row.get(3)?,
                created_at: row.get(4)?,
                width: row.get(5)?,
                height: row.get(6)?,
                file_size: row.get(7)?,
                model_name: row.get(8)?,
                negative_prompt: row.get(9)?,
                steps: row.get(10)?,
                cfg_scale: row.get(11)?,
                seed: row.get(12)?,
                sampler: row.get(13)?,
                generation_time_ms: row.get(14)?,
                status: row.get(15)?,
                session_id: row.get(16)?,
                updated_at: row.get(17)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(images)
    }

    pub fn save_preset(&self, preset: &GenerationPreset) -> Result<()> {
        self.conn.execute(
            "INSERT INTO presets
             (id, name, mode, prompt, negative_prompt, steps, cfg_scale, width, height,
              seed, model_id, lora_ids, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                mode = excluded.mode,
                prompt = excluded.prompt,
                negative_prompt = excluded.negative_prompt,
                steps = excluded.steps,
                cfg_scale = excluded.cfg_scale,
                width = excluded.width,
                height = excluded.height,
                seed = excluded.seed,
                model_id = excluded.model_id,
                lora_ids = excluded.lora_ids,
                updated_at = excluded.updated_at",
            params![
                preset.id,
                preset.name,
                preset.mode,
                preset.prompt,
                preset.negative_prompt,
                preset.steps,
                preset.cfg_scale,
                preset.width,
                preset.height,
                preset.seed,
                preset.model_id,
                preset.lora_ids,
                preset.created_at,
                preset.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn load_presets(&self) -> Result<Vec<GenerationPreset>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, mode, prompt, negative_prompt, steps, cfg_scale,
                    width, height, seed, model_id, lora_ids, created_at, updated_at
             FROM presets ORDER BY updated_at DESC"
        )?;

        let presets = stmt.query_map([], |row| {
            Ok(GenerationPreset {
                id: row.get(0)?,
                name: row.get(1)?,
                mode: row.get(2)?,
                prompt: row.get(3)?,
                negative_prompt: row.get(4)?,
                steps: row.get(5)?,
                cfg_scale: row.get(6)?,
                width: row.get(7)?,
                height: row.get(8)?,
                seed: row.get(9)?,
                model_id: row.get(10)?,
                lora_ids: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(presets)
    }

    pub fn delete_preset(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM presets WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_gallery_images(&self, limit: usize) -> Result<Vec<GalleryImage>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_path, thumbnail_path, created_at, width, height,
                    file_size, is_favorite, prompt, negative_prompt, model_name,
                    steps, cfg_scale, seed, sampler, status, session_id, updated_at
             FROM images
             ORDER BY created_at DESC
             LIMIT ?1"
        )?;

        let mut images: Vec<GalleryImage> = stmt.query_map(params![limit], |row| {
            Ok(GalleryImage {
                id: row.get(0)?,
                file_path: row.get(1)?,
                thumbnail_path: row.get(2)?,
                created_at: row.get(3)?,
                width: row.get(4)?,
                height: row.get(5)?,
                file_size: row.get(6)?,
                is_favorite: row.get::<_, i32>(7)? != 0,
                prompt: row.get(8)?,
                negative_prompt: row.get(9)?,
                model_name: row.get(10)?,
                steps: row.get(11)?,
                cfg_scale: row.get(12)?,
                seed: row.get(13)?,
                sampler: row.get(14)?,
                tags: Vec::new(),
                folder_ids: Vec::new(),
                status: row.get(15)?,
                session_id: row.get(16)?,
                updated_at: row.get(17)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        // Fetch tags and folders for each image
        for image in &mut images {
            image.tags = self.get_image_tags(&image.id)?;
            image.folder_ids = self.get_image_folder_ids(&image.id)?;
        }

        Ok(images)
    }

    pub fn search_gallery_images(&self, query: &str) -> Result<Vec<ImageMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT i.id, i.file_path, i.thumbnail_path, i.prompt, i.created_at,
                    i.width, i.height, i.file_size, i.model_name, i.negative_prompt,
                    i.steps, i.cfg_scale, i.seed, i.sampler, i.generation_time_ms,
                    i.status, i.session_id, i.updated_at
             FROM images i
             JOIN images_fts fts ON i.id = fts.image_id
             WHERE images_fts MATCH ?1
             ORDER BY i.created_at DESC
             LIMIT 100"
        )?;

        let images = stmt.query_map(params![query], |row| {
            Ok(ImageMetadata {
                id: row.get(0)?,
                file_path: row.get(1)?,
                thumbnail_path: row.get(2)?,
                prompt: row.get(3)?,
                created_at: row.get(4)?,
                width: row.get(5)?,
                height: row.get(6)?,
                file_size: row.get(7)?,
                model_name: row.get(8)?,
                negative_prompt: row.get(9)?,
                steps: row.get(10)?,
                cfg_scale: row.get(11)?,
                seed: row.get(12)?,
                sampler: row.get(13)?,
                generation_time_ms: row.get(14)?,
                status: row.get(15)?,
                session_id: row.get(16)?,
                updated_at: row.get(17)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(images)
    }

    pub fn toggle_favorite(&self, image_id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE images SET is_favorite = NOT is_favorite WHERE id = ?1",
            params![image_id],
        )?;

        Ok(())
    }

    pub fn add_image_tag(&self, image_id: &str, tag: &str) -> Result<()> {
        // Get or create tag
        self.conn.execute(
            "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
            params![tag],
        )?;

        let tag_id: i64 = self.conn.query_row(
            "SELECT id FROM tags WHERE name = ?1",
            params![tag],
            |row| row.get(0),
        )?;

        // Link tag to image
        self.conn.execute(
            "INSERT OR IGNORE INTO image_tags (image_id, tag_id) VALUES (?1, ?2)",
            params![image_id, tag_id],
        )?;

        Ok(())
    }

    pub fn remove_image_tag(&self, image_id: &str, tag: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM image_tags
             WHERE image_id = ?1 AND tag_id = (SELECT id FROM tags WHERE name = ?2)",
            params![image_id, tag],
        )?;

        Ok(())
    }

    pub fn get_image_by_id(&self, image_id: &str) -> Result<Option<ImageMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_path, thumbnail_path, prompt, created_at,
                    width, height, file_size, model_name, negative_prompt,
                    steps, cfg_scale, seed, sampler, generation_time_ms,
                    status, session_id, updated_at
             FROM images
             WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![image_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(ImageMetadata {
                id: row.get(0)?,
                file_path: row.get(1)?,
                thumbnail_path: row.get(2)?,
                prompt: row.get(3)?,
                created_at: row.get(4)?,
                width: row.get(5)?,
                height: row.get(6)?,
                file_size: row.get(7)?,
                model_name: row.get(8)?,
                negative_prompt: row.get(9)?,
                steps: row.get(10)?,
                cfg_scale: row.get(11)?,
                seed: row.get(12)?,
                sampler: row.get(13)?,
                generation_time_ms: row.get(14)?,
                status: row.get(15)?,
                session_id: row.get(16)?,
                updated_at: row.get(17)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn delete_gallery_image(&self, image_id: &str) -> Result<()> {
        // Delete from FTS first
        self.conn.execute(
            "DELETE FROM images_fts WHERE image_id = ?1",
            params![image_id],
        )?;

        // Then delete from main table (cascades to image_tags and generation_stats)
        self.conn.execute(
            "DELETE FROM images WHERE id = ?1",
            params![image_id],
        )?;

        Ok(())
    }

    /// Insert generation statistics for an image
    pub fn insert_generation_stats(&self, image_id: &str, stats: &GenerationStats) -> Result<()> {
        self.conn.execute(
            "INSERT INTO generation_stats (
                image_id, model_load_ms, t5_load_ms, clip_load_ms, vae_load_ms, flux_load_ms,
                t5_encode_ms, clip_encode_ms, denoise_ms, vae_decode_ms, png_encode_ms,
                total_ms, steps, model_type,
                t5_embedding_shape, clip_embedding_shape, latent_shape, image_shape
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            params![
                image_id,
                stats.model_load_ms.map(|v| v as i64),
                stats.t5_load_ms.map(|v| v as i64),
                stats.clip_load_ms.map(|v| v as i64),
                stats.vae_load_ms.map(|v| v as i64),
                stats.flux_load_ms.map(|v| v as i64),
                stats.t5_encode_ms as i64,
                stats.clip_encode_ms as i64,
                stats.denoise_ms as i64,
                stats.vae_decode_ms as i64,
                stats.png_encode_ms as i64,
                stats.total_ms as i64,
                stats.steps as i64,
                stats.model_type,
                serde_json::to_string(&stats.t5_embedding_shape).ok(),
                serde_json::to_string(&stats.clip_embedding_shape).ok(),
                serde_json::to_string(&stats.latent_shape).ok(),
                serde_json::to_string(&stats.image_shape).ok(),
            ],
        )?;

        // Also update the generation_time_ms in the images table for quick access
        self.conn.execute(
            "UPDATE images SET generation_time_ms = ?1 WHERE id = ?2",
            params![stats.total_ms, image_id],
        )?;

        Ok(())
    }

    /// Get generation statistics for an image
    pub fn get_generation_stats(&self, image_id: &str) -> Result<Option<GenerationStats>> {
        let mut stmt = self.conn.prepare(
            "SELECT model_load_ms, t5_load_ms, clip_load_ms, vae_load_ms, flux_load_ms,
                    t5_encode_ms, clip_encode_ms, denoise_ms, vae_decode_ms, png_encode_ms,
                    total_ms, steps, model_type,
                    t5_embedding_shape, clip_embedding_shape, latent_shape, image_shape
             FROM generation_stats
             WHERE image_id = ?1"
        )?;

        let mut rows = stmt.query(params![image_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(GenerationStats {
                model_load_ms: row.get::<_, Option<i64>>(0)?.map(|v| v as u64),
                t5_load_ms: row.get::<_, Option<i64>>(1)?.map(|v| v as u64),
                clip_load_ms: row.get::<_, Option<i64>>(2)?.map(|v| v as u64),
                vae_load_ms: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
                flux_load_ms: row.get::<_, Option<i64>>(4)?.map(|v| v as u64),
                t5_encode_ms: row.get::<_, i64>(5)? as u64,
                clip_encode_ms: row.get::<_, i64>(6)? as u64,
                denoise_ms: row.get::<_, i64>(7)? as u64,
                vae_decode_ms: row.get::<_, i64>(8)? as u64,
                png_encode_ms: row.get::<_, i64>(9)? as u64,
                total_ms: row.get::<_, i64>(10)? as u64,
                steps: row.get::<_, i64>(11)? as usize,
                model_type: row.get(12)?,
                t5_embedding_shape: row.get::<_, Option<String>>(13)?
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default(),
                clip_embedding_shape: row.get::<_, Option<String>>(14)?
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default(),
                latent_shape: row.get::<_, Option<String>>(15)?
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default(),
                image_shape: row.get::<_, Option<String>>(16)?
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    // ========== Folder Operations ==========

    /// Create a new folder
    pub fn create_folder(
        &self,
        name: &str,
        parent_id: Option<&str>,
        color: Option<&str>,
        icon: Option<&str>,
    ) -> Result<Folder> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // Get next sort_order for this parent
        let sort_order: i32 = self.conn.query_row(
            "SELECT COALESCE(MAX(sort_order) + 1, 0) FROM folders WHERE parent_id IS ?1",
            params![parent_id],
            |row| row.get(0),
        ).unwrap_or(0);

        self.conn.execute(
            "INSERT INTO folders (id, name, parent_id, color, icon, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, name, parent_id, color, icon, sort_order, now, now],
        )?;

        Ok(Folder {
            id,
            name: name.to_string(),
            parent_id: parent_id.map(String::from),
            color: color.map(String::from),
            icon: icon.map(String::from),
            sort_order,
            created_at: now,
            updated_at: now,
        })
    }

    /// Update an existing folder
    pub fn update_folder(
        &self,
        id: &str,
        name: Option<&str>,
        color: Option<&str>,
        icon: Option<&str>,
    ) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        self.conn.execute(
            "UPDATE folders SET
                name = COALESCE(?2, name),
                color = COALESCE(?3, color),
                icon = COALESCE(?4, icon),
                updated_at = ?1
             WHERE id = ?5",
            params![now, name, color, icon, id],
        )?;

        Ok(())
    }

    /// Delete a folder and all its contents (cascade)
    pub fn delete_folder(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM folders WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Move a folder to a new parent
    pub fn move_folder(&self, id: &str, new_parent_id: Option<&str>) -> Result<()> {
        // Prevent moving folder into itself or its descendants
        if let Some(new_parent) = new_parent_id {
            if new_parent == id {
                anyhow::bail!("Cannot move folder into itself");
            }
            // Check if new_parent is a descendant of id
            let mut current = Some(new_parent.to_string());
            while let Some(ref parent_id) = current {
                if parent_id == id {
                    anyhow::bail!("Cannot move folder into its own descendant");
                }
                current = self.conn.query_row(
                    "SELECT parent_id FROM folders WHERE id = ?1",
                    params![parent_id],
                    |row| row.get::<_, Option<String>>(0),
                ).ok().flatten();
            }
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // Get next sort_order in new parent
        let sort_order: i32 = self.conn.query_row(
            "SELECT COALESCE(MAX(sort_order) + 1, 0) FROM folders WHERE parent_id IS ?1",
            params![new_parent_id],
            |row| row.get(0),
        ).unwrap_or(0);

        self.conn.execute(
            "UPDATE folders SET parent_id = ?1, sort_order = ?2, updated_at = ?3 WHERE id = ?4",
            params![new_parent_id, sort_order, now, id],
        )?;

        Ok(())
    }

    /// Get all folders as a flat list
    fn get_all_folders(&self) -> Result<Vec<Folder>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, parent_id, color, icon, sort_order, created_at, updated_at
             FROM folders
             ORDER BY parent_id NULLS FIRST, sort_order, name"
        )?;

        let folders = stmt.query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                color: row.get(3)?,
                icon: row.get(4)?,
                sort_order: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(folders)
    }

    /// Get image count for each folder
    fn get_folder_image_counts(&self) -> Result<std::collections::HashMap<String, i32>> {
        let mut stmt = self.conn.prepare(
            "SELECT folder_id, COUNT(*) FROM image_folders GROUP BY folder_id"
        )?;

        let counts = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(counts)
    }

    /// Build folder tree with computed counts and paths
    pub fn get_folder_tree(&self) -> Result<Vec<FolderNode>> {
        let folders = self.get_all_folders()?;
        let counts = self.get_folder_image_counts()?;

        // Build parent-to-children map
        let mut children_map: std::collections::HashMap<Option<String>, Vec<&Folder>> =
            std::collections::HashMap::new();

        for folder in &folders {
            children_map
                .entry(folder.parent_id.clone())
                .or_default()
                .push(folder);
        }

        // Recursive function to build tree
        fn build_node(
            folder: &Folder,
            children_map: &std::collections::HashMap<Option<String>, Vec<&Folder>>,
            counts: &std::collections::HashMap<String, i32>,
            path: Vec<String>,
        ) -> FolderNode {
            let direct_count = counts.get(&folder.id).copied().unwrap_or(0);

            let children: Vec<FolderNode> = children_map
                .get(&Some(folder.id.clone()))
                .map(|kids| {
                    let mut child_path = path.clone();
                    child_path.push(folder.name.clone());
                    kids.iter()
                        .map(|child| build_node(child, children_map, counts, child_path.clone()))
                        .collect()
                })
                .unwrap_or_default();

            let total_count = direct_count + children.iter().map(|c| c.total_image_count).sum::<i32>();

            FolderNode {
                id: folder.id.clone(),
                name: folder.name.clone(),
                parent_id: folder.parent_id.clone(),
                color: folder.color.clone(),
                icon: folder.icon.clone(),
                sort_order: folder.sort_order,
                created_at: folder.created_at,
                updated_at: folder.updated_at,
                children,
                image_count: direct_count,
                total_image_count: total_count,
                path,
            }
        }

        // Build root-level nodes
        let root_folders = children_map.get(&None).cloned().unwrap_or_default();
        let tree: Vec<FolderNode> = root_folders
            .into_iter()
            .map(|f| build_node(f, &children_map, &counts, vec![]))
            .collect();

        Ok(tree)
    }

    /// Add images to a folder
    pub fn add_images_to_folder(&self, image_ids: &[String], folder_id: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        for image_id in image_ids {
            self.conn.execute(
                "INSERT OR IGNORE INTO image_folders (image_id, folder_id, added_at)
                 VALUES (?1, ?2, ?3)",
                params![image_id, folder_id, now],
            )?;
        }

        Ok(())
    }

    /// Remove images from a folder
    pub fn remove_images_from_folder(&self, image_ids: &[String], folder_id: &str) -> Result<()> {
        for image_id in image_ids {
            self.conn.execute(
                "DELETE FROM image_folders WHERE image_id = ?1 AND folder_id = ?2",
                params![image_id, folder_id],
            )?;
        }

        Ok(())
    }

    /// Get images in a folder, optionally including descendants
    pub fn get_folder_images(
        &self,
        folder_id: &str,
        include_descendants: bool,
        limit: usize,
    ) -> Result<Vec<GalleryImage>> {
        let query = if include_descendants {
            // Recursive CTE to get all descendant folder IDs
            "WITH RECURSIVE descendants AS (
                SELECT id FROM folders WHERE id = ?1
                UNION ALL
                SELECT f.id FROM folders f
                INNER JOIN descendants d ON f.parent_id = d.id
             )
             SELECT DISTINCT i.id, i.file_path, i.thumbnail_path, i.created_at, i.width, i.height,
                    i.file_size, i.is_favorite, i.prompt, i.negative_prompt, i.model_name,
                    i.steps, i.cfg_scale, i.seed, i.sampler
             FROM images i
             INNER JOIN image_folders if2 ON i.id = if2.image_id
             WHERE if2.folder_id IN (SELECT id FROM descendants)
             ORDER BY i.created_at DESC
             LIMIT ?2"
        } else {
            "SELECT i.id, i.file_path, i.thumbnail_path, i.created_at, i.width, i.height,
                    i.file_size, i.is_favorite, i.prompt, i.negative_prompt, i.model_name,
                    i.steps, i.cfg_scale, i.seed, i.sampler
             FROM images i
             INNER JOIN image_folders if2 ON i.id = if2.image_id
             WHERE if2.folder_id = ?1
             ORDER BY i.created_at DESC
             LIMIT ?2"
        };

        let mut stmt = self.conn.prepare(query)?;
        let image_ids: Vec<String> = stmt.query_map(params![folder_id, limit], |row| {
            row.get(0)
        })?.filter_map(|r| r.ok()).collect();

        // Fetch full images with tags and folders
        self.get_gallery_images_by_ids(&image_ids)
    }

    /// Get images that are not in any folder
    pub fn get_uncategorized_images(&self, limit: usize) -> Result<Vec<GalleryImage>> {
        let mut stmt = self.conn.prepare(
            "SELECT i.id
             FROM images i
             LEFT JOIN image_folders if2 ON i.id = if2.image_id
             WHERE if2.folder_id IS NULL
             ORDER BY i.created_at DESC
             LIMIT ?1"
        )?;

        let image_ids: Vec<String> = stmt.query_map(params![limit], |row| {
            row.get(0)
        })?.filter_map(|r| r.ok()).collect();

        self.get_gallery_images_by_ids(&image_ids)
    }

    /// Get folder IDs for an image
    fn get_image_folder_ids(&self, image_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT folder_id FROM image_folders WHERE image_id = ?1"
        )?;

        let ids = stmt.query_map(params![image_id], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(ids)
    }

    /// Get gallery images by IDs with tags and folders
    fn get_gallery_images_by_ids(&self, image_ids: &[String]) -> Result<Vec<GalleryImage>> {
        if image_ids.is_empty() {
            return Ok(vec![]);
        }

        // Create placeholders for IN clause
        let placeholders: Vec<String> = (1..=image_ids.len()).map(|i| format!("?{}", i)).collect();
        let query = format!(
            "SELECT id, file_path, thumbnail_path, created_at, width, height,
                    file_size, is_favorite, prompt, negative_prompt, model_name,
                    steps, cfg_scale, seed, sampler, status, session_id, updated_at
             FROM images
             WHERE id IN ({})
             ORDER BY created_at DESC",
            placeholders.join(", ")
        );

        let mut stmt = self.conn.prepare(&query)?;
        let params: Vec<&dyn rusqlite::ToSql> = image_ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

        let mut images: Vec<GalleryImage> = stmt.query_map(params.as_slice(), |row| {
            Ok(GalleryImage {
                id: row.get(0)?,
                file_path: row.get(1)?,
                thumbnail_path: row.get(2)?,
                created_at: row.get(3)?,
                width: row.get(4)?,
                height: row.get(5)?,
                file_size: row.get(6)?,
                is_favorite: row.get::<_, i32>(7)? != 0,
                prompt: row.get(8)?,
                negative_prompt: row.get(9)?,
                model_name: row.get(10)?,
                steps: row.get(11)?,
                cfg_scale: row.get(12)?,
                seed: row.get(13)?,
                sampler: row.get(14)?,
                tags: Vec::new(),
                folder_ids: Vec::new(),
                status: row.get(15)?,
                session_id: row.get(16)?,
                updated_at: row.get(17)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        // Fetch tags and folders for each image
        for image in &mut images {
            image.tags = self.get_image_tags(&image.id)?;
            image.folder_ids = self.get_image_folder_ids(&image.id)?;
        }

        Ok(images)
    }

    /// Get tags for an image
    fn get_image_tags(&self, image_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.name FROM tags t
             INNER JOIN image_tags it ON t.id = it.tag_id
             WHERE it.image_id = ?1"
        )?;

        let tags = stmt.query_map(params![image_id], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(tags)
    }

    // ========== Enhanced Tag Operations ==========

    /// Get all tags with usage counts
    pub fn get_all_tags(&self) -> Result<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name, t.color, t.category, COUNT(it.image_id) as usage_count
             FROM tags t
             LEFT JOIN image_tags it ON t.id = it.tag_id
             GROUP BY t.id
             ORDER BY usage_count DESC, t.name"
        )?;

        let tags = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                category: row.get(3)?,
                usage_count: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(tags)
    }

    /// Update tag properties
    pub fn update_tag(
        &self,
        id: i64,
        name: Option<&str>,
        color: Option<&str>,
        category: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE tags SET
                name = COALESCE(?2, name),
                color = COALESCE(?3, color),
                category = COALESCE(?4, category)
             WHERE id = ?1",
            params![id, name, color, category],
        )?;

        Ok(())
    }

    /// Bulk add tag to multiple images
    pub fn bulk_add_tag(&self, image_ids: &[String], tag: &str) -> Result<()> {
        // Get or create tag
        self.conn.execute(
            "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
            params![tag],
        )?;

        let tag_id: i64 = self.conn.query_row(
            "SELECT id FROM tags WHERE name = ?1",
            params![tag],
            |row| row.get(0),
        )?;

        // Add tag to all images
        for image_id in image_ids {
            self.conn.execute(
                "INSERT OR IGNORE INTO image_tags (image_id, tag_id) VALUES (?1, ?2)",
                params![image_id, tag_id],
            )?;
        }

        Ok(())
    }

    /// Bulk remove tag from multiple images
    pub fn bulk_remove_tag(&self, image_ids: &[String], tag: &str) -> Result<()> {
        let tag_id: Option<i64> = self.conn.query_row(
            "SELECT id FROM tags WHERE name = ?1",
            params![tag],
            |row| row.get(0),
        ).ok();

        if let Some(tag_id) = tag_id {
            for image_id in image_ids {
                self.conn.execute(
                    "DELETE FROM image_tags WHERE image_id = ?1 AND tag_id = ?2",
                    params![image_id, tag_id],
                )?;
            }
        }

        Ok(())
    }

    /// Delete a tag completely (removes from all images)
    pub fn delete_tag(&self, tag_id: i64) -> Result<()> {
        // First remove all image associations (cascades via FK, but explicit is clearer)
        self.conn.execute(
            "DELETE FROM image_tags WHERE tag_id = ?1",
            params![tag_id],
        )?;

        // Then delete the tag itself
        self.conn.execute(
            "DELETE FROM tags WHERE id = ?1",
            params![tag_id],
        )?;

        Ok(())
    }

    /// Reorder folders within the same parent
    pub fn reorder_folders(&self, folder_ids: &[String]) -> Result<()> {
        for (index, folder_id) in folder_ids.iter().enumerate() {
            self.conn.execute(
                "UPDATE folders SET sort_order = ?1, updated_at = ?2 WHERE id = ?3",
                params![index as i32, chrono::Utc::now().timestamp(), folder_id],
            )?;
        }
        Ok(())
    }

    // ========== App Settings ==========

    /// Get auto-tag settings from database
    pub fn get_auto_tag_settings(&self) -> Result<crate::vision::AutoTagSettings> {
        let result: Option<String> = self.conn.query_row(
            "SELECT value FROM app_settings WHERE key = 'auto_tag_settings'",
            [],
            |row| row.get(0),
        ).ok();

        match result {
            Some(json) => {
                serde_json::from_str(&json)
                    .map_err(|e| anyhow::anyhow!("Failed to parse auto-tag settings: {}", e))
            }
            None => Ok(crate::vision::AutoTagSettings::default()),
        }
    }

    /// Save auto-tag settings to database
    pub fn set_auto_tag_settings(&self, settings: &crate::vision::AutoTagSettings) -> Result<()> {
        let json = serde_json::to_string(settings)?;
        let now = chrono::Utc::now().timestamp();

        self.conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value, updated_at)
             VALUES ('auto_tag_settings', ?1, ?2)",
            params![json, now],
        )?;

        Ok(())
    }

    /// Bulk add tags with category and confidence (for auto-tagging)
    pub fn bulk_add_tags_with_category(
        &self,
        image_id: &str,
        tags: &[crate::vision::TagWithConfidence],
    ) -> Result<()> {
        for tag_info in tags {
            // Get or create tag with category
            self.conn.execute(
                "INSERT INTO tags (name, category) VALUES (?1, ?2)
                 ON CONFLICT(name) DO UPDATE SET category = COALESCE(excluded.category, category)",
                params![tag_info.tag, format!("{:?}", tag_info.category).to_lowercase()],
            )?;

            let tag_id: i64 = self.conn.query_row(
                "SELECT id FROM tags WHERE name = ?1",
                params![tag_info.tag],
                |row| row.get(0),
            )?;

            // Add tag to image
            self.conn.execute(
                "INSERT OR IGNORE INTO image_tags (image_id, tag_id) VALUES (?1, ?2)",
                params![image_id, tag_id],
            )?;
        }

        Ok(())
    }



    // ========== Template History Operations ==========

    /// Get recent template history (last 5 used templates)
    pub fn get_recent_template_history(&self) -> Result<Vec<crate::batch::TemplateHistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, template, used_at, image_count FROM batch_template_history ORDER BY used_at DESC LIMIT 5"
        )?;

        let entries = stmt.query_map([], |row| {
            Ok(crate::batch::TemplateHistoryEntry {
                id: row.get(0)?,
                template: row.get(1)?,
                used_at: row.get(2)?,
                image_count: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Save a template to history after generation
    pub fn save_template_to_history(&self, template: &str, image_count: i64) -> Result<()> {
        use chrono::Utc;
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO batch_template_history (template, used_at, image_count) VALUES (?1, ?2, ?3)",
            params![template, now, image_count],
        )?;

        Ok(())
    }

    // ========== Settings Operations ==========

    /// Get a setting value
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Set a setting value
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        self.conn.execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![key, value, now],
        )?;

        Ok(())
    }

    /// Delete a setting
    pub fn delete_setting(&self, key: &str) -> Result<()> {
        self.conn.execute("DELETE FROM app_settings WHERE key = ?1", params![key])?;
        Ok(())
    }

    /// Get all settings as key-value pairs
    pub fn get_all_settings(&self) -> Result<std::collections::HashMap<String, String>> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM app_settings")?;
        let settings = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<std::collections::HashMap<_, _>, _>>()?;

        Ok(settings)
    }

    // ========== LoRA Operations ==========

    /// Get all LoRAs from database
    pub fn get_all_loras(&self) -> Result<Vec<crate::models::LoraInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, path, trigger_words, base_model, size_bytes, created_at, metadata
             FROM loras ORDER BY name"
        )?;

        let loras = stmt.query_map([], |row| {
            let metadata_json: Option<String> = row.get(7)?;
            let metadata = metadata_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            Ok(crate::models::LoraInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                trigger_words: row.get(3)?,
                base_model: row.get(4)?,
                size_bytes: row.get(5)?,
                created_at: row.get(6)?,
                metadata,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(loras)
    }

    /// Insert or update a LoRA
    pub fn upsert_lora(&self, lora: &crate::models::LoraInfo) -> Result<()> {
        let metadata_json = serde_json::to_string(&lora.metadata)?;

        self.conn.execute(
            "INSERT INTO loras (id, name, path, trigger_words, base_model, size_bytes, created_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                path = excluded.path,
                trigger_words = excluded.trigger_words,
                base_model = excluded.base_model,
                size_bytes = excluded.size_bytes,
                metadata = excluded.metadata",
            params![
                lora.id,
                lora.name,
                lora.path,
                lora.trigger_words,
                lora.base_model,
                lora.size_bytes as i64,
                lora.created_at,
                metadata_json,
            ],
        )?;

        Ok(())
    }

    /// Delete a LoRA by ID
    pub fn delete_lora(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM loras WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Get a LoRA by ID
    pub fn get_lora(&self, id: &str) -> Result<Option<crate::models::LoraInfo>> {
        let result = self.conn.query_row(
            "SELECT id, name, path, trigger_words, base_model, size_bytes, created_at, metadata
             FROM loras WHERE id = ?1",
            params![id],
            |row| {
                let metadata_json: Option<String> = row.get(7)?;
                let metadata = metadata_json
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default();

                Ok(crate::models::LoraInfo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    trigger_words: row.get(3)?,
                    base_model: row.get(4)?,
                    size_bytes: row.get(5)?,
                    created_at: row.get(6)?,
                    metadata,
                })
            },
        );

        match result {
            Ok(lora) => Ok(Some(lora)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    // ========== Model Bundle System Operations ==========

    // --- Component Operations ---

    /// Insert a model component
    pub fn insert_component(&self, comp: &ComponentRecord) -> Result<()> {
        let metadata_json = comp.metadata.as_ref()
            .map(|m| serde_json::to_string(m).ok())
            .flatten();

        self.conn.execute(
            "INSERT INTO model_components (
                id, component_type, format, file_path, file_size, file_hash, name,
                repo_id, repo_snapshot, architecture, quantization, supports_loras,
                is_sharded, shard_count, vram_mb, discovered_at, last_verified_at,
                is_available, metadata
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
             ON CONFLICT(file_path) DO UPDATE SET
                name = excluded.name,
                component_type = excluded.component_type,
                format = excluded.format,
                file_size = excluded.file_size,
                last_verified_at = excluded.last_verified_at,
                is_available = excluded.is_available",
            params![
                comp.id,
                comp.component_type,
                comp.format,
                comp.file_path,
                comp.file_size,
                comp.file_hash,
                comp.name,
                comp.repo_id,
                comp.repo_snapshot,
                comp.architecture,
                comp.quantization,
                comp.supports_loras as i32,
                comp.is_sharded as i32,
                comp.shard_count,
                comp.vram_mb,
                comp.discovered_at,
                comp.last_verified_at,
                comp.is_available as i32,
                metadata_json,
            ],
        )?;

        Ok(())
    }

    /// Get component by ID
    pub fn get_component(&self, id: &str) -> Result<ComponentRecord> {
        self.conn.query_row(
            "SELECT id, component_type, format, file_path, file_size, file_hash, name,
                    repo_id, repo_snapshot, architecture, quantization, supports_loras,
                    is_sharded, shard_count, vram_mb, discovered_at, last_verified_at,
                    is_available, metadata
             FROM model_components WHERE id = ?1",
            params![id],
            |row| {
                let metadata_json: Option<String> = row.get(18)?;
                let metadata = metadata_json.and_then(|s| serde_json::from_str(&s).ok());

                Ok(ComponentRecord {
                    id: row.get(0)?,
                    component_type: row.get(1)?,
                    format: row.get(2)?,
                    file_path: row.get(3)?,
                    file_size: row.get(4)?,
                    file_hash: row.get(5)?,
                    name: row.get(6)?,
                    repo_id: row.get(7)?,
                    repo_snapshot: row.get(8)?,
                    architecture: row.get(9)?,
                    quantization: row.get(10)?,
                    supports_loras: row.get::<_, i32>(11)? != 0,
                    is_sharded: row.get::<_, i32>(12)? != 0,
                    shard_count: row.get(13)?,
                    vram_mb: row.get(14)?,
                    discovered_at: row.get(15)?,
                    last_verified_at: row.get(16)?,
                    is_available: row.get::<_, i32>(17)? != 0,
                    metadata,
                })
            },
        ).map_err(Into::into)
    }

    /// Get all components
    pub fn get_all_components(&self) -> Result<Vec<ComponentRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, component_type, format, file_path, file_size, file_hash, name,
                    repo_id, repo_snapshot, architecture, quantization, supports_loras,
                    is_sharded, shard_count, vram_mb, discovered_at, last_verified_at,
                    is_available, metadata
             FROM model_components ORDER BY name"
        )?;

        let components = stmt.query_map([], |row| {
            let metadata_json: Option<String> = row.get(18)?;
            let metadata = metadata_json.and_then(|s| serde_json::from_str(&s).ok());

            Ok(ComponentRecord {
                id: row.get(0)?,
                component_type: row.get(1)?,
                format: row.get(2)?,
                file_path: row.get(3)?,
                file_size: row.get(4)?,
                file_hash: row.get(5)?,
                name: row.get(6)?,
                repo_id: row.get(7)?,
                repo_snapshot: row.get(8)?,
                architecture: row.get(9)?,
                quantization: row.get(10)?,
                supports_loras: row.get::<_, i32>(11)? != 0,
                is_sharded: row.get::<_, i32>(12)? != 0,
                shard_count: row.get(13)?,
                vram_mb: row.get(14)?,
                discovered_at: row.get(15)?,
                last_verified_at: row.get(16)?,
                is_available: row.get::<_, i32>(17)? != 0,
                metadata,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(components)
    }

    /// Get components by type
    pub fn get_components_by_type(&self, comp_type: &str) -> Result<Vec<ComponentRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, component_type, format, file_path, file_size, file_hash, name,
                    repo_id, repo_snapshot, architecture, quantization, supports_loras,
                    is_sharded, shard_count, vram_mb, discovered_at, last_verified_at,
                    is_available, metadata
             FROM model_components WHERE component_type = ?1 ORDER BY name"
        )?;

        let components = stmt.query_map(params![comp_type], |row| {
            let metadata_json: Option<String> = row.get(18)?;
            let metadata = metadata_json.and_then(|s| serde_json::from_str(&s).ok());

            Ok(ComponentRecord {
                id: row.get(0)?,
                component_type: row.get(1)?,
                format: row.get(2)?,
                file_path: row.get(3)?,
                file_size: row.get(4)?,
                file_hash: row.get(5)?,
                name: row.get(6)?,
                repo_id: row.get(7)?,
                repo_snapshot: row.get(8)?,
                architecture: row.get(9)?,
                quantization: row.get(10)?,
                supports_loras: row.get::<_, i32>(11)? != 0,
                is_sharded: row.get::<_, i32>(12)? != 0,
                shard_count: row.get(13)?,
                vram_mb: row.get(14)?,
                discovered_at: row.get(15)?,
                last_verified_at: row.get(16)?,
                is_available: row.get::<_, i32>(17)? != 0,
                metadata,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(components)
    }

    /// Find a component by repo_id and component_type
    /// Used to find tokenizers associated with encoders from the same repo
    pub fn find_component_by_repo_and_type(&self, repo_id: &str, comp_type: &str) -> Result<ComponentRecord> {
        self.conn.query_row(
            "SELECT id, component_type, format, file_path, file_size, file_hash, name,
                    repo_id, repo_snapshot, architecture, quantization, supports_loras,
                    is_sharded, shard_count, vram_mb, discovered_at, last_verified_at,
                    is_available, metadata
             FROM model_components
             WHERE repo_id = ?1 AND component_type = ?2 AND is_available = 1
             LIMIT 1",
            params![repo_id, comp_type],
            |row| {
                let metadata_json: Option<String> = row.get(18)?;
                let metadata = metadata_json.and_then(|s| serde_json::from_str(&s).ok());

                Ok(ComponentRecord {
                    id: row.get(0)?,
                    component_type: row.get(1)?,
                    format: row.get(2)?,
                    file_path: row.get(3)?,
                    file_size: row.get(4)?,
                    file_hash: row.get(5)?,
                    name: row.get(6)?,
                    repo_id: row.get(7)?,
                    repo_snapshot: row.get(8)?,
                    architecture: row.get(9)?,
                    quantization: row.get(10)?,
                    supports_loras: row.get::<_, i32>(11)? != 0,
                    is_sharded: row.get::<_, i32>(12)? != 0,
                    shard_count: row.get(13)?,
                    vram_mb: row.get(14)?,
                    discovered_at: row.get(15)?,
                    last_verified_at: row.get(16)?,
                    is_available: row.get::<_, i32>(17)? != 0,
                    metadata,
                })
            },
        ).map_err(|e| anyhow::anyhow!("Component not found: {}", e))
    }

    /// Update component availability
    pub fn update_component_availability(&self, id: &str, available: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE model_components SET is_available = ?1, last_verified_at = ?2 WHERE id = ?3",
            params![available as i32, chrono::Utc::now().timestamp(), id],
        )?;
        Ok(())
    }

    /// Check if a repo has been scanned with a specific snapshot hash
    /// Returns true if components from this repo+snapshot exist in the database
    pub fn has_repo_snapshot(&self, repo_id: &str, snapshot_hash: &str) -> Result<bool> {
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM model_components WHERE repo_id = ?1 AND repo_snapshot = ?2",
            params![repo_id, snapshot_hash],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    // --- Bundle Operations ---

    /// Insert a model bundle
    pub fn insert_bundle(&self, bundle: &BundleRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO model_bundles (
                id, name, description, bundle_type, model_family, default_steps,
                default_guidance, step_min, step_max, total_vram_mb, is_complete,
                is_active, created_at, updated_at, validation_errors
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                bundle.id,
                bundle.name,
                bundle.description,
                bundle.bundle_type,
                bundle.model_family,
                bundle.default_steps,
                bundle.default_guidance,
                bundle.step_min,
                bundle.step_max,
                bundle.total_vram_mb,
                bundle.is_complete as i32,
                bundle.is_active as i32,
                bundle.created_at,
                bundle.updated_at,
                bundle.validation_errors,
            ],
        )?;

        Ok(())
    }

    /// Get bundle by ID with all component details
    pub fn get_bundle(&self, id: &str) -> Result<BundleInfo> {
        // Get bundle record
        let bundle: BundleRecord = self.conn.query_row(
            "SELECT id, name, description, bundle_type, model_family, default_steps,
                    default_guidance, step_min, step_max, total_vram_mb, is_complete,
                    is_active, created_at, updated_at, validation_errors
             FROM model_bundles WHERE id = ?1",
            params![id],
            |row| {
                Ok(BundleRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    bundle_type: row.get(3)?,
                    model_family: row.get(4)?,
                    default_steps: row.get(5)?,
                    default_guidance: row.get(6)?,
                    step_min: row.get(7)?,
                    step_max: row.get(8)?,
                    total_vram_mb: row.get(9)?,
                    is_complete: row.get::<_, i32>(10)? != 0,
                    is_active: row.get::<_, i32>(11)? != 0,
                    created_at: row.get(12)?,
                    updated_at: row.get(13)?,
                    validation_errors: row.get(14)?,
                })
            },
        )?;

        // Get associated components
        let components = self.get_bundle_components(&bundle.id)?;

        Ok(BundleInfo {
            id: bundle.id,
            name: bundle.name,
            description: bundle.description,
            bundle_type: bundle.bundle_type,
            model_family: bundle.model_family,
            default_steps: bundle.default_steps,
            default_guidance: bundle.default_guidance,
            step_min: bundle.step_min,
            step_max: bundle.step_max,
            total_vram_mb: bundle.total_vram_mb,
            is_complete: bundle.is_complete,
            is_active: bundle.is_active,
            created_at: bundle.created_at,
            updated_at: bundle.updated_at,
            validation_errors: bundle.validation_errors,
            components,
        })
    }

    /// Get all bundles with component details
    pub fn get_all_bundles(&self) -> Result<Vec<BundleInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, bundle_type, model_family, default_steps,
                    default_guidance, step_min, step_max, total_vram_mb, is_complete,
                    is_active, created_at, updated_at, validation_errors
             FROM model_bundles ORDER BY name"
        )?;

        let bundle_records: Vec<BundleRecord> = stmt.query_map([], |row| {
            Ok(BundleRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                bundle_type: row.get(3)?,
                model_family: row.get(4)?,
                default_steps: row.get(5)?,
                default_guidance: row.get(6)?,
                step_min: row.get(7)?,
                step_max: row.get(8)?,
                total_vram_mb: row.get(9)?,
                is_complete: row.get::<_, i32>(10)? != 0,
                is_active: row.get::<_, i32>(11)? != 0,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
                validation_errors: row.get(14)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        // Load components for each bundle
        let mut bundles = Vec::new();
        for bundle in bundle_records {
            let components = self.get_bundle_components(&bundle.id)?;
            bundles.push(BundleInfo {
                id: bundle.id,
                name: bundle.name,
                description: bundle.description,
                bundle_type: bundle.bundle_type,
                model_family: bundle.model_family,
                default_steps: bundle.default_steps,
                default_guidance: bundle.default_guidance,
                step_min: bundle.step_min,
                step_max: bundle.step_max,
                total_vram_mb: bundle.total_vram_mb,
                is_complete: bundle.is_complete,
                is_active: bundle.is_active,
                created_at: bundle.created_at,
                updated_at: bundle.updated_at,
                validation_errors: bundle.validation_errors,
                components,
            });
        }

        Ok(bundles)
    }

    /// Update bundle
    pub fn update_bundle(&self, id: &str, name: Option<&str>, description: Option<&str>) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        if let Some(n) = name {
            self.conn.execute(
                "UPDATE model_bundles SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![n, now, id],
            )?;
        }

        if let Some(d) = description {
            self.conn.execute(
                "UPDATE model_bundles SET description = ?1, updated_at = ?2 WHERE id = ?3",
                params![d, now, id],
            )?;
        }

        Ok(())
    }

    /// Delete bundle
    pub fn delete_bundle(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM model_bundles WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Set bundle as active (deactivates all others)
    pub fn set_bundle_active(&self, id: &str, active: bool) -> Result<()> {
        if active {
            // Deactivate all bundles first
            self.conn.execute("UPDATE model_bundles SET is_active = 0", [])?;
        }

        // Activate the specified bundle
        self.conn.execute(
            "UPDATE model_bundles SET is_active = ?1, updated_at = ?2 WHERE id = ?3",
            params![active as i32, chrono::Utc::now().timestamp(), id],
        )?;

        Ok(())
    }

    /// Deactivate all bundles
    pub fn deactivate_all_bundles(&self) -> Result<()> {
        self.conn.execute("UPDATE model_bundles SET is_active = 0", [])?;
        Ok(())
    }

    /// Get active bundle
    pub fn get_active_bundle(&self) -> Result<Option<BundleInfo>> {
        let result = self.conn.query_row(
            "SELECT id FROM model_bundles WHERE is_active = 1 LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        );

        match result {
            Ok(id) => Ok(Some(self.get_bundle(&id)?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    // --- Bundle Component Relationships ---

    /// Add component to bundle
    pub fn add_component_to_bundle(&self, bundle_id: &str, comp_id: &str, role: &str, is_required: bool, priority: i32) -> Result<()> {
        self.conn.execute(
            "INSERT INTO bundle_components (bundle_id, component_id, component_role, is_required, priority)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(bundle_id, component_role, component_id) DO UPDATE SET
                is_required = excluded.is_required,
                priority = excluded.priority",
            params![bundle_id, comp_id, role, is_required as i32, priority],
        )?;
        Ok(())
    }

    /// Remove component from bundle by role
    pub fn remove_component_from_bundle(&self, bundle_id: &str, role: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM bundle_components WHERE bundle_id = ?1 AND component_role = ?2",
            params![bundle_id, role],
        )?;
        Ok(())
    }

    /// Get components for a bundle (with details)
    pub fn get_bundle_components(&self, bundle_id: &str) -> Result<Vec<ComponentInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                c.id, bc.component_role, c.name, c.component_type, c.format,
                c.file_path, c.file_size, c.quantization, c.vram_mb, c.is_available,
                bc.is_required, bc.priority
             FROM bundle_components bc
             JOIN model_components c ON bc.component_id = c.id
             WHERE bc.bundle_id = ?1
             ORDER BY bc.priority DESC, c.component_type"
        )?;

        let components = stmt.query_map(params![bundle_id], |row| {
            Ok(ComponentInfo {
                id: row.get(0)?,
                role: row.get(1)?,
                name: row.get(2)?,
                component_type: row.get(3)?,
                format: row.get(4)?,
                file_path: row.get(5)?,
                file_size: row.get(6)?,
                quantization: row.get(7)?,
                vram_mb: row.get(8)?,
                is_available: row.get::<_, i32>(9)? != 0,
                is_required: row.get::<_, i32>(10)? != 0,
                priority: row.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(components)
    }

    // ========== Session-Based Image Management Methods ==========

    /// Create a pending image entry before generation starts
    pub fn create_pending_image(
        &self,
        image_id: &str,
        params: &crate::queue::GenerationParams,
        session_id: &str,
    ) -> Result<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

        // Construct model name from bundle or component
        let model_name = if let Some(ref bundle_id) = params.bundle_id {
            format!("bundle:{}", bundle_id)
        } else {
            format!("component:{}", params.model_component_id)
        };

        // Convert sampler enum to string
        let sampler = params.sampler.as_ref().map(|s| format!("{:?}", s));

        self.conn.execute(
            "INSERT INTO images (
                id, prompt, negative_prompt, model_name, steps, cfg_scale, seed, sampler,
                created_at, updated_at, status, session_id
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                image_id,
                params.prompt,
                params.negative_prompt,
                model_name,
                params.steps,
                params.cfg_scale,
                params.seed,
                sampler,
                now,
                now,
                "pending",
                session_id,
            ],
        )?;

        // Insert into FTS table for search
        self.conn.execute(
            "INSERT INTO images_fts (image_id, prompt, negative_prompt)
             VALUES (?1, ?2, ?3)",
            params![
                image_id,
                params.prompt,
                params.negative_prompt.as_deref().unwrap_or("")
            ],
        )?;

        Ok(())
    }

    /// Update image when generation completes successfully
    pub fn update_image_on_completion(
        &self,
        image_id: &str,
        file_path: &str,
        thumbnail_path: Option<String>,
        width: i32,
        height: i32,
        file_size: i64,
        generation_time_ms: i64,
    ) -> Result<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

        self.conn.execute(
            "UPDATE images
             SET file_path = ?1, thumbnail_path = ?2, width = ?3, height = ?4,
                 file_size = ?5, generation_time_ms = ?6, status = ?7, updated_at = ?8
             WHERE id = ?9",
            params![
                file_path,
                thumbnail_path,
                width,
                height,
                file_size,
                generation_time_ms,
                "completed",
                now,
                image_id,
            ],
        )?;

        Ok(())
    }

    /// Update image when generation fails
    pub fn update_image_on_failure(&self, image_id: &str, _error: &str) -> Result<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

        self.conn.execute(
            "UPDATE images SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params!["failed", now, image_id],
        )?;

        Ok(())
    }

    /// Update only the status of an image
    pub fn update_image_status(&self, image_id: &str, status: &str) -> Result<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

        self.conn.execute(
            "UPDATE images SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, now, image_id],
        )?;

        Ok(())
    }

    /// Cleanup pending images on startup (crash recovery)
    /// Returns the number of images marked as failed
    pub fn cleanup_pending_images(&self) -> Result<usize> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

        let count = self.conn.execute(
            "UPDATE images SET status = ?1, updated_at = ?2
             WHERE status = ?3 OR status = ?4",
            params!["failed", now, "pending", "processing"],
        )?;

        Ok(count)
    }

    /// Query images by session and status
    pub fn get_images_by_session_status(
        &self,
        session_id: &str,
        status: &str,
        limit: i64,
    ) -> Result<Vec<GalleryImage>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_path, thumbnail_path, created_at, width, height,
                    file_size, is_favorite, prompt, negative_prompt, model_name,
                    steps, cfg_scale, seed, sampler, status, session_id, updated_at
             FROM images
             WHERE session_id = ?1 AND status = ?2
             ORDER BY created_at DESC
             LIMIT ?3"
        )?;

        let images = stmt.query_map(params![session_id, status, limit], |row| {
            let image_id: String = row.get(0)?;
            let tags = self.get_image_tags(&image_id).unwrap_or_default();
            let folder_ids = self.get_image_folder_ids(&image_id).unwrap_or_default();

            Ok(GalleryImage {
                id: image_id,
                file_path: row.get(1)?,
                thumbnail_path: row.get(2)?,
                created_at: row.get(3)?,
                width: row.get(4)?,
                height: row.get(5)?,
                file_size: row.get(6)?,
                is_favorite: row.get::<_, i32>(7)? != 0,
                prompt: row.get(8)?,
                negative_prompt: row.get(9)?,
                model_name: row.get(10)?,
                steps: row.get(11)?,
                cfg_scale: row.get(12)?,
                seed: row.get(13)?,
                sampler: row.get(14)?,
                tags,
                folder_ids,
                status: row.get(15)?,
                session_id: row.get(16)?,
                updated_at: row.get(17)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(images)
    }

    /// Query images by status only (for gallery - all sessions)
    pub fn get_images_by_status(&self, status: &str, limit: i64) -> Result<Vec<GalleryImage>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_path, thumbnail_path, created_at, width, height,
                    file_size, is_favorite, prompt, negative_prompt, model_name,
                    steps, cfg_scale, seed, sampler, status, session_id, updated_at
             FROM images
             WHERE status = ?1
             ORDER BY created_at DESC
             LIMIT ?2"
        )?;

        let images = stmt.query_map(params![status, limit], |row| {
            let image_id: String = row.get(0)?;
            let tags = self.get_image_tags(&image_id).unwrap_or_default();
            let folder_ids = self.get_image_folder_ids(&image_id).unwrap_or_default();

            Ok(GalleryImage {
                id: image_id,
                file_path: row.get(1)?,
                thumbnail_path: row.get(2)?,
                created_at: row.get(3)?,
                width: row.get(4)?,
                height: row.get(5)?,
                file_size: row.get(6)?,
                is_favorite: row.get::<_, i32>(7)? != 0,
                prompt: row.get(8)?,
                negative_prompt: row.get(9)?,
                model_name: row.get(10)?,
                steps: row.get(11)?,
                cfg_scale: row.get(12)?,
                seed: row.get(13)?,
                sampler: row.get(14)?,
                tags,
                folder_ids,
                status: row.get(15)?,
                session_id: row.get(16)?,
                updated_at: row.get(17)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(images)
    }

    // ========== New Model/Bundle Response Methods ==========

    /// Convert a ComponentRecord into a ModelInfoResponse (for frontend API)
    fn component_to_model_info(&self, comp: &ComponentRecord) -> Result<ModelInfoResponse> {
        let tags = self.get_model_tags(&comp.id)?;
        let examples = self.get_examples("model", &comp.id)?;

        Ok(ModelInfoResponse {
            id: comp.id.clone(),
            model_type: component_type_to_model_type(&comp.component_type).to_string(),
            family: infer_family(comp.architecture.as_deref()).to_string(),
            display_name: comp.name.clone(),
            description: None,
            files: vec![ModelFileInfoResponse {
                id: comp.id.clone(),
                model_id: comp.id.clone(),
                path: comp.file_path.clone(),
                resolved_path: comp.file_path.clone(),
                sha256: comp.file_hash.clone(),
                size_bytes: comp.file_size,
                is_symlink: false,
            }],
            tags,
            examples,
            prefs_base: None,
            prefs_lora: None,
            trigger_words: vec![],
            architecture: comp.architecture.clone(),
            quantization: comp.quantization.clone(),
            vram_mb: comp.vram_mb,
            created_at: chrono::DateTime::from_timestamp(comp.discovered_at, 0)
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_default(),
            updated_at: comp.last_verified_at
                .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_default(),
        })
    }

    /// Get all models as frontend-compatible ModelInfoResponse
    pub fn get_all_models_response(&self) -> Result<Vec<ModelInfoResponse>> {
        let components = self.get_all_components()?;
        let mut models = Vec::with_capacity(components.len());
        for comp in &components {
            models.push(self.component_to_model_info(comp)?);
        }
        Ok(models)
    }

    /// Convert BundleInfo to BundleInfoResponse
    fn bundle_to_response(&self, bundle: &BundleInfo) -> Result<BundleInfoResponse> {
        let items: Vec<BundleItemInfoResponse> = bundle.components.iter().map(|c| {
            BundleItemInfoResponse {
                id: c.id.clone(),
                model_id: c.id.clone(),
                role: c.role.clone(),
                model_display_name: c.name.clone(),
                model_family: {
                    // Look up the component to get architecture for family inference
                    self.get_component(&c.id).ok()
                        .map(|comp| infer_family(comp.architecture.as_deref()).to_string())
                        .unwrap_or_else(|| "other".to_string())
                },
                model_type: component_type_to_model_type(&c.component_type).to_string(),
                model_vram_mb: c.vram_mb,
                model_quantization: c.quantization.clone(),
            }
        }).collect();

        let examples = self.get_examples("bundle", &bundle.id)?;

        Ok(BundleInfoResponse {
            id: bundle.id.clone(),
            display_name: bundle.name.clone(),
            description: bundle.description.clone(),
            is_active: bundle.is_active,
            is_complete: bundle.is_complete,
            total_vram_mb: bundle.total_vram_mb.unwrap_or(0),
            tags: vec![],
            items,
            examples,
            created_at: chrono::DateTime::from_timestamp(bundle.created_at, 0)
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_default(),
            updated_at: chrono::DateTime::from_timestamp(bundle.updated_at, 0)
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_default(),
        })
    }

    /// Get all bundles as frontend-compatible BundleInfoResponse
    pub fn get_all_bundles_response(&self) -> Result<Vec<BundleInfoResponse>> {
        let bundles = self.get_all_bundles()?;
        let mut responses = Vec::with_capacity(bundles.len());
        for bundle in &bundles {
            responses.push(self.bundle_to_response(bundle)?);
        }
        Ok(responses)
    }

    /// Update model display name
    pub fn update_model_name(&self, model_id: &str, name: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE model_components SET name = ?1 WHERE id = ?2",
            params![name, model_id],
        )?;
        Ok(())
    }

    /// Get tags for a model
    pub fn get_model_tags(&self, model_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT tag FROM model_tags WHERE model_id = ?1 ORDER BY tag"
        )?;
        let tags = stmt.query_map(params![model_id], |row| {
            row.get::<_, String>(0)
        })?.collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    /// Add a tag to a model
    pub fn add_model_tag(&self, model_id: &str, tag: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO model_tags (model_id, tag) VALUES (?1, ?2)",
            params![model_id, tag],
        )?;
        Ok(())
    }

    /// Remove a tag from a model
    pub fn remove_model_tag(&self, model_id: &str, tag: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM model_tags WHERE model_id = ?1 AND tag = ?2",
            params![model_id, tag],
        )?;
        Ok(())
    }

    /// Get examples for an entity
    pub fn get_examples(&self, entity_type: &str, entity_id: &str) -> Result<Vec<ExampleResponse>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, entity_type, entity_id, example_type, content, created_at
             FROM examples WHERE entity_type = ?1 AND entity_id = ?2 ORDER BY created_at"
        )?;
        let examples = stmt.query_map(params![entity_type, entity_id], |row| {
            Ok(ExampleResponse {
                id: row.get(0)?,
                entity_type: row.get(1)?,
                entity_id: row.get(2)?,
                example_type: row.get(3)?,
                content: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(examples)
    }

    /// Add an example
    pub fn add_example(&self, entity_type: &str, entity_id: &str, example_type: &str, content: &str) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        self.conn.execute(
            "INSERT INTO examples (id, entity_type, entity_id, example_type, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, entity_type, entity_id, example_type, content, now],
        )?;
        Ok(id)
    }

    /// Remove an example by ID
    pub fn remove_example(&self, example_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM examples WHERE id = ?1",
            params![example_id],
        )?;
        Ok(())
    }

    /// Get compatible models for a given base model and target type
    pub fn get_compatible_models(&self, _base_model_id: &str, target_type: &str) -> Result<Vec<ModelInfoResponse>> {
        // Compatibility is family-based: return all models of target_type matching the base model's family
        let base = self.get_component(_base_model_id)?;
        let family = infer_family(base.architecture.as_deref());

        let all_components = self.get_all_components()?;
        let mut results = Vec::new();
        for comp in &all_components {
            let comp_model_type = component_type_to_model_type(&comp.component_type);
            let comp_family = infer_family(comp.architecture.as_deref());
            if comp_model_type == target_type && comp_family == family {
                results.push(self.component_to_model_info(comp)?);
            }
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gallery_db_init_and_insert() {
        // Create in-memory database
        let db = GalleryDb::new(":memory:").unwrap();

        // Initialize schema
        db.init_schema().unwrap();

        // Insert test ImageMetadata
        let test_metadata = ImageMetadata {
            id: "test-id-123".to_string(),
            file_path: Some("/path/to/test.png".to_string()),
            thumbnail_path: None,
            prompt: "test prompt".to_string(),
            created_at: 1234567890,
            width: Some(1024),
            height: Some(1024),
            file_size: Some(123456),
            model_name: "flux-schnell".to_string(),
            negative_prompt: None,
            steps: Some(4),
            cfg_scale: Some(3.5),
            seed: Some(42),
            sampler: Some("Euler".to_string()),
            generation_time_ms: Some(5000),
            status: "completed".to_string(),
            session_id: Some("test-session".to_string()),
            updated_at: 1234567890,
        };
        db.insert_image(&test_metadata).unwrap();

        // Retrieve with get_recent_images
        let images = db.get_recent_images(10).unwrap();

        // Assert length is 1 and id matches
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].id, "test-id-123");
    }
}
