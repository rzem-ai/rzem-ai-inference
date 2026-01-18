# Performance Optimization & FLUX Dev - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add persistent model loading, batch generation, real-time progress tracking, FLUX Dev support, and a full-featured CLI.

**Architecture:** Introduce a `ModelManager` singleton that owns all models and stays loaded between generations. Progress callbacks emit events through Tauri. CLI binary shares the library crate with the Tauri app.

**Tech Stack:** Rust, Candle, Tauri, clap (CLI), tokio, serde

---

## Task 1: Create Progress Types

**Files:**
- Create: `src-tauri/src/inference/progress.rs`
- Modify: `src-tauri/src/inference/mod.rs`

**Step 1: Create progress types file**

Create `src-tauri/src/inference/progress.rs`:

```rust
//! Progress tracking for generation pipeline

use serde::{Deserialize, Serialize};

/// Pipeline stages with their weight in overall progress
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStage {
    LoadingModels,
    EncodingT5,
    EncodingClip,
    Denoising,
    DecodingVae,
    EncodingPng,
}

impl PipelineStage {
    /// Get the start percentage for this stage (0.0-1.0)
    pub fn start_percent(&self) -> f32 {
        match self {
            Self::LoadingModels => 0.0,
            Self::EncodingT5 => 0.10,
            Self::EncodingClip => 0.20,
            Self::Denoising => 0.25,
            Self::DecodingVae => 0.85,
            Self::EncodingPng => 0.95,
        }
    }

    /// Get the end percentage for this stage (0.0-1.0)
    pub fn end_percent(&self) -> f32 {
        match self {
            Self::LoadingModels => 0.10,
            Self::EncodingT5 => 0.20,
            Self::EncodingClip => 0.25,
            Self::Denoising => 0.85,
            Self::DecodingVae => 0.95,
            Self::EncodingPng => 1.0,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::LoadingModels => "Loading models",
            Self::EncodingT5 => "Encoding prompt (T5)",
            Self::EncodingClip => "Encoding prompt (CLIP)",
            Self::Denoising => "Denoising",
            Self::DecodingVae => "Decoding image",
            Self::EncodingPng => "Saving image",
        }
    }
}

/// Progress update during generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationProgress {
    /// Current pipeline stage
    pub stage: PipelineStage,
    /// Progress within current stage (0.0-1.0)
    pub stage_progress: f32,
    /// Overall generation progress (0.0-1.0)
    pub overall_progress: f32,
    /// Human-readable status message
    pub message: String,
    /// Estimated seconds remaining
    pub eta_seconds: Option<f32>,
    /// For batch jobs: current image index (1-indexed)
    pub batch_index: Option<usize>,
    /// For batch jobs: total image count
    pub batch_total: Option<usize>,
}

impl GenerationProgress {
    /// Create a new progress update
    pub fn new(stage: PipelineStage, stage_progress: f32) -> Self {
        let stage_start = stage.start_percent();
        let stage_end = stage.end_percent();
        let stage_range = stage_end - stage_start;
        let overall = stage_start + (stage_range * stage_progress.clamp(0.0, 1.0));

        Self {
            stage,
            stage_progress: stage_progress.clamp(0.0, 1.0),
            overall_progress: overall,
            message: stage.display_name().to_string(),
            eta_seconds: None,
            batch_index: None,
            batch_total: None,
        }
    }

    /// Create progress for a denoising step
    pub fn denoising_step(current_step: usize, total_steps: usize) -> Self {
        let stage_progress = current_step as f32 / total_steps as f32;
        let mut progress = Self::new(PipelineStage::Denoising, stage_progress);
        progress.message = format!("Denoising step {}/{}", current_step, total_steps);
        progress
    }

    /// Set batch information
    pub fn with_batch(mut self, index: usize, total: usize) -> Self {
        self.batch_index = Some(index);
        self.batch_total = Some(total);
        if total > 1 {
            self.message = format!("Image {}/{}: {}", index, total, self.message);
        }
        self
    }

    /// Set ETA
    pub fn with_eta(mut self, seconds: f32) -> Self {
        self.eta_seconds = Some(seconds);
        self
    }
}

/// Callback type for progress updates
pub type ProgressCallback = Box<dyn Fn(GenerationProgress) + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_percentages() {
        assert_eq!(PipelineStage::LoadingModels.start_percent(), 0.0);
        assert_eq!(PipelineStage::EncodingPng.end_percent(), 1.0);
    }

    #[test]
    fn test_progress_calculation() {
        let progress = GenerationProgress::new(PipelineStage::Denoising, 0.5);
        // Denoising is 25-85%, so 50% through = 25 + (60 * 0.5) = 55%
        assert!((progress.overall_progress - 0.55).abs() < 0.01);
    }

    #[test]
    fn test_denoising_step() {
        let progress = GenerationProgress::denoising_step(2, 4);
        assert_eq!(progress.message, "Denoising step 2/4");
        assert!((progress.stage_progress - 0.5).abs() < 0.01);
    }
}
```

**Step 2: Export progress module**

Add to `src-tauri/src/inference/mod.rs` after other module declarations:

```rust
mod progress;

pub use progress::{GenerationProgress, PipelineStage, ProgressCallback};
```

**Step 3: Run tests**

Run: `cd src-tauri && cargo test progress --lib`
Expected: All 3 tests pass

**Step 4: Commit**

```bash
git add src-tauri/src/inference/progress.rs src-tauri/src/inference/mod.rs
git commit -m "feat: add progress tracking types for generation pipeline"
```

---

## Task 2: Create ModelType Enum

