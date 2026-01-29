//! Model cache scanner - discovers ALL model components in HuggingFace cache

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::path::{Path, PathBuf};
use std::io::Read;
use tracing::{debug, info, warn};

/// Component types that can be discovered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComponentType {
    Transformer,
    T5Encoder,
    ClipEncoder,
    VaeDecoder,
    ClipTokenizer,
    T5Tokenizer,
}

impl ComponentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Transformer => "transformer",
            Self::T5Encoder => "t5_encoder",
            Self::ClipEncoder => "clip_encoder",
            Self::VaeDecoder => "vae",
            Self::ClipTokenizer => "clip_tokenizer",
            Self::T5Tokenizer => "t5_tokenizer",
        }
    }
}

/// Model format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelFormat {
    Safetensors,
    Gguf,
    Json,
}

impl ModelFormat {
    fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "safetensors" => Some(Self::Safetensors),
                "gguf" => Some(Self::Gguf),
                "json" => Some(Self::Json),
                _ => None,
            })
    }
}

/// Information about a discovered component
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredComponent {
    pub component_type: ComponentType,
    pub architecture: String,
    pub format: ModelFormat,
    pub path: PathBuf,
    pub repo_id: String,
    pub repo_snapshot: Option<String>,
    pub file_size: u64,
    pub file_hash: Option<String>, // SHA256 hash for deduplication
    pub quantization: Option<String>,
    pub is_sharded: bool,
    pub shard_count: Option<usize>,
    pub vram_mb: Option<usize>,
    pub metadata: serde_json::Value,
}

/// Scans the HuggingFace cache for ALL model components
pub fn scan_all_components() -> Result<Vec<DiscoveredComponent>> {
    let cache_dir = get_hf_cache_dir()?;
    let mut components = Vec::new();

    info!(cache_dir = %cache_dir.display(), "Scanning HuggingFace cache for all model components");

    if !cache_dir.exists() {
        warn!("HuggingFace cache directory does not exist: {}", cache_dir.display());
        return Ok(components);
    }

    let entries = std::fs::read_dir(&cache_dir)?;

    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }

        let dir_name = entry.file_name();
        let dir_name_str = dir_name.to_string_lossy();

        // Only process model directories (models--owner--name)
        if !dir_name_str.starts_with("models--") {
            continue;
        }

        // Scan this repository for all component types
        match scan_repo_directory(&entry.path()) {
            Ok(repo_components) => {
                debug!(repo = %entry.path().display(), count = repo_components.len(), "Discovered components");
                components.extend(repo_components);
            }
            Err(e) => {
                warn!(repo = %entry.path().display(), error = %e, "Failed to scan repository");
            }
        }
    }

    info!(count = components.len(), "Component scan complete");
    Ok(components)
}

/// Scan a single repository directory for all component types
fn scan_repo_directory(repo_dir: &Path) -> Result<Vec<DiscoveredComponent>> {
    let dir_name = repo_dir.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid directory name"))?;

    // Parse repo ID from directory name (models--owner--name)
    let parts: Vec<&str> = dir_name.split("--").collect();
    if parts.len() < 3 {
        return Ok(Vec::new());
    }

    let repo_id = format!("{}/{}", parts[1], parts[2]);

    // Get the active snapshot directory
    let snapshot_dir = get_active_snapshot(repo_dir)?;
    let snapshot_hash = snapshot_dir.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());

    let mut components = Vec::new();

    // Look for transformers
    components.extend(find_transformers(&snapshot_dir, &repo_id, snapshot_hash.as_deref())?);

    // Look for T5 encoders
    components.extend(find_t5_encoders(&snapshot_dir, &repo_id, snapshot_hash.as_deref())?);

    // Look for CLIP encoders
    components.extend(find_clip_encoders(&snapshot_dir, &repo_id, snapshot_hash.as_deref())?);

    // Look for VAE decoders
    components.extend(find_vae_decoders(&snapshot_dir, &repo_id, snapshot_hash.as_deref())?);

    // Look for tokenizers
    components.extend(find_tokenizers(&snapshot_dir, &repo_id, snapshot_hash.as_deref())?);

    Ok(components)
}

