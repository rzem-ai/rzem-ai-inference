use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tracing::warn;

/// Statistics gathered during a search pass.
#[derive(Debug, Default, Clone)]
pub struct SearchStats {
    pub items_scanned: usize,
    pub models_found: usize,
    pub models_filtered: usize,
}

/// Simple recursive model search mirroring the Python helper. Callbacks allow callers to
/// plug in filtering and progress reporting.
pub struct ModelSearch {
    on_search_started: Option<Box<dyn Fn(&Path) + Send + Sync>>, 
    on_model_found: Option<Box<dyn Fn(&Path) -> bool + Send + Sync>>, 
    on_search_completed: Option<Box<dyn Fn(&HashSet<PathBuf>) + Send + Sync>>, 
    pub stats: SearchStats,
    models_found: HashSet<PathBuf>,
    max_depth: usize,
}

impl Default for ModelSearch {
    fn default() -> Self {
        Self {
            on_search_started: None,
            on_model_found: None,
            on_search_completed: None,
            stats: SearchStats::default(),
            models_found: HashSet::new(),
            max_depth: 20,
        }
    }
}

impl ModelSearch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_on_search_started(mut self, cb: impl Fn(&Path) + Send + Sync + 'static) -> Self {
        self.on_search_started = Some(Box::new(cb));
        self
    }

    pub fn with_on_model_found(mut self, cb: impl Fn(&Path) -> bool + Send + Sync + 'static) -> Self {
        self.on_model_found = Some(Box::new(cb));
        self
    }

    pub fn with_on_search_completed(
        mut self,
        cb: impl Fn(&HashSet<PathBuf>) + Send + Sync + 'static,
    ) -> Self {
        self.on_search_completed = Some(Box::new(cb));
        self
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn search(&mut self, directory: impl AsRef<Path>) -> Result<HashSet<PathBuf>> {
        let root = directory.as_ref().canonicalize()?;
        self.stats = SearchStats::default();
        self.models_found.clear();
        if let Some(cb) = &self.on_search_started {
            cb(&root);
        }

        self.walk(&root, 0);

        if let Some(cb) = &self.on_search_completed {
            cb(&self.models_found);
        }
        Ok(self.models_found.clone())
    }

    fn walk(&mut self, path: &Path, depth: usize) {
        if depth > self.max_depth {
            return;
        }

        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(err) => {
                warn!(error = %err, "Skipping unreadable directory");
                return;
            }
        };

        let mut file_names = Vec::new();
        let mut dirs = Vec::new();

        for entry in entries.flatten() {
            self.stats.items_scanned += 1;
            let path = entry.path();
            let name = entry.file_name();
            if name.to_string_lossy().starts_with('.') {
                continue;
            }

            match entry.file_type() {
                Ok(ft) if ft.is_dir() => dirs.push(path.clone()),
                Ok(ft) if ft.is_file() => file_names.push((name.to_string_lossy().into_owned(), path.clone())),
                _ => continue,
            }
        }

        let model_marker = [
            "config.json",
            "model_index.json",
            "learned_embeds.bin",
            "pytorch_lora_weights.bin",
            "image_encoder.txt",
        ];

        if file_names.iter().any(|(n, _)| model_marker.contains(&n.as_str())) {
            self.handle_model_candidate(path);
            return;
        }

        for (name, file_path) in &file_names {
            if name.ends_with(".ckpt")
                || name.ends_with(".bin")
                || name.ends_with(".pth")
                || name.ends_with(".safetensors")
                || name.ends_with(".pt")
                || name.ends_with(".gguf")
                || name.ends_with(".onnx")
            {
                self.handle_model_candidate(file_path);
            }
        }

        for dir in dirs {
            self.walk(&dir, depth + 1);
        }
    }

    fn handle_model_candidate(&mut self, candidate: &Path) {
        self.stats.models_found += 1;
        let include = self
            .on_model_found
            .as_ref()
            .map(|cb| cb(candidate))
            .unwrap_or(true);
        if include {
            self.stats.models_filtered += 1;
            self.models_found.insert(candidate.to_path_buf());
        }
    }
}
