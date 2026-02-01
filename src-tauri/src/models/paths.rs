//! Path management for model files - Bundle-based system

use anyhow::Result;
use std::path::PathBuf;
use std::collections::HashMap;

/// Component roles in a model bundle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentRole {
    Transformer,
    T5,
    Clip,
    Vae,
    ClipTokenizer,
    T5Tokenizer,
}

impl ComponentRole {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "transformer" => Ok(Self::Transformer),
            "t5" => Ok(Self::T5),
            "clip" => Ok(Self::Clip),
            "vae" => Ok(Self::Vae),
            "clip_tokenizer" => Ok(Self::ClipTokenizer),
            "t5_tokenizer" => Ok(Self::T5Tokenizer),
            _ => anyhow::bail!("Unknown component role: {}", s),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Transformer => "transformer",
            Self::T5 => "t5",
            Self::Clip => "clip",
            Self::Vae => "vae",
            Self::ClipTokenizer => "clip_tokenizer",
            Self::T5Tokenizer => "t5_tokenizer",
        }
    }
}

/// Manages paths for FLUX model files using bundle system
pub struct ModelPaths {
    bundle_id: Option<String>,
    bundle_components: HashMap<ComponentRole, PathBuf>,
}

impl ModelPaths {
    /// Create new ModelPaths - loads from active bundle
    /// This is the primary constructor that should be used
    pub fn new(db: &crate::db::InferenceDb) -> Result<Self> {
        Self::from_active_bundle(db)
    }

    /// Create ModelPaths from active bundle in database
    pub fn from_active_bundle(db: &crate::db::InferenceDb) -> Result<Self> {
        let bundle_info = db.get_active_bundle()?
            .ok_or_else(|| anyhow::anyhow!("No active bundle configured"))?;

        Self::from_bundle_info(&bundle_info)
    }

    /// Create ModelPaths from a BundleInfo
    pub fn from_bundle_info(bundle: &crate::db::BundleInfo) -> Result<Self> {
        let mut component_paths = HashMap::new();

        for comp in &bundle.components {
            let role = ComponentRole::from_str(&comp.role)?;
            let path = PathBuf::from(&comp.file_path);
            component_paths.insert(role, path);
        }

        Ok(Self {
            bundle_id: Some(bundle.id.clone()),
            bundle_components: component_paths,
        })
    }

    /// Create ModelPaths from individual component IDs
    /// Tokenizers are automatically derived from their associated encoders
    pub fn from_component_ids(
        db: &crate::db::InferenceDb,
        transformer_id: &str,
        t5_id: &str,
        clip_id: &str,
        vae_id: &str,
    ) -> Result<Self> {
        let mut component_paths = HashMap::new();

        // Load transformer (required)
        let transformer = db.get_component(transformer_id)?;
        component_paths.insert(ComponentRole::Transformer, PathBuf::from(&transformer.file_path));

        // Load T5 encoder and find its tokenizer
        let t5 = db.get_component(t5_id)?;
        let t5_path = PathBuf::from(&t5.file_path);
        component_paths.insert(ComponentRole::T5, t5_path.clone());

        // Find T5 tokenizer: first try database, then derive from encoder path
        let t5_tokenizer_path = Self::find_tokenizer_for_encoder(db, &t5, "t5_tokenizer")?;
        component_paths.insert(ComponentRole::T5Tokenizer, t5_tokenizer_path);

        // Load CLIP encoder and find its tokenizer
        let clip = db.get_component(clip_id)?;
        let clip_path = PathBuf::from(&clip.file_path);
        component_paths.insert(ComponentRole::Clip, clip_path.clone());

        // Find CLIP tokenizer: first try database, then derive from encoder path
        let clip_tokenizer_path = Self::find_tokenizer_for_encoder(db, &clip, "clip_tokenizer")?;
        component_paths.insert(ComponentRole::ClipTokenizer, clip_tokenizer_path);

        // Load VAE
        let vae = db.get_component(vae_id)?;
        component_paths.insert(ComponentRole::Vae, PathBuf::from(&vae.file_path));

        // Generate sanitized bundle ID using hash for safe filesystem/DB storage
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        transformer_id.hash(&mut hasher);
        t5_id.hash(&mut hasher);
        clip_id.hash(&mut hasher);
        vae_id.hash(&mut hasher);
        let hash = hasher.finish();

        Ok(Self {
            bundle_id: Some(format!("custom-{:x}", hash)),
            bundle_components: component_paths,
        })
    }

