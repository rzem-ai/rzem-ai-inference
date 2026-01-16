mod inference;
mod models;
mod queue;
mod gallery;
mod utils;

use tauri::command;
use std::sync::Mutex;
use tauri::State;

struct AppState {
    gallery_db: Mutex<Option<gallery::GalleryDb>>,
}

#[command]
fn health_check() -> String {
    "OK".to_string()
}

#[command]
fn init_database(app_state: State<AppState>, db_path: String) -> Result<String, String> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create database directory: {}", e))?;
    }

    let db = gallery::GalleryDb::new(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    db.init_schema()
        .map_err(|e| format!("Failed to initialize schema: {}", e))?;

    *app_state.gallery_db.lock().unwrap() = Some(db);

    Ok("Database initialized".to_string())
}

/// Generate an image from a text prompt using Flux model
///
/// # Arguments
/// * `prompt` - Text description of image to generate
/// * `steps` - Number of diffusion steps
/// * `width` - Image width in pixels
/// * `height` - Image height in pixels
/// * `seed` - Random seed for generation (-1 for random)
///
/// # Returns
/// Success message with generated data size
///
/// # Note
/// Currently uses stub implementation. Real Flux model integration pending.
#[command]
fn generate_image(
    _app_state: State<AppState>,
    prompt: String,
    steps: u32,
    _width: u32,
    _height: u32,
    _seed: i64,
) -> Result<String, String> {
    use crate::inference::{InferenceEngine, FluxPipeline};

    // Get or create inference engine
    let engine = InferenceEngine::new()
        .map_err(|e| format!("Failed to initialize inference engine: {}", e))?;

    let device = engine.get_device().clone();
    let pipeline = FluxPipeline::new(device)
        .map_err(|e| format!("Failed to create pipeline: {}", e))?;

    // Generate stub image (will be real model later)
    let image_data = pipeline.generate_stub(&prompt, steps as usize)
        .map_err(|e| format!("Generation failed: {}", e))?;

    // For now, just return success with data size
    Ok(format!("Generated {} bytes", image_data.len()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState {
        gallery_db: Mutex::new(None),
    };

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            health_check,
            init_database,
            generate_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
