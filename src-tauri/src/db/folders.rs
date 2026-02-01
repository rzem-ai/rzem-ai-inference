use super::{InferenceDb, Folder, FolderNode, GalleryImage};
use rusqlite::params;
use anyhow::Result;

impl InferenceDb {
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
    pub(super) fn get_image_folder_ids(&self, image_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT folder_id FROM image_folders WHERE image_id = ?1"
        )?;

        let ids = stmt.query_map(params![image_id], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(ids)
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

}
