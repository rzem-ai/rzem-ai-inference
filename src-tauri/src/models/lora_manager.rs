//! LoRA collection management
//!
//! Handles scanning, importing, removing, and caching LoRA adapters.

use super::lora::{LoraAdapter, LoraInfo, LoraFileInfo, get_lora_file_info};
use anyhow::{Context, Result};
use candle_core::{DType, Device};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Manages the collection of available LoRAs
pub struct LoraManager {
    /// Directory where LoRAs are stored
    loras_dir: PathBuf,
    /// Path to the metadata index file
    index_path: PathBuf,
    /// Cached loaded adapters (keyed by ID)
    loaded: Arc<RwLock<HashMap<String, Arc<LoraAdapter>>>>,
    /// Metadata index (keyed by ID)
    index: Arc<RwLock<LoraIndex>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct LoraIndex {
    loras: HashMap<String, LoraInfo>,
}

impl LoraManager {
    /// Create a new LoRA manager
    ///
    /// Uses ~/.rzem-ai-inference/loras/ as the storage directory
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        let base_dir = home.join(".rzem-ai-inference");
        let loras_dir = base_dir.join("loras");
        let index_path = base_dir.join("loras.json");

        // Ensure directories exist
        std::fs::create_dir_all(&loras_dir)?;

        // Load existing index
        let index = if index_path.exists() {
            let content = std::fs::read_to_string(&index_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            LoraIndex::default()
        };

        Ok(Self {
            loras_dir,
            index_path,
            loaded: Arc::new(RwLock::new(HashMap::new())),
            index: Arc::new(RwLock::new(index)),
        })
    }

    /// Get the LoRAs directory path
    pub fn loras_dir(&self) -> &Path {
        &self.loras_dir
    }

    /// Scan the LoRAs directory and return all available LoRAs
    pub async fn scan_loras(&self) -> Result<Vec<LoraInfo>> {
        let mut index = self.index.write().await;
        let mut found_ids = Vec::new();

        // Scan directory for .safetensors files
        if let Ok(entries) = std::fs::read_dir(&self.loras_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "safetensors") {
                    // Check if this file is already indexed
                    let existing = index.loras.values().find(|l| l.path == path.to_string_lossy());

                    if let Some(lora) = existing {
                        found_ids.push(lora.id.clone());
                    } else {
                        // New LoRA file - add to index
                        let id = Uuid::new_v4().to_string();
                        let name = path.file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Unknown".to_string());

                        let metadata = std::fs::metadata(&path)?;

                        let info = LoraInfo {
                            id: id.clone(),
                            name,
                            path: path.to_string_lossy().to_string(),
                            trigger_words: None,
                            base_model: Some("flux".to_string()),
                            size_bytes: metadata.len(),
                            created_at: chrono::Utc::now().timestamp(),
                            metadata: HashMap::new(),
                        };

                        index.loras.insert(id.clone(), info);
                        found_ids.push(id);
                    }
                }
            }
        }

        // Remove entries for files that no longer exist
        let to_remove: Vec<String> = index.loras.iter()
            .filter(|(_, info)| !Path::new(&info.path).exists())
            .map(|(id, _)| id.clone())
            .collect();

        for id in to_remove {
            index.loras.remove(&id);
            // Also remove from loaded cache
            self.loaded.write().await.remove(&id);
        }

        // Save updated index
        self.save_index(&index)?;