/// Find transformer model files
fn find_transformers(snapshot: &Path, repo_id: &str, snapshot_hash: Option<&str>) -> Result<Vec<DiscoveredComponent>> {
    let mut components = Vec::new();

    // Known transformer filenames
    let transformer_patterns = [
        "flux1-schnell.safetensors",
        "flux1-schnell.gguf",
        "flux1-dev.safetensors",
        "flux1-dev.gguf",
        "flux1-dev-Q8_0.gguf",
        "flux1-dev-Q5_K_S.gguf",
        "diffusion_pytorch_model.safetensors",
    ];

    for pattern in &transformer_patterns {
        let path = snapshot.join(pattern);
        if path.exists() {
            if let Some(component) = create_component_from_file(
                &path,
                ComponentType::Transformer,
                repo_id,
                snapshot_hash,
            ) {
                components.push(component);
            }
        }
    }

    // Also check transformer subdirectory (for sharded models like Z-Image)
    let transformer_dir = snapshot.join("transformer");
    if transformer_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&transformer_dir) {
            // Look for sharded safetensors (e.g., model-00001-of-00003.safetensors)
            let mut shards: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext == "safetensors")
                        .unwrap_or(false)
                })
                .collect();

            if !shards.is_empty() {
                shards.sort();
                // Use first shard as representative
                if let Some(component) = create_sharded_component(
                    &shards,
                    ComponentType::Transformer,
                    repo_id,
                    snapshot_hash,
                ) {
                    components.push(component);
                }
            }
        }
    }

    Ok(components)
}

/// Find T5 encoder files
fn find_t5_encoders(snapshot: &Path, repo_id: &str, snapshot_hash: Option<&str>) -> Result<Vec<DiscoveredComponent>> {
    let mut components = Vec::new();

    // Check for T5 GGUF (quantized)
    let t5_gguf_patterns = [
        "t5-v1_1-xxl-encoder-Q5_K_M.gguf",
        "t5-v1_1-xxl-encoder-Q8_0.gguf",
        "t5-v1_1-xxl-encoder.gguf",
    ];

    for pattern in &t5_gguf_patterns {
        let path = snapshot.join(pattern);
        if path.exists() {
            if let Some(component) = create_component_from_file(
                &path,
                ComponentType::T5Encoder,
                repo_id,
                snapshot_hash,
            ) {
                components.push(component);
            }
        }
    }

    // Check text_encoder_2 directory (split safetensors)
    let t5_dir = snapshot.join("text_encoder_2");
    if t5_dir.exists() {
        let config_path = t5_dir.join("config.json");
        if config_path.exists() {
            // Check for sharded model files
            if let Ok(entries) = std::fs::read_dir(&t5_dir) {
                let shards: Vec<PathBuf> = entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| {
                        p.file_name()
                            .and_then(|n| n.to_str())
                            .map(|s| s.starts_with("model-") && s.ends_with(".safetensors"))
                            .unwrap_or(false)
                    })
                    .collect();

                if !shards.is_empty() {
                    if let Some(component) = create_sharded_component(
                        &shards,
                        ComponentType::T5Encoder,
                        repo_id,
                        snapshot_hash,
                    ) {
                        components.push(component);
                    }
                }
            }
        }
    }

    Ok(components)
}

/// Find CLIP encoder files
fn find_clip_encoders(snapshot: &Path, repo_id: &str, snapshot_hash: Option<&str>) -> Result<Vec<DiscoveredComponent>> {
    let mut components = Vec::new();

    // Check text_encoder directory
    let clip_dir = snapshot.join("text_encoder");
    let clip_model = clip_dir.join("model.safetensors");

    if clip_model.exists() {
        if let Some(component) = create_component_from_file(
            &clip_model,
            ComponentType::ClipEncoder,
            repo_id,
            snapshot_hash,
        ) {
            components.push(component);
        }
    }

    Ok(components)
}

