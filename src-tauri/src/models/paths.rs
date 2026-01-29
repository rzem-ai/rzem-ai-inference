//! Path management for model files - Bundle-aware with legacy fallback

use anyhow::{Context, Result};
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

/// Manages paths for FLUX model files
/// New: Bundle-aware with automatic fallback to legacy hardcoded paths
pub struct ModelPaths {
    /// Bundle mode fields (new)
    bundle_id: Option<String>,
    bundle_components: Option<HashMap<ComponentRole, PathBuf>>,

    /// Legacy mode fields (backward compatibility)
    /// Base cache directory (~/.cache/huggingface)
    pub cache_dir: PathBuf,
    /// FLUX Schnell model directory
    pub schnell_dir: PathBuf,
}

impl ModelPaths {
    /// Create new ModelPaths with optional bundle/component overrides
    /// Priority: custom bundle > individual components > active bundle > legacy paths
    pub fn new_with_context(
        bundle_id: Option<&str>,
        t5_component_id: Option<&str>,
        clip_component_id: Option<&str>,
        vae_component_id: Option<&str>,
    ) -> Result<Self> {
        // Priority 1: Explicit bundle ID provided
        if let Some(bundle_id) = bundle_id {
            let db_path = Self::get_db_path()?;
            let db = crate::gallery::GalleryDb::new(&db_path)?;
            let bundle_info = db.get_bundle(bundle_id)?;
            tracing::info!(bundle_id = %bundle_id, "Using specified bundle");
            return Self::from_bundle_info(&bundle_info);
        }

        // Priority 2: Individual component IDs provided
        if t5_component_id.is_some() || clip_component_id.is_some() || vae_component_id.is_some() {
            tracing::info!("Using individual component selection");
            // For now, we need at least the transformer ID, which we don't have here
            // This will be handled by a separate method from_component_ids
            // Fall through to next priority
        }

        // Priority 3: Try to load from active bundle
        if let Ok(paths) = Self::from_active_bundle() {
            tracing::info!(bundle_id = ?paths.bundle_id, "Using active model bundle");
            return Ok(paths);
        }

        // Priority 4: Fallback to legacy hardcoded paths
        tracing::debug!("No active bundle found, using legacy hardcoded paths");
        Self::new_legacy()
    }

    /// Create new ModelPaths - tries bundle first, falls back to legacy
    /// This is the primary constructor that should be used
    pub fn new() -> Result<Self> {
        Self::new_with_context(None, None, None, None)
    }

    /// Create ModelPaths from active bundle in database
    pub fn from_active_bundle() -> Result<Self> {
        // We need to access the database to get the active bundle
        // This requires the database path
        let db_path = Self::get_db_path()?;
        let db = crate::gallery::GalleryDb::new(&db_path)?;

        // Get active bundle
        let bundle_info = db.get_active_bundle()?
            .ok_or_else(|| anyhow::anyhow!("No active bundle"))?;

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

        // Create legacy fields for fallback compatibility
        let (cache_dir, schnell_dir) = Self::get_legacy_dirs()?;

        Ok(Self {
            bundle_id: Some(bundle.id.clone()),
            bundle_components: Some(component_paths),
            cache_dir,
            schnell_dir,
        })
    }

    /// Create ModelPaths from individual component IDs
    pub fn from_component_ids(
        transformer_id: &str,
        t5_id: Option<&str>,
        clip_id: Option<&str>,
        vae_id: Option<&str>,
    ) -> Result<Self> {
        let db_path = Self::get_db_path()?;
        let db = crate::gallery::GalleryDb::new(&db_path)?;

        let mut component_paths = HashMap::new();

        // Load transformer (required)
        let transformer = db.get_component(transformer_id)?;
        component_paths.insert(ComponentRole::Transformer, PathBuf::from(&transformer.file_path));

        // Load required components
        if let Some(t5_id) = t5_id {
            let t5 = db.get_component(t5_id)?;
            component_paths.insert(ComponentRole::T5, PathBuf::from(&t5.file_path));
        } else {
            anyhow::bail!("T5 component ID required");
        }

        if let Some(clip_id) = clip_id {
            let clip = db.get_component(clip_id)?;
            component_paths.insert(ComponentRole::Clip, PathBuf::from(&clip.file_path));
        } else {
            anyhow::bail!("CLIP component ID required");
        }

        if let Some(vae_id) = vae_id {
            let vae = db.get_component(vae_id)?;
            component_paths.insert(ComponentRole::Vae, PathBuf::from(&vae.file_path));
        } else {
            anyhow::bail!("VAE component ID required");
        }

        // No legacy fields needed
        let (cache_dir, schnell_dir) = Self::get_legacy_dirs()?;

        Ok(Self {
            bundle_id: Some(format!("custom-{}", transformer_id)),
            bundle_components: Some(component_paths),
            cache_dir,
            schnell_dir,
        })
    }

