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
        println!("FLUX CLI - Image Generation");
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

    // TODO: Implement actual generation in Task 7
    if json {
        println!(r#"{{"success": false, "error": "Generation not yet implemented"}}"#);
    } else if !quiet {
        println!("[INFO] Generation not yet implemented - CLI structure ready");
    }

    Ok(())
}

fn cmd_models_list() -> Result<()> {
    let paths = ModelPaths::new()?;

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

    // Dev
    let dev_status = if paths.is_dev_downloaded() {
        "✓ downloaded"
    } else {
        "✗ not downloaded"
    };
    println!("{:<12} {:<18} {:<12}", "dev", dev_status, "~24 GB");

    println!();
    Ok(())
}

fn cmd_models_download(model: String) -> Result<()> {
    let model_type: ModelType = model.parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    println!("Downloading {}...", model_type);
    println!("(Download not yet implemented - use the GUI or huggingface-cli)");

    Ok(())
}

fn cmd_models_info(model: String) -> Result<()> {
    let model_type: ModelType = model.parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

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
    use flux_generator_lib::inference::InferenceEngine;

    println!("FLUX Generator CLI");
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

    let paths = ModelPaths::new()?;
    println!("Cache:   {}", paths.cache_dir.display());

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
