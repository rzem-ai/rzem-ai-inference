//! Rzem AI Inference CLI

use anyhow::Result;
use clap::{Parser, Subcommand};
use rzem_ai_inference::models::{ModelPaths, ModelType};
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
    // Initialize logging (defaults to info, use RUST_LOG to override)
    // Note: quiet flag in generate command will suppress progress output but not logs
    rzem_ai_inference::init_logging("info");

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
    use rzem_ai_inference::inference::{InferenceEngine, FluxPipeline, GenerationProgress, ImageMetadata};
    use std::io::{Write, stdout};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let model_type: ModelType = model.parse().unwrap();

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
        println!("Rzem CLI - Image Generation");
        println!("════════════════════════════════════════");
        println!("  Prompt:   {}", prompt);
        println!("  Model:    {}", model_type);
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
        let mut pipeline = FluxPipeline::with_model_type(device.clone(), model_type.clone())?;

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
            model: model_type.to_string(),
            sampler: Some("euler".to_string()),
            scheduler: Some("normal".to_string()),
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
            "model": model_type.to_string(),
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
    let db = rzem_ai_inference::gallery::GalleryDb::new(&db_path)?;
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
    let db = rzem_ai_inference::gallery::GalleryDb::new(&db_path)?;
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
