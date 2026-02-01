use super::{InferenceDb, Tag};
use rusqlite::params;
use anyhow::Result;

impl InferenceDb {

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

    /// Get tags for an image
    pub(super) fn get_image_tags(&self, image_id: &str) -> Result<Vec<String>> {
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
