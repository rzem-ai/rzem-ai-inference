//! Rzem AI Inference CLI

use anyhow::Result;
use clap::{Parser, Subcommand};
use rzem_ai_inference::models::{ModelPaths, ModelType};
use rzem_ai_inference::models::scanner::{
    scan_all_components_with_callbacks, scan_all_components_fast, DiscoveredComponent,
};
use rzem_ai_inference::models::bundle_builder::{BundleBuilder, to_component_record};
use rzem_ai_inference::db::InferenceDb;
use rzem_ai_inference::inference::samplers::{SamplerType, SchedulerType};

#[derive(Parser)]
#[command(name = "rzem-cli")]
#[command(about = "Rzem AI image generation CLI", long_about = None)]
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

        /// Bundle to use (name or ID). Use 'bundles list' to see available bundles.
        #[arg(short, long)]
        bundle: Option<String>,

        /// Number of denoising steps (overrides bundle default)
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
        #[arg(short = 'n', long, default_value = "1")]
        batch: usize,

        /// Guidance scale (overrides bundle default)
        #[arg(short, long)]
        guidance: Option<f64>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Suppress progress output
        #[arg(short, long)]
        quiet: bool,
    },

    /// Manage model bundles
    Bundles {
        #[command(subcommand)]
        action: BundleCommands,
    },

    /// Manage models (deprecated - use 'bundles' instead)
    Models {
        #[command(subcommand)]
        action: ModelCommands,
    },

    /// Show system information
    Info,
}

#[derive(Subcommand)]
enum BundleCommands {
    /// List available bundles
    List,

    /// Show bundle details
    Info {
        /// Bundle name or ID
        bundle: String,
    },

    /// Set the active bundle for generation
    Use {
        /// Bundle name or ID to activate
        bundle: String,
    },