/// Find VAE decoder files
fn find_vae_decoders(snapshot: &Path, repo_id: &str, snapshot_hash: Option<&str>) -> Result<Vec<DiscoveredComponent>> {
    let mut components = Vec::new();

    // Check for ae.safetensors (native FLUX VAE)
    let vae_path = snapshot.join("ae.safetensors");
    if vae_path.exists() {
        if let Some(component) = create_component_from_file(
            &vae_path,
            ComponentType::VaeDecoder,
            repo_id,
            snapshot_hash,
        ) {
            components.push(component);
        }
    }

    // Check for vae directory (alternative location)
    let vae_dir = snapshot.join("vae");
    if vae_dir.exists() {
        let vae_model = vae_dir.join("diffusion_pytorch_model.safetensors");
        if vae_model.exists() {
            if let Some(component) = create_component_from_file(
                &vae_model,
                ComponentType::VaeDecoder,
                repo_id,
                snapshot_hash,
            ) {
                components.push(component);
            }
        }
    }

    Ok(components)
}

/// Find tokenizer files
fn find_tokenizers(snapshot: &Path, repo_id: &str, snapshot_hash: Option<&str>) -> Result<Vec<DiscoveredComponent>> {
    let mut components = Vec::new();

    // Check for CLIP tokenizer
    let clip_tokenizer_dir = snapshot.join("tokenizer");
    let clip_tokenizer_file = clip_tokenizer_dir.join("tokenizer.json");
    if clip_tokenizer_file.exists() {
        if let Some(component) = create_component_from_file(
            &clip_tokenizer_file,
            ComponentType::ClipTokenizer,
            repo_id,
            snapshot_hash,
        ) {
            components.push(component);
        }
    }

    // Check for T5 tokenizer
    let t5_tokenizer_dir = snapshot.join("tokenizer_2");
    let t5_tokenizer_file = t5_tokenizer_dir.join("tokenizer.json");
    if t5_tokenizer_file.exists() {
        if let Some(component) = create_component_from_file(
            &t5_tokenizer_file,
            ComponentType::T5Tokenizer,
            repo_id,
            snapshot_hash,
        ) {
            components.push(component);
        }
    }

    // Also check for standalone tokenizer files (e.g., lmz/mt5-tokenizers)
    let tokenizer_file = snapshot.join("t5-v1_1-xxl.tokenizer.json");
    if tokenizer_file.exists() {
        if let Some(component) = create_component_from_file(
            &tokenizer_file,
            ComponentType::T5Tokenizer,
            repo_id,
            snapshot_hash,
        ) {
            components.push(component);
        }
    }

    Ok(components)
}

/// Create a DiscoveredComponent from a single file
fn create_component_from_file(
    path: &Path,
    comp_type: ComponentType,
    repo_id: &str,
    snapshot_hash: Option<&str>,
) -> Option<DiscoveredComponent> {
    let format = ModelFormat::from_path(path)?;
    let metadata = std::fs::metadata(path).ok()?;
    let file_size = metadata.len();

    // Compute SHA256 hash for deduplication
    let file_hash = match compute_file_hash(path) {
        Ok(hash) => {
            debug!(path = %path.display(), hash = %hash, "Computed file hash");
            Some(hash)
        }
        Err(e) => {
            warn!(path = %path.display(), error = %e, "Failed to compute file hash");
            None
        }
    };

    let quantization = extract_quantization(path);
    let architecture = infer_architecture(path, repo_id, comp_type);
    let vram_mb = estimate_vram(file_size, comp_type, &quantization);

    let name = generate_component_name(repo_id, comp_type, &quantization, false);

    Some(DiscoveredComponent {
        component_type: comp_type,
        architecture,
        format,
        path: path.to_path_buf(),
        repo_id: repo_id.to_string(),
        repo_snapshot: snapshot_hash.map(|s| s.to_string()),
        file_size,
        file_hash,
        quantization,
        is_sharded: false,
        shard_count: None,
        vram_mb,
        metadata: serde_json::json!({}),
    })
}

