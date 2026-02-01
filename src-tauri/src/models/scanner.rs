//! Model cache scanner - discovers ALL model components in HuggingFace cache

use anyhow::Result;
use serde::{Deserialize, Serialize};
use blake2::{Blake2b512, Digest};
use std::path::{Path, PathBuf};
use std::io::{Read, BufReader};
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
    Lora,
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
            Self::Lora => "lora",
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
    pub file_hash: Option<String>, // BLAKE2b hash for deduplication
    pub quantization: Option<String>,
    pub is_sharded: bool,
    pub shard_count: Option<usize>,
    pub vram_mb: Option<usize>,
    pub metadata: serde_json::Value,
}

/// Progress callback for scan operations
pub type ScanProgressCallback = Box<dyn Fn(usize, usize, &str) + Send>;

/// Hash progress callback for detailed hash computation updates
pub type HashProgressCallback = Box<dyn Fn(&str, u8, u64, u64) + Send + Sync>;

/// Repo completion callback - called after each repo is fully scanned with its components
pub type RepoCompletionCallback = Box<dyn Fn(&str, Vec<DiscoveredComponent>) + Send>;

/// Repo skip callback - returns true if repo should be skipped (repo_id, snapshot_hash) -> should_skip
pub type RepoSkipCallback = Box<dyn Fn(&str, &str) -> bool + Send>;

/// Scans the HuggingFace cache for ALL model components
pub fn scan_all_components() -> Result<Vec<DiscoveredComponent>> {
    scan_all_components_with_progress(None, None)
}

/// Scans the HuggingFace cache for ALL model components with optional progress callbacks
pub fn scan_all_components_with_progress(
    progress_callback: Option<ScanProgressCallback>,
    hash_progress_callback: Option<HashProgressCallback>,
) -> Result<Vec<DiscoveredComponent>> {
    scan_all_components_with_callbacks(
        progress_callback,
        hash_progress_callback,
        None,
        None,
    )
}

/// Scans the HuggingFace cache for ALL model components with full callback support
/// Skips hash computation by default for faster scanning
pub fn scan_all_components_with_callbacks(
    progress_callback: Option<ScanProgressCallback>,
    hash_progress_callback: Option<HashProgressCallback>,
    repo_completion_callback: Option<RepoCompletionCallback>,
    repo_skip_callback: Option<RepoSkipCallback>,
) -> Result<Vec<DiscoveredComponent>> {
    scan_all_components_full(
        progress_callback,
        hash_progress_callback,
        repo_completion_callback,
        repo_skip_callback,
        true, // skip hashes by default
    )
}

/// Scans with option to skip hash computation (faster for debugging)
pub fn scan_all_components_fast(
    progress_callback: Option<ScanProgressCallback>,
    repo_completion_callback: Option<RepoCompletionCallback>,
) -> Result<Vec<DiscoveredComponent>> {
    scan_all_components_full(
        progress_callback,
        None,
        repo_completion_callback,
        None,
        true, // skip hashes
    )
}

