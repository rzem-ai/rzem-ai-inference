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
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub prompt: String,
    pub created_at: i64,
}

/// Image metadata for frontend API (camelCase)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryImage {
    pub id: String,
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub created_at: i64,
    pub width: i32,
    pub height: i32,
    pub file_size: i64,
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

pub struct GalleryDb {
    conn: Connection,
}

impl GalleryDb {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    /// Run schema migrations for existing databases
    fn run_migrations(&self) -> Result<()> {
        // Add color and category columns to tags table if they don't exist
        // SQLite doesn't have IF NOT EXISTS for ALTER TABLE, so we check first
        let has_color: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('tags') WHERE name = 'color'",
            [],
            |row| row.get(0),
        ).unwrap_or(false);

        if !has_color {
            self.conn.execute("ALTER TABLE tags ADD COLUMN color TEXT", [])?;
        }

        let has_category: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('tags') WHERE name = 'category'",
            [],
            |row| row.get(0),
        ).unwrap_or(false);

        if !has_category {
            self.conn.execute("ALTER TABLE tags ADD COLUMN category TEXT", [])?;
        }

        Ok(())
    }

    pub fn init_schema(&self) -> Result<()> {
        // Create images table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS images (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL UNIQUE,
                thumbnail_path TEXT,
                created_at INTEGER NOT NULL,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                file_size INTEGER NOT NULL,
                is_favorite INTEGER DEFAULT 0,
                prompt TEXT NOT NULL,
                negative_prompt TEXT,
                model_name TEXT NOT NULL,
                steps INTEGER,
                cfg_scale REAL,
                seed INTEGER,
                sampler TEXT,
                server_id TEXT,
                generation_time_ms INTEGER
            )",
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

        // Create models table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS models (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                path TEXT,
                size_bytes INTEGER,
                is_downloaded INTEGER DEFAULT 0,
                is_active INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                last_used_at INTEGER,
                metadata TEXT
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

        // Run migrations for existing databases
        self.run_migrations()?;

        Ok(())
    }

    pub fn insert_image(&self, metadata: &ImageMetadata) -> Result<()> {
        // Insert into images table with hardcoded defaults
        self.conn.execute(
            "INSERT INTO images (id, file_path, thumbnail_path, prompt, created_at, width, height, file_size, model_name)
             VALUES (?1, ?2, ?3, ?4, ?5, 1024, 1024, 0, 'flux-schnell')",
            params![metadata.id, metadata.file_path, metadata.thumbnail_path, metadata.prompt, metadata.created_at],
        )?;

        // Insert into FTS table
        self.conn.execute(
            "INSERT INTO images_fts (image_id, prompt, negative_prompt)
             VALUES (?1, ?2, ?3)",
            params![
                metadata.id,
                metadata.prompt,
                ""
            ],
        )?;

        Ok(())
    }

    pub fn get_recent_images(&self, limit: usize) -> Result<Vec<ImageMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_path, thumbnail_path, prompt, created_at
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
                    steps, cfg_scale, seed, sampler
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
            "SELECT i.id, i.file_path, i.thumbnail_path, i.prompt, i.created_at
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
            "SELECT id, file_path, thumbnail_path, prompt, created_at
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
                stats.model_load_ms,
                stats.t5_load_ms,
                stats.clip_load_ms,
                stats.vae_load_ms,
                stats.flux_load_ms,
                stats.t5_encode_ms,
                stats.clip_encode_ms,
                stats.denoise_ms,
                stats.vae_decode_ms,
                stats.png_encode_ms,
                stats.total_ms,
                stats.steps,
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
                model_load_ms: row.get(0)?,
                t5_load_ms: row.get(1)?,
                clip_load_ms: row.get(2)?,
                vae_load_ms: row.get(3)?,
                flux_load_ms: row.get(4)?,
                t5_encode_ms: row.get(5)?,
                clip_encode_ms: row.get(6)?,
                denoise_ms: row.get(7)?,
                vae_decode_ms: row.get(8)?,
                png_encode_ms: row.get(9)?,
                total_ms: row.get(10)?,
                steps: row.get(11)?,
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
                    steps, cfg_scale, seed, sampler
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
            file_path: "/path/to/test.png".to_string(),
            thumbnail_path: None,
            prompt: "test prompt".to_string(),
            created_at: 1234567890,
        };
        db.insert_image(&test_metadata).unwrap();

        // Retrieve with get_recent_images
        let images = db.get_recent_images(10).unwrap();

        // Assert length is 1 and id matches
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].id, "test-id-123");
    }
}