    /// Scan for new models and create bundles
    Scan {
        /// Skip hash computation for faster scanning
        #[arg(long)]
        skip_hash: bool,
    },
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
    // Initialize logging (defaults to info, use RUST_LOG to override)
    // Note: quiet flag in generate command will suppress progress output but not logs
    rzem_ai_inference::init_logging("info");

    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            prompt,
            output,
            bundle,
            steps,
            width,
            height,
            seed,
            batch,
            guidance,
            json,
            quiet,
        } => {
            cmd_generate(prompt, output, bundle, steps, width, height, seed, batch, guidance, json, quiet)
        }
        Commands::Bundles { action } => match action {
            BundleCommands::List => cmd_bundles_list(),
            BundleCommands::Info { bundle } => cmd_bundles_info(bundle),
            BundleCommands::Use { bundle } => cmd_bundles_use(bundle),
            BundleCommands::Scan { skip_hash } => cmd_bundles_scan(skip_hash),
        },
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
    bundle_name: Option<String>,
    steps: Option<usize>,
    width: usize,
    height: usize,
    seed: i64,
    batch: usize,
    guidance: Option<f64>,
    json: bool,
    quiet: bool,
) -> Result<()> {
    use rzem_ai_inference::inference::{InferenceEngine, FluxPipeline, BundleContext, GenerationProgress, ImageMetadata};
    use std::io::{Write, stdout};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    // Get database and resolve bundle
    let db_path = ModelPaths::get_db_path()?;
    let db = InferenceDb::new(&db_path)?;

    // Find the bundle to use
    let bundle = if let Some(name) = bundle_name {
        // Try to find by name or ID
        find_bundle_by_name_or_id(&db, &name)?
            .ok_or_else(|| anyhow::anyhow!("Bundle not found: {}. Run 'rzem-cli bundles list' to see available bundles.", name))?
    } else {
        // Use active bundle
        db.get_active_bundle()?
            .ok_or_else(|| anyhow::anyhow!("No active bundle. Use 'rzem-cli bundles use <name>' to set one, or specify --bundle."))?
    };

    // Get default values from bundle
    let steps = steps.unwrap_or_else(|| bundle.default_steps.unwrap_or(4) as usize);
    let guidance = guidance.unwrap_or_else(|| bundle.default_guidance.unwrap_or(0.0));

    // Create bundle context for the pipeline
    let bundle_context = BundleContext {
        bundle_id: Some(bundle.id.clone()),
        model_component_id: None,
        t5_component_id: None,
        clip_component_id: None,
        vae_component_id: None,
    };

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
        println!("Rzem CLI - Image Generation");
        println!("════════════════════════════════════════");
        println!("  Prompt:   {}", prompt);
        println!("  Bundle:   {} ({})", bundle.name, bundle.model_family);
        println!("  Steps:    {}", steps);
        println!("  Size:     {}x{}", width, height);
        println!("  Seed:     {}", actual_seed);
        println!("  Guidance: {}", guidance);
        println!("  Batch:    {}", batch);
        println!("  Output:   {}", output);
        println!();
    }

    // Initialize inference engine
    let engine = InferenceEngine::new()?;
    let device = engine.get_device().clone();

    // Track last progress line length for proper clearing
    let last_len = Arc::new(AtomicUsize::new(0));

    // Results for JSON output
    let mut results: Vec<serde_json::Value> = Vec::new();
    let start_time = std::time::Instant::now();

    for batch_idx in 0..batch {
        // Create fresh pipeline for each image (models get unloaded during generation)
        let mut pipeline = FluxPipeline::with_model_type(device.clone(), ModelType::schnell())?;

        // Set bundle context so pipeline knows which model files to load
        pipeline.set_bundle_context(bundle_context.clone());

        // Determine output filename for this batch item
        let output_path = if batch > 1 {
            let path = std::path::Path::new(&output);
            let stem = path.file_stem().unwrap_or_default().to_string_lossy();
            let ext = path.extension().unwrap_or_default().to_string_lossy();
            let parent = path.parent().unwrap_or(std::path::Path::new("."));
            parent.join(format!("{}_{}.{}", stem, batch_idx + 1, ext))
        } else {
            std::path::PathBuf::from(&output)
        };

        if !quiet && !json {
            if batch > 1 {
                println!("Generating image {}/{}...", batch_idx + 1, batch);
            }
        }

        // Progress callback for terminal display
        let last_len_clone = last_len.clone();
        let quiet_clone = quiet;
        let json_clone = json;
        let batch_clone = batch;
        let batch_idx_clone = batch_idx;

        let on_progress = move |progress: GenerationProgress| {
            if quiet_clone || json_clone {
                return;
            }

            // Format progress bar
            let bar_width = 30;
            let filled = (progress.overall_progress * bar_width as f32) as usize;
            let empty = bar_width - filled;
            let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));

            // Format message
            let percent = (progress.overall_progress * 100.0) as u32;
            let batch_prefix = if batch_clone > 1 {
                format!("[{}/{}] ", batch_idx_clone + 1, batch_clone)
            } else {
                String::new()
            };

            let line = format!(
                "\r{}{} {:>3}% {}",
                batch_prefix,
                bar,
                percent,
                progress.message
            );

            // Clear previous line if it was longer
            let prev_len = last_len_clone.load(Ordering::Relaxed);
            if line.len() < prev_len {
                print!("\r{}", " ".repeat(prev_len));
            }
            last_len_clone.store(line.len(), Ordering::Relaxed);

            print!("{}", line);
            let _ = stdout().flush();
        };

        // Create metadata for embedding in the PNG
        let metadata = ImageMetadata {
            prompt: prompt.clone(),
            negative_prompt: None,
            steps: steps as u32,
            cfg_scale: guidance,
            width: width as u32,
            height: height as u32,
            seed: actual_seed,
            model: bundle.name.clone(),
            sampler: Some("euler".to_string()),
            scheduler: Some("normal".to_string()),
            loras: Vec::new(),  // No LoRAs in CLI
        };

        // Generate image
        let result = pipeline.generate_with_progress(
            &prompt,
            steps,
            width,
            height,
            guidance,
            actual_seed as u64,
            Some(metadata),
            SamplerType::default(),
            SchedulerType::default(),
            on_progress,
        )?;

        // Clear progress line
        if !quiet && !json {
            let prev_len = last_len.load(Ordering::Relaxed);
            print!("\r{}\r", " ".repeat(prev_len));
            let _ = stdout().flush();
        }

        // Create parent directory if needed
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Save image
        std::fs::write(&output_path, &result.image_data)?;

        if json {
            results.push(serde_json::json!({
                "path": output_path.to_string_lossy(),
                "batch_index": batch_idx + 1,
                "stats": {
                    "total_ms": result.stats.total_ms,
                    "denoise_ms": result.stats.denoise_ms,
                    "model_type": result.stats.model_type,
                    "steps": result.stats.steps,
                }
            }));
        } else if !quiet {
            println!("✓ Saved: {} ({}ms)", output_path.display(), result.stats.total_ms);
        }
    }

    let total_elapsed = start_time.elapsed().as_millis();

    if json {
        println!("{}", serde_json::json!({
            "success": true,
            "images": results,
            "total_ms": total_elapsed,
            "prompt": prompt,
            "bundle": bundle.name,
            "bundle_id": bundle.id,
            "model_family": bundle.model_family,
            "steps": steps,
            "guidance": guidance,
            "seed": actual_seed,
            "width": width,
            "height": height,
        }));
    } else if !quiet {
        println!();
        println!("════════════════════════════════════════");
        println!("Generation complete! Total: {}ms", total_elapsed);
    }

    Ok(())
}

