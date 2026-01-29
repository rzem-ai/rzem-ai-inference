//! Application settings management
//!
//! Stores user settings in database (app_settings table)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    /// HuggingFace API token for downloading gated models
    #[serde(default)]
    pub hf_token: Option<String>,

    /// Claude API key for AI features
    #[serde(default)]
    pub claude_api_key: Option<String>,

    /// Fal.ai API key for cloud inference
    #[serde(default)]
    pub fal_key: Option<String>,
}

impl AppSettings {
    /// Get the settings file path
    fn settings_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("Could not determine home directory")?;
        Ok(home.join(".rzem-ai-inference").join("settings.json"))
    }

    /// Load settings from disk, or return defaults if not found
    pub fn load() -> Result<Self> {
        let path = Self::settings_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .context("Failed to read settings file")?;

        let settings: AppSettings = serde_json::from_str(&content)
            .context("Failed to parse settings file")?;

        Ok(settings)
    }

    /// Save settings to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::settings_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create settings directory")?;
        }

        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize settings")?;

        fs::write(&path, content)
            .context("Failed to write settings file")?;

        Ok(())
    }

    /// Get the HuggingFace token
    pub fn get_hf_token(&self) -> Option<&str> {
        self.hf_token.as_deref()
    }

    /// Set the HuggingFace token
    pub fn set_hf_token(&mut self, token: Option<String>) {
        self.hf_token = token.filter(|t| !t.trim().is_empty());
    }

    /// Get the Claude API key
    pub fn get_claude_api_key(&self) -> Option<&str> {
        self.claude_api_key.as_deref()
    }

    /// Set the Claude API key
    pub fn set_claude_api_key(&mut self, key: Option<String>) {
        self.claude_api_key = key.filter(|k| !k.trim().is_empty());
    }

    /// Get the Fal.ai API key
    pub fn get_fal_key(&self) -> Option<&str> {
        self.fal_key.as_deref()
    }

    /// Set the Fal.ai API key
    pub fn set_fal_key(&mut self, key: Option<String>) {
        self.fal_key = key.filter(|k| !k.trim().is_empty());
    }

    /// Load settings from database
    pub fn load_from_db(db: &crate::gallery::GalleryDb) -> Result<Self> {
        Ok(Self {
            hf_token: db.get_setting("hf_token")?,
            claude_api_key: db.get_setting("claude_api_key")?,
            fal_key: db.get_setting("fal_key")?,
        })
    }

    /// Save settings to database
    pub fn save_to_db(&self, db: &crate::gallery::GalleryDb) -> Result<()> {
        if let Some(token) = &self.hf_token {
            db.set_setting("hf_token", token)?;
        } else {
            db.delete_setting("hf_token")?;
        }

        if let Some(key) = &self.claude_api_key {
            db.set_setting("claude_api_key", key)?;
        } else {
            db.delete_setting("claude_api_key")?;
        }

        if let Some(key) = &self.fal_key {
            db.set_setting("fal_key", key)?;
        } else {
            db.delete_setting("fal_key")?;
        }

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert!(settings.hf_token.is_none());
    }

    #[test]
    fn test_settings_serialization() {
        let mut settings = AppSettings::default();
        settings.set_hf_token(Some("test_token".to_string()));

        let json = serde_json::to_string(&settings).unwrap();
        let parsed: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.hf_token, Some("test_token".to_string()));
    }
}