**Files:**
- Create: `src-tauri/src/models/model_type.rs`
- Modify: `src-tauri/src/models/mod.rs`

**Step 1: Create model type file**

Create `src-tauri/src/models/model_type.rs`:

```rust
//! Model type definitions for FLUX variants

use serde::{Deserialize, Serialize};

/// Available FLUX model types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    /// FLUX.1 [schnell] - Fast, 4 steps
    Schnell,
    /// FLUX.1 [dev] - Higher quality, 28+ steps
    Dev,
}

impl ModelType {
    /// Default number of denoising steps
    pub fn default_steps(&self) -> usize {
        match self {
            Self::Schnell => 4,
            Self::Dev => 28,
        }
    }

    /// Default guidance scale
    pub fn default_guidance(&self) -> f64 {
        match self {
            Self::Schnell => 4.0,
            Self::Dev => 3.5,
        }
    }

    /// Valid step range (min, max)
    pub fn step_range(&self) -> (usize, usize) {
        match self {
            Self::Schnell => (1, 8),
            Self::Dev => (20, 100),
        }
    }

    /// Approximate VRAM usage in MB (full precision)
    pub fn vram_full_precision(&self) -> usize {
        match self {
            Self::Schnell => 23_000,
            Self::Dev => 24_000,
        }
    }

    /// Approximate VRAM usage in MB (quantized)
    pub fn vram_quantized(&self) -> usize {
        match self {
            Self::Schnell => 12_000,
            Self::Dev => 12_000,
        }
    }

    /// HuggingFace repository ID
    pub fn repo_id(&self) -> &'static str {
        match self {
            Self::Schnell => "black-forest-labs/FLUX.1-schnell",
            Self::Dev => "black-forest-labs/FLUX.1-dev",
        }
    }

    /// Transformer filename
    pub fn transformer_filename(&self) -> &'static str {
        match self {
            Self::Schnell => "flux1-schnell.safetensors",
            Self::Dev => "flux1-dev.safetensors",
        }
    }

    /// Quantized transformer filename
    pub fn quantized_filename(&self) -> &'static str {
        match self {
            Self::Schnell => "flux1-schnell.gguf",
            Self::Dev => "flux1-dev.gguf",
        }
    }

    /// Display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Schnell => "FLUX.1 [schnell]",
            Self::Dev => "FLUX.1 [dev]",
        }
    }
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for ModelType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "schnell" | "flux-schnell" | "flux.1-schnell" => Ok(Self::Schnell),
            "dev" | "flux-dev" | "flux.1-dev" => Ok(Self::Dev),
            _ => Err(format!("Unknown model type: {}. Use 'schnell' or 'dev'", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_steps() {
        assert_eq!(ModelType::Schnell.default_steps(), 4);
        assert_eq!(ModelType::Dev.default_steps(), 28);
    }

    #[test]
    fn test_parse() {
        assert_eq!("schnell".parse::<ModelType>().unwrap(), ModelType::Schnell);
        assert_eq!("dev".parse::<ModelType>().unwrap(), ModelType::Dev);
        assert_eq!("SCHNELL".parse::<ModelType>().unwrap(), ModelType::Schnell);
    }

    #[test]
    fn test_vram() {
        assert!(ModelType::Dev.vram_full_precision() > ModelType::Dev.vram_quantized());
    }
}
```

**Step 2: Export model type**

Add to `src-tauri/src/models/mod.rs`:

```rust
mod model_type;

pub use model_type::ModelType;
```

**Step 3: Run tests**

Run: `cd src-tauri && cargo test model_type --lib`
Expected: All 3 tests pass

**Step 4: Commit**

```bash
git add src-tauri/src/models/model_type.rs src-tauri/src/models/mod.rs
git commit -m "feat: add ModelType enum for Schnell and Dev variants"
```

---

## Task 3: Extend ModelPaths for Dev

**Files:**
- Modify: `src-tauri/src/models/paths.rs`

**Step 1: Add Dev path methods**

Add these methods to `impl ModelPaths` in `src-tauri/src/models/paths.rs`:

```rust
    /// Get the FLUX Dev model directory
    pub fn dev_dir(&self) -> PathBuf {
        self.cache_dir.join("models--black-forest-labs--FLUX.1-dev")
    }

    /// Get snapshot hash for Dev model
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

    /// Get path to FLUX Dev transformer
    pub fn dev_transformer_path(&self) -> PathBuf {
        self.get_dev_snapshot_hash()
            .map(|hash| self.dev_dir().join("snapshots").join(hash).join("flux1-dev.safetensors"))
            .unwrap_or_else(|_| self.dev_dir().join("snapshots").join("main").join("flux1-dev.safetensors"))
    }

    /// Get path to quantized FLUX Dev transformer
    pub fn quantized_dev_transformer_path(&self) -> PathBuf {
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

        lmz_dir.join("snapshots").join("main").join("flux1-dev.gguf")
    }

    /// Check if Dev model is downloaded
    pub fn is_dev_downloaded(&self) -> bool {
        self.dev_transformer_path().exists() || self.quantized_dev_transformer_path().exists()
    }

    /// Check if quantized Dev transformer is available
    pub fn has_quantized_dev(&self) -> bool {
        self.quantized_dev_transformer_path().exists()
    }

    /// Get transformer path for a given model type
    pub fn transformer_path_for(&self, model_type: super::ModelType) -> PathBuf {
        match model_type {
            super::ModelType::Schnell => self.transformer_path(),
            super::ModelType::Dev => self.dev_transformer_path(),
        }
    }

    /// Get quantized transformer path for a given model type
    pub fn quantized_transformer_path_for(&self, model_type: super::ModelType) -> PathBuf {
        match model_type {
            super::ModelType::Schnell => self.quantized_transformer_path(),
            super::ModelType::Dev => self.quantized_dev_transformer_path(),
        }
    }

    /// Check if quantized version exists for model type
    pub fn has_quantized_for(&self, model_type: super::ModelType) -> bool {
        match model_type {
            super::ModelType::Schnell => self.has_quantized_transformer(),
            super::ModelType::Dev => self.has_quantized_dev(),
        }
    }
```