/// Create a DiscoveredComponent from sharded files
fn create_sharded_component(
    shards: &[PathBuf],
    comp_type: ComponentType,
    repo_id: &str,
    snapshot_hash: Option<&str>,
) -> Option<DiscoveredComponent> {
    if shards.is_empty() {
        return None;
    }

    let total_size: u64 = shards.iter()
        .filter_map(|p| std::fs::metadata(p).ok())
        .map(|m| m.len())
        .sum();

    // Compute combined hash of all shards for deduplication
    let file_hash = match compute_sharded_hash(shards) {
        Ok(hash) => {
            debug!(shard_count = shards.len(), hash = %hash, "Computed sharded file hash");
            Some(hash)
        }
        Err(e) => {
            warn!(shard_count = shards.len(), error = %e, "Failed to compute sharded hash");
            None
        }
    };

    let shard_count = shards.len();
    let architecture = infer_architecture(&shards[0], repo_id, comp_type);
    let vram_mb = estimate_vram(total_size, comp_type, &None);
    let name = generate_component_name(repo_id, comp_type, &None, true);

    // Use directory as the path for sharded models
    let path = shards[0].parent()?.to_path_buf();

    Some(DiscoveredComponent {
        component_type: comp_type,
        architecture,
        format: ModelFormat::Safetensors,
        path,
        repo_id: repo_id.to_string(),
        repo_snapshot: snapshot_hash.map(|s| s.to_string()),
        file_size: total_size,
        file_hash,
        quantization: None,
        is_sharded: true,
        shard_count: Some(shard_count),
        vram_mb,
        metadata: serde_json::json!({ "shards": shard_count }),
    })
}

/// Extract quantization info from filename
fn extract_quantization(path: &Path) -> Option<String> {
    let filename = path.file_name()?.to_str()?;

    // Known quantization patterns
    for quant in ["Q3_K_L", "Q4_K_S", "Q5_K_M", "Q8_0", "Q4_0", "Q5_0"] {
        if filename.contains(quant) {
            return Some(quant.to_string());
        }
    }

    None
}

/// Infer architecture from path and repo
fn infer_architecture(path: &Path, repo_id: &str, comp_type: ComponentType) -> String {
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    match comp_type {
        ComponentType::Transformer => {
            if filename.contains("schnell") {
                "flux-schnell".to_string()
            } else if filename.contains("dev") {
                "flux-dev".to_string()
            } else if repo_id.contains("Z-Image") {
                "z-image-turbo".to_string()
            } else {
                "flux".to_string()
            }
        }
        ComponentType::T5Encoder => "t5-v1_1-xxl".to_string(),
        ComponentType::ClipEncoder => "clip-l".to_string(),
        ComponentType::VaeDecoder => "flux-vae".to_string(),
        ComponentType::ClipTokenizer => "clip-tokenizer".to_string(),
        ComponentType::T5Tokenizer => "t5-tokenizer".to_string(),
    }
}

/// Estimate VRAM usage based on file size and type
fn estimate_vram(file_size: u64, comp_type: ComponentType, quantization: &Option<String>) -> Option<usize> {
    let size_mb = (file_size / 1_000_000) as usize;

    match comp_type {
        ComponentType::Transformer => {
            if quantization.is_some() {
                Some(size_mb + 1000) // Quantized + overhead
            } else {
                Some(size_mb + 1000) // Full precision + overhead
            }
        }
        ComponentType::T5Encoder => {
            if quantization.is_some() {
                Some(3500) // ~3.3GB quantized + overhead
            } else {
                Some(9000) // ~9GB full + overhead
            }
        }
        ComponentType::ClipEncoder => Some(500), // ~300MB + overhead
        ComponentType::VaeDecoder => Some(200),  // ~150MB + overhead
        _ => None,
    }
}