    /// Create with legacy hardcoded paths (backward compatibility)
    fn new_legacy() -> Result<Self> {
        let (cache_dir, schnell_dir) = Self::get_legacy_dirs()?;

        Ok(Self {
            bundle_id: None,
            bundle_components: None,
            cache_dir,
            schnell_dir,
        })
    }

    /// Get legacy HuggingFace cache directories
    fn get_legacy_dirs() -> Result<(PathBuf, PathBuf)> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        // Check for HF_HUB_CACHE or HF_HOME environment variables first
        let cache_dir = if let Ok(hf_cache) = std::env::var("HF_HUB_CACHE") {
            PathBuf::from(hf_cache)
        } else if let Ok(hf_home) = std::env::var("HF_HOME") {
            PathBuf::from(hf_home).join("hub")
        } else {
            // Check possible cache locations
            let possible_locations = vec![
                home.join(".cache").join("huggingface").join("hub"),
                // macOS alternative location
                #[cfg(target_os = "macos")]
                home.join("Library").join("Caches").join("huggingface").join("hub"),
            ];

            // Find the first location that exists and has the model, or default to first
            let schnell_model = "models--black-forest-labs--FLUX.1-schnell";
            possible_locations
                .iter()
                .find(|p| p.join(schnell_model).exists())
                .cloned()
                .unwrap_or_else(|| possible_locations[0].clone())
        };

        let schnell_dir = cache_dir.join("models--black-forest-labs--FLUX.1-schnell");