    /// Find the tokenizer associated with an encoder component
    /// Strategy:
    /// 1. Look in database for tokenizer with same repo_id
    /// 2. Look in encoder's directory structure for tokenizer files
    fn find_tokenizer_for_encoder(
        db: &crate::db::InferenceDb,
        encoder: &crate::db::ComponentRecord,
        tokenizer_type: &str,
    ) -> Result<PathBuf> {
        // Strategy 1: Find tokenizer in database with matching repo_id
        if let Some(ref repo_id) = encoder.repo_id {
            if let Ok(tokenizer) = db.find_component_by_repo_and_type(repo_id, tokenizer_type) {
                return Ok(PathBuf::from(&tokenizer.file_path));
            }
        }

        // Strategy 2: Derive tokenizer path from encoder's location
        let encoder_path = PathBuf::from(&encoder.file_path);

        // For T5 tokenizer, look in common locations relative to encoder
        if tokenizer_type == "t5_tokenizer" {
            // Check if encoder is a GGUF file (quantized T5)
            if encoder_path.extension().and_then(|e| e.to_str()) == Some("gguf") {
                // For GGUF T5, tokenizer is usually in a separate location
                // Try to find it via the HuggingFace cache structure
                if let Some(tokenizer_path) = Self::find_t5_tokenizer_in_cache()? {
                    return Ok(tokenizer_path);
                }
            }

            // For safetensors T5, look in parent directory structure
            if let Some(parent) = encoder_path.parent() {
                // Check for tokenizer_2 directory (FLUX structure)
                let tokenizer_dir = parent.join("tokenizer_2");
                let tokenizer_json = tokenizer_dir.join("tokenizer.json");
                if tokenizer_json.exists() {
                    return Ok(tokenizer_json);
                }

                // Check for spiece.model (SentencePiece format)
                let spiece = tokenizer_dir.join("spiece.model");
                if spiece.exists() {
                    return Ok(spiece);
                }

                // Check sibling directory for standalone tokenizers
                if let Some(grandparent) = parent.parent() {
                    let standalone = grandparent.join("t5-v1_1-xxl.tokenizer.json");
                    if standalone.exists() {
                        return Ok(standalone);
                    }
                }
            }
        }

        // For CLIP tokenizer
        if tokenizer_type == "clip_tokenizer" {
            if let Some(parent) = encoder_path.parent() {
                // CLIP encoder is usually in text_encoder/model.safetensors
                // Tokenizer is in sibling tokenizer/ directory
                let tokenizer_dir = if parent.ends_with("text_encoder") {
                    parent.parent().map(|p| p.join("tokenizer"))
                } else {
                    Some(parent.join("tokenizer"))
                };

                if let Some(dir) = tokenizer_dir {
                    let tokenizer_json = dir.join("tokenizer.json");
                    if tokenizer_json.exists() {
                        return Ok(tokenizer_json);
                    }

                    let vocab_json = dir.join("vocab.json");
                    if vocab_json.exists() {
                        return Ok(vocab_json);
                    }
                }
            }
        }

        anyhow::bail!(
            "Could not find {} for encoder at {}. \
            Tokenizer must be in the same repository or directory as the encoder.",
            tokenizer_type,
            encoder_path.display()
        )
    }

