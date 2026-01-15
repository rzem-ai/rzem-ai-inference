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
