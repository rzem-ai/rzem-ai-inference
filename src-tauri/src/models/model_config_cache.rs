use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use std::collections::HashMap;
use anyhow::Result;
use super::ModelConfig;

/// In-memory cache for model configurations
pub struct ModelConfigCache {
    configs: Arc<RwLock<HashMap<String, ModelConfig>>>,
    db: Arc<Mutex<Option<crate::db::InferenceDb>>>,
}

impl ModelConfigCache {
    pub fn new(db: Arc<Mutex<Option<crate::db::InferenceDb>>>) -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            db,
        }
    }

    /// Load all model configs - now empty since we use bundle/component system
    pub async fn load_all(&self) -> Result<()> {
        // Model configs are no longer loaded from old models table
        // Use bundle/component system instead
        let mut configs = self.configs.write().await;
        configs.clear();

        tracing::info!("Model config cache cleared - use bundle/component system");
        Ok(())
    }

    /// Get config by model ID
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

        // No fallback - require database config
        Err(anyhow::anyhow!(
            "Model config '{}' not found in database. Please ensure the model is properly registered.",
            model_id
        ))
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