    /// Search HuggingFace cache for T5 tokenizer
    fn find_t5_tokenizer_in_cache() -> Result<Option<PathBuf>> {
        let cache_dir = Self::get_hf_cache_dir()?;

        // Look for common T5 tokenizer repos
        let tokenizer_repos = [
            "models--lmz--mt5-tokenizers",
            "models--google--t5-v1_1-xxl",
        ];

        for repo_name in &tokenizer_repos {
            let repo_dir = cache_dir.join(repo_name);
            if !repo_dir.exists() {
                continue;
            }

            // Find the active snapshot
            let refs_main = repo_dir.join("refs").join("main");
            let snapshot_hash = if refs_main.exists() {
                std::fs::read_to_string(&refs_main).ok()
            } else {
                None
            };

            let snapshot_dir = if let Some(hash) = snapshot_hash {
                repo_dir.join("snapshots").join(hash.trim())
            } else {
                // Find first snapshot directory
                let snapshots = repo_dir.join("snapshots");
                if let Ok(entries) = std::fs::read_dir(&snapshots) {
                    entries
                        .flatten()
                        .find(|e| e.path().is_dir())
                        .map(|e| e.path())
                        .unwrap_or_default()
                } else {
                    continue;
                }
            };

            if !snapshot_dir.exists() {
                continue;
            }

            // Check for tokenizer files
            let tokenizer_json = snapshot_dir.join("t5-v1_1-xxl.tokenizer.json");
            if tokenizer_json.exists() {
                return Ok(Some(tokenizer_json));
            }

            // Check tokenizer_2 subdirectory
            let tokenizer_2 = snapshot_dir.join("tokenizer_2").join("tokenizer.json");
            if tokenizer_2.exists() {
                return Ok(Some(tokenizer_2));
            }
        }

        Ok(None)
    }

    /// Get HuggingFace cache directory
    fn get_hf_cache_dir() -> Result<PathBuf> {
        // Check environment variables first
        if let Ok(hf_cache) = std::env::var("HF_HUB_CACHE") {
            return Ok(PathBuf::from(hf_cache));
        }

        if let Ok(hf_home) = std::env::var("HF_HOME") {
            return Ok(PathBuf::from(hf_home).join("hub"));
        }

        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        // macOS location
        #[cfg(target_os = "macos")]
        {
            let macos_cache = home.join("Library/Caches/huggingface/hub");
            if macos_cache.exists() {
                return Ok(macos_cache);
            }
        }

        // Default Linux/fallback location
        Ok(home.join(".cache/huggingface/hub"))
    }

    /// Get database path (~/.rzem-ai-inference/inference.db)
    pub fn get_db_path() -> Result<String> {
        let app_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            .join(".rzem-ai-inference");

        std::fs::create_dir_all(&app_dir)?;
        Ok(app_dir.join("inference.db").to_string_lossy().to_string())
    }

    /// Get component path by role
    pub fn component_path(&self, role: ComponentRole) -> Result<PathBuf> {
        self.bundle_components.get(&role)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Component {:?} not in bundle", role))
    }

    /// Get active bundle ID
    pub fn bundle_id(&self) -> Option<&str> {
        self.bundle_id.as_deref()
    }

    // ===== Public API =====

    /// Get path to CLIP text encoder
    pub fn clip_path(&self) -> Result<PathBuf> {
        self.component_path(ComponentRole::Clip)
    }

    /// Get path to VAE decoder
    pub fn vae_path(&self) -> Result<PathBuf> {
        self.component_path(ComponentRole::Vae)
    }

    /// Get path to FLUX transformer
    pub fn transformer_path(&self) -> Result<PathBuf> {
        self.component_path(ComponentRole::Transformer)
    }

    /// Get path to CLIP tokenizer
    pub fn tokenizer_path(&self) -> Result<PathBuf> {
        self.component_path(ComponentRole::ClipTokenizer)
    }

    /// Get path to T5 text encoder
    pub fn t5_path(&self) -> Result<PathBuf> {
        self.component_path(ComponentRole::T5)
    }

    /// Get path to T5 tokenizer
    pub fn t5_tokenizer_path(&self) -> Result<PathBuf> {
        self.component_path(ComponentRole::T5Tokenizer)
    }

    // ===== Validation =====

    /// Check if all required files exist
    pub fn all_files_exist(&self) -> bool {
        self.validate_components().is_ok()
    }

    /// Validate bundle components exist
    pub fn validate_components(&self) -> Result<()> {
        let required_roles = [
            ComponentRole::Transformer,
            ComponentRole::T5,
            ComponentRole::Clip,
            ComponentRole::Vae,
            ComponentRole::ClipTokenizer,
            ComponentRole::T5Tokenizer,
        ];

        for role in &required_roles {
            let path = self.component_path(*role)?;
            if !path.exists() {
                anyhow::bail!("Required component {:?} not found at {}", role, path.display());
            }
        }

        Ok(())
    }