fn cmd_models_list() -> Result<()> {
    let db_path = ModelPaths::get_db_path()?;
    let db = rzem_ai_inference::db::InferenceDb::new(&db_path)?;
    let paths = ModelPaths::new(&db)?;

    println!("FLUX Models");
    println!("════════════════════════════════════════════════════════════");
    println!("{:<12} {:<18} {:<12}", "MODEL", "STATUS", "SIZE");
    println!("────────────────────────────────────────────────────────────");

    // Schnell
    let schnell_status = if paths.all_files_exist() {
        "✓ downloaded"
    } else {
        "✗ not downloaded"
    };
    println!("{:<12} {:<18} {:<12}", "schnell", schnell_status, "~23 GB");

    // Bundle status
    let bundle_status = if let Some(bundle_id) = paths.bundle_id() {
        format!("Bundle: {}", bundle_id)
    } else {
        "No active bundle".to_string()
    };
    println!("\n{}", bundle_status);

    println!();
    Ok(())
}

fn cmd_models_download(model: String) -> Result<()> {
    let model_type: ModelType = model.parse().unwrap();

    println!("Downloading {}...", model_type);
    println!("(Download not yet implemented - use the GUI or huggingface-cli)");

    Ok(())
}

fn cmd_models_info(model: String) -> Result<()> {
    let model_type: ModelType = model.parse().unwrap();

    println!("{}", model_type.display_name());
    println!("════════════════════════════════════════");
    println!("Default steps:    {}", model_type.default_steps());
    println!("Step range:       {}-{}", model_type.step_range().0, model_type.step_range().1);
    println!("Default guidance: {}", model_type.default_guidance());
    println!("VRAM (full):      {} GB", model_type.vram_full_precision() / 1000);
    println!("VRAM (quantized): {} GB", model_type.vram_quantized() / 1000);
    println!("Repository:       {}", model_type.repo_id());
    println!();

    Ok(())
}

fn cmd_info() -> Result<()> {
    use rzem_ai_inference::inference::InferenceEngine;

    println!("Rzem AI Inference CLI");
    println!("════════════════════════════════════════");

    let engine = InferenceEngine::new()?;
    let device = engine.get_device();

    let device_name = if device.is_cuda() {
        "CUDA GPU"
    } else if device.is_metal() {
        "Metal GPU (Apple Silicon)"
    } else {
        "CPU"
    };

    println!("Device:  {}", device_name);

    let db_path = ModelPaths::get_db_path()?;
    let db = rzem_ai_inference::db::InferenceDb::new(&db_path)?;
    let paths = ModelPaths::new(&db)?;
    if let Some(bundle_id) = paths.bundle_id() {
        println!("Bundle:  {}", bundle_id);
    }

    println!();
    println!("Model Status:");
    println!("────────────────────────────────────────");
    for (name, exists, _path) in paths.get_status() {
        let status = if exists { "✓" } else { "✗" };
        println!("  {} {}", status, name);
    }

    println!();
    Ok(())
}

// ===== Bundle Commands =====