**Step 2: Update get_status to include Dev**

Find the `get_status` method and add Dev entries:

```rust
    pub fn get_status(&self) -> Vec<(String, bool, String)> {
        let clip_path = self.clip_path().join("model.safetensors");
        let vae_path = self.vae_path();
        let transformer_path = self.transformer_path();
        let quantized_transformer_path = self.quantized_transformer_path();
        let dev_transformer_path = self.dev_transformer_path();
        let quantized_dev_path = self.quantized_dev_transformer_path();
        let t5_path = self.t5_path();
        let t5_model_path = t5_path.join("model-00001-of-00002.safetensors");
        let t5_config_path = t5_path.join("config.json");
        let quantized_t5_path = self.quantized_t5_path();
        let t5_tokenizer_path = self.t5_tokenizer_path();

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
```

**Step 3: Run tests**

Run: `cd src-tauri && cargo test paths --lib`
Expected: Tests pass

**Step 4: Commit**

```bash
git add src-tauri/src/models/paths.rs
git commit -m "feat: add FLUX Dev model paths and status checking"
```

---

## Task 4: Create Model Manager

**Files:**
- Create: `src-tauri/src/models/manager.rs`
- Modify: `src-tauri/src/models/mod.rs`

**Step 1: Create model manager**

Create `src-tauri/src/models/manager.rs`:

```rust
//! Persistent model manager for efficient generation

use anyhow::Result;
use candle_core::Device;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{ClipTextEncoder, FluxTransformer, ModelPaths, ModelType, T5TextEncoder, VaeDecoder};

/// Precision level for model loading
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precision {
    Full,
    Quantized,
}

/// Manages model lifecycle and VRAM
pub struct ModelManager {
    device: Device,
    paths: ModelPaths,

    // Shared components (kept loaded)
    t5: Option<T5TextEncoder>,
    clip: Option<ClipTextEncoder>,
    vae: Option<VaeDecoder>,

    // Model-specific transformer (swapped between Schnell/Dev)
    flux: Option<FluxTransformer>,
    current_model: Option<ModelType>,
    current_precision: Option<Precision>,
}

impl ModelManager {
    /// Create a new model manager
    pub fn new(device: Device) -> Result<Self> {
        let paths = ModelPaths::new()?;

        Ok(Self {
            device,
            paths,
            t5: None,
            clip: None,
            vae: None,
            flux: None,
            current_model: None,
            current_precision: None,
        })
    }

    /// Get the device
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get model paths
    pub fn paths(&self) -> &ModelPaths {
        &self.paths
    }

    /// Check if shared components are loaded
    pub fn shared_loaded(&self) -> bool {
        self.t5.is_some() && self.clip.is_some() && self.vae.is_some()
    }

    /// Get current model type
    pub fn current_model(&self) -> Option<ModelType> {
        self.current_model
    }

    /// Check available VRAM in MB (returns total if can't query)
    pub fn available_vram_mb(&self) -> usize {
        #[cfg(feature = "cuda")]
        {
            if self.device.is_cuda() {
                // Try to get free memory from CUDA
                // For now, estimate based on what's loaded
                let total = 32_000; // Assume 32GB for RTX 5090
                let used = self.estimate_loaded_vram();
                return total.saturating_sub(used);
            }
        }
        // CPU or Metal - return large number
        64_000
    }

    /// Estimate VRAM used by loaded models
    fn estimate_loaded_vram(&self) -> usize {
        let mut used = 0;
        if self.t5.is_some() {
            used += 9_000; // T5 ~9GB
        }
        if self.clip.is_some() {
            used += 250; // CLIP ~250MB
        }
        if self.vae.is_some() {
            used += 160; // VAE ~160MB
        }
        if let (Some(model), Some(precision)) = (self.current_model, self.current_precision) {
            used += match precision {
                Precision::Full => model.vram_full_precision(),
                Precision::Quantized => model.vram_quantized(),
            };
        }
        used
    }

    /// Select precision based on available VRAM
    pub fn select_precision(&self, model: ModelType) -> Precision {
        let available = self.available_vram_mb();
        let full_requirement = model.vram_full_precision();

        // Need 2GB headroom
        if available > full_requirement + 2_000 {
            Precision::Full
        } else {
            Precision::Quantized
        }
    }

    /// Load shared components (T5, CLIP, VAE)
    pub fn load_shared(&mut self) -> Result<()> {
        if self.shared_loaded() {
            return Ok(());
        }

        if !self.paths.all_files_exist() {
            anyhow::bail!("Required model files not downloaded");
        }

        println!("Loading shared components...");

        if self.t5.is_none() {
            println!("  Loading T5 encoder...");
            self.t5 = Some(T5TextEncoder::load(
                self.paths.t5_path(),
                self.paths.t5_tokenizer_path(),
                self.device.clone(),
            )?);
        }

        if self.clip.is_none() {
            println!("  Loading CLIP encoder...");
            self.clip = Some(ClipTextEncoder::load(
                self.paths.clip_path().join("model.safetensors"),
                self.paths.tokenizer_path(),
                self.device.clone(),
            )?);
        }

        if self.vae.is_none() {
            println!("  Loading VAE decoder...");
            self.vae = Some(VaeDecoder::load(
                self.paths.vae_path(),
                self.device.clone(),
            )?);
        }

        Ok(())
    }

    /// Load or switch to a specific model
    pub fn load_model(&mut self, model: ModelType) -> Result<()> {
        // Load shared components first
        self.load_shared()?;

        // Check if already loaded
        if self.current_model == Some(model) && self.flux.is_some() {
            return Ok(());
        }

        // Unload current transformer if different model
        if self.current_model.is_some() && self.current_model != Some(model) {
            println!("  Unloading {} transformer...", self.current_model.unwrap());
            self.flux = None;
            self.current_model = None;
            self.current_precision = None;
        }

        // Determine precision
        let precision = self.select_precision(model);
        let use_quantized = precision == Precision::Quantized;

        println!(
            "  Loading {} transformer ({})...",
            model,
            if use_quantized { "quantized" } else { "full precision" }
        );

        // Load transformer
        let flux = if use_quantized && self.paths.has_quantized_for(model) {
            FluxTransformer::load_quantized(
                self.paths.quantized_transformer_path_for(model),
                self.device.clone(),
            )?
        } else {
            FluxTransformer::load(
                self.paths.transformer_path_for(model),
                self.device.clone(),
            )?
        };

        self.flux = Some(flux);
        self.current_model = Some(model);
        self.current_precision = Some(precision);

        println!("  {} loaded successfully!", model);
        Ok(())
    }

    /// Unload transformer to free VRAM (keeps shared components)
    pub fn unload_transformer(&mut self) {
        self.flux = None;
        self.current_model = None;
        self.current_precision = None;
    }

    /// Unload everything
    pub fn unload_all(&mut self) {
        self.t5 = None;
        self.clip = None;
        self.vae = None;
        self.flux = None;
        self.current_model = None;
        self.current_precision = None;
    }

    /// Get T5 encoder reference
    pub fn t5(&self) -> Option<&T5TextEncoder> {
        self.t5.as_ref()
    }

    /// Get mutable T5 encoder reference
    pub fn t5_mut(&mut self) -> Option<&mut T5TextEncoder> {
        self.t5.as_mut()
    }

    /// Get CLIP encoder reference
    pub fn clip(&self) -> Option<&ClipTextEncoder> {
        self.clip.as_ref()
    }

    /// Get VAE decoder reference
    pub fn vae(&self) -> Option<&VaeDecoder> {
        self.vae.as_ref()
    }

    /// Get FLUX transformer reference
    pub fn flux(&self) -> Option<&FluxTransformer> {
        self.flux.as_ref()
    }
}

/// Thread-safe shared model manager
pub type SharedModelManager = Arc<RwLock<ModelManager>>;

/// Create a shared model manager
pub fn create_shared_manager(device: Device) -> Result<SharedModelManager> {
    Ok(Arc::new(RwLock::new(ModelManager::new(device)?)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let device = Device::Cpu;
        let manager = ModelManager::new(device).unwrap();
        assert!(!manager.shared_loaded());
        assert!(manager.current_model().is_none());
    }

    #[test]
    fn test_precision_selection() {
        let device = Device::Cpu;
        let manager = ModelManager::new(device).unwrap();
        // On CPU, should always have "enough" VRAM
        let precision = manager.select_precision(ModelType::Schnell);
        assert_eq!(precision, Precision::Full);
    }
}
```