/// Full scan function with all options
fn scan_all_components_full(
    progress_callback: Option<ScanProgressCallback>,
    hash_progress_callback: Option<HashProgressCallback>,
    repo_completion_callback: Option<RepoCompletionCallback>,
    repo_skip_callback: Option<RepoSkipCallback>,
    skip_hash: bool,
) -> Result<Vec<DiscoveredComponent>> {
    let cache_dir = get_hf_cache_dir()?;
    let mut components = Vec::new();

    info!("📂 Starting HuggingFace cache scan");
    info!("   Cache directory: {}", cache_dir.display());

    if !cache_dir.exists() {
        warn!("⚠️  HuggingFace cache directory does not exist: {}", cache_dir.display());
        warn!("   No models will be discovered");
        return Ok(components);
    }

    let entries = std::fs::read_dir(&cache_dir)?;
    let model_dirs: Vec<_> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("models--")
        })
        .collect();

    let total_repos = model_dirs.len();
    info!("   Found {} model repositories to scan", total_repos);

    for (idx, entry) in model_dirs.iter().enumerate() {
        let dir_name = entry.file_name();
        let dir_name_str = dir_name.to_string_lossy();

        // Parse repo name for display
        let repo_name = dir_name_str
            .strip_prefix("models--")
            .map(|s| s.replace("--", "/"))
            .unwrap_or_else(|| dir_name_str.to_string());

        // Check if we should skip this repo
        if let Some(ref skip_callback) = repo_skip_callback {
            // Get current snapshot hash for this repo
            match get_repo_snapshot_hash(&entry.path()) {
                Ok(snapshot_hash) => {
                    if skip_callback(&repo_name, &snapshot_hash) {
                        info!("⏭️  [{}/{}] Skipping (unchanged): {}", idx + 1, total_repos, repo_name);
                        continue;
                    }
                }
                Err(e) => {
                    warn!("   Could not get snapshot hash for {}: {} - will scan", repo_name, e);
                }
            }
        }

        info!("🔍 [{}/{}] Scanning: {}", idx + 1, total_repos, repo_name);

        // Call progress callback if provided
        if let Some(ref callback) = progress_callback {
            callback(idx + 1, total_repos, &repo_name);
        }

        // Scan this repository for all component types
        match scan_repo_directory(&entry.path(), hash_progress_callback.as_ref(), skip_hash) {
            Ok(repo_components) => {
                if !repo_components.is_empty() {
                    info!("   ✓ Found {} component(s)", repo_components.len());
                    for comp in &repo_components {
                        let size_mb = comp.file_size / 1_000_000;
                        let quant_info = comp.quantization.as_ref()
                            .map(|q| format!(" [{}]", q))
                            .unwrap_or_default();
                        let shard_info = if comp.is_sharded {
                            format!(" [Sharded: {}]", comp.shard_count.unwrap_or(0))
                        } else {
                            String::new()
                        };

                        info!("     • {} ({} MB){}{}",
                            generate_component_name(&comp.repo_id, comp.component_type, &comp.quantization, comp.is_sharded),
                            size_mb,
                            quant_info,
                            shard_info
                        );
                    }

                    // Call repo completion callback if provided
                    if let Some(ref callback) = repo_completion_callback {
                        callback(&repo_name, repo_components.clone());
                    }

                    components.extend(repo_components);
                } else {
                    debug!("   No usable components found in {}", repo_name);
                }
            }
            Err(e) => {
                warn!("   ✗ Failed to scan {}: {}", repo_name, e);
            }
        }
    }

    // Summary by type
    let transformers = components.iter().filter(|c| matches!(c.component_type, ComponentType::Transformer)).count();
    let t5_encoders = components.iter().filter(|c| matches!(c.component_type, ComponentType::T5Encoder)).count();
    let clip_encoders = components.iter().filter(|c| matches!(c.component_type, ComponentType::ClipEncoder)).count();
    let vae_decoders = components.iter().filter(|c| matches!(c.component_type, ComponentType::VaeDecoder)).count();
    let tokenizers = components.iter().filter(|c| matches!(c.component_type, ComponentType::ClipTokenizer | ComponentType::T5Tokenizer)).count();

    info!("✅ Component scan complete");
    info!("   Total components found: {}", components.len());
    info!("   - Transformers: {}", transformers);
    info!("   - T5 Encoders: {}", t5_encoders);
    info!("   - CLIP Encoders: {}", clip_encoders);
    info!("   - VAE Decoders: {}", vae_decoders);
    info!("   - Tokenizers: {}", tokenizers);

    Ok(components)
}

/// Scan a single repository directory for all component types
fn scan_repo_directory(
    repo_dir: &Path,
    hash_progress_callback: Option<&HashProgressCallback>,
    skip_hash: bool,
) -> Result<Vec<DiscoveredComponent>> {
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
    components.extend(find_transformers(&snapshot_dir, &repo_id, snapshot_hash.as_deref(), hash_progress_callback, skip_hash)?);

    // Look for T5 encoders
    components.extend(find_t5_encoders(&snapshot_dir, &repo_id, snapshot_hash.as_deref(), hash_progress_callback, skip_hash)?);

    // Look for CLIP encoders
    components.extend(find_clip_encoders(&snapshot_dir, &repo_id, snapshot_hash.as_deref(), hash_progress_callback, skip_hash)?);

    // Look for VAE decoders
    components.extend(find_vae_decoders(&snapshot_dir, &repo_id, snapshot_hash.as_deref(), hash_progress_callback, skip_hash)?);

    // Look for tokenizers
    components.extend(find_tokenizers(&snapshot_dir, &repo_id, snapshot_hash.as_deref(), hash_progress_callback)?);

    Ok(components)
}

/// Find transformer model files
fn find_transformers(
    snapshot: &Path,
    repo_id: &str,
    snapshot_hash: Option<&str>,
    hash_progress_callback: Option<&HashProgressCallback>,
    skip_hash: bool,
) -> Result<Vec<DiscoveredComponent>> {
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
                hash_progress_callback,
                skip_hash,
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
                    hash_progress_callback,
                    skip_hash,
                ) {
                    components.push(component);
                }
            }
        }
    }

    Ok(components)
}

/// Find T5 encoder files
fn find_t5_encoders(
    snapshot: &Path,
    repo_id: &str,
    snapshot_hash: Option<&str>,
    hash_progress_callback: Option<&HashProgressCallback>,
    skip_hash: bool,
) -> Result<Vec<DiscoveredComponent>> {
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
                hash_progress_callback,
                skip_hash,
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
                        hash_progress_callback,
                        skip_hash,
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
fn find_clip_encoders(
    snapshot: &Path,
    repo_id: &str,
    snapshot_hash: Option<&str>,
    hash_progress_callback: Option<&HashProgressCallback>,
    skip_hash: bool,
) -> Result<Vec<DiscoveredComponent>> {
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
            hash_progress_callback,
            skip_hash,
        ) {
            components.push(component);
        }
    }

    Ok(components)
}