        Ok(index.loras.values().cloned().collect())
    }

    /// Get all known LoRAs without rescanning
    pub async fn get_loras(&self) -> Vec<LoraInfo> {
        self.index.read().await.loras.values().cloned().collect()
    }

    /// Get info about a specific LoRA
    pub async fn get_lora_info(&self, id: &str) -> Option<LoraInfo> {
        self.index.read().await.loras.get(id).cloned()
    }

    /// Import a LoRA from an external path
    ///
    /// Copies the file to the LoRAs directory and adds to index
    pub async fn import_lora(
        &self,
        source_path: &Path,
        name: &str,
        trigger_words: Option<&str>,
    ) -> Result<LoraInfo> {
        // Validate source exists and is a safetensors file
        if !source_path.exists() {
            anyhow::bail!("Source file does not exist: {}", source_path.display());
        }

        if source_path.extension().is_none_or(|e| e != "safetensors") {
            anyhow::bail!("LoRA file must be a .safetensors file");
        }

        // Generate new ID and destination path
        let id = Uuid::new_v4().to_string();
        let dest_filename = format!("{}.safetensors", id);
        let dest_path = self.loras_dir.join(&dest_filename);

        // Copy file
        std::fs::copy(source_path, &dest_path)
            .with_context(|| format!("Failed to copy LoRA file to {}", dest_path.display()))?;

        let metadata = std::fs::metadata(&dest_path)?;

        let info = LoraInfo {
            id: id.clone(),
            name: name.to_string(),
            path: dest_path.to_string_lossy().to_string(),
            trigger_words: trigger_words.map(|s| s.to_string()),
            base_model: Some("flux".to_string()),
            size_bytes: metadata.len(),
            created_at: chrono::Utc::now().timestamp(),
            metadata: HashMap::new(),
        };

        // Add to index
        {
            let mut index = self.index.write().await;
            index.loras.insert(id.clone(), info.clone());
            self.save_index(&index)?;
        }

        info!(id = %id, name = %name, "Imported LoRA");

        Ok(info)
    }

    /// Remove a LoRA from the collection
    ///
    /// Deletes the file and removes from index
    pub async fn remove_lora(&self, id: &str) -> Result<()> {
        let mut index = self.index.write().await;

        let info = index.loras.remove(id)
            .ok_or_else(|| anyhow::anyhow!("LoRA not found: {}", id))?;

        // Delete the file if it's in our loras directory
        let path = Path::new(&info.path);
        if path.starts_with(&self.loras_dir) && path.exists() {
            std::fs::remove_file(path)
                .with_context(|| format!("Failed to delete LoRA file: {}", path.display()))?;
        }

        // Remove from loaded cache
        self.loaded.write().await.remove(id);

        // Save updated index
        self.save_index(&index)?;

        info!(id = %id, name = %info.name, "Removed LoRA");

        Ok(())
    }

    /// Update LoRA metadata
    pub async fn update_lora(
        &self,
        id: &str,
        name: Option<&str>,
        trigger_words: Option<Option<&str>>,
    ) -> Result<LoraInfo> {
        let mut index = self.index.write().await;

        let info = index.loras.get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("LoRA not found: {}", id))?;

        if let Some(new_name) = name {
            info.name = new_name.to_string();
        }

        if let Some(new_trigger_words) = trigger_words {
            info.trigger_words = new_trigger_words.map(|s| s.to_string());
        }

        let info = info.clone();
        self.save_index(&index)?;

        Ok(info)
    }

    /// Load a LoRA adapter, using cache if available
    pub async fn get_or_load_lora(
        &self,
        id: &str,
        device: &Device,
        dtype: DType,
    ) -> Result<Arc<LoraAdapter>> {
        // Check cache first
        {
            let loaded = self.loaded.read().await;
            if let Some(adapter) = loaded.get(id) {
                return Ok(Arc::clone(adapter));
            }
        }

        // Load from disk
        let info = self.get_lora_info(id).await
            .ok_or_else(|| anyhow::anyhow!("LoRA not found: {}", id))?;

        let mut adapter = LoraAdapter::load(&info.path, id.to_string(), info.name.clone(), device, dtype)?;

        // Apply metadata
        if let Some(trigger_words) = &info.trigger_words {
            adapter.trigger_words = Some(trigger_words.clone());
        }

        let adapter = Arc::new(adapter);

        // Cache it
        {
            let mut loaded = self.loaded.write().await;
            loaded.insert(id.to_string(), Arc::clone(&adapter));
        }

        Ok(adapter)
    }

    /// Clear the loaded adapter cache
    pub async fn clear_cache(&self) {
        self.loaded.write().await.clear();
        debug!("Cleared LoRA adapter cache");
    }

    /// Get file info for a LoRA without fully loading it
    pub fn get_file_info(&self, path: &Path) -> Result<LoraFileInfo> {
        get_lora_file_info(path)
    }

    /// Save the index to disk
    fn save_index(&self, index: &LoraIndex) -> Result<()> {
        let content = serde_json::to_string_pretty(index)?;
        std::fs::write(&self.index_path, content)?;
        Ok(())
    }
}

impl Default for LoraManager {
    fn default() -> Self {
        Self::new().expect("Failed to create LoRA manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_lora_manager_creation() {
        // Just test that we can create a manager
        let manager = LoraManager::new();
        assert!(manager.is_ok());
    }
}