**Step 2: Export model manager**

Add to `src-tauri/src/models/mod.rs`:

```rust
mod manager;

pub use manager::{create_shared_manager, ModelManager, Precision, SharedModelManager};
```

**Step 3: Run tests**

Run: `cd src-tauri && cargo test manager --lib`
Expected: Tests pass

**Step 4: Commit**

```bash
git add src-tauri/src/models/manager.rs src-tauri/src/models/mod.rs
git commit -m "feat: add ModelManager for persistent model loading"
```

---

## Task 5: Create CLI Binary Structure

**Files:**
- Create: `src-tauri/src/bin/flux-cli.rs`
- Modify: `src-tauri/Cargo.toml`

**Step 1: Add clap dependency and binary target**

Add to `src-tauri/Cargo.toml` in `[dependencies]`:

```toml
clap = { version = "4.4", features = ["derive"] }
```

Add at end of file:

```toml
[[bin]]
name = "flux-cli"
path = "src/bin/flux-cli.rs"
```

**Step 2: Create CLI binary**

Create `src-tauri/src/bin/flux-cli.rs`:

```rust
//! FLUX Generator CLI

use anyhow::Result;
use clap::{Parser, Subcommand};
use flux_generator_lib::models::{ModelPaths, ModelType};

#[derive(Parser)]
#[command(name = "flux-cli")]
#[command(about = "FLUX image generation CLI", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate images from text prompts
    Generate {
        /// Text prompt describing the image
        #[arg(short, long)]
        prompt: String,

        /// Output file path
        #[arg(short, long, default_value = "output.png")]
        output: String,

        /// Model to use (schnell or dev)
        #[arg(short, long, default_value = "schnell")]
        model: String,

        /// Number of denoising steps
        #[arg(short, long)]
        steps: Option<usize>,

        /// Image width
        #[arg(short = 'W', long, default_value = "1024")]
        width: usize,

        /// Image height
        #[arg(short = 'H', long, default_value = "1024")]
        height: usize,

        /// Random seed (-1 for random)
        #[arg(long, default_value = "-1")]
        seed: i64,

        /// Number of images to generate
        #[arg(short, long, default_value = "1")]
        batch: usize,

        /// Guidance scale
        #[arg(short, long)]
        guidance: Option<f64>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Suppress progress output
        #[arg(short, long)]
        quiet: bool,
    },

    /// Manage models
    Models {
        #[command(subcommand)]
        action: ModelCommands,
    },

    /// Show system information
    Info,
}

#[derive(Subcommand)]
enum ModelCommands {
    /// List available models
    List,

    /// Download a model
    Download {
        /// Model to download (schnell or dev)
        model: String,
    },

    /// Show model details
    #[command(name = "info")]
    ModelInfo {
        /// Model to show info for
        model: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            prompt,
            output,
            model,
            steps,
            width,
            height,
            seed,
            batch,
            guidance,
            json,
            quiet,
        } => {
            cmd_generate(prompt, output, model, steps, width, height, seed, batch, guidance, json, quiet)
        }
        Commands::Models { action } => match action {
            ModelCommands::List => cmd_models_list(),
            ModelCommands::Download { model } => cmd_models_download(model),
            ModelCommands::ModelInfo { model } => cmd_models_info(model),
        },
        Commands::Info => cmd_info(),
    }
}

fn cmd_generate(
    prompt: String,
    output: String,
    model: String,
    steps: Option<usize>,
    width: usize,
    height: usize,
    seed: i64,
    batch: usize,
    guidance: Option<f64>,
    json: bool,
    quiet: bool,
) -> Result<()> {
    let model_type: ModelType = model.parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    let steps = steps.unwrap_or_else(|| model_type.default_steps());
    let guidance = guidance.unwrap_or_else(|| model_type.default_guidance());

    if !quiet && !json {
        println!("Generating image...");
        println!("  Prompt: {}", prompt);
        println!("  Model: {}", model_type);
        println!("  Steps: {}", steps);
        println!("  Size: {}x{}", width, height);
        println!("  Seed: {}", if seed == -1 { "random".to_string() } else { seed.to_string() });
        println!("  Batch: {}", batch);
        println!("  Guidance: {}", guidance);
    }

    // TODO: Implement actual generation in Task 7
    if json {
        println!(r#"{{"success": false, "error": "Generation not yet implemented"}}"#);
    } else if !quiet {
        println!("Generation not yet implemented - CLI structure ready");
    }

    Ok(())
}

fn cmd_models_list() -> Result<()> {
    let paths = ModelPaths::new()?;

    println!("MODEL          STATUS           SIZE");
    println!("─────────────────────────────────────────");

    // Schnell
    let schnell_status = if paths.all_files_exist() {
        "downloaded"
    } else {
        "not downloaded"
    };
    println!("schnell        {:16} ~23 GB", schnell_status);

    // Dev
    let dev_status = if paths.is_dev_downloaded() {
        "downloaded"
    } else {
        "not downloaded"
    };
    println!("dev            {:16} ~24 GB", dev_status);

    Ok(())
}

fn cmd_models_download(model: String) -> Result<()> {
    let model_type: ModelType = model.parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    println!("Downloading {}...", model_type);
    println!("(Download not yet implemented)");

    Ok(())
}

fn cmd_models_info(model: String) -> Result<()> {
    let model_type: ModelType = model.parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    println!("{}", model_type.display_name());
    println!("─────────────────────────────────────────");
    println!("Default steps:    {}", model_type.default_steps());
    println!("Step range:       {}-{}", model_type.step_range().0, model_type.step_range().1);
    println!("Default guidance: {}", model_type.default_guidance());
    println!("VRAM (full):      {} GB", model_type.vram_full_precision() / 1000);
    println!("VRAM (quantized): {} GB", model_type.vram_quantized() / 1000);
    println!("Repository:       {}", model_type.repo_id());

    Ok(())
}

fn cmd_info() -> Result<()> {
    use flux_generator_lib::inference::InferenceEngine;

    println!("FLUX Generator CLI");
    println!("─────────────────────────────────────────");

    let engine = InferenceEngine::new()?;
    let device = engine.get_device();

    let device_name = if device.is_cuda() {
        "CUDA GPU"
    } else if device.is_metal() {
        "Metal GPU"
    } else {
        "CPU"
    };

    println!("Device: {}", device_name);

    let paths = ModelPaths::new()?;
    println!("Cache:  {}", paths.cache_dir.display());

    println!("\nModel Status:");
    for (name, exists, _path) in paths.get_status() {
        let status = if exists { "✓" } else { "✗" };
        println!("  {} {}", status, name);
    }

    Ok(())
}
```

