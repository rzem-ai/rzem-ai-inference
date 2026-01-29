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
    pub fn new(db: &crate::gallery::GalleryDb) -> Result<Self> {
        Self::from_active_bundle(db)
    }

    /// Create ModelPaths from active bundle in database
    pub fn from_active_bundle(db: &crate::gallery::GalleryDb) -> Result<Self> {
        let bundle_info = db.get_active_bundle()?
            .ok_or_else(|| anyhow::anyhow!("No active bundle configured"))?;

        Self::from_bundle_info(&bundle_info)
    }

    /// Create ModelPaths from a BundleInfo
    pub fn from_bundle_info(bundle: &crate::gallery::BundleInfo) -> Result<Self> {
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
    pub fn from_component_ids(
        db: &crate::gallery::GalleryDb,
        transformer_id: &str,
        t5_id: &str,
        clip_id: &str,
        vae_id: &str,
    ) -> Result<Self> {
        let mut component_paths = HashMap::new();

        // Load transformer (required)
        let transformer = db.get_component(transformer_id)?;
        component_paths.insert(ComponentRole::Transformer, PathBuf::from(&transformer.file_path));

        // Load required components
        let t5 = db.get_component(t5_id)?;
        component_paths.insert(ComponentRole::T5, PathBuf::from(&t5.file_path));

        let clip = db.get_component(clip_id)?;
        component_paths.insert(ComponentRole::Clip, PathBuf::from(&clip.file_path));

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

    /// Get database path
    pub fn get_db_path() -> Result<String> {
        let app_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?
            .join("rzem-ai-inference");

        std::fs::create_dir_all(&app_dir)?;
        Ok(app_dir.join("rzem.db").to_string_lossy().to_string())
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

    // ===== Legacy Z-Index stubs (TODO: Remove when Z-Index removed) =====

    /// DEPRECATED: Qwen3 is part of Z-Index which is being removed
    pub fn qwen3_path(&self) -> Result<PathBuf> {
        anyhow::bail!("Qwen3 paths are deprecated - Z-Index is being removed")
    }

    /// DEPRECATED: Qwen3 is part of Z-Index which is being removed
    pub fn qwen3_tokenizer_path(&self) -> Result<PathBuf> {
        anyhow::bail!("Qwen3 tokenizer paths are deprecated - Z-Index is being removed")
    }

    /// DEPRECATED: Z-Image paths are deprecated
    pub fn zimage_vae_path(&self) -> Result<PathBuf> {
        anyhow::bail!("Z-Image paths are deprecated")
    }

    /// DEPRECATED: Z-Image paths are deprecated
    pub fn zimage_transformer_path(&self) -> Result<PathBuf> {
        anyhow::bail!("Z-Image paths are deprecated")
    }

    /// DEPRECATED: Legacy quantization check - use component metadata instead
    pub fn has_quantized_transformer(&self) -> bool {
        false
    }

    /// DEPRECATED: Legacy quantization check - use component metadata instead
    pub fn quantized_transformer_path(&self) -> Result<PathBuf> {
        anyhow::bail!("Quantized paths are deprecated - use component metadata")
    }

    /// DEPRECATED: Z-Image download check - use bundle system
    pub fn is_zimage_downloaded(&self) -> bool {
        false
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
        let db = crate::gallery::GalleryDb::new(&db_path)?;
        Self::new(&db)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_role_conversion() {
        assert_eq!(ComponentRole::from_str("transformer").unwrap(), ComponentRole::Transformer);
        assert_eq!(ComponentRole::Transformer.as_str(), "transformer");
    }
}
