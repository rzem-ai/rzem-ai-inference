use super::InferenceDb;
use crate::models::{StyleInfo, StyleLoraWithInfo, StyleExample, StyleDetail, StyleRequest};
use rusqlite::params;
use anyhow::{Result, Context};

impl InferenceDb {
    /// Get all styles from database
    pub fn get_all_styles(&self) -> Result<Vec<StyleInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, prompt_template, default_strength, strength_min,
                    strength_max, category, thumbnail_path, is_favorite, usage_count,
                    created_at, updated_at
             FROM styles ORDER BY name"
        )?;

        let styles = stmt.query_map([], |row| {
            Ok(StyleInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                prompt_template: row.get(3)?,
                default_strength: row.get(4)?,
                strength_min: row.get(5)?,
                strength_max: row.get(6)?,
                category: row.get(7)?,
                thumbnail_path: row.get(8)?,
                is_favorite: row.get::<_, i32>(9)? != 0,
                usage_count: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(styles)
    }

    /// Get complete style detail including LoRAs and examples
    pub fn get_style_detail(&self, id: &str) -> Result<Option<StyleDetail>> {
        // Get style info
        let style_info = self.conn.query_row(
            "SELECT id, name, description, prompt_template, default_strength, strength_min,
                    strength_max, category, thumbnail_path, is_favorite, usage_count,
                    created_at, updated_at
             FROM styles WHERE id = ?1",
            params![id],
            |row| {
                Ok(StyleInfo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    prompt_template: row.get(3)?,
                    default_strength: row.get(4)?,
                    strength_min: row.get(5)?,
                    strength_max: row.get(6)?,
                    category: row.get(7)?,
                    thumbnail_path: row.get(8)?,
                    is_favorite: row.get::<_, i32>(9)? != 0,
                    usage_count: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            },
        );

        let info = match style_info {
            Ok(info) => info,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        // Get associated LoRAs
        let mut lora_stmt = self.conn.prepare(
            "SELECT sl.lora_id, l.name, l.trigger_words, sl.strength, sl.priority
             FROM style_loras sl
             JOIN loras l ON sl.lora_id = l.id
             WHERE sl.style_id = ?1
             ORDER BY sl.priority"
        )?;

        let loras = lora_stmt.query_map(params![id], |row| {
            Ok(StyleLoraWithInfo {
                lora_id: row.get(0)?,
                lora_name: row.get(1)?,
                lora_trigger_words: row.get(2)?,
                strength: row.get(3)?,
                priority: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        // Get examples
        let mut example_stmt = self.conn.prepare(
            "SELECT id, style_id, example_type, content, generation_params, created_at
             FROM style_examples
             WHERE style_id = ?1
             ORDER BY created_at DESC"
        )?;

        let examples = example_stmt.query_map(params![id], |row| {
            Ok(StyleExample {
                id: row.get(0)?,
                style_id: row.get(1)?,
                example_type: row.get(2)?,
                content: row.get(3)?,
                generation_params: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(Some(StyleDetail {
            info,
            loras,
            examples,
        }))
    }

    /// Insert or update a style
    pub fn upsert_style(&self, id: &str, request: &StyleRequest) -> Result<StyleInfo> {
        let now = chrono::Utc::now().timestamp();

        self.conn.execute(
            "INSERT INTO styles (id, name, description, prompt_template, default_strength,
                                strength_min, strength_max, category, is_favorite,
                                usage_count, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, ?10)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                prompt_template = excluded.prompt_template,
                default_strength = excluded.default_strength,
                strength_min = excluded.strength_min,
                strength_max = excluded.strength_max,
                category = excluded.category,
                is_favorite = excluded.is_favorite,
                updated_at = excluded.updated_at",
            params![
                id,
                request.name,
                request.description,
                request.prompt_template,
                request.default_strength,
                request.strength_min,
                request.strength_max,
                request.category,
                if request.is_favorite { 1 } else { 0 },
                now,
            ],
        )?;

        // Retrieve and return the complete StyleInfo
        self.conn.query_row(
            "SELECT id, name, description, prompt_template, default_strength, strength_min,
                    strength_max, category, thumbnail_path, is_favorite, usage_count,
                    created_at, updated_at
             FROM styles WHERE id = ?1",
            params![id],
            |row| {
                Ok(StyleInfo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    prompt_template: row.get(3)?,
                    default_strength: row.get(4)?,
                    strength_min: row.get(5)?,
                    strength_max: row.get(6)?,
                    category: row.get(7)?,
                    thumbnail_path: row.get(8)?,
                    is_favorite: row.get::<_, i32>(9)? != 0,
                    usage_count: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            },
        ).context("Failed to retrieve style after upsert")
    }

    /// Delete a style (cascades to style_loras and style_examples)
    pub fn delete_style(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM styles WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Add a LoRA to a style
    pub fn add_lora_to_style(&self, style_id: &str, lora_id: &str, strength: f32, priority: i32) -> Result<()> {
        self.conn.execute(
            "INSERT INTO style_loras (style_id, lora_id, strength, priority)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(style_id, lora_id) DO UPDATE SET
                strength = excluded.strength,
                priority = excluded.priority",
            params![style_id, lora_id, strength, priority],
        )?;
        Ok(())
    }

    /// Remove a LoRA from a style
    pub fn remove_lora_from_style(&self, style_id: &str, lora_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM style_loras WHERE style_id = ?1 AND lora_id = ?2",
            params![style_id, lora_id],
        )?;
        Ok(())
    }

    /// Add an example to a style
    pub fn add_style_example(&self, style_id: &str, example_type: &str, content: &str, generation_params: Option<&str>) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        self.conn.execute(
            "INSERT INTO style_examples (id, style_id, example_type, content, generation_params, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, style_id, example_type, content, generation_params, now],
        )?;

        Ok(id)
    }

    /// Remove an example from a style
    pub fn remove_style_example(&self, example_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM style_examples WHERE id = ?1",
            params![example_id],
        )?;
        Ok(())
    }

    /// Increment usage count for a style
    pub fn increment_style_usage(&self, style_id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE styles SET usage_count = usage_count + 1 WHERE id = ?1",
            params![style_id],
        )?;
        Ok(())
    }

    /// Update thumbnail path for a style
    pub fn update_style_thumbnail(&self, style_id: &str, thumbnail_path: Option<&str>) -> Result<()> {
        self.conn.execute(
            "UPDATE styles SET thumbnail_path = ?1, updated_at = ?2 WHERE id = ?3",
            params![thumbnail_path, chrono::Utc::now().timestamp(), style_id],
        )?;
        Ok(())
    }
}
