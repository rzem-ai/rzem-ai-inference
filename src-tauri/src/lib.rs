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
    let db = gallery::GalleryDb::new(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    db.init_schema()
        .map_err(|e| format!("Failed to initialize schema: {}", e))?;

    *app_state.gallery_db.lock().unwrap() = Some(db);

    Ok("Database initialized".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState {
        gallery_db: Mutex::new(None),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            health_check,
            init_database,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