        Ok((cache_dir, schnell_dir))
    }

    /// Get database path
    pub fn get_db_path() -> Result<String> {
        let app_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?
            .join("rzem-ai-inference");

        std::fs::create_dir_all(&app_dir)?;
        Ok(app_dir.join("rzem.db").to_string_lossy().to_string())
    }

    // ===== Bundle-aware Path Methods =====

    /// Get component path by role (bundle-aware)
    pub fn component_path(&self, role: ComponentRole) -> Result<PathBuf> {
        if let Some(ref components) = self.bundle_components {
            components.get(&role)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Component {:?} not in bundle", role))
        } else {
            // Fallback to legacy paths
            self.legacy_component_path(role)
        }
    }

    /// Legacy component path lookup
    fn legacy_component_path(&self, role: ComponentRole) -> Result<PathBuf> {
        match role {
            ComponentRole::Transformer => Ok(self.legacy_transformer_path()),
            ComponentRole::T5 => Ok(self.legacy_t5_path()),
            ComponentRole::Clip => Ok(self.legacy_clip_path()),
            ComponentRole::Vae => Ok(self.legacy_vae_path()),
            ComponentRole::ClipTokenizer => Ok(self.legacy_tokenizer_path()),
            ComponentRole::T5Tokenizer => Ok(self.legacy_t5_tokenizer_path()),
        }
    }

    /// Check if bundle mode is active
    pub fn is_bundle_mode(&self) -> bool {
        self.bundle_id.is_some()
    }

    /// Get active bundle ID
    pub fn bundle_id(&self) -> Option<&str> {
        self.bundle_id.as_deref()
    }

    // ===== Public API (Bundle-aware with legacy fallback) =====

    /// Get path to CLIP text encoder
    pub fn clip_path(&self) -> PathBuf {
        self.component_path(ComponentRole::Clip)
            .unwrap_or_else(|_| self.legacy_clip_path())
    }

    /// Get path to VAE decoder
    pub fn vae_path(&self) -> PathBuf {
        self.component_path(ComponentRole::Vae)
            .unwrap_or_else(|_| self.legacy_vae_path())
    }

    /// Get path to FLUX transformer
    pub fn transformer_path(&self) -> PathBuf {
        self.component_path(ComponentRole::Transformer)
            .unwrap_or_else(|_| self.legacy_transformer_path())
    }

    /// Get path to CLIP tokenizer
    pub fn tokenizer_path(&self) -> PathBuf {
        self.component_path(ComponentRole::ClipTokenizer)
            .unwrap_or_else(|_| self.legacy_tokenizer_path())
    }

    /// Get path to T5 text encoder
    pub fn t5_path(&self) -> PathBuf {
        self.component_path(ComponentRole::T5)
            .unwrap_or_else(|_| self.legacy_t5_path())
    }

    /// Get path to T5 tokenizer
    pub fn t5_tokenizer_path(&self) -> PathBuf {
        self.component_path(ComponentRole::T5Tokenizer)
            .unwrap_or_else(|_| self.legacy_t5_tokenizer_path())
    }

    // ===== Legacy Hardcoded Path Methods =====

    fn get_snapshot_hash(&self) -> Result<String> {
        let refs_main = self.schnell_dir.join("refs").join("main");

        if refs_main.exists() {
            let hash = std::fs::read_to_string(&refs_main)
                .context("Failed to read refs/main")?
                .trim()
                .to_string();
            Ok(hash)
        } else {
            let snapshots_dir = self.schnell_dir.join("snapshots");
            if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        if entry.path().is_dir() {
                            if let Some(name) = entry.file_name().to_str() {
                                return Ok(name.to_string());
                            }
                        }
                    }
                }
            }
            anyhow::bail!("Could not find snapshot directory")
        }
    }

    fn snapshot_path(&self) -> Result<PathBuf> {
        let hash = self.get_snapshot_hash()?;
        Ok(self.schnell_dir.join("snapshots").join(hash))
    }

    fn legacy_clip_path(&self) -> PathBuf {
        let schnell_path = self.snapshot_path()
            .map(|p| p.join("text_encoder"))
            .unwrap_or_else(|_| self.schnell_dir.join("snapshots").join("main").join("text_encoder"));

        if schnell_path.join("model.safetensors").exists() {
            return schnell_path;
        }

        self.get_dev_snapshot_hash()
            .map(|hash| self.dev_dir().join("snapshots").join(hash).join("text_encoder"))
            .unwrap_or(schnell_path)
    }

    fn legacy_vae_path(&self) -> PathBuf {
        let schnell_path = self.snapshot_path()
            .map(|p| p.join("ae.safetensors"))
            .unwrap_or_else(|_| self.schnell_dir.join("snapshots").join("main").join("ae.safetensors"));

        if schnell_path.exists() {
            return schnell_path;
        }

        self.get_dev_snapshot_hash()
            .map(|hash| self.dev_dir().join("snapshots").join(hash).join("ae.safetensors"))
            .unwrap_or(schnell_path)
    }

    fn legacy_transformer_path(&self) -> PathBuf {
        self.snapshot_path()
            .map(|p| p.join("flux1-schnell.safetensors"))
            .unwrap_or_else(|_| self.schnell_dir.join("snapshots").join("main").join("flux1-schnell.safetensors"))
    }

    fn legacy_tokenizer_path(&self) -> PathBuf {
        let schnell_path = self.snapshot_path()
            .map(|p| p.join("tokenizer"))
            .unwrap_or_else(|_| self.schnell_dir.join("snapshots").join("main").join("tokenizer"));

        if schnell_path.exists() {
            return schnell_path;
        }

        self.get_dev_snapshot_hash()
            .map(|hash| self.dev_dir().join("snapshots").join(hash).join("tokenizer"))
            .unwrap_or(schnell_path)
    }

    fn legacy_t5_path(&self) -> PathBuf {
        self.snapshot_path()
            .map(|p| p.join("text_encoder_2"))
            .unwrap_or_else(|_| self.schnell_dir.join("snapshots").join("main").join("text_encoder_2"))
    }

    fn legacy_t5_tokenizer_path(&self) -> PathBuf {
        let mt5_dir = self.cache_dir.join("models--lmz--mt5-tokenizers");

        if let Ok(refs_main) = std::fs::read_to_string(mt5_dir.join("refs").join("main")) {
            let hash = refs_main.trim();
            return mt5_dir.join("snapshots").join(hash).join("t5-v1_1-xxl.tokenizer.json");
        }

        if let Ok(entries) = std::fs::read_dir(mt5_dir.join("snapshots")) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    return entry.path().join("t5-v1_1-xxl.tokenizer.json");
                }
            }
        }

        mt5_dir.join("snapshots").join("main").join("t5-v1_1-xxl.tokenizer.json")
    }

    // ===== Quantized Model Paths =====

    pub fn quantized_transformer_path(&self) -> PathBuf {
        let lmz_dir = self.cache_dir.join("models--lmz--candle-flux");

        if let Ok(refs_main) = std::fs::read_to_string(lmz_dir.join("refs").join("main")) {
            let hash = refs_main.trim();
            return lmz_dir.join("snapshots").join(hash).join("flux1-schnell.gguf");
        }

        if let Ok(entries) = std::fs::read_dir(lmz_dir.join("snapshots")) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    return entry.path().join("flux1-schnell.gguf");
                }
            }
        }

        lmz_dir.join("snapshots").join("main").join("flux1-schnell.gguf")
    }

    pub fn has_quantized_transformer(&self) -> bool {
        self.quantized_transformer_path().exists()
    }

    pub fn quantized_t5_path(&self) -> PathBuf {
        let t5_gguf_dir = self.cache_dir.join("models--city96--t5-v1_1-xxl-encoder-gguf");

        if let Ok(refs_main) = std::fs::read_to_string(t5_gguf_dir.join("refs").join("main")) {
            let hash = refs_main.trim();
            return t5_gguf_dir.join("snapshots").join(hash).join("t5-v1_1-xxl-encoder-Q5_K_M.gguf");
        }

        if let Ok(entries) = std::fs::read_dir(t5_gguf_dir.join("snapshots")) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    return entry.path().join("t5-v1_1-xxl-encoder-Q5_K_M.gguf");
                }
            }
        }

        t5_gguf_dir.join("snapshots").join("main").join("t5-v1_1-xxl-encoder-Q5_K_M.gguf")
    }

    pub fn has_quantized_t5(&self) -> bool {
        self.quantized_t5_path().exists()
    }

    // ===== FLUX Dev Methods =====

    pub fn dev_dir(&self) -> PathBuf {
        self.cache_dir.join("models--black-forest-labs--FLUX.1-dev")
    }

    fn get_dev_snapshot_hash(&self) -> Result<String> {
        let refs_main = self.dev_dir().join("refs").join("main");

        if refs_main.exists() {
            let hash = std::fs::read_to_string(&refs_main)
                .context("Failed to read Dev refs/main")?
                .trim()
                .to_string();
            Ok(hash)
        } else {
            let snapshots_dir = self.dev_dir().join("snapshots");
            if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            return Ok(name.to_string());
                        }
                    }
                }
            }
            anyhow::bail!("Could not find Dev snapshot directory")
        }
    }

    pub fn dev_transformer_path(&self) -> PathBuf {
        self.get_dev_snapshot_hash()
            .map(|hash| self.dev_dir().join("snapshots").join(hash).join("flux1-dev.safetensors"))
            .unwrap_or_else(|_| self.dev_dir().join("snapshots").join("main").join("flux1-dev.safetensors"))
    }

    pub fn quantized_dev_transformer_path(&self) -> PathBuf {
        let city96_dir = self.cache_dir.join("models--city96--FLUX.1-dev-gguf");
        if let Ok(refs_main) = std::fs::read_to_string(city96_dir.join("refs").join("main")) {
            let hash = refs_main.trim();
            let path = city96_dir.join("snapshots").join(hash).join("flux1-dev-Q8_0.gguf");
            if path.exists() {
                return path;
            }
        }
        if let Ok(entries) = std::fs::read_dir(city96_dir.join("snapshots")) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let path = entry.path().join("flux1-dev-Q8_0.gguf");
                    if path.exists() {
                        return path;
                    }
                }
            }
        }

        let lmz_dir = self.cache_dir.join("models--lmz--candle-flux");
        if let Ok(refs_main) = std::fs::read_to_string(lmz_dir.join("refs").join("main")) {
            let hash = refs_main.trim();
            return lmz_dir.join("snapshots").join(hash).join("flux1-dev.gguf");
        }

        if let Ok(entries) = std::fs::read_dir(lmz_dir.join("snapshots")) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    return entry.path().join("flux1-dev.gguf");
                }
            }
        }

        city96_dir.join("snapshots").join("main").join("flux1-dev-Q8_0.gguf")
    }

    pub fn is_dev_downloaded(&self) -> bool {
        self.dev_transformer_path().exists() || self.quantized_dev_transformer_path().exists()
    }

    pub fn has_quantized_dev(&self) -> bool {
        self.quantized_dev_transformer_path().exists()
    }

    // ===== Z-Image-Turbo Paths =====

    pub fn zimage_dir(&self) -> PathBuf {
        self.cache_dir.join("models--Tongyi-MAI--Z-Image-Turbo")
    }

    fn get_zimage_snapshot_hash(&self) -> Result<String> {
        let refs_main = self.zimage_dir().join("refs").join("main");
        if refs_main.exists() {
            let hash = std::fs::read_to_string(&refs_main)
                .context("Failed to read Z-Image refs/main")?
                .trim()
                .to_string();
            Ok(hash)
        } else {
            let snapshots_dir = self.zimage_dir().join("snapshots");
            if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            return Ok(name.to_string());
                        }
                    }
                }
            }
            anyhow::bail!("Could not find Z-Image snapshot directory")
        }
    }

    pub fn qwen3_path(&self) -> PathBuf {
        self.get_zimage_snapshot_hash()
            .map(|hash| self.zimage_dir().join("snapshots").join(hash).join("text_encoder"))
            .unwrap_or_else(|_| self.zimage_dir().join("snapshots").join("main").join("text_encoder"))
    }

    pub fn qwen3_tokenizer_path(&self) -> PathBuf {
        self.get_zimage_snapshot_hash()
            .map(|hash| self.zimage_dir().join("snapshots").join(hash).join("tokenizer"))
            .unwrap_or_else(|_| self.zimage_dir().join("snapshots").join("main").join("tokenizer"))
    }

    pub fn zimage_transformer_path(&self) -> PathBuf {
        self.get_zimage_snapshot_hash()
            .map(|hash| self.zimage_dir().join("snapshots").join(hash).join("transformer"))
            .unwrap_or_else(|_| self.zimage_dir().join("snapshots").join("main").join("transformer"))
    }

    pub fn zimage_vae_path(&self) -> PathBuf {
        self.get_zimage_snapshot_hash()
            .map(|hash| self.zimage_dir().join("snapshots").join(hash).join("vae"))
            .unwrap_or_else(|_| self.zimage_dir().join("snapshots").join("main").join("vae"))
    }

    pub fn quantized_zimage_transformer_path(&self) -> PathBuf {
        self.cache_dir
            .join("models--lmz--candle-zimage")
            .join("snapshots")
            .join("main")
            .join("zimage-turbo.gguf")
    }

    pub fn is_zimage_downloaded(&self) -> bool {
        let transformer_dir = self.zimage_transformer_path();
        transformer_dir.join("diffusion_pytorch_model-00001-of-00003.safetensors").exists()
            && transformer_dir.join("diffusion_pytorch_model-00002-of-00003.safetensors").exists()
            && transformer_dir.join("diffusion_pytorch_model-00003-of-00003.safetensors").exists()
    }

    pub fn has_quantized_zimage(&self) -> bool {
        self.quantized_zimage_transformer_path().exists()
    }

    // ===== Model Type Helpers =====

    pub fn transformer_path_for(&self, model_type: super::ModelType) -> PathBuf {
        match model_type.id() {
            "schnell" => self.transformer_path(),
            "dev" => self.dev_transformer_path(),
            "zimage-turbo" => self.zimage_transformer_path(),
            _ => self.transformer_path(),
        }
    }

    pub fn quantized_transformer_path_for(&self, model_type: super::ModelType) -> PathBuf {
        match model_type.id() {
            "schnell" => self.quantized_transformer_path(),
            "dev" => self.quantized_dev_transformer_path(),
            "zimage-turbo" => self.quantized_zimage_transformer_path(),
            _ => self.quantized_transformer_path(),
        }
    }

    pub fn has_quantized_for(&self, model_type: super::ModelType) -> bool {
        match model_type.id() {
            "schnell" => self.has_quantized_transformer(),
            "dev" => self.has_quantized_dev(),
            "zimage-turbo" => self.has_quantized_zimage(),
            _ => false,
        }
    }

    // ===== Validation =====

    /// Check if all required files exist
    /// Bundle mode: checks bundle component paths
    /// Legacy mode: checks hardcoded paths
    pub fn all_files_exist(&self) -> bool {
        if let Some(ref components) = self.bundle_components {
            // Bundle mode: check all required components are available
            self.validate_bundle_components().is_ok()
        } else {
            // Legacy mode: check hardcoded paths
            self.validate_legacy_paths()
        }
    }

    /// Validate bundle components exist
    fn validate_bundle_components(&self) -> Result<()> {
        let required_roles = [
            ComponentRole::Transformer,
            ComponentRole::T5,
            ComponentRole::Clip,
            ComponentRole::Vae,
        ];

        for role in &required_roles {
            let path = self.component_path(*role)?;
            if !path.exists() {
                anyhow::bail!("Required component {:?} not found at {}", role, path.display());
            }
        }

        Ok(())
    }

    /// Validate legacy hardcoded paths
    fn validate_legacy_paths(&self) -> bool {
        let has_clip = self.legacy_clip_path().join("model.safetensors").exists();
        let has_vae = self.legacy_vae_path().exists();
        let has_transformer = self.legacy_transformer_path().exists()
            || self.has_quantized_transformer();

        let t5_dir = self.legacy_t5_path();
        let has_t5_full = t5_dir.join("model-00001-of-00002.safetensors").exists()
            && t5_dir.join("config.json").exists();
        let has_t5 = has_t5_full || self.has_quantized_t5();

        let has_t5_tokenizer = self.legacy_t5_tokenizer_path().exists();

        has_clip && has_vae && has_transformer && has_t5 && has_t5_tokenizer
    }

    /// Get detailed status of which model files exist (for debugging)
    pub fn get_status(&self) -> Vec<(String, bool, String)> {
        if self.is_bundle_mode() {
            self.get_bundle_status()
        } else {
            self.get_legacy_status()
        }
    }

    fn get_bundle_status(&self) -> Vec<(String, bool, String)> {
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

    fn get_legacy_status(&self) -> Vec<(String, bool, String)> {
        let clip_path = self.legacy_clip_path().join("model.safetensors");
        let vae_path = self.legacy_vae_path();
        let transformer_path = self.legacy_transformer_path();
        let quantized_transformer_path = self.quantized_transformer_path();
        let dev_transformer_path = self.dev_transformer_path();
        let quantized_dev_path = self.quantized_dev_transformer_path();
        let t5_path = self.legacy_t5_path();
        let t5_model_path = t5_path.join("model-00001-of-00002.safetensors");
        let t5_config_path = t5_path.join("config.json");
        let quantized_t5_path = self.quantized_t5_path();
        let t5_tokenizer_path = self.legacy_t5_tokenizer_path();

        vec![
            ("CLIP text encoder".to_string(), clip_path.exists(), clip_path.display().to_string()),
            ("VAE (ae.safetensors)".to_string(), vae_path.exists(), vae_path.display().to_string()),
            ("Schnell transformer (full)".to_string(), transformer_path.exists(), transformer_path.display().to_string()),
            ("Schnell transformer (quantized)".to_string(), quantized_transformer_path.exists(), quantized_transformer_path.display().to_string()),
            ("Dev transformer (full)".to_string(), dev_transformer_path.exists(), dev_transformer_path.display().to_string()),
            ("Dev transformer (quantized)".to_string(), quantized_dev_path.exists(), quantized_dev_path.display().to_string()),
            ("T5 model".to_string(), t5_model_path.exists(), t5_model_path.display().to_string()),
            ("T5 config".to_string(), t5_config_path.exists(), t5_config_path.display().to_string()),
            ("T5 (quantized)".to_string(), quantized_t5_path.exists(), quantized_t5_path.display().to_string()),
            ("T5 tokenizer".to_string(), t5_tokenizer_path.exists(), t5_tokenizer_path.display().to_string()),
        ]
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

    #[test]
    fn test_paths_creation() {
        let paths = ModelPaths::new().unwrap();
        assert!(paths.cache_dir.to_string_lossy().contains("huggingface"));
    }
}
