//! Gallery database and metadata management

use rusqlite::{Connection, params};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub id: String,
    pub file_path: String,
    pub prompt: String,
    pub created_at: i64,
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
            "INSERT OR REPLACE INTO presets
             (id, name, mode, prompt, negative_prompt, steps, cfg_scale, width, height,
              seed, model_id, lora_ids, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
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