    /// Get detailed status of which model files exist (for debugging)
    pub fn get_status(&self) -> Vec<(String, bool, String)> {
        let mut status = vec![];

        if let Some(bundle_id) = &self.bundle_id {
            status.push((
                format!("Active Bundle: {}", bundle_id),
                true,
                "Using bundle mode".to_string(),
            ));
        }

        for role in [
            ComponentRole::Transformer,
            ComponentRole::T5,
            ComponentRole::Clip,
            ComponentRole::Vae,
            ComponentRole::ClipTokenizer,
            ComponentRole::T5Tokenizer,
        ] {
            if let Ok(path) = self.component_path(role) {
                let exists = path.exists();
                status.push((
                    format!("{:?}", role),
                    exists,
                    path.display().to_string(),
                ));
            }
        }

        status
    }

    /// Helper function to create ModelPaths for tests (creates its own DB connection)
    #[cfg(test)]
    pub fn new_for_test() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        let db = crate::db::InferenceDb::new(&db_path)?;
        Self::new(&db)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{InferenceDb, BundleRecord, ComponentRecord};

    #[test]
    fn test_component_role_conversion() {
        assert_eq!(ComponentRole::from_str("transformer").unwrap(), ComponentRole::Transformer);
        assert_eq!(ComponentRole::Transformer.as_str(), "transformer");

        // Test all roles
        assert_eq!(ComponentRole::from_str("t5").unwrap(), ComponentRole::T5);
        assert_eq!(ComponentRole::from_str("clip").unwrap(), ComponentRole::Clip);
        assert_eq!(ComponentRole::from_str("vae").unwrap(), ComponentRole::Vae);
        assert_eq!(ComponentRole::from_str("clip_tokenizer").unwrap(), ComponentRole::ClipTokenizer);
        assert_eq!(ComponentRole::from_str("t5_tokenizer").unwrap(), ComponentRole::T5Tokenizer);

        // Test invalid role
        assert!(ComponentRole::from_str("invalid").is_err());
    }

    #[test]
    fn test_paths_requires_active_bundle() {
        // Create in-memory database with no active bundle
        let db = InferenceDb::new(":memory:").unwrap();
        db.init_schema().unwrap();

        // ModelPaths::new() should fail without active bundle
        let result = ModelPaths::new(&db);
        assert!(result.is_err());
        let err_msg = format!("{}", result.err().unwrap());
        assert!(err_msg.contains("No active bundle"));
    }

    #[test]
    fn test_paths_from_complete_bundle() {
        // Create in-memory database
        let db = InferenceDb::new(":memory:").unwrap();
        db.init_schema().unwrap();

        // Create bundle
        let bundle = BundleRecord {
            id: "test-bundle".to_string(),
            name: "Test Bundle".to_string(),
            description: Some("Complete test bundle".to_string()),
            bundle_type: "user_created".to_string(),
            model_family: "flux".to_string(),
            default_steps: Some(4),
            default_guidance: Some(3.5),
            step_min: Some(1),
            step_max: Some(50),
            total_vram_mb: Some(24000),
            is_complete: true,
            is_active: true,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            validation_errors: None,
        };

        db.insert_bundle(&bundle).unwrap();

        // Add all required components
        let component_roles = [
            "transformer", "t5", "clip", "vae", "clip_tokenizer", "t5_tokenizer"
        ];

        for (i, role) in component_roles.iter().enumerate() {
            let component = ComponentRecord {
                id: format!("comp-{}", i),
                component_type: role.to_string(),
                format: "safetensors".to_string(),
                file_path: format!("/tmp/{}.safetensors", role),
                file_size: 1000000,
                file_hash: None,
                name: format!("Test {}", role),
                repo_id: Some("test/repo".to_string()),
                repo_snapshot: None,
                architecture: Some("test-arch".to_string()),
                quantization: None,
                supports_loras: false,
                is_sharded: false,
                shard_count: None,
                vram_mb: Some(4000),
                discovered_at: chrono::Utc::now().timestamp(),
                last_verified_at: Some(chrono::Utc::now().timestamp()),
                is_available: true,
                metadata: None,
            };

            db.insert_component(&component).unwrap();
            db.add_component_to_bundle(&bundle.id, &component.id, role, true, i as i32)
                .unwrap();
        }

        // Now ModelPaths::new() should succeed
        let paths = ModelPaths::new(&db).unwrap();
        assert_eq!(paths.bundle_id(), Some("test-bundle"));

        // Test component path access
        assert!(paths.transformer_path().is_ok());
        assert!(paths.t5_path().is_ok());
        assert!(paths.clip_path().is_ok());
        assert!(paths.vae_path().is_ok());
        assert!(paths.tokenizer_path().is_ok());
        assert!(paths.t5_tokenizer_path().is_ok());
    }

