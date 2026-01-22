//! LRU cache for T5 and CLIP prompt embeddings
//!
//! Caches text encoder outputs to avoid re-encoding identical prompts.
//! Uses simple hash-based keys with LRU eviction.

use candle_core::Tensor;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};

/// Cached T5 and CLIP embeddings for a prompt
#[derive(Clone)]
pub struct CachedEmbedding {
    pub t5_embedding: Tensor,
    pub clip_embedding: Tensor,
}

/// LRU cache for prompt embeddings
///
/// Keeps the most recently used embeddings in memory to avoid
/// re-encoding the same prompts when generating variations.
pub struct EmbeddingCache {
    entries: HashMap<u64, CachedEmbedding>,
    access_order: VecDeque<u64>, // Front = oldest, Back = newest
    max_size: usize,
    hits: u64,
    misses: u64,
}

impl EmbeddingCache {
    /// Create a new embedding cache with the given maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            access_order: VecDeque::new(),
            max_size,
            hits: 0,
            misses: 0,
        }
    }

    /// Hash a prompt to a cache key
    fn hash_prompt(prompt: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        prompt.hash(&mut hasher);
        hasher.finish()
    }

    /// Get cached embeddings for a prompt
    pub fn get(&mut self, prompt: &str) -> Option<CachedEmbedding> {
        let key = Self::hash_prompt(prompt);

        if let Some(entry) = self.entries.get(&key) {
            // Move to back of access order (most recently used)
            self.access_order.retain(|&k| k != key);
            self.access_order.push_back(key);
            self.hits += 1;
            Some(entry.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    /// Insert embeddings for a prompt
    pub fn insert(&mut self, prompt: &str, embedding: CachedEmbedding) {
        let key = Self::hash_prompt(prompt);

        // Evict oldest if at capacity
        while self.entries.len() >= self.max_size && !self.access_order.is_empty() {
            if let Some(oldest_key) = self.access_order.pop_front() {
                self.entries.remove(&oldest_key);
            }
        }

        // Remove existing entry from access order if updating
        self.access_order.retain(|&k| k != key);

        self.entries.insert(key, embedding);
        self.access_order.push_back(key);
    }

    /// Clear all cached embeddings
    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
    }

    /// Get cache statistics (hits, misses)
    pub fn stats(&self) -> (u64, u64) {
        (self.hits, self.misses)
    }

    /// Get the number of cached entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Device;

    fn make_dummy_embedding() -> CachedEmbedding {
        let device = Device::Cpu;
        CachedEmbedding {
            t5_embedding: Tensor::zeros((1, 512, 4096), candle_core::DType::F32, &device).unwrap(),
            clip_embedding: Tensor::zeros((1, 768), candle_core::DType::F32, &device).unwrap(),
        }
    }

    #[test]
    fn test_cache_hit_miss() {
        let mut cache = EmbeddingCache::new(5);

        // Miss on first access
        assert!(cache.get("prompt 1").is_none());
        assert_eq!(cache.stats(), (0, 1));

        // Insert and hit
        cache.insert("prompt 1", make_dummy_embedding());
        assert!(cache.get("prompt 1").is_some());
        assert_eq!(cache.stats(), (1, 1));
    }

    #[test]
    fn test_lru_eviction() {
        let mut cache = EmbeddingCache::new(2);

        cache.insert("a", make_dummy_embedding());
        cache.insert("b", make_dummy_embedding());

        // Access "a" to make it most recent
        cache.get("a");

        // Insert "c" should evict "b" (oldest)
        cache.insert("c", make_dummy_embedding());

        assert!(cache.get("a").is_some()); // Still present
        assert!(cache.get("b").is_none()); // Evicted
        assert!(cache.get("c").is_some()); // Just inserted
    }
}