fn cmd_bundles_list() -> Result<()> {
    let db_path = ModelPaths::get_db_path()?;
    let db = InferenceDb::new(&db_path)?;

    let bundles = db.get_all_bundles()?;

    println!("Model Bundles");
    println!("════════════════════════════════════════════════════════════════════════════════");

    if bundles.is_empty() {
        println!("No bundles found. Run 'rzem-cli bundles scan' to discover models.");
        return Ok(());
    }

    println!("{:<3} {:<40} {:<15} {:<10} {:<10}",
        "", "NAME", "FAMILY", "COMPLETE", "VRAM");
    println!("────────────────────────────────────────────────────────────────────────────────");

    for bundle in &bundles {
        let active_marker = if bundle.is_active { "→" } else { " " };
        let complete_marker = if bundle.is_complete { "✓" } else { "✗" };
        let vram = bundle.total_vram_mb.map(|v| format!("{}MB", v)).unwrap_or_default();

        println!("{} {:<40} {:<15} {:<10} {:<10}",
            active_marker,
            truncate(&bundle.name, 40),
            bundle.model_family,
            complete_marker,
            vram
        );
    }

    println!();
    println!("Total: {} bundles (→ = active)", bundles.len());
    println!();
    println!("Use 'rzem-cli bundles info <name>' for details");
    println!("Use 'rzem-cli bundles use <name>' to set active bundle");

    Ok(())
}

fn cmd_bundles_info(bundle_name: String) -> Result<()> {
    let db_path = ModelPaths::get_db_path()?;
    let db = InferenceDb::new(&db_path)?;

    let bundle = find_bundle_by_name_or_id(&db, &bundle_name)?
        .ok_or_else(|| anyhow::anyhow!("Bundle not found: {}", bundle_name))?;

    println!("{}", bundle.name);
    println!("════════════════════════════════════════════════════════════════════════════════");
    println!("ID:          {}", bundle.id);
    println!("Family:      {}", bundle.model_family);
    println!("Type:        {}", bundle.bundle_type);
    println!("Complete:    {}", if bundle.is_complete { "Yes" } else { "No" });
    println!("Active:      {}", if bundle.is_active { "Yes" } else { "No" });

    if let Some(steps) = bundle.default_steps {
        println!("Steps:       {} (range: {}-{})",
            steps,
            bundle.step_min.unwrap_or(1),
            bundle.step_max.unwrap_or(50)
        );
    }
    if let Some(guidance) = bundle.default_guidance {
        println!("Guidance:    {:.1}", guidance);
    }
    if let Some(vram) = bundle.total_vram_mb {
        println!("VRAM:        {} MB", vram);
    }

    if let Some(desc) = &bundle.description {
        println!();
        println!("Description: {}", desc);
    }

    println!();
    println!("Components:");
    println!("────────────────────────────────────────────────────────────────────────────────");

    for comp in &bundle.components {
        let available = if comp.is_available { "✓" } else { "✗" };
        let size_mb = comp.file_size / 1_000_000;
        let quant = comp.quantization.as_ref()
            .map(|q| format!(" [{}]", q))
            .unwrap_or_default();

        println!("  {} {:<15} {} ({}MB){}",
            available,
            comp.role,
            comp.component_type,
            size_mb,
            quant
        );
        println!("                    {}", comp.file_path);
    }

    println!();
    Ok(())
}

fn cmd_bundles_use(bundle_name: String) -> Result<()> {
    let db_path = ModelPaths::get_db_path()?;
    let db = InferenceDb::new(&db_path)?;

    let bundle = find_bundle_by_name_or_id(&db, &bundle_name)?
        .ok_or_else(|| anyhow::anyhow!("Bundle not found: {}", bundle_name))?;

    if !bundle.is_complete {
        println!("Warning: Bundle '{}' is incomplete. Some components may be missing.", bundle.name);
    }

    db.set_bundle_active(&bundle.id, true)?;
    println!("✓ Activated bundle: {}", bundle.name);

    Ok(())
}