**Step 3: Verify CLI builds**

Run: `cd src-tauri && cargo build --bin flux-cli`
Expected: Compiles successfully

**Step 4: Test CLI help**

Run: `cd src-tauri && cargo run --bin flux-cli -- --help`
Expected: Shows help text

**Step 5: Test CLI commands**

Run: `cd src-tauri && cargo run --bin flux-cli -- info`
Expected: Shows system info

Run: `cd src-tauri && cargo run --bin flux-cli -- models list`
Expected: Shows model list

**Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/bin/flux-cli.rs
git commit -m "feat: add flux-cli binary with basic commands

- generate command (structure only)
- models list/download/info commands
- info command showing GPU and model status"
```

---

## Task 6: Add Progress Callbacks to Pipeline

**Files:**
- Modify: `src-tauri/src/inference/pipeline.rs`

**Step 1: Add generate_with_progress method**

Add this method to `impl FluxPipeline` in `src-tauri/src/inference/pipeline.rs`:

```rust
    /// Generate image with progress callbacks
    pub fn generate_with_progress<F>(
        &mut self,
        prompt: &str,
        steps: usize,
        width: usize,
        height: usize,
        guidance: f64,
        on_progress: F,
    ) -> Result<GenerationResult>
    where
        F: Fn(super::GenerationProgress),
    {
        use super::{GenerationProgress, PipelineStage};

        let total_timer = Timer::start();
        let mut stats = GenerationStats::default();
        stats.steps = steps;

        // Loading stage
        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 0.0));
        self.ensure_models_loaded(&mut stats)?;
        on_progress(GenerationProgress::new(PipelineStage::LoadingModels, 1.0));

        // T5 encoding
        on_progress(GenerationProgress::new(PipelineStage::EncodingT5, 0.0));
        let t5 = self.t5.as_mut()
            .ok_or_else(|| anyhow::anyhow!("T5 model not loaded"))?;

        let t5_timer = Timer::start();
        let t5_emb = t5.encode(prompt)?;
        stats.t5_encode_ms = t5_timer.stop();
        stats.t5_embedding_shape = t5_emb.dims().to_vec();
        on_progress(GenerationProgress::new(PipelineStage::EncodingT5, 1.0));

        // CLIP encoding
        on_progress(GenerationProgress::new(PipelineStage::EncodingClip, 0.0));
        let clip = self.clip.as_ref()
            .ok_or_else(|| anyhow::anyhow!("CLIP model not loaded"))?;

        let clip_timer = Timer::start();
        let clip_emb = clip.encode(prompt)?;
        stats.clip_encode_ms = clip_timer.stop();
        stats.clip_embedding_shape = clip_emb.dims().to_vec();
        on_progress(GenerationProgress::new(PipelineStage::EncodingClip, 1.0));

        // Unload T5
        self.t5 = None;

        // Denoising with per-step progress
        let flux = self.flux.as_ref()
            .ok_or_else(|| anyhow::anyhow!("FLUX model not loaded"))?;

        stats.model_type = if flux.is_quantized() {
            "quantized".to_string()
        } else {
            "full_precision".to_string()
        };

        // Initial denoising progress
        on_progress(GenerationProgress::denoising_step(0, steps));

        let denoise_timer = Timer::start();
        let latents = flux.denoise(&t5_emb, &clip_emb, height, width, steps, guidance)?;
        stats.denoise_ms = denoise_timer.stop();
        stats.latent_shape = latents.dims().to_vec();

        // Final denoising progress
        on_progress(GenerationProgress::denoising_step(steps, steps));

        // Unload FLUX
        drop(t5_emb);
        drop(clip_emb);
        self.flux = None;
        self.clip = None;

        // VAE decoding
        on_progress(GenerationProgress::new(PipelineStage::DecodingVae, 0.0));
        let vae = self.vae.as_ref()
            .ok_or_else(|| anyhow::anyhow!("VAE model not loaded"))?;

        let vae_timer = Timer::start();
        let image = vae.decode(&latents)?;
        stats.vae_decode_ms = vae_timer.stop();
        stats.image_shape = image.dims().to_vec();
        on_progress(GenerationProgress::new(PipelineStage::DecodingVae, 1.0));

        // PNG encoding
        on_progress(GenerationProgress::new(PipelineStage::EncodingPng, 0.0));
        let png_timer = Timer::start();
        let rgb_data = vae.tensor_to_rgb(&image)?;

        let img = image::RgbImage::from_raw(width as u32, height as u32, rgb_data)
            .ok_or_else(|| anyhow::anyhow!("Failed to create image from raw data"))?;

        let mut png_data = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut png_data),
            image::ImageFormat::Png
        )?;
        stats.png_encode_ms = png_timer.stop();
        on_progress(GenerationProgress::new(PipelineStage::EncodingPng, 1.0));

        stats.total_ms = total_timer.stop();

        Ok(GenerationResult {
            image_data: png_data,
            stats,
        })
    }
