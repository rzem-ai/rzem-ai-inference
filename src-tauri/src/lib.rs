mod inference;
mod models;
mod queue;
mod gallery;
mod utils;

use tauri::{command, Manager};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use queue::QueueProcessor;

struct AppState {
    gallery_db: Arc<Mutex<Option<gallery::GalleryDb>>>,
    queue_manager: Arc<queue::QueueManager>,
    queue_processor: Arc<QueueProcessor>,
}

#[command]
fn health_check() -> String {
    "OK".to_string()
}

#[command]
async fn init_database(app_state: State<'_, AppState>, db_path: String) -> Result<String, String> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create database directory: {}", e))?;
    }

    let db = gallery::GalleryDb::new(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    db.init_schema()
        .map_err(|e| format!("Failed to initialize schema: {}", e))?;

    *app_state.gallery_db.lock().await = Some(db);

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
    let filename = format!("flux_{}_{}.png", timestamp, seed);
    let output_path = output_dir.join(&filename);

    // Save PNG image data
    fs::write(&output_path, &image_data)
        .map_err(|e| format!("Failed to write image: {}", e))?;

    Ok(output_path.to_string_lossy().to_string())
}

#[command]
async fn get_gallery_images(
    app_state: State<'_, AppState>,
    limit: usize,
) -> Result<Vec<gallery::ImageMetadata>, String> {
    let db = app_state.gallery_db.lock().await;
    let db = db.as_ref().ok_or("Database not initialized")?;

    db.get_gallery_images(limit)
        .map_err(|e| format!("Failed to get images: {}", e))
}

#[command]
async fn search_gallery_images(
    app_state: State<'_, AppState>,
    query: String,
) -> Result<Vec<gallery::ImageMetadata>, String> {
    let db = app_state.gallery_db.lock().await;
    let db = db.as_ref().ok_or("Database not initialized")?;

    db.search_gallery_images(&query)
        .map_err(|e| format!("Failed to search images: {}", e))
}

#[command]
async fn toggle_favorite(
    app_state: State<'_, AppState>,
    image_id: String,
) -> Result<String, String> {
    let db = app_state.gallery_db.lock().await;
    let db = db.as_ref().ok_or("Database not initialized")?;

    db.toggle_favorite(&image_id)
        .map_err(|e| format!("Failed to toggle favorite: {}", e))?;

    Ok("Favorite toggled".to_string())
}

#[command]
async fn add_image_tag(
    app_state: State<'_, AppState>,
    image_id: String,
    tag: String,
) -> Result<String, String> {
    let db = app_state.gallery_db.lock().await;
    let db = db.as_ref().ok_or("Database not initialized")?;

    db.add_image_tag(&image_id, &tag)
        .map_err(|e| format!("Failed to add tag: {}", e))?;

    Ok("Tag added".to_string())
}

#[command]
async fn remove_image_tag(
    app_state: State<'_, AppState>,
    image_id: String,
    tag: String,
) -> Result<String, String> {
    let db = app_state.gallery_db.lock().await;
    let db = db.as_ref().ok_or("Database not initialized")?;

    db.remove_image_tag(&image_id, &tag)
        .map_err(|e| format!("Failed to remove tag: {}", e))?;

    Ok("Tag removed".to_string())
}

#[command]
async fn delete_gallery_image(
    app_state: State<'_, AppState>,
    image_id: String,
) -> Result<String, String> {
    let db = app_state
        .gallery_db
        .lock()
        .await;

    let db = db.as_ref().ok_or("Database not initialized")?;

    // Get image metadata
    let image = db.get_image_by_id(&image_id)
        .map_err(|e| format!("Failed to get image: {}", e))?
        .ok_or_else(|| "Image not found".to_string())?;

    // Delete file from filesystem
    std::fs::remove_file(&image.file_path)
        .map_err(|e| format!("Failed to delete file: {}", e))?;

    // Delete from database (this will cascade to image_tags)
    db.delete_gallery_image(&image_id)
        .map_err(|e| format!("Failed to delete from database: {}", e))?;

    Ok("Image deleted".to_string())
}

#[command]
async fn add_to_queue(
    app_state: State<'_, AppState>,
    params: queue::GenerationParams,
) -> Result<String, String> {
    let job_id = app_state.queue_manager.add_job(params).await;
    Ok(job_id)
}

#[command]
async fn get_queue_jobs(
    app_state: State<'_, AppState>,
) -> Result<Vec<queue::GenerationJob>, String> {
    let jobs = app_state.queue_manager.get_jobs().await;
    Ok(jobs)
}

#[command]
async fn get_queue_job(
    app_state: State<'_, AppState>,
    job_id: String,
) -> Result<Option<queue::GenerationJob>, String> {
    let job = app_state.queue_manager.get_job(&job_id).await;
    Ok(job)
}

#[command]
async fn cancel_queue_job(
    app_state: State<'_, AppState>,
    job_id: String,
) -> Result<bool, String> {
    let cancelled = app_state.queue_manager.cancel_job(&job_id).await;
    Ok(cancelled)
}

#[command]
async fn clear_completed_jobs(
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    app_state.queue_manager.clear_completed().await;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let gallery_db = Arc::new(Mutex::new(None));
    let queue_manager = Arc::new(queue::QueueManager::new(1)); // Max 1 concurrent for now

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            // Get app handle
            let app_handle = app.handle().clone();

            // Create processor with app handle
            let queue_processor = Arc::new(
                QueueProcessor::new(
                    queue_manager.clone(),
                    gallery_db.clone(),
                    app_handle,
                ).expect("Failed to create queue processor")
            );

            // Store in managed state
            app.manage(AppState {
                gallery_db: gallery_db.clone(),
                queue_manager: queue_manager.clone(),
                queue_processor: queue_processor.clone(),
            });

            // Start processor
            tauri::async_runtime::spawn(async move {
                queue_processor.start().await;
            });

            Ok(())
        })
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
            add_to_queue,
            get_queue_jobs,
            get_queue_job,
            cancel_queue_job,
            clear_completed_jobs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
