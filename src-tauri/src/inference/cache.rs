//! Model cache manager for pipeline reuse
//!
//! Manages FluxPipeline instances to avoid reloading models between jobs.
//! Includes configurable model retention, LRU embedding cache, and idle timeout.

use crate::models::ModelType;
use anyhow::Result;
use candle_core::Device;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, info, trace};

use super::embedding_cache::EmbeddingCache;
use super::flux_pipeline::FluxPipeline;

/// Configuration for model caching behavior
#[derive(Clone, Debug)]
pub struct ModelCacheConfig {
    /// Keep VAE loaded between generations (small, always needed)
    pub keep_vae_loaded: bool,
    /// Keep FLUX transformer loaded (large ~12GB, unload to free VRAM)
    pub keep_flux_loaded: bool,
    /// Keep T5 encoder loaded (huge ~9GB, unload after encoding)
    pub keep_t5_loaded: bool,
    /// Keep CLIP encoder loaded
    pub keep_clip_loaded: bool,
    /// Maximum number of cached embeddings
    pub embedding_cache_size: usize,
    /// Auto-unload all models after this many seconds of inactivity (None = never)
    pub idle_timeout_secs: Option<u64>,
}

impl Default for ModelCacheConfig {
    fn default() -> Self {
        Self {
            keep_vae_loaded: true,       // Always keep VAE (small)
            keep_flux_loaded: true,      // Keep FLUX loaded for fast batch processing
            keep_t5_loaded: false,       // Unload T5 after encoding (large, ~9GB)
            keep_clip_loaded: false,     // Unload CLIP between generations
            embedding_cache_size: 10,    // Cache up to 10 prompts
            idle_timeout_secs: Some(300), // 5 minute idle timeout
        }
    }
}

impl ModelCacheConfig {
    /// Configuration that keeps all models loaded for fast batch processing
    pub fn keep_all_loaded() -> Self {
        Self {
            keep_vae_loaded: true,
            keep_flux_loaded: true,
            keep_t5_loaded: true,
            keep_clip_loaded: true,
            embedding_cache_size: 20,
            idle_timeout_secs: Some(600), // 10 minute timeout
        }
    }

    /// Configuration that aggressively frees memory after each generation
    pub fn memory_saver() -> Self {
        Self {
            keep_vae_loaded: false,
            keep_flux_loaded: false,
            keep_t5_loaded: false,
            keep_clip_loaded: false,
            embedding_cache_size: 5,
            idle_timeout_secs: Some(60), // 1 minute timeout
        }
    }
}