```

**Step 2: Run tests**

Run: `cd src-tauri && cargo test pipeline --lib`
Expected: Tests pass

**Step 3: Commit**

```bash
git add src-tauri/src/inference/pipeline.rs
git commit -m "feat: add generate_with_progress for real-time progress tracking"
```

---

## Task 7: Implement CLI Generation

**Files:**
- Modify: `src-tauri/src/bin/flux-cli.rs`

**Step 1: Implement actual generation in cmd_generate**

Replace the `cmd_generate` function in `src-tauri/src/bin/flux-cli.rs`:

```rust
fn cmd_generate(
    prompt: String,
    output: String,
    model: String,
    steps: Option<usize>,
    width: usize,
    height: usize,
    seed: i64,
    batch: usize,
    guidance: Option<f64>,
    json: bool,
    quiet: bool,
) -> Result<()> {
    use flux_generator_lib::inference::{InferenceEngine, FluxPipeline, GenerationProgress};
    use std::time::Instant;

    let model_type: ModelType = model.parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    let steps = steps.unwrap_or_else(|| model_type.default_steps());
    let guidance = guidance.unwrap_or_else(|| model_type.default_guidance());

    // Determine actual seed
    let actual_seed = if seed == -1 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    } else {
        seed
    };

    if !quiet && !json {
        println!("Generating image...");
        println!("  Prompt: {}", prompt);
        println!("  Model: {}", model_type);
        println!("  Steps: {}", steps);
        println!("  Size: {}x{}", width, height);
        println!("  Seed: {}", actual_seed);
        println!("  Guidance: {}", guidance);
        println!();
    }

    let start = Instant::now();

    // Initialize engine and pipeline
    let engine = InferenceEngine::new()?;
    let device = engine.get_device().clone();
    let mut pipeline = FluxPipeline::new(device)?;

    // Generate with progress callback
    let result = pipeline.generate_with_progress(
        &prompt,
        steps,
        width,
        height,
        guidance,
        |progress: GenerationProgress| {
            if !quiet && !json {
                let pct = (progress.overall_progress * 100.0) as u32;
                print!("\r[{:3}%] {}", pct, progress.message);
                use std::io::Write;
                std::io::stdout().flush().ok();
            }
        },
    )?;

    if !quiet && !json {
        println!("\r[100%] Complete!                    ");
    }

    // Save image
    std::fs::write(&output, &result.image_data)?;

    let elapsed = start.elapsed();

    if json {
        let output_json = serde_json::json!({
            "success": true,
            "output_path": std::fs::canonicalize(&output)?.to_string_lossy(),
            "seed": actual_seed,
            "model": model_type.to_string(),
            "steps": steps,
            "width": width,
            "height": height,
            "generation_time_ms": elapsed.as_millis(),
            "stats": {
                "t5_encode_ms": result.stats.t5_encode_ms,
                "clip_encode_ms": result.stats.clip_encode_ms,
                "denoise_ms": result.stats.denoise_ms,
                "vae_decode_ms": result.stats.vae_decode_ms,
                "total_ms": result.stats.total_ms,
            }
        });
        println!("{}", serde_json::to_string_pretty(&output_json)?);
    } else if !quiet {
        println!("\nSaved to: {}", output);
        println!("Time: {:.2}s", elapsed.as_secs_f64());
    }

    Ok(())
}
```

**Step 2: Add serde_json to imports at top of file**

Add near the top of the file after other imports:

```rust
use serde_json;
```

**Step 3: Build and test CLI**

Run: `cd src-tauri && cargo build --bin flux-cli --release`
Expected: Compiles

Run: `cd src-tauri && cargo run --release --bin flux-cli -- generate -p "a cat" -o /tmp/test_cli.png`
Expected: Generates an image (may take ~13 seconds)

**Step 4: Verify output**

Run: `file /tmp/test_cli.png`
Expected: Shows PNG image info

**Step 5: Test JSON output**

Run: `cd src-tauri && cargo run --release --bin flux-cli -- generate -p "a dog" -o /tmp/test_json.png --json`
Expected: JSON output with stats

**Step 6: Commit**

```bash
git add src-tauri/src/bin/flux-cli.rs
git commit -m "feat: implement CLI image generation with progress