/// Find VAE decoder files
fn find_vae_decoders(
    snapshot: &Path,
    repo_id: &str,
    snapshot_hash: Option<&str>,
    hash_progress_callback: Option<&HashProgressCallback>,
    skip_hash: bool,
) -> Result<Vec<DiscoveredComponent>> {
    let mut components = Vec::new();

    // Check for ae.safetensors (native FLUX VAE)
    let vae_path = snapshot.join("ae.safetensors");
    if vae_path.exists() {
        if let Some(component) = create_component_from_file(
            &vae_path,
            ComponentType::VaeDecoder,
            repo_id,
            snapshot_hash,
            hash_progress_callback,
            skip_hash,
        ) {
            components.push(component);
        }
    }

    // Check for vae directory (alternative location) — only if ae.safetensors wasn't found,
    // since both are the same model in different layouts (native vs diffusers).
    let vae_dir = snapshot.join("vae");
    if components.is_empty() && vae_dir.exists() {
        let vae_model = vae_dir.join("diffusion_pytorch_model.safetensors");
        if vae_model.exists() {
            if let Some(component) = create_component_from_file(
                &vae_model,
                ComponentType::VaeDecoder,
                repo_id,
                snapshot_hash,
                hash_progress_callback,
                skip_hash,
            ) {
                components.push(component);
            }
        }
    }

    Ok(components)
}

/// Find tokenizer files
fn find_tokenizers(
    snapshot: &Path,
    repo_id: &str,
    snapshot_hash: Option<&str>,
    _hash_progress_callback: Option<&HashProgressCallback>,
) -> Result<Vec<DiscoveredComponent>> {
    let mut components = Vec::new();

    // Check for CLIP tokenizer directory
    // Priority: tokenizer.json > vocab.json (BPE format)
    let clip_tokenizer_dir = snapshot.join("tokenizer");
    if clip_tokenizer_dir.is_dir() {
        let tokenizer_json = clip_tokenizer_dir.join("tokenizer.json");
        let vocab_json = clip_tokenizer_dir.join("vocab.json");

        // Prefer tokenizer.json as it works directly with Tokenizer::from_file
        let tokenizer_file = if tokenizer_json.exists() {
            Some(tokenizer_json)
        } else if vocab_json.exists() {
            // BPE format - store vocab.json path, loader can handle BPE directory
            Some(vocab_json)
        } else {
            None
        };

        if let Some(file_path) = tokenizer_file {
            if let Some(component) = create_tokenizer_component_from_file(
                &file_path,
                ComponentType::ClipTokenizer,
                repo_id,
                snapshot_hash,
            ) {
                components.push(component);
            }
        }
    }

    // Check for T5 tokenizer (tokenizer_2 directory)
    // Priority: tokenizer.json > spiece.model
    let t5_tokenizer_dir = snapshot.join("tokenizer_2");
    if t5_tokenizer_dir.is_dir() {
        let tokenizer_json = t5_tokenizer_dir.join("tokenizer.json");
        let spiece_model = t5_tokenizer_dir.join("spiece.model");

        // Prefer tokenizer.json as it works directly with Tokenizer::from_file
        let tokenizer_file = if tokenizer_json.exists() {
            Some(tokenizer_json)
        } else if spiece_model.exists() {
            // SentencePiece format - will need special handling in loader
            Some(spiece_model)
        } else {
            None
        };

        if let Some(file_path) = tokenizer_file {
            if let Some(component) = create_tokenizer_component_from_file(
                &file_path,
                ComponentType::T5Tokenizer,
                repo_id,
                snapshot_hash,
            ) {
                components.push(component);
            }
        }
    }

    // Also check for standalone tokenizer files (e.g., lmz/mt5-tokenizers)
    let standalone_tokenizer = snapshot.join("t5-v1_1-xxl.tokenizer.json");
    if standalone_tokenizer.exists() {
        if let Some(component) = create_tokenizer_component_from_file(
            &standalone_tokenizer,
            ComponentType::T5Tokenizer,
            repo_id,
            snapshot_hash,
        ) {
            components.push(component);
        }
    }

    Ok(components)
}

/// Create a DiscoveredComponent from a tokenizer file path
///
/// Stores the actual file path (tokenizer.json, vocab.json, or spiece.model)
/// so model loaders can use it directly with Tokenizer::from_file()
fn create_tokenizer_component_from_file(
    tokenizer_file: &Path,
    comp_type: ComponentType,
    repo_id: &str,
    snapshot_hash: Option<&str>,
) -> Option<DiscoveredComponent> {
    if !tokenizer_file.is_file() {
        return None;
    }

    let file_size = std::fs::metadata(tokenizer_file).ok()?.len();

    // Determine format from file extension
    let format = if tokenizer_file.extension().and_then(|e| e.to_str()) == Some("model") {
        // spiece.model is binary SentencePiece format
        ModelFormat::Json // Treat as JSON for compatibility (it's a tokenizer)
    } else {
        ModelFormat::Json
    };

    // Tokenizers are small, no need to compute hash
    let _name = generate_component_name(repo_id, comp_type, &None, false);

    debug!(
        path = %tokenizer_file.display(),
        component_type = ?comp_type,
        "Found tokenizer file"
    );

    Some(DiscoveredComponent {
        component_type: comp_type,
        architecture: "tokenizer".to_string(),
        format,
        path: tokenizer_file.to_path_buf(),
        repo_id: repo_id.to_string(),
        repo_snapshot: snapshot_hash.map(|s| s.to_string()),
        file_size,
        file_hash: None, // Skip hash for small tokenizer files
        quantization: None,
        is_sharded: false,
        shard_count: None,
        vram_mb: None, // Tokenizers use negligible VRAM
        metadata: serde_json::json!({}),
    })
}