    #[test]
    fn test_from_component_ids_requires_all_four() {
        // Create in-memory database
        let db = InferenceDb::new(":memory:").unwrap();
        db.init_schema().unwrap();

        // Create test components
        let component_ids = ["trans-1", "t5-1", "clip-1", "vae-1"];
        for id in &component_ids {
            let component = ComponentRecord {
                id: id.to_string(),
                component_type: "test".to_string(),
                format: "safetensors".to_string(),
                file_path: format!("/tmp/{}.safetensors", id),
                file_size: 1000000,
                file_hash: None,
                name: format!("Test {}", id),
                repo_id: Some("test/repo".to_string()),
                repo_snapshot: None,
                architecture: Some("test-arch".to_string()),
                quantization: None,
                supports_loras: false,
                is_sharded: false,
                shard_count: None,
                vram_mb: Some(4000),
                discovered_at: chrono::Utc::now().timestamp(),
                last_verified_at: Some(chrono::Utc::now().timestamp()),
                is_available: true,
                metadata: None,
            };
            db.insert_component(&component).unwrap();
        }

        // Should succeed with all 4 component IDs
        let result = ModelPaths::from_component_ids(
            &db,
            "trans-1",
            "t5-1",
            "clip-1",
            "vae-1"
        );
        assert!(result.is_ok());
        let paths = result.unwrap();

        // Verify custom bundle ID was generated
        assert!(paths.bundle_id().is_some());
        assert!(paths.bundle_id().unwrap().starts_with("custom-"));

        // Should have all 4 required components
        assert!(paths.component_path(ComponentRole::Transformer).is_ok());
        assert!(paths.component_path(ComponentRole::T5).is_ok());
        assert!(paths.component_path(ComponentRole::Clip).is_ok());
        assert!(paths.component_path(ComponentRole::Vae).is_ok());
    }

    #[test]
    fn test_from_component_ids_fails_with_missing_component() {
        // Create in-memory database
        let db = InferenceDb::new(":memory:").unwrap();
        db.init_schema().unwrap();

        // Create only 3 components (missing one)
        for id in &["trans-1", "t5-1", "clip-1"] {
            let component = ComponentRecord {
                id: id.to_string(),
                component_type: "test".to_string(),
                format: "safetensors".to_string(),
                file_path: format!("/tmp/{}.safetensors", id),
                file_size: 1000000,
                file_hash: None,
                name: format!("Test {}", id),
                repo_id: Some("test/repo".to_string()),
                repo_snapshot: None,
                architecture: Some("test-arch".to_string()),
                quantization: None,
                supports_loras: false,
                is_sharded: false,
                shard_count: None,
                vram_mb: Some(4000),
                discovered_at: chrono::Utc::now().timestamp(),
                last_verified_at: Some(chrono::Utc::now().timestamp()),
                is_available: true,
                metadata: None,
            };
            db.insert_component(&component).unwrap();
        }

        // Should fail - vae-1 doesn't exist
        let result = ModelPaths::from_component_ids(
            &db,
            "trans-1",
            "t5-1",
            "clip-1",
            "vae-missing"
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_component_path_missing_role() {
        // Create empty ModelPaths
        let paths = ModelPaths {
            bundle_id: Some("test".to_string()),
            bundle_components: HashMap::new(),
        };

        // Should fail when requesting missing component
        let result = paths.component_path(ComponentRole::Transformer);
        assert!(result.is_err());
        let err_msg = format!("{}", result.err().unwrap());
        assert!(err_msg.contains("not in bundle"));
    }

    #[test]
    fn test_validate_components() {
        // Create ModelPaths with non-existent paths
        let mut bundle_components = HashMap::new();
        bundle_components.insert(ComponentRole::Transformer, PathBuf::from("/nonexistent/path"));

        let paths = ModelPaths {
            bundle_id: Some("test".to_string()),
            bundle_components,
        };

        // Validation should fail for missing files
        let result = paths.validate_components();
        assert!(result.is_err());
    }
}
