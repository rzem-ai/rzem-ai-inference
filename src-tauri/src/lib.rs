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
/// File path to the generated image
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
    seed: i64,
) -> Result<String, String> {
    use crate::inference::{InferenceEngine, FluxPipeline};
    use std::fs;

    // Get or create inference engine
    let engine = InferenceEngine::new()
        .map_err(|e| format!("Failed to initialize inference engine: {}", e))?;

    let device = engine.get_device().clone();
    let pipeline = FluxPipeline::new(device)
        .map_err(|e| format!("Failed to create pipeline: {}", e))?;

    // Generate stub image
    let image_data = pipeline.generate_stub(&prompt, steps as usize)
        .map_err(|e| format!("Generation failed: {}", e))?;

    // Determine output path
    let home = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?;
    let output_dir = home.join(".flux-generator").join("outputs");

    // Create output directory
    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    // Generate filename with timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("System time error: {}", e))?
        .as_secs();
    let filename = format!("flux_{}_{}.raw", timestamp, seed);
    let output_path = output_dir.join(&filename);

    // Save raw image data (will be PNG later)
    fs::write(&output_path, &image_data)
        .map_err(|e| format!("Failed to write image: {}", e))?;

    Ok(output_path.to_string_lossy().to_string())
}

#[command]
fn get_gallery_images(
    app_state: State<AppState>,
    limit: usize,
) -> Result<Vec<gallery::ImageMetadata>, String> {
    let db = app_state.gallery_db.lock().unwrap();
    let db = db.as_ref().ok_or("Database not initialized")?;

    db.get_gallery_images(limit)
        .map_err(|e| format!("Failed to get images: {}", e))
}

#[command]
fn search_gallery_images(
    app_state: State<AppState>,
    query: String,
) -> Result<Vec<gallery::ImageMetadata>, String> {
    let db = app_state.gallery_db.lock().unwrap();
    let db = db.as_ref().ok_or("Database not initialized")?;

    db.search_gallery_images(&query)
        .map_err(|e| format!("Failed to search images: {}", e))
}

#[command]
fn toggle_favorite(
    app_state: State<AppState>,
    image_id: String,
) -> Result<String, String> {
    let db = app_state.gallery_db.lock().unwrap();
    let db = db.as_ref().ok_or("Database not initialized")?;

    db.toggle_favorite(&image_id)
        .map_err(|e| format!("Failed to toggle favorite: {}", e))?;

    Ok("Favorite toggled".to_string())
}

#[command]
fn add_image_tag(
    app_state: State<AppState>,
    image_id: String,
    tag: String,
) -> Result<String, String> {
    let db = app_state.gallery_db.lock().unwrap();
    let db = db.as_ref().ok_or("Database not initialized")?;

    db.add_image_tag(&image_id, &tag)
        .map_err(|e| format!("Failed to add tag: {}", e))?;

    Ok("Tag added".to_string())
}

#[command]
fn remove_image_tag(
    app_state: State<AppState>,
    image_id: String,
    tag: String,
) -> Result<String, String> {
    let db = app_state.gallery_db.lock().unwrap();
    let db = db.as_ref().ok_or("Database not initialized")?;

    db.remove_image_tag(&image_id, &tag)
        .map_err(|e| format!("Failed to remove tag: {}", e))?;

    Ok("Tag removed".to_string())
}

#[command]
fn delete_gallery_image(
    app_state: State<AppState>,
    image_id: String,
) -> Result<String, String> {
    let db = app_state.gallery_db.lock().unwrap();
    let db = db.as_ref().ok_or("Database not initialized")?;

    // Get file path before deleting
    let images = db.get_gallery_images(1000)
        .map_err(|e| format!("Failed to get images: {}", e))?;
    if let Some(image) = images.iter().find(|img| img.id == image_id) {
        std::fs::remove_file(&image.file_path)
            .map_err(|e| format!("Failed to delete file: {}", e))?;
    }

    // Then delete from database
    db.delete_gallery_image(&image_id)
        .map_err(|e| format!("Failed to delete from database: {}", e))?;

    Ok("Image deleted".to_string())
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
            get_gallery_images,
            search_gallery_images,
            toggle_favorite,
            add_image_tag,
            remove_image_tag,
            delete_gallery_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