/// Create a DiscoveredComponent from a single file
fn create_component_from_file(
    path: &Path,
    comp_type: ComponentType,
    repo_id: &str,
    snapshot_hash: Option<&str>,
    hash_progress_callback: Option<&HashProgressCallback>,
    skip_hash: bool,
) -> Option<DiscoveredComponent> {
    let format = ModelFormat::from_path(path)?;
    let metadata = std::fs::metadata(path).ok()?;
    let file_size = metadata.len();

    // Compute BLAKE2b hash for deduplication (unless skipped)
    let file_hash = if skip_hash {
        None
    } else {
        match compute_file_hash(path, hash_progress_callback) {
            Ok(hash) => {
                debug!(path = %path.display(), hash = %hash[..16], "File hash computed");
                Some(hash)
            }
            Err(e) => {
                warn!(path = %path.display(), error = %e, "Failed to compute file hash");
                None
            }
        }
    };

    let quantization = extract_quantization(path);
    let architecture = infer_architecture(path, repo_id, comp_type);
    let vram_mb = estimate_vram(file_size, comp_type, &quantization);

    let _name = generate_component_name(repo_id, comp_type, &quantization, false);

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
    hash_progress_callback: Option<&HashProgressCallback>,
    skip_hash: bool,
) -> Option<DiscoveredComponent> {
    if shards.is_empty() {
        return None;
    }

    let total_size: u64 = shards.iter()
        .filter_map(|p| std::fs::metadata(p).ok())
        .map(|m| m.len())
        .sum();

    // Compute combined BLAKE2b hash of all shards for deduplication (unless skipped)
    let file_hash = if skip_hash {
        None
    } else {
        match compute_sharded_hash(shards, hash_progress_callback) {
            Ok(hash) => {
                debug!(shard_count = shards.len(), hash = %hash, "Computed sharded file hash");
                Some(hash)
            }
            Err(e) => {
                warn!(shard_count = shards.len(), error = %e, "Failed to compute sharded hash");
                None
            }
        }
    };

    let shard_count = shards.len();
    let architecture = infer_architecture(&shards[0], repo_id, comp_type);
    let vram_mb = estimate_vram(total_size, comp_type, &None);
    let _name = generate_component_name(repo_id, comp_type, &None, true);

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
        ComponentType::Lora => "flux-lora".to_string(),
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
        ComponentType::Lora => "LoRA Adapter",
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

/// Compute BLAKE2b hash of a file with progress logging and callbacks
fn compute_file_hash(
    path: &Path,
    progress_callback: Option<&HashProgressCallback>,
) -> Result<String> {
    let file_metadata = std::fs::metadata(path)?;
    let total_size = file_metadata.len();
    let file = std::fs::File::open(path)?;
    let mut file = BufReader::with_capacity(8 * 1024 * 1024, file); // 8MB BufReader
    let mut hasher = Blake2b512::new();
    let mut buffer = vec![0; 8 * 1024 * 1024]; // 8MB buffer for better I/O performance

    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    // Only log progress for large files (>100MB)
    let should_log = total_size > 100_000_000;
    let mut bytes_processed = 0u64;
    let mut last_log_percent = 0;
    let mut last_callback_percent = 0;

    if should_log {
        let size_gb = total_size as f64 / 1_000_000_000.0;
        info!("   🔐 Computing hash for {} ({:.1} GB)...", filename, size_gb);
    }

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
        bytes_processed += bytes_read as u64;

        let percent = (bytes_processed * 100 / total_size) as u8;

        // Log progress every 10% for large files
        if should_log && percent >= last_log_percent + 10 && percent < 100 {
            let processed_gb = bytes_processed as f64 / 1_000_000_000.0;
            let total_gb = total_size as f64 / 1_000_000_000.0;
            info!("      Progress: {}% ({:.1} GB / {:.1} GB)",
                percent, processed_gb, total_gb);
            last_log_percent = percent;
        }

        // Call progress callback every 5% for large files
        if should_log && percent >= last_callback_percent + 5 && percent < 100 {
            if let Some(callback) = progress_callback {
                callback(filename, percent, bytes_processed, total_size);
            }
            last_callback_percent = percent;
        }
    }

    // Final callback at 100%
    if should_log {
        info!("      ✓ Hash computed successfully");
        if let Some(callback) = progress_callback {
            callback(filename, 100, total_size, total_size);
        }
    }

    let hash_bytes = hasher.finalize();
    Ok(format!("{:x}", hash_bytes))
}