fn cmd_bundles_scan(skip_hash: bool) -> Result<()> {
    let db_path = ModelPaths::get_db_path()?;
    let db = InferenceDb::new(&db_path)?;

    println!("Rzem Model Scanner");
    println!("════════════════════════════════════════════════════════════════════════════════");
    println!("Database: {}", db_path);
    if skip_hash {
        println!("Mode: Fast (skipping hash computation)");
    }
    println!();

    // Scan for components
    println!("Scanning HuggingFace cache...");
    println!("────────────────────────────────────────────────────────────────────────────────");

    let components = scan_with_progress(skip_hash)?;

    println!();
    println!("Found {} components", components.len());
    println!();

    // Store components
    println!("Storing components...");
    let mut stored = 0;
    let mut skipped = 0;

    for comp in &components {
        let record = to_component_record(comp);
        match db.get_component(&record.id) {
            Ok(_) => skipped += 1,
            Err(_) => {
                db.insert_component(&record)?;
                stored += 1;
            }
        }
    }

    println!("  Stored: {}, Skipped (existing): {}", stored, skipped);
    println!();

    // Build bundles
    println!("Building bundles...");
    println!("────────────────────────────────────────────────────────────────────────────────");

    let builder = BundleBuilder::new(components);
    let bundle_defs = builder.auto_create_bundles();

    let mut bundle_count = 0;
    for bundle_def in &bundle_defs {
        // Check if bundle exists
        if db.get_bundle(&bundle_def.id).is_ok() {
            continue;
        }

        // Calculate VRAM
        let total_vram: i32 = bundle_def.component_map.iter()
            .filter_map(|(_, comp_id)| {
                db.get_component(comp_id).ok()
                    .and_then(|c| c.vram_mb)
            })
            .sum();

        // Check completeness
        let missing: Vec<_> = bundle_def.component_map.iter()
            .filter(|(_, comp_id)| db.get_component(comp_id).is_err())
            .map(|(role, _)| role.clone())
            .collect();

        let is_complete = missing.is_empty() && bundle_def.component_map.len() >= 4;

        // Create bundle
        let bundle_record = bundle_def.to_record(total_vram, is_complete);
        db.insert_bundle(&bundle_record)?;

        // Add component relationships
        for (priority, (role, comp_id)) in bundle_def.component_map.iter().enumerate() {
            let is_required = matches!(role.as_str(), "transformer" | "t5" | "clip" | "vae");
            db.add_component_to_bundle(&bundle_def.id, comp_id, role, is_required, priority as i32)?;
        }

        let status = if is_complete { "complete" } else { "incomplete" };
        println!("  Created: {} ({})", bundle_def.name, status);
        bundle_count += 1;
    }

    if bundle_count == 0 {
        println!("  (no new bundles created)");
    }

    // Summary
    println!();
    println!("════════════════════════════════════════════════════════════════════════════════");

    let all_bundles = db.get_all_bundles()?;
    let complete = all_bundles.iter().filter(|b| b.is_complete).count();
    println!("Total bundles: {} ({} complete)", all_bundles.len(), complete);

    if db.get_active_bundle()?.is_none() && !all_bundles.is_empty() {
        // Auto-activate first complete bundle
        if let Some(first_complete) = all_bundles.iter().find(|b| b.is_complete) {
            db.set_bundle_active(&first_complete.id, true)?;
            println!("Auto-activated: {}", first_complete.name);
        }
    }

    println!();
    println!("Use 'rzem-cli bundles list' to see all bundles");

    Ok(())
}

// ===== Helper Functions =====

fn find_bundle_by_name_or_id(db: &InferenceDb, name_or_id: &str) -> Result<Option<rzem_ai_inference::db::BundleInfo>> {
    // First try exact ID match
    if let Ok(bundle) = db.get_bundle(name_or_id) {
        return Ok(Some(bundle));
    }

    // Then search by name (case-insensitive partial match)
    let all_bundles = db.get_all_bundles()?;
    let name_lower = name_or_id.to_lowercase();

    // Exact name match first
    if let Some(bundle) = all_bundles.iter().find(|b| b.name.to_lowercase() == name_lower) {
        return Ok(Some(bundle.clone()));
    }

    // Partial name match
    if let Some(bundle) = all_bundles.iter().find(|b| b.name.to_lowercase().contains(&name_lower)) {
        return Ok(Some(bundle.clone()));
    }

    Ok(None)
}

fn scan_with_progress(skip_hash: bool) -> Result<Vec<DiscoveredComponent>> {
    let progress_callback = Box::new(|current: usize, total: usize, repo: &str| {
        println!("[{}/{}] {}", current, total, repo);
    });

    let repo_callback = Box::new(|repo: &str, components: Vec<DiscoveredComponent>| {
        if !components.is_empty() {
            println!("  Found {} components in {}", components.len(), repo);
            for comp in &components {
                let size_mb = comp.file_size / 1_000_000;
                let quant = comp.quantization.as_ref()
                    .map(|q| format!(" [{}]", q))
                    .unwrap_or_default();
                println!("    - {:?}: {}MB{}",
                    comp.component_type,
                    size_mb,
                    quant
                );
            }
        }
    });

    if skip_hash {
        scan_all_components_fast(
            Some(progress_callback),
            Some(repo_callback),
        )
    } else {
        scan_all_components_with_callbacks(
            Some(progress_callback),
            None, // hash callback
            Some(repo_callback),
            None, // skip callback
        )
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
