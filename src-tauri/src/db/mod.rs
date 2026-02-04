//! Database layer — schema, shared types, and per-table sub-modules

use rusqlite::Connection;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod images;
pub mod tags;
pub mod folders;
pub mod generation;
pub mod loras;
pub mod settings;
pub mod models;
pub mod styles;

pub use models::{
    ModelFileInfoResponse,
    ExampleResponse,
    ModelPrefsBaseResponse,
    ModelPrefsLoraResponse,
    ModelInfoResponse,
    BundleItemInfoResponse,
    BundleInfoResponse,
};

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

/// LoRA configuration for database storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraInfo {
    pub id: String,
    pub strength: f32,
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
    pub scheduler: Option<String>,
    pub generation_time_ms: Option<i64>,
    pub status: String,  // "pending", "processing", "completed", "failed"
    pub session_id: Option<String>,  // UUID for session tracking
    pub updated_at: i64,  // Unix timestamp of last update
    pub loras: Option<Vec<LoraInfo>>,  // LoRA adapters with strengths
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
    pub scheduler: Option<String>,
    pub tags: Vec<String>,
    /// IDs of folders this image belongs to
    pub folder_ids: Vec<String>,
    pub status: String,  // "pending", "processing", "completed", "failed"
    pub session_id: Option<String>,  // UUID for session tracking
    pub updated_at: i64,  // Unix timestamp of last update
    pub loras: Option<Vec<LoraInfo>>,  // LoRA adapters with strengths
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

pub struct InferenceDb {
    conn: Connection,
}

impl InferenceDb {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        // Always initialize schema on new connection
        //db.init_schema()?;
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

        // Add loras column if it doesn't exist (stores JSON array of {id, strength})
        self.conn.execute(
            "ALTER TABLE images ADD COLUMN loras TEXT",
            [],
        ).ok(); // Ignore if column already exists

        // Add scheduler column if it doesn't exist
        self.conn.execute(
            "ALTER TABLE images ADD COLUMN scheduler TEXT",
            [],
        ).ok(); // Ignore if column already exists

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

        // ===== Style Management System Tables =====

        // Create styles table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS styles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                prompt_template TEXT NOT NULL,
                default_strength REAL DEFAULT 1.0,
                strength_min REAL DEFAULT 0.5,
                strength_max REAL DEFAULT 1.5,
                category TEXT,
                thumbnail_path TEXT,
                is_favorite INTEGER DEFAULT 0,
                usage_count INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Create indexes for styles
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_styles_category ON styles(category)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_styles_favorite ON styles(is_favorite)",
            [],
        )?;

        // Create style_loras association table (many-to-many)
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS style_loras (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                style_id TEXT NOT NULL,
                lora_id TEXT NOT NULL,
                strength REAL NOT NULL DEFAULT 1.0,
                priority INTEGER DEFAULT 0,
                FOREIGN KEY (style_id) REFERENCES styles(id) ON DELETE CASCADE,
                FOREIGN KEY (lora_id) REFERENCES loras(id) ON DELETE CASCADE,
                UNIQUE (style_id, lora_id)
            )",
            [],
        )?;

        // Create indexes for style_loras
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_style_loras_style ON style_loras(style_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_style_loras_lora ON style_loras(lora_id)",
            [],
        )?;

        // Create style_examples table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS style_examples (
                id TEXT PRIMARY KEY,
                style_id TEXT NOT NULL,
                example_type TEXT NOT NULL,  -- 'prompt' or 'image'
                content TEXT NOT NULL,       -- prompt text or image_id
                generation_params TEXT,      -- JSON
                created_at INTEGER NOT NULL,
                FOREIGN KEY (style_id) REFERENCES styles(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create index for style_examples
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_style_examples_style ON style_examples(style_id)",
            [],
        )?;

        // Add strength range columns to loras table if they don't exist
        self.conn.execute(
            "ALTER TABLE loras ADD COLUMN default_strength REAL DEFAULT 1.0",
            [],
        ).ok(); // Ignore if column already exists

        self.conn.execute(
            "ALTER TABLE loras ADD COLUMN strength_min REAL DEFAULT 0.5",
            [],
        ).ok();

        self.conn.execute(
            "ALTER TABLE loras ADD COLUMN strength_max REAL DEFAULT 1.5",
            [],
        ).ok();

        // Add CivitAI download fields
        self.conn.execute(
            "ALTER TABLE loras ADD COLUMN download_url TEXT",
            [],
        ).ok(); // Ignore if column already exists

        self.conn.execute(
            "ALTER TABLE loras ADD COLUMN civitai_model_id INTEGER",
            [],
        ).ok();

        self.conn.execute(
            "ALTER TABLE loras ADD COLUMN civitai_version_id INTEGER",
            [],
        ).ok();

        tracing::info!("✅ Model bundle tables created");
        tracing::info!("✅ Database schema initialization complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_db_init_and_insert() {
        // Create in-memory database
        let db = InferenceDb::new(":memory:").unwrap();

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
            scheduler: Some("Normal".to_string()),
            generation_time_ms: Some(5000),
            status: "completed".to_string(),
            session_id: Some("test-session".to_string()),
            updated_at: 1234567890,
            loras: None,
        };
        db.insert_image(&test_metadata).unwrap();

        // Retrieve with get_recent_images
        let images = db.get_recent_images(10).unwrap();

        // Assert length is 1 and id matches
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].id, "test-id-123");
    }
}
