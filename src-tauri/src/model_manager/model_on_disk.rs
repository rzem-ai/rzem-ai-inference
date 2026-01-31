use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use blake2::{digest::Digest, Blake2b512};
use tracing::debug;

use crate::model_manager::taxonomy::ModelRepoVariant;

/// Hashing algorithms supported by the Rust port. The Python version defaults to blake3;
/// we keep both blake3 and blake2b for flexibility without pulling in heavy crypto deps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Blake3,
    Blake2b,
}

impl Default for HashAlgorithm {
    fn default() -> Self {
        HashAlgorithm::Blake3
    }
}

/// Represents a model stored on disk. Mirrors the Python helper used during model
/// identification and loading.
#[derive(Debug, Clone)]
pub struct ModelOnDisk {
    path: PathBuf,
    pub name: String,
    hash_algo: HashAlgorithm,
    metadata_cache: HashMap<PathBuf, HashMap<String, String>>,
}

impl ModelOnDisk {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self::with_hash_algo(path, HashAlgorithm::default())
    }

    pub fn with_hash_algo(path: impl Into<PathBuf>, hash_algo: HashAlgorithm) -> Self {
        let path_buf = path.into();
        let name = match path_buf.extension().and_then(|ext| ext.to_str()) {
            Some("safetensors" | "bin" | "pt" | "ckpt") => path_buf
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string(),
            _ => path_buf
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string(),
        };

        Self {
            path: path_buf,
            name,
            hash_algo,
            metadata_cache: HashMap::new(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Hash the underlying weights. Directories are hashed by walking all recognised weight files
    /// in a stable order.
    pub fn hash(&self) -> Result<String> {
        let mut files = self.weight_files()?;
        if files.is_empty() {
            files.push(self.path.clone());
        }
        files.sort();
        match self.hash_algo {
            HashAlgorithm::Blake3 => self.hash_blake3(&files),
            HashAlgorithm::Blake2b => self.hash_blake2b(&files),
        }
    }

    fn hash_blake3(&self, files: &[PathBuf]) -> Result<String> {
        let mut hasher = blake3::Hasher::new();
        for path in files {
            self.feed_file_to_hasher(path, |chunk| hasher.update(chunk))?;
        }
        Ok(hasher.finalize().to_hex().to_string())
    }

    fn hash_blake2b(&self, files: &[PathBuf]) -> Result<String> {
        let mut hasher = Blake2b512::new();
        for path in files {
            self.feed_file_to_hasher(path, |chunk| hasher.update(chunk))?;
        }
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn feed_file_to_hasher<F>(&self, path: &Path, mut update: F) -> Result<()>
    where
        F: FnMut(&[u8]),
    {
        let file = File::open(path).with_context(|| format!("Failed to open {:?}", path))?;
        let mut reader = BufReader::new(file);
        let mut buffer = [0u8; 8 * 1024];
        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            update(&buffer[..read]);
        }
        Ok(())
    }

    /// Total size in bytes across recognised weight files.
    pub fn size(&self) -> Result<u64> {
        if self.path.is_file() {
            let meta = fs::metadata(&self.path)?;
            return Ok(meta.len());
        }

        let mut total = 0u64;
        for path in self.weight_files()? {
            total = total.saturating_add(fs::metadata(path)?.len());
        }
        Ok(total)
    }

    /// Enumerate candidate weight files beneath the model path.
    pub fn weight_files(&self) -> Result<Vec<PathBuf>> {
        if self.path.is_file() {
            return Ok(vec![self.path.clone()]);
        }

        let mut files = Vec::new();
        let mut stack = vec![self.path.clone()];
        while let Some(dir) = stack.pop() {
            let entries = match fs::read_dir(&dir) {
                Ok(entries) => entries,
                Err(err) => {
                    debug!(error = %err, "Skipping unreadable directory" );
                    continue;
                }
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if entry.file_name().to_string_lossy().starts_with('.') {
                    continue;
                }
                if path.is_dir() {
                    stack.push(path);
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if matches!(ext, "safetensors" | "pt" | "pth" | "ckpt" | "bin" | "gguf" | "onnx") {
                        files.push(path);
                    }
                }
            }
        }

        Ok(files)
    }

    /// Extract safetensors metadata when available. For other formats an empty map is returned.
    pub fn metadata(&mut self, path: Option<&Path>) -> Result<HashMap<String, String>> {
        let target = path.unwrap_or(&self.path);
        if let Some(cached) = self.metadata_cache.get(target) {
            return Ok(cached.clone());
        }

        let metadata = if target.extension().and_then(|e| e.to_str()) == Some("safetensors") {
            read_safetensors_metadata(target)?
        } else {
            HashMap::new()
        };

        self.metadata_cache.insert(target.to_path_buf(), metadata.clone());
        Ok(metadata)
    }

    /// Infer the huggingface-style repo variant for diffusers directories.
    pub fn repo_variant(&self) -> Result<Option<ModelRepoVariant>> {
        if self.path.is_file() {
            return Ok(None);
        }

        let mut safes = Vec::new();
        let mut bins = Vec::new();
        for entry in fs::read_dir(&self.path)? {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    debug!(error = %err, "Skipping unreadable entry while inferring variant");
                    continue;
                }
            };
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            match path.extension().and_then(|e| e.to_str()) {
                Some("safetensors") => safes.push(path.clone()),
                Some("bin") => bins.push(path.clone()),
                Some("onnx") => return Ok(Some(ModelRepoVariant::ONNX)),
                _ => {}
            }
        }

        let mut check_variant = |paths: &[PathBuf]| -> Option<ModelRepoVariant> {
            for p in paths {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.contains(".fp16") {
                    return Some(ModelRepoVariant::FP16);
                }
                if name.contains("openvino_model") {
                    return Some(ModelRepoVariant::OpenVINO);
                }
                if name.contains("flax_model") {
                    return Some(ModelRepoVariant::Flax);
                }
            }
            None
        };

        Ok(check_variant(&safes).or_else(|| check_variant(&bins)).or(Some(ModelRepoVariant::Default)))
    }

    /// Load raw bytes of the resolved weight file. This is intentionally low-level; callers can
    /// deserialize using their preferred framework.
    pub fn load_bytes(&self, path: Option<&Path>) -> Result<Vec<u8>> {
        let resolved = self.resolve_weight_file(path)?;
        fs::read(&resolved).with_context(|| format!("Failed to read {:?}", resolved))
    }

    /// Resolve the weight file to operate on, mirroring the Python helper's semantics.
    pub fn resolve_weight_file(&self, path: Option<&Path>) -> Result<PathBuf> {
        if let Some(provided) = path {
            if !provided.exists() {
                bail!("Provided weight path {:?} does not exist", provided);
            }
            return Ok(provided.to_path_buf());
        }

        if self.path.is_file() {
            return Ok(self.path.clone());
        }

        let weights = self.weight_files()?;
        match weights.len() {
            0 => bail!("No weight files found for model at {:?}", self.path),
            1 => Ok(weights[0].clone()),
            _ => bail!(
                "Multiple weight files found for model at {:?}. Specify one explicitly.",
                self.path
            ),
        }
    }
}

fn read_safetensors_metadata(path: &Path) -> Result<HashMap<String, String>> {
    let mut file = File::open(path).with_context(|| format!("Failed to open {:?} for metadata", path))?;
    let mut header_len_buf = [0u8; 8];
    file.read_exact(&mut header_len_buf)?;
    let header_len = u64::from_le_bytes(header_len_buf) as usize;
    let mut header_bytes = vec![0u8; header_len];
    file.read_exact(&mut header_bytes)?;
    let header: serde_json::Value = serde_json::from_slice(&header_bytes)?;

    let mut metadata = HashMap::new();
    if let Some(meta_obj) = header.get("__metadata__").and_then(|v| v.as_object()) {
        for (k, v) in meta_obj {
            let value = if let Some(s) = v.as_str() {
                s.to_string()
            } else {
                v.to_string()
            };
            metadata.insert(k.clone(), value);
        }
    }

    Ok(metadata)
}
