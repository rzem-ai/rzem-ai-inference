use super::{InferenceDb, ImageMetadata, GalleryImage};
use rusqlite::params;
use anyhow::Result;

impl InferenceDb {

    pub fn insert_image(&self, metadata: &ImageMetadata) -> Result<()> {
        // Serialize loras to JSON if present
        let loras_json = metadata.loras.as_ref()
            .map(|loras| serde_json::to_string(loras).ok())
            .flatten();

        // Insert into images table with all metadata
        self.conn.execute(
            "INSERT INTO images (
                id, file_path, thumbnail_path, prompt, created_at,
                width, height, file_size, model_name, negative_prompt,
                steps, cfg_scale, seed, sampler, generation_time_ms,
                status, session_id, updated_at, loras
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
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
                loras_json,
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
                    status, session_id, updated_at, loras
             FROM images
             ORDER BY created_at DESC
             LIMIT ?1"
        )?;

        let images = stmt.query_map(params![limit], |row| {
            // Parse loras JSON if present
            let loras: Option<Vec<super::LoraInfo>> = row.get::<_, Option<String>>(18)?
                .and_then(|s| serde_json::from_str(&s).ok());

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
                loras,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(images)
    }


    pub fn get_gallery_images(&self, limit: usize) -> Result<Vec<GalleryImage>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_path, thumbnail_path, created_at, width, height,
                    file_size, is_favorite, prompt, negative_prompt, model_name,
                    steps, cfg_scale, seed, sampler, status, session_id, updated_at, loras
             FROM images
             ORDER BY created_at DESC
             LIMIT ?1"
        )?;

        let mut images: Vec<GalleryImage> = stmt.query_map(params![limit], |row| {
            // Parse loras JSON if present
            let loras: Option<Vec<super::LoraInfo>> = row.get::<_, Option<String>>(18)?
                .and_then(|s| serde_json::from_str(&s).ok());

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
                loras,
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
                    i.status, i.session_id, i.updated_at, i.loras
             FROM images i
             JOIN images_fts fts ON i.id = fts.image_id
             WHERE images_fts MATCH ?1
             ORDER BY i.created_at DESC
             LIMIT 100"
        )?;

        let images = stmt.query_map(params![query], |row| {
            // Parse loras JSON if present
            let loras: Option<Vec<super::LoraInfo>> = row.get::<_, Option<String>>(18)?
                .and_then(|s| serde_json::from_str(&s).ok());

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
                loras,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(images)
    }


    pub fn get_image_by_id(&self, image_id: &str) -> Result<Option<ImageMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_path, thumbnail_path, prompt, created_at,
                    width, height, file_size, model_name, negative_prompt,
                    steps, cfg_scale, seed, sampler, generation_time_ms,
                    status, session_id, updated_at, loras
             FROM images
             WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![image_id])?;

        if let Some(row) = rows.next()? {
            // Parse loras JSON if present
            let loras: Option<Vec<super::LoraInfo>> = row.get::<_, Option<String>>(18)?
                .and_then(|s| serde_json::from_str(&s).ok());

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
                loras,
            }))
        } else {
            Ok(None)
        }
    }


    pub fn toggle_favorite(&self, image_id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE images SET is_favorite = NOT is_favorite WHERE id = ?1",
            params![image_id],
        )?;

        Ok(())
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

        // Serialize loras to JSON if present
        let loras_json = if !params.loras.is_empty() {
            Some(serde_json::to_string(&params.loras).ok()).flatten()
        } else {
            None
        };

        self.conn.execute(
            "INSERT INTO images (
                id, prompt, negative_prompt, model_name, steps, cfg_scale, seed, sampler,
                created_at, updated_at, status, session_id, loras
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
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
                loras_json,
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
                    steps, cfg_scale, seed, sampler, status, session_id, updated_at, loras
             FROM images
             WHERE session_id = ?1 AND status = ?2
             ORDER BY created_at DESC
             LIMIT ?3"
        )?;

        let images = stmt.query_map(params![session_id, status, limit], |row| {
            let image_id: String = row.get(0)?;
            let tags = self.get_image_tags(&image_id).unwrap_or_default();
            let folder_ids = self.get_image_folder_ids(&image_id).unwrap_or_default();

            // Parse loras JSON if present
            let loras: Option<Vec<super::LoraInfo>> = row.get::<_, Option<String>>(18)?
                .and_then(|s| serde_json::from_str(&s).ok());

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
                loras,
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
                    steps, cfg_scale, seed, sampler, status, session_id, updated_at, loras
             FROM images
             WHERE status = ?1
             ORDER BY created_at DESC
             LIMIT ?2"
        )?;

        let images = stmt.query_map(params![status, limit], |row| {
            let image_id: String = row.get(0)?;
            let tags = self.get_image_tags(&image_id).unwrap_or_default();
            let folder_ids = self.get_image_folder_ids(&image_id).unwrap_or_default();

            // Parse loras JSON if present
            let loras: Option<Vec<super::LoraInfo>> = row.get::<_, Option<String>>(18)?
                .and_then(|s| serde_json::from_str(&s).ok());

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
                loras,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(images)
    }

    /// Get gallery images by IDs with tags and folders
    pub(super) fn get_gallery_images_by_ids(&self, image_ids: &[String]) -> Result<Vec<GalleryImage>> {
        if image_ids.is_empty() {
            return Ok(vec![]);
        }

        // Create placeholders for IN clause
        let placeholders: Vec<String> = (1..=image_ids.len()).map(|i| format!("?{}", i)).collect();
        let query = format!(
            "SELECT id, file_path, thumbnail_path, created_at, width, height,
                    file_size, is_favorite, prompt, negative_prompt, model_name,
                    steps, cfg_scale, seed, sampler, status, session_id, updated_at, loras
             FROM images
             WHERE id IN ({})
             ORDER BY created_at DESC",
            placeholders.join(", ")
        );

        let mut stmt = self.conn.prepare(&query)?;
        let params: Vec<&dyn rusqlite::ToSql> = image_ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

        let mut images: Vec<GalleryImage> = stmt.query_map(params.as_slice(), |row| {
            // Parse loras JSON if present
            let loras: Option<Vec<super::LoraInfo>> = row.get::<_, Option<String>>(18)?
                .and_then(|s| serde_json::from_str(&s).ok());

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
                loras,
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

}
