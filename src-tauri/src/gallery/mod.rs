//! Gallery database and metadata management

use rusqlite::{Connection, params};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::inference::GenerationStats;

/// Image metadata for internal use (snake_case)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub id: String,
    pub file_path: String,
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

        // Create tags table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL
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

        Ok(())
    }

    pub fn insert_image(&self, metadata: &ImageMetadata) -> Result<()> {
        // Insert into images table with hardcoded defaults
        self.conn.execute(
            "INSERT INTO images (id, file_path, prompt, created_at, width, height, file_size, model_name)
             VALUES (?1, ?2, ?3, ?4, 1024, 1024, 0, 'flux-schnell')",
            params![metadata.id, metadata.file_path, metadata.prompt, metadata.created_at],
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
            "SELECT id, file_path, prompt, created_at
             FROM images
             ORDER BY created_at DESC
             LIMIT ?1"
        )?;

        let images = stmt.query_map(params![limit], |row| {
            Ok(ImageMetadata {
                id: row.get(0)?,
                file_path: row.get(1)?,
                prompt: row.get(2)?,
                created_at: row.get(3)?,
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

        let images = stmt.query_map(params![limit], |row| {
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
                tags: Vec::new(), // TODO: fetch from image_tags table
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(images)
    }

    pub fn search_gallery_images(&self, query: &str) -> Result<Vec<ImageMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT i.id, i.file_path, i.prompt, i.created_at
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
                prompt: row.get(2)?,
                created_at: row.get(3)?,
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
            "SELECT id, file_path, prompt, created_at
             FROM images
             WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![image_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(ImageMetadata {
                id: row.get(0)?,
                file_path: row.get(1)?,
                prompt: row.get(2)?,
                created_at: row.get(3)?,
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
