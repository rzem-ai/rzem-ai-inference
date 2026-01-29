use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use std::collections::HashMap;
use anyhow::Result;
use super::ModelConfig;

/// In-memory cache for model configurations
pub struct ModelConfigCache {
    configs: Arc<RwLock<HashMap<String, ModelConfig>>>,
    db: Arc<Mutex<Option<crate::gallery::GalleryDb>>>,
}

impl ModelConfigCache {
    pub fn new(db: Arc<Mutex<Option<crate::gallery::GalleryDb>>>) -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            db,
        }
    }

    /// Load all model configs from database
    pub async fn load_all(&self) -> Result<()> {
        let db_guard = self.db.lock().await;
        let db = db_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;

        let records = db.get_all_models()
            .map_err(|e| anyhow::anyhow!("Failed to load models: {}", e))?;

        let mut configs = self.configs.write().await;
        configs.clear();

        for record in records {
            // Only load transformer models
            if record.component_type.as_deref() == Some("transformer") {
                if let Ok(config) = ModelConfig::from_record(&record) {
                    configs.insert(config.id.clone(), config);
                }
            }
        }

        tracing::info!(count = configs.len(), "Loaded model configurations");
        Ok(())
    }

    /// Get config by model ID with fallback
    pub async fn get_config(&self, model_id: &str) -> Result<ModelConfig> {
        // Check cache
        {
            let configs = self.configs.read().await;
            if let Some(config) = configs.get(model_id) {
                return Ok(config.clone());
            }
        }

        // Try reloading from DB
        if self.load_all().await.is_ok() {
            let configs = self.configs.read().await;
            if let Some(config) = configs.get(model_id) {
                return Ok(config.clone());
            }
        }

        // Fallback to hard-coded defaults
        tracing::warn!(
            model_id = model_id,
            "Using fallback config - model not in database"
        );
        Ok(ModelConfig::fallback(model_id))
    }

    /// Get all available model configs
    pub async fn get_all(&self) -> Vec<ModelConfig> {
        let configs = self.configs.read().await;
        configs.values().cloned().collect()
    }

    /// Invalidate cache (call after DB updates)
    pub async fn invalidate(&self) {
        let mut configs = self.configs.write().await;
        configs.clear();
    }
}