- Real-time progress output
- JSON mode for scripting
- Saves to specified output path"
```

---

## Task 8: Integrate Progress with Tauri Events

**Files:**
- Modify: `src-tauri/src/queue/processor.rs`

**Step 1: Update execute_generation to emit progress events**

Find the `execute_generation` function in `src-tauri/src/queue/processor.rs` and update it to use progress callbacks:

```rust
async fn execute_generation(
    job_id: &str,
    params: &super::GenerationParams,
    queue_manager: &Arc<QueueManager>,
    inference_engine: &Arc<InferenceEngine>,
    app_handle: &tauri::AppHandle,
) -> Result<String> {
    use crate::inference::{FluxPipeline, GenerationProgress};

    let device = inference_engine.get_device().clone();
    let mut pipeline = FluxPipeline::new(device)?;

    let job_id_clone = job_id.to_string();
    let app_handle_clone = app_handle.clone();

    // Generate with progress callback
    let result = pipeline.generate_with_progress(
        &params.prompt,
        params.steps as usize,
        params.width as usize,
        params.height as usize,
        params.guidance.unwrap_or(4.0),
        move |progress: GenerationProgress| {
            // Emit progress event to frontend
            let _ = app_handle_clone.emit("generation-progress", serde_json::json!({
                "job_id": job_id_clone,
                "stage": progress.stage,
                "stage_progress": progress.stage_progress,
                "overall_progress": progress.overall_progress,
                "message": progress.message,
                "eta_seconds": progress.eta_seconds,
            }));
        },
    )?;

    // Save to file
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let output_dir = home.join(".flux-generator").join("outputs");
    std::fs::create_dir_all(&output_dir)?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let filename = format!("flux_{}_{}.png", timestamp, params.seed);
    let output_path = output_dir.join(&filename);

    std::fs::write(&output_path, &result.image_data)?;

    // Update final progress
    queue_manager.update_job_progress(job_id, 1.0).await
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(output_path.to_string_lossy().to_string())
}
```

**Step 2: Update function signature if needed**

If the function doesn't have `app_handle` parameter, add it and update the call site in `process_jobs`.

**Step 3: Test compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles

**Step 4: Commit**

```bash
git add src-tauri/src/queue/processor.rs
git commit -m "feat: emit generation progress events to frontend"
```

---

## Task 9: Add Dev Model Download Support

**Files:**
- Modify: `src-tauri/src/models/downloader.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add download_dev method to ModelDownloader**

Add to `impl ModelDownloader` in `src-tauri/src/models/downloader.rs`:

```rust
    /// Check if FLUX Dev is already downloaded
    pub fn is_dev_downloaded(&self) -> bool {
        self.paths.is_dev_downloaded()
    }

    /// Download FLUX Dev model from HuggingFace Hub
    pub async fn download_dev(&self) -> Result<()> {
        if self.is_dev_downloaded() {
            println!("FLUX Dev already downloaded");
            return Ok(());
        }

        println!("Downloading FLUX Dev from HuggingFace Hub...");
        println!("Note: FLUX Dev requires authentication for non-commercial use");

        // Dev only needs the transformer - shared components come from Schnell
        let api = hf_hub::api::tokio::Api::new()?;
        let repo = api.model("black-forest-labs/FLUX.1-dev".to_string());

        println!("Downloading flux1-dev.safetensors (~24GB)...");
        repo.get("flux1-dev.safetensors").await?;

        println!("FLUX Dev download complete!");
        Ok(())
    }
```