/// Cache statistics for monitoring
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct CacheStats {
    pub embedding_hits: u64,
    pub embedding_misses: u64,
    pub cached_embeddings: usize,
    pub pipeline_reuses: u64,
    pub pipeline_recreations: u64,
    pub current_model_type: Option<String>,
    pub models_loaded: ModelsLoaded,
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct ModelsLoaded {
    pub t5: bool,
    pub clip: bool,
    pub vae: bool,
    pub flux: bool,
}

/// Cached pipeline manager
///
/// Thread-safe manager for FluxPipeline instances. Handles:
/// - Pipeline reuse when generating with the same model type
/// - Embedding caching to skip re-encoding identical prompts
/// - Configurable model retention between generations
/// - Automatic cleanup on idle timeout
pub struct ModelCache {
    pipeline: Mutex<Option<FluxPipeline>>,
    current_model_type: Mutex<Option<ModelType>>,
    device: Device,
    config: Mutex<ModelCacheConfig>,
    last_activity: Mutex<Instant>,
    embedding_cache: Mutex<EmbeddingCache>,
    stats: Mutex<CacheStats>,
}

impl ModelCache {
    /// Create a new model cache with the given device and configuration
    pub fn new(device: Device, config: ModelCacheConfig) -> Arc<Self> {
        let embedding_cache_size = config.embedding_cache_size;
        Arc::new(Self {
            pipeline: Mutex::new(None),
            current_model_type: Mutex::new(None),
            device,
            config: Mutex::new(config),
            last_activity: Mutex::new(Instant::now()),
            embedding_cache: Mutex::new(EmbeddingCache::new(embedding_cache_size)),
            stats: Mutex::new(CacheStats::default()),
        })
    }

    /// Get device reference
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get or create pipeline for the given model type
    ///
    /// Returns Ok(true) if a new pipeline was created, Ok(false) if existing was reused
    pub async fn get_or_create_pipeline(&self, model_type: ModelType) -> Result<bool> {
        self.touch_activity().await;

        let mut pipeline_guard = self.pipeline.lock().await;
        let mut model_type_guard = self.current_model_type.lock().await;
        let mut stats = self.stats.lock().await;

        let needs_new = match (&*pipeline_guard, &*model_type_guard) {
            (None, _) => true,
            (_, Some(current)) if *current != model_type => {
                info!(from = ?current, to = ?model_type, "Model type changed, recreating pipeline");
                true
            }
            _ => false,
        };

        if needs_new {
            info!(model = ?model_type, "Creating new pipeline");
            *pipeline_guard = Some(FluxPipeline::with_model_type(self.device.clone(), model_type.clone())?);
            *model_type_guard = Some(model_type.clone());
            stats.pipeline_recreations += 1;
            stats.current_model_type = Some(model_type.to_string());
            Ok(true)
        } else {
            debug!(model = ?model_type, "Reusing existing pipeline");
            stats.pipeline_reuses += 1;
            Ok(false)
        }
    }

    /// Access the pipeline for generation
    ///
    /// The callback receives a mutable reference to the pipeline.
    /// This pattern ensures the pipeline lock is held only during generation.
    pub async fn with_pipeline<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut FluxPipeline) -> Result<R>,
    {
        trace!("with_pipeline: touching activity");
        self.touch_activity().await;
        trace!("with_pipeline: acquiring pipeline lock");
        let mut guard = self.pipeline.lock().await;
        trace!("with_pipeline: lock acquired");
        let pipeline = guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Pipeline not initialized - call get_or_create_pipeline first"))?;
        trace!("with_pipeline: calling closure");
        let result = f(pipeline);
        trace!("with_pipeline: closure returned");
        result
    }

    /// Access the embedding cache
    pub async fn with_embedding_cache<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut EmbeddingCache) -> R,
    {
        let mut guard = self.embedding_cache.lock().await;
        f(&mut guard)
    }

    /// Apply cache config to unload models based on settings
    pub async fn apply_post_generation_cleanup(&self) {
        let config = self.config.lock().await.clone();
        let mut guard = self.pipeline.lock().await;

        if let Some(pipeline) = guard.as_mut() {
            pipeline.apply_cache_config(&config);
            debug!(
                keep_vae = config.keep_vae_loaded,
                keep_flux = config.keep_flux_loaded,
                keep_t5 = config.keep_t5_loaded,
                "Post-generation cleanup applied"
            );
        }
    }

    /// Check and handle idle timeout
    pub async fn check_idle_timeout(&self) -> bool {
        let config = self.config.lock().await;
        if let Some(timeout_secs) = config.idle_timeout_secs {
            let last = *self.last_activity.lock().await;
            if last.elapsed() > Duration::from_secs(timeout_secs) {
                drop(config); // Release config lock before clearing
                info!(timeout_secs = timeout_secs, "Idle timeout reached, clearing cache");
                self.clear().await;
                return true;
            }
        }
        false
    }

    /// Clear all cached models and embeddings
    pub async fn clear(&self) {
        info!("Clearing all cached models and embeddings");
        *self.pipeline.lock().await = None;
        *self.current_model_type.lock().await = None;
        self.embedding_cache.lock().await.clear();

        let mut stats = self.stats.lock().await;
        stats.current_model_type = None;
        stats.models_loaded = ModelsLoaded::default();

        // Synchronize device to ensure GPU memory is actually freed
        if let Err(e) = self.device.synchronize() {
            tracing::warn!("Failed to synchronize device during cache clear: {}", e);
        }
        info!("GPU memory freed and device synchronized");
    }

    /// Update the cache configuration
    pub async fn set_config(&self, config: ModelCacheConfig) {
        let new_cache_size = config.embedding_cache_size;
        *self.config.lock().await = config;

        // Recreate embedding cache with new size if changed
        let mut emb_cache = self.embedding_cache.lock().await;
        if new_cache_size != emb_cache.len() {
            *emb_cache = EmbeddingCache::new(new_cache_size);
        }
    }

    /// Get current configuration
    pub async fn get_config(&self) -> ModelCacheConfig {
        self.config.lock().await.clone()
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let mut stats = self.stats.lock().await.clone();
        let emb_cache = self.embedding_cache.lock().await;
        let (hits, misses) = emb_cache.stats();
        stats.embedding_hits = hits;
        stats.embedding_misses = misses;
        stats.cached_embeddings = emb_cache.len();
        stats
    }

    async fn touch_activity(&self) {
        *self.last_activity.lock().await = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_creation() {
        let config = ModelCacheConfig::default();
        let cache = ModelCache::new(Device::Cpu, config);
        assert!(cache.pipeline.lock().await.is_none());
    }

    #[tokio::test]
    async fn test_config_presets() {
        let default = ModelCacheConfig::default();
        assert!(default.keep_flux_loaded);  // Keep FLUX loaded by default for performance
        assert!(default.keep_vae_loaded);

        let keep_all = ModelCacheConfig::keep_all_loaded();
        assert!(keep_all.keep_flux_loaded);
        assert!(keep_all.keep_t5_loaded);

        let memory_saver = ModelCacheConfig::memory_saver();
        assert!(!memory_saver.keep_vae_loaded);
        assert_eq!(memory_saver.idle_timeout_secs, Some(60));
    }
}