/// Compute BLAKE2b hash of multiple sharded files combined with progress logging
fn compute_sharded_hash(
    shard_paths: &[PathBuf],
    progress_callback: Option<&HashProgressCallback>,
) -> Result<String> {
    let mut hasher = Blake2b512::new();
    let mut buffer = vec![0; 8 * 1024 * 1024]; // 8MB buffer

    // Calculate total size
    let total_size: u64 = shard_paths.iter()
        .filter_map(|p| std::fs::metadata(p).ok())
        .map(|m| m.len())
        .sum();

    let should_log = total_size > 100_000_000; // Log if >100MB
    let mut bytes_processed = 0u64;
    let mut last_log_percent = 0;
    let mut last_callback_percent = 0;

    let display_name = format!("{} sharded files", shard_paths.len());

    if should_log {
        let size_gb = total_size as f64 / 1_000_000_000.0;
        info!("   🔐 Computing hash for {} sharded files ({:.1} GB)...",
            shard_paths.len(), size_gb);
    }

    for (idx, shard_path) in shard_paths.iter().enumerate() {
        if should_log && shard_paths.len() > 1 {
            info!("      Processing shard {}/{}", idx + 1, shard_paths.len());
        }

        let file = std::fs::File::open(shard_path)?;
        let mut file = BufReader::with_capacity(8 * 1024 * 1024, file);
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
            bytes_processed += bytes_read as u64;

            let percent = if total_size > 0 {
                (bytes_processed * 100 / total_size) as u8
            } else {
                0
            };

            // Log progress every 10%
            if should_log && percent >= last_log_percent + 10 && percent < 100 {
                let processed_gb = bytes_processed as f64 / 1_000_000_000.0;
                let total_gb = total_size as f64 / 1_000_000_000.0;
                info!("      Progress: {}% ({:.1} GB / {:.1} GB)",
                    percent, processed_gb, total_gb);
                last_log_percent = percent;
            }

            // Call progress callback every 5%
            if should_log && percent >= last_callback_percent + 5 && percent < 100 {
                if let Some(callback) = progress_callback {
                    callback(&display_name, percent, bytes_processed, total_size);
                }
                last_callback_percent = percent;
            }
        }
    }

    // Final callback at 100%
    if should_log {
        info!("      ✓ Hash computed successfully");
        if let Some(callback) = progress_callback {
            callback(&display_name, 100, total_size, total_size);
        }
    }

    let hash_bytes = hasher.finalize();
    Ok(format!("{:x}", hash_bytes))
}

/// Get the current snapshot hash for a repo directory
pub fn get_repo_snapshot_hash(repo_dir: &Path) -> Result<String> {
    let snapshot_dir = get_active_snapshot(repo_dir)?;
    let snapshot_hash = snapshot_dir.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid snapshot directory name"))?;
    Ok(snapshot_hash.to_string())
}