/// Generate a descriptive name for the component
fn generate_component_name(repo_id: &str, comp_type: ComponentType, quantization: &Option<String>, is_sharded: bool) -> String {
    let base_name = match comp_type {
        ComponentType::Transformer => {
            if repo_id.contains("schnell") {
                "FLUX.1 Schnell Transformer"
            } else if repo_id.contains("dev") {
                "FLUX.1 Dev Transformer"
            } else if repo_id.contains("Z-Image") {
                "Z-Image Turbo Transformer"
            } else {
                "FLUX Transformer"
            }
        }
        ComponentType::T5Encoder => "T5-XXL Text Encoder",
        ComponentType::ClipEncoder => "CLIP-L Text Encoder",
        ComponentType::VaeDecoder => "FLUX VAE Decoder",
        ComponentType::ClipTokenizer => "CLIP Tokenizer",
        ComponentType::T5Tokenizer => "T5 Tokenizer",
    };

    let mut name = base_name.to_string();

    if let Some(quant) = quantization {
        name.push_str(&format!(" ({})", quant));
    }

    if is_sharded {
        name.push_str(" [Sharded]");
    }

    name
}

/// Get the active snapshot directory for a model
fn get_active_snapshot(model_dir: &Path) -> Result<PathBuf> {
    // Try to read refs/main
    let refs_main = model_dir.join("refs").join("main");
    if let Ok(hash) = std::fs::read_to_string(&refs_main) {
        let snapshot = model_dir.join("snapshots").join(hash.trim());
        if snapshot.exists() {
            return Ok(snapshot);
        }
    }

    // Fallback: find first snapshot directory
    let snapshots_dir = model_dir.join("snapshots");
    if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                return Ok(entry.path());
            }
        }
    }

    anyhow::bail!("Could not find snapshot directory in {}", model_dir.display())
}

/// Compute SHA256 hash of a file
fn compute_file_hash(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 8192]; // 8KB buffer for efficient reading

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash_bytes = hasher.finalize();
    Ok(format!("{:x}", hash_bytes))
}

/// Compute SHA256 hash of multiple sharded files combined
fn compute_sharded_hash(shard_paths: &[PathBuf]) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 8192];

    for shard_path in shard_paths {
        let mut file = std::fs::File::open(shard_path)?;
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
    }

    let hash_bytes = hasher.finalize();
    Ok(format!("{:x}", hash_bytes))
}

/// Get the HuggingFace cache directory
fn get_hf_cache_dir() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    // Check environment variables
    if let Ok(hf_cache) = std::env::var("HF_HUB_CACHE") {
        return Ok(PathBuf::from(hf_cache));
    }

    if let Ok(hf_home) = std::env::var("HF_HOME") {
        return Ok(PathBuf::from(hf_home).join("hub"));
    }

    // Default location
    #[cfg(target_os = "macos")]
    {
        let macos_cache = home.join("Library").join("Caches").join("huggingface").join("hub");
        if macos_cache.exists() {
            return Ok(macos_cache);
        }
    }

    Ok(home.join(".cache").join("huggingface").join("hub"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_type_serialization() {
        let ct = ComponentType::Transformer;
        assert_eq!(ct.as_str(), "transformer");
    }

    #[test]
    fn test_model_format_detection() {
        assert_eq!(
            ModelFormat::from_path(Path::new("model.safetensors")),
            Some(ModelFormat::Safetensors)
        );
        assert_eq!(
            ModelFormat::from_path(Path::new("model.gguf")),
            Some(ModelFormat::Gguf)
        );
        assert_eq!(
            ModelFormat::from_path(Path::new("tokenizer.json")),
            Some(ModelFormat::Json)
        );
    }

    #[test]
    fn test_quantization_extraction() {
        assert_eq!(
            extract_quantization(Path::new("t5-encoder-Q5_K_M.gguf")),
            Some("Q5_K_M".to_string())
        );
        assert_eq!(
            extract_quantization(Path::new("flux-Q8_0.gguf")),
            Some("Q8_0".to_string())
        );
        assert_eq!(
            extract_quantization(Path::new("model.safetensors")),
            None
        );
    }

    #[test]
    fn test_architecture_inference() {
        assert_eq!(
            infer_architecture(
                Path::new("flux1-schnell.safetensors"),
                "black-forest-labs/FLUX.1-schnell",
                ComponentType::Transformer
            ),
            "flux-schnell"
        );

        assert_eq!(
            infer_architecture(
                Path::new("model.safetensors"),
                "openai/clip",
                ComponentType::ClipEncoder
            ),
            "clip-l"
        );
    }
}
