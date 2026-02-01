use super::InferenceDb;
use rusqlite::params;
use anyhow::Result;

impl InferenceDb {
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

}
