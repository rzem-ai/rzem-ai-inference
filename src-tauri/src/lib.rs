mod inference;
mod models;
mod queue;
mod gallery;
mod utils;

use tauri::command;

#[command]
fn health_check() -> String {
    "OK".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            health_check,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
