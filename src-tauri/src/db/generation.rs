use super::{InferenceDb, GenerationPreset};
use rusqlite::params;
use anyhow::Result;
use crate::inference::GenerationStats;

impl InferenceDb {
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
                stats.model_load_ms.map(|v| v as i64),
                stats.t5_load_ms.map(|v| v as i64),
                stats.clip_load_ms.map(|v| v as i64),
                stats.vae_load_ms.map(|v| v as i64),
                stats.flux_load_ms.map(|v| v as i64),
                stats.t5_encode_ms as i64,
                stats.clip_encode_ms as i64,
                stats.denoise_ms as i64,
                stats.vae_decode_ms as i64,
                stats.png_encode_ms as i64,
                stats.total_ms as i64,
                stats.steps as i64,
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
                model_load_ms: row.get::<_, Option<i64>>(0)?.map(|v| v as u64),
                t5_load_ms: row.get::<_, Option<i64>>(1)?.map(|v| v as u64),
                clip_load_ms: row.get::<_, Option<i64>>(2)?.map(|v| v as u64),
                vae_load_ms: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
                flux_load_ms: row.get::<_, Option<i64>>(4)?.map(|v| v as u64),
                t5_encode_ms: row.get::<_, i64>(5)? as u64,
                clip_encode_ms: row.get::<_, i64>(6)? as u64,
                denoise_ms: row.get::<_, i64>(7)? as u64,
                vae_decode_ms: row.get::<_, i64>(8)? as u64,
                png_encode_ms: row.get::<_, i64>(9)? as u64,
                total_ms: row.get::<_, i64>(10)? as u64,
                steps: row.get::<_, i64>(11)? as usize,
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

    /// Get recent template history (last 5 used templates)
    pub fn get_recent_template_history(&self) -> Result<Vec<crate::batch::TemplateHistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, template, used_at, image_count FROM batch_template_history ORDER BY used_at DESC LIMIT 5"
        )?;

        let entries = stmt.query_map([], |row| {
            Ok(crate::batch::TemplateHistoryEntry {
                id: row.get(0)?,
                template: row.get(1)?,
                used_at: row.get(2)?,
                image_count: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Save a template to history after generation
    pub fn save_template_to_history(&self, template: &str, image_count: i64) -> Result<()> {
        use chrono::Utc;
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO batch_template_history (template, used_at, image_count) VALUES (?1, ?2, ?3)",
            params![template, now, image_count],
        )?;

        Ok(())
    }

}