**Step 2: Add Tauri command for Dev download**

Add to `src-tauri/src/lib.rs` after `download_flux_schnell`:

```rust
#[command]
async fn download_flux_dev(app_state: State<'_, AppState>) -> Result<String, String> {
    use crate::models::ModelDownloader;

    // Check if download is already in progress
    {
        let mut in_progress = app_state.download_in_progress.lock().await;
        if *in_progress {
            return Err("Download already in progress".to_string());
        }
        *in_progress = true;
    }

    let result = async {
        let downloader = ModelDownloader::new()
            .map_err(|e| e.to_string())?;

        if downloader.is_dev_downloaded() {
            return Ok("FLUX Dev is already downloaded".to_string());
        }

        downloader.download_dev()
            .await
            .map_err(|e| e.to_string())?;

        Ok("FLUX Dev downloaded successfully".to_string())
    }.await;

    *app_state.download_in_progress.lock().await = false;
    result
}
```

**Step 3: Register command in invoke_handler**

Add `download_flux_dev` to the `invoke_handler` list.

**Step 4: Test compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles

**Step 5: Commit**

```bash
git add src-tauri/src/models/downloader.rs src-tauri/src/lib.rs
git commit -m "feat: add FLUX Dev model download support"
```

---

## Task 10: Update Frontend for Progress Display

**Files:**
- Modify: `src/stores/generation.ts` (or create if doesn't exist)
- Modify: `src/views/GenerateView.vue`

**Step 1: Create generation store with progress**

Create or update `src/stores/generation.ts`:

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { listen } from '@tauri-apps/api/event'

export interface GenerationProgress {
  job_id: string
  stage: string
  stage_progress: number
  overall_progress: number
  message: string
  eta_seconds: number | null
}

export const useGenerationStore = defineStore('generation', () => {
  const currentProgress = ref<GenerationProgress | null>(null)
  const isGenerating = ref(false)

  // Computed
  const progressPercent = computed(() => {
    if (!currentProgress.value) return 0
    return Math.round(currentProgress.value.overall_progress * 100)
  })

  const progressMessage = computed(() => {
    if (!currentProgress.value) return ''
    return currentProgress.value.message
  })

  const etaFormatted = computed(() => {
    if (!currentProgress.value?.eta_seconds) return null
    const seconds = Math.round(currentProgress.value.eta_seconds)
    if (seconds < 60) return `${seconds}s`
    const minutes = Math.floor(seconds / 60)
    const secs = seconds % 60
    return `${minutes}m ${secs}s`
  })

  // Actions
  function setGenerating(value: boolean) {
    isGenerating.value = value
    if (!value) {
      currentProgress.value = null
    }
  }

  function updateProgress(progress: GenerationProgress) {
    currentProgress.value = progress
  }

  // Listen for progress events
  async function setupListeners() {
    await listen<GenerationProgress>('generation-progress', (event) => {
      updateProgress(event.payload)
    })
  }

  return {
    currentProgress,
    isGenerating,
    progressPercent,
    progressMessage,
    etaFormatted,
    setGenerating,
    updateProgress,
    setupListeners,
  }
})
```

**Step 2: Update GenerateView to show progress**

Add progress display to the generate view (location depends on existing structure). Add something like:

```vue
<template>
  <!-- Add inside the generate panel -->
  <div v-if="generationStore.isGenerating" class="generation-progress">
    <ProgressBar :value="generationStore.progressPercent" />
    <div class="progress-info">
      <span class="progress-message">{{ generationStore.progressMessage }}</span>
      <span v-if="generationStore.etaFormatted" class="progress-eta">
        ETA: {{ generationStore.etaFormatted }}
      </span>
    </div>
  </div>
</template>

<script setup>
import { useGenerationStore } from '@/stores/generation'
import { onMounted } from 'vue'

const generationStore = useGenerationStore()

onMounted(() => {
  generationStore.setupListeners()
})
</script>

<style scoped>
.generation-progress {
  margin-top: 1rem;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  margin-top: 0.5rem;
  font-size: 0.875rem;
  color: #6b7280;
}
</style>
```

**Step 3: Test frontend compilation**

Run: `cd /home/alex/Dev/Work/rzem-ai-inference && npx vue-tsc --noEmit`
Expected: No TypeScript errors

**Step 4: Commit**

```bash
git add src/stores/generation.ts src/views/GenerateView.vue
git commit -m "feat: add real-time progress display in frontend"
```

---

## Success Criteria

After completing all tasks:

- [ ] `cargo test` passes all tests
- [ ] `flux-cli generate -p "a cat" -o test.png` generates an image
- [ ] `flux-cli models list` shows model status
- [ ] `flux-cli info` shows GPU and model info
- [ ] Progress events emit during generation
- [ ] Frontend shows progress bar with ETA
- [ ] Dev model can be downloaded
- [ ] Models stay loaded between generations (verify with timing)

---

## Notes

- Task 6-10 can be parallelized after Tasks 1-5 complete
- Test on both CUDA (Linux) and Metal (macOS) if possible
- The ModelManager (Task 4) is the foundation - do this carefully
- CLI batch generation is a future enhancement (not in this plan)
