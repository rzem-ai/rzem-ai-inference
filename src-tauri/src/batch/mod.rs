pub mod combinatorial;
pub mod parser;
pub mod renderer;
pub mod types;

#[cfg(test)]
mod tests;

pub use parser::{parse_csv, parse_json};
pub use renderer::render_template;
pub use types::{BatchData, RenderError, RenderResult, TemplateHistoryEntry};

use std::collections::HashMap;
use tauri::State;
use crate::AppState;

/// Tauri command: Parse CSV or JSON data
/// Format: "csv" or "json"
#[tauri::command]
pub fn batch_parse_data(content: String, format: String) -> Result<BatchData, String> {
    match format.to_lowercase().as_str() {
        "csv" => parse_csv(&content).map_err(|e| format!("CSV parse error: {}", e)),
        "json" => parse_json(&content).map_err(|e| format!("JSON parse error: {}", e)),
        _ => Err(format!("Unsupported format: {}. Use 'csv' or 'json'", format)),
    }
}

/// Tauri command: Render template with data rows
#[tauri::command]
pub fn batch_render_template(
    template: String,
    rows: Vec<HashMap<String, String>>,
) -> Result<RenderResult, String> {
    if template.trim().is_empty() {
        return Err("Template cannot be empty".to_string());
    }

    if rows.is_empty() {
        return Err("No data rows provided".to_string());
    }

    Ok(render_template(&template, rows))
}

/// Tauri command: Generate combinatorial batch data from input
#[tauri::command]
pub fn batch_generate_combinations(data: BatchData) -> Result<BatchData, String> {
    combinatorial::generate_combinations(data)
        .map_err(|e| format!("Failed to generate combinations: {}", e))
}

/// Tauri command: Get recent template history (last 5 used templates)
#[tauri::command]
pub async fn batch_get_recent_templates(
    app_state: State<'_, AppState>,
) -> Result<Vec<TemplateHistoryEntry>, String> {
    let db = app_state.gallery_db.lock().await;
    let db = db.as_ref().ok_or("Database not initialized")?;

    db.get_recent_template_history()
        .map_err(|e| format!("Failed to get recent templates: {}", e))
}

/// Tauri command: Save a template to history after generation
#[tauri::command]
pub async fn batch_save_template(
    app_state: State<'_, AppState>,
    template: String,
    image_count: i64,
) -> Result<(), String> {
    let db = app_state.gallery_db.lock().await;
    let db = db.as_ref().ok_or("Database not initialized")?;

    db.save_template_to_history(&template, image_count)
        .map_err(|e| format!("Failed to save template: {}", e))
}
