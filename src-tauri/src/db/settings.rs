use super::InferenceDb;
use rusqlite::params;
use anyhow::Result;

impl InferenceDb {
    /// Get a setting value
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Set a setting value
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        self.conn.execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![key, value, now],
        )?;

        Ok(())
    }

    /// Delete a setting
    pub fn delete_setting(&self, key: &str) -> Result<()> {
        self.conn.execute("DELETE FROM app_settings WHERE key = ?1", params![key])?;
        Ok(())
    }

    /// Get all settings as key-value pairs
    pub fn get_all_settings(&self) -> Result<std::collections::HashMap<String, String>> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM app_settings")?;
        let settings = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<std::collections::HashMap<_, _>, _>>()?;

        Ok(settings)
    }

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

}