/// Get the repo_id from a repo directory path
pub fn get_repo_id_from_path(repo_dir: &Path) -> Result<String> {
    let dir_name = repo_dir.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid directory name"))?;

    // Parse repo ID from directory name (models--owner--name)
    let parts: Vec<&str> = dir_name.split("--").collect();
    if parts.len() < 3 {
        anyhow::bail!("Invalid repo directory format: {}", dir_name);
    }

    Ok(format!("{}/{}", parts[1], parts[2]))
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

// ============================================================================
// Directory Scanner - Scans arbitrary directories for model files
// ============================================================================

/// Scan a directory recursively for model files, identifying types by inspecting file headers
pub fn scan_directory_for_models(
    dir_path: &Path,
    progress_callback: Option<ScanProgressCallback>,
    hash_progress_callback: Option<HashProgressCallback>,
) -> Result<Vec<DiscoveredComponent>> {
    scan_directory_for_models_full(dir_path, progress_callback, hash_progress_callback, false)
}

/// Fast directory scan without hash computation
pub fn scan_directory_for_models_fast(
    dir_path: &Path,
    progress_callback: Option<ScanProgressCallback>,
) -> Result<Vec<DiscoveredComponent>> {
    scan_directory_for_models_full(dir_path, progress_callback, None, true)
}

/// Full directory scan with all options
fn scan_directory_for_models_full(
    dir_path: &Path,
    progress_callback: Option<ScanProgressCallback>,
    hash_progress_callback: Option<HashProgressCallback>,
    skip_hash: bool,
) -> Result<Vec<DiscoveredComponent>> {
    info!("📂 Starting directory scan: {}", dir_path.display());

    if !dir_path.exists() {
        anyhow::bail!("Directory does not exist: {}", dir_path.display());
    }

    if !dir_path.is_dir() {
        anyhow::bail!("Path is not a directory: {}", dir_path.display());
    }

    // First pass: discover all model files
    info!("   Phase 1: Discovering model files...");
    let model_files = discover_model_files(dir_path)?;
    let total_files = model_files.len();
    info!("   Found {} potential model files", total_files);

    if total_files == 0 {
        return Ok(Vec::new());
    }

    // Second pass: identify and create components
    info!("   Phase 2: Identifying component types...");
    let mut components = Vec::new();

    for (idx, file_path) in model_files.iter().enumerate() {
        let filename = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Emit progress
        if let Some(ref callback) = progress_callback {
            callback(idx + 1, total_files, filename);
        }

        // Check if this is an unsupported model variant (Kontext, Fill, etc.)
        let filename_lower = filename.to_lowercase();
        if is_unsupported_model(&filename_lower) {
            info!("   ⏭ [{}/{}] {} -> Unsupported model variant, skipping", idx + 1, total_files, filename);
            continue;
        }

        // Try to identify the component type
        match identify_model_file(file_path) {
            Ok(Some(component_type)) => {
                info!("   ✓ [{}/{}] {} -> {:?}", idx + 1, total_files, filename, component_type);

                // Create component from identified file
                if let Some(component) = create_component_from_loose_file(
                    file_path,
                    component_type,
                    hash_progress_callback.as_ref(),
                    skip_hash,
                ) {
                    // Skip sharded models (not currently supported)
                    if component.is_sharded {
                        info!("   ⏭ Skipping sharded model: {}", filename);
                        continue;
                    }
                    components.push(component);
                }
            }
            Ok(None) => {
                debug!("   - [{}/{}] {} -> Unknown type, skipping", idx + 1, total_files, filename);
            }
            Err(e) => {
                warn!("   ✗ [{}/{}] {} -> Error: {}", idx + 1, total_files, filename, e);
            }
        }
    }

    // Summary
    let transformers = components.iter().filter(|c| matches!(c.component_type, ComponentType::Transformer)).count();
    let t5_encoders = components.iter().filter(|c| matches!(c.component_type, ComponentType::T5Encoder)).count();
    let clip_encoders = components.iter().filter(|c| matches!(c.component_type, ComponentType::ClipEncoder)).count();
    let vae_decoders = components.iter().filter(|c| matches!(c.component_type, ComponentType::VaeDecoder)).count();

    info!("✅ Directory scan complete: {}", dir_path.display());
    info!("   Total components identified: {}", components.len());
    info!("   - Transformers: {}", transformers);
    info!("   - T5 Encoders: {}", t5_encoders);
    info!("   - CLIP Encoders: {}", clip_encoders);
    info!("   - VAE Decoders: {}", vae_decoders);

    Ok(components)
}

/// Recursively discover all model files in a directory
fn discover_model_files(dir_path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    discover_model_files_recursive(dir_path, &mut files)?;
    Ok(files)
}

fn discover_model_files_recursive(dir_path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let entries = std::fs::read_dir(dir_path)?;

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            // Skip hidden directories and common non-model directories
            let dir_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if !dir_name.starts_with('.') && dir_name != "node_modules" && dir_name != "__pycache__" {
                discover_model_files_recursive(&path, files)?;
            }
        } else if path.is_file() {
            // Check if it's a model file by extension
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                match ext.to_lowercase().as_str() {
                    "safetensors" | "gguf" => {
                        files.push(path);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

/// Identify a model file's component type by inspecting its contents
fn identify_model_file(path: &Path) -> Result<Option<ComponentType>> {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    match ext.as_deref() {
        Some("safetensors") => identify_safetensors_component(path),
        Some("gguf") => identify_gguf_component(path),
        _ => Ok(None),
    }
}

/// Identify component type from a safetensors file by reading its header
fn identify_safetensors_component(path: &Path) -> Result<Option<ComponentType>> {
    use std::io::{Read, Seek, SeekFrom};

    let mut file = std::fs::File::open(path)?;

    // Read header length (first 8 bytes, little-endian u64)
    let mut header_len_bytes = [0u8; 8];
    file.read_exact(&mut header_len_bytes)?;
    let header_len = u64::from_le_bytes(header_len_bytes) as usize;

    // Sanity check: header shouldn't be too large (max 10MB)
    if header_len > 10 * 1024 * 1024 {
        warn!("Safetensors header too large: {} bytes", header_len);
        return Ok(None);
    }

    // Read the JSON header
    let mut header_bytes = vec![0u8; header_len];
    file.read_exact(&mut header_bytes)?;

    let header_str = String::from_utf8_lossy(&header_bytes);

    // Parse as JSON to get tensor names
    let header: serde_json::Value = serde_json::from_str(&header_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse safetensors header: {}", e))?;

    // Get tensor names from the header (keys of the JSON object, excluding __metadata__)
    let tensor_names: Vec<&str> = header.as_object()
        .map(|obj| obj.keys().filter(|k| *k != "__metadata__").map(|s| s.as_str()).collect())
        .unwrap_or_default();

    if tensor_names.is_empty() {
        return Ok(None);
    }

    // Identify component type based on tensor naming patterns
    Ok(identify_component_from_tensor_names(&tensor_names))
}

/// Identify component type from GGUF file metadata
fn identify_gguf_component(path: &Path) -> Result<Option<ComponentType>> {
    use std::io::{Read, Seek, SeekFrom};

    let mut file = std::fs::File::open(path)?;

    // Read magic bytes
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;

    if &magic != b"GGUF" {
        return Ok(None);
    }

    // Read version (u32 LE)
    let mut version_bytes = [0u8; 4];
    file.read_exact(&mut version_bytes)?;
    let version = u32::from_le_bytes(version_bytes);

    if version < 2 || version > 3 {
        warn!("Unsupported GGUF version: {}", version);
        return Ok(None);
    }

    // Read tensor count and metadata kv count (both u64 LE)
    let mut tensor_count_bytes = [0u8; 8];
    let mut kv_count_bytes = [0u8; 8];
    file.read_exact(&mut tensor_count_bytes)?;
    file.read_exact(&mut kv_count_bytes)?;

    let _tensor_count = u64::from_le_bytes(tensor_count_bytes);
    let kv_count = u64::from_le_bytes(kv_count_bytes);

    // Read metadata key-value pairs to find architecture
    let mut architecture: Option<String> = None;

    for _ in 0..kv_count.min(100) { // Limit to first 100 keys for safety
        // Read key length and key
        let key = match read_gguf_string(&mut file) {
            Ok(k) => k,
            Err(_) => break,
        };

        // Read value type (u32 LE)
        let mut value_type_bytes = [0u8; 4];
        if file.read_exact(&mut value_type_bytes).is_err() {
            break;
        }
        let value_type = u32::from_le_bytes(value_type_bytes);

        // We only care about string values (type 8)
        if value_type == 8 {
            if let Ok(value) = read_gguf_string(&mut file) {
                if key == "general.architecture" {
                    architecture = Some(value.clone());
                }
                // Also check general.name for hints
                if key == "general.name" {
                    let lower = value.to_lowercase();
                    if lower.contains("flux") {
                        return Ok(Some(ComponentType::Transformer));
                    }
                    if lower.contains("t5") {
                        return Ok(Some(ComponentType::T5Encoder));
                    }
                    if lower.contains("clip") {
                        return Ok(Some(ComponentType::ClipEncoder));
                    }
                }
            }
        } else {
            // Skip other value types
            if skip_gguf_value(&mut file, value_type).is_err() {
                break;
            }
        }
    }

    // Classify based on architecture
    if let Some(arch) = architecture {
        let arch_lower = arch.to_lowercase();
        if arch_lower.contains("flux") || arch_lower == "diffusion" {
            return Ok(Some(ComponentType::Transformer));
        }
        if arch_lower.contains("t5") {
            return Ok(Some(ComponentType::T5Encoder));
        }
        if arch_lower.contains("clip") {
            return Ok(Some(ComponentType::ClipEncoder));
        }
    }

    // Fallback: check filename for hints
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    if filename.contains("flux") || filename.contains("schnell") || filename.contains("dev") {
        return Ok(Some(ComponentType::Transformer));
    }
    if filename.contains("t5") {
        return Ok(Some(ComponentType::T5Encoder));
    }
    if filename.contains("clip") {
        return Ok(Some(ComponentType::ClipEncoder));
    }
    if filename.contains("vae") || filename.contains("ae.") {
        return Ok(Some(ComponentType::VaeDecoder));
    }

    Ok(None)
}

/// Read a GGUF string (length-prefixed)
fn read_gguf_string(file: &mut std::fs::File) -> Result<String> {
    use std::io::Read;

    let mut len_bytes = [0u8; 8];
    file.read_exact(&mut len_bytes)?;
    let len = u64::from_le_bytes(len_bytes) as usize;

    if len > 1024 * 1024 { // Max 1MB string
        anyhow::bail!("GGUF string too long: {}", len);
    }

    let mut bytes = vec![0u8; len];
    file.read_exact(&mut bytes)?;

    Ok(String::from_utf8_lossy(&bytes).to_string())
}

/// Skip a GGUF value of a given type
fn skip_gguf_value(file: &mut std::fs::File, value_type: u32) -> Result<()> {
    use std::io::{Read, Seek, SeekFrom};

    let skip_bytes: i64 = match value_type {
        0 => 1,  // u8
        1 => 1,  // i8
        2 => 2,  // u16
        3 => 2,  // i16
        4 => 4,  // u32
        5 => 4,  // i32
        6 => 4,  // f32
        7 => 1,  // bool
        8 => {   // string
            let mut len_bytes = [0u8; 8];
            file.read_exact(&mut len_bytes)?;
            u64::from_le_bytes(len_bytes) as i64
        }
        9 => {   // array
            // Read array type and count
            let mut arr_type_bytes = [0u8; 4];
            let mut arr_count_bytes = [0u8; 8];
            file.read_exact(&mut arr_type_bytes)?;
            file.read_exact(&mut arr_count_bytes)?;
            let arr_type = u32::from_le_bytes(arr_type_bytes);
            let arr_count = u64::from_le_bytes(arr_count_bytes);

            // Skip array elements (simplified - just skip fixed-size types)
            let elem_size: i64 = match arr_type {
                0 | 1 | 7 => 1,
                2 | 3 => 2,
                4 | 5 | 6 => 4,
                10 | 11 => 8,
                12 => 8,
                _ => return Ok(()), // Unknown type, can't skip safely
            };
            elem_size * arr_count as i64
        }
        10 => 8, // u64
        11 => 8, // i64
        12 => 8, // f64
        _ => 0,
    };

    if skip_bytes > 0 {
        file.seek(SeekFrom::Current(skip_bytes))?;
    }

    Ok(())
}

/// Identify component type from tensor names (for safetensors)
fn identify_component_from_tensor_names(tensor_names: &[&str]) -> Option<ComponentType> {
    // Sample tensor names for pattern matching
    let sample: Vec<&str> = tensor_names.iter().take(50).copied().collect();

    // LoRA patterns - CHECK FIRST as LoRAs can contain base model layer names
    // LoRA tensors typically have: lora_up, lora_down, lora_A, lora_B, .alpha
    let has_lora_up_down = sample.iter().any(|n| n.contains("lora_up") || n.contains("lora_down"));
    let has_lora_ab = sample.iter().any(|n| n.contains("lora_A") || n.contains("lora_B") || n.contains(".lora_a.") || n.contains(".lora_b."));
    let has_lora_alpha = sample.iter().any(|n| n.ends_with(".alpha"));

    // LoRAs are typically small and have specific naming patterns
    if has_lora_up_down || has_lora_ab || has_lora_alpha {
        return Some(ComponentType::Lora);
    }

    // FLUX Transformer patterns
    let has_double_blocks = sample.iter().any(|n| n.contains("double_blocks"));
    let has_single_blocks = sample.iter().any(|n| n.contains("single_blocks"));
    let has_img_attn = sample.iter().any(|n| n.contains("img_attn") || n.contains("img_mlp"));

    if has_double_blocks || has_single_blocks || has_img_attn {
        return Some(ComponentType::Transformer);
    }

    // Also check for generic diffusion transformer patterns
    let has_transformer_blocks = sample.iter().any(|n| n.contains("transformer_blocks"));
    if has_transformer_blocks {
        return Some(ComponentType::Transformer);
    }

    // T5 Encoder patterns
    let has_encoder_block = sample.iter().any(|n| n.contains("encoder.block"));
    let has_shared_weight = sample.iter().any(|n| n.contains("shared.weight"));
    let has_t5_layer = sample.iter().any(|n| n.contains("SelfAttention") && n.contains("layer"));

    if has_encoder_block || (has_shared_weight && has_t5_layer) {
        return Some(ComponentType::T5Encoder);
    }

    // CLIP Encoder patterns
    let has_text_model = sample.iter().any(|n| n.contains("text_model.encoder"));
    let has_clip_layers = sample.iter().any(|n| n.contains("text_model.encoder.layers"));

    if has_text_model || has_clip_layers {
        return Some(ComponentType::ClipEncoder);
    }

    // VAE patterns
    let has_decoder_up = sample.iter().any(|n| n.contains("decoder.up_blocks") || n.contains("decoder.mid_block"));
    let has_encoder_down = sample.iter().any(|n| n.contains("encoder.down_blocks") || n.contains("encoder.mid_block"));
    let has_quant_conv = sample.iter().any(|n| n.contains("quant_conv") || n.contains("post_quant_conv"));

    if has_decoder_up || has_encoder_down || has_quant_conv {
        return Some(ComponentType::VaeDecoder);
    }

    None
}

/// Check if a model filename indicates an unsupported variant
/// Filters out FLUX Kontext, FLUX Fill, and other unsupported model types
fn is_unsupported_model(filename_lower: &str) -> bool {
    // FLUX Kontext - context-aware models, not standard FLUX
    filename_lower.contains("kontext") ||
    // FLUX Fill - inpainting models, different architecture
    filename_lower.contains("fill") ||
    // FLUX Canny - ControlNet variants
    filename_lower.contains("canny") ||
    // FLUX Depth - ControlNet variants
    filename_lower.contains("depth") ||
    // Redux - image prompt models
    filename_lower.contains("redux")
}

/// Create a DiscoveredComponent from a loose file (not in HF cache structure)
fn create_component_from_loose_file(
    path: &Path,
    comp_type: ComponentType,
    hash_progress_callback: Option<&HashProgressCallback>,
    skip_hash: bool,
) -> Option<DiscoveredComponent> {
    let format = ModelFormat::from_path(path)?;
    let metadata = std::fs::metadata(path).ok()?;
    let file_size = metadata.len();

    // Compute hash for deduplication (unless skipped)
    let file_hash = if skip_hash {
        None
    } else {
        compute_file_hash(path, hash_progress_callback).ok()
    };

    let quantization = extract_quantization(path);
    let vram_mb = estimate_vram(file_size, comp_type, &quantization);

    // For loose files, derive repo_id from parent directory name
    let repo_id = path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Infer architecture from filename
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let architecture = match comp_type {
        ComponentType::Transformer => {
            if filename.to_lowercase().contains("schnell") {
                "flux-schnell".to_string()
            } else if filename.to_lowercase().contains("dev") {
                "flux-dev".to_string()
            } else {
                "flux".to_string()
            }
        }
        ComponentType::T5Encoder => "t5-v1_1-xxl".to_string(),
        ComponentType::ClipEncoder => "clip-l".to_string(),
        ComponentType::VaeDecoder => "flux-vae".to_string(),
        ComponentType::ClipTokenizer => "clip-tokenizer".to_string(),
        ComponentType::T5Tokenizer => "t5-tokenizer".to_string(),
        ComponentType::Lora => "flux-lora".to_string(),
    };

    Some(DiscoveredComponent {
        component_type: comp_type,
        architecture,
        format,
        path: path.to_path_buf(),
        repo_id,
        repo_snapshot: None, // No snapshot for loose files
        file_size,
        file_hash,
        quantization,
        is_sharded: false,
        shard_count: None,
        vram_mb,
        metadata: serde_json::json!({"source": "directory_scan"}),
    })
}

/// Filter out sharded components when non-sharded alternatives exist in the same repo.
/// Sharded models are only kept when they are the only models found for that repo.
pub fn filter_sharded_components(components: Vec<DiscoveredComponent>) -> Vec<DiscoveredComponent> {
    use std::collections::HashSet;

    let repos_with_non_sharded: HashSet<String> = components.iter()
        .filter(|c| !c.is_sharded)
        .map(|c| c.repo_id.clone())
        .collect();

    components.into_iter()
        .filter(|c| !c.is_sharded || !repos_with_non_sharded.contains(&c.repo_id))
        .collect()
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
