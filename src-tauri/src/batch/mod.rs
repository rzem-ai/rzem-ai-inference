pub mod combinatorial;
pub mod parser;
pub mod renderer;
pub mod types;

#[cfg(test)]
mod tests;

pub use parser::{parse_csv, parse_json};
pub use renderer::render_template;
pub use types::{BatchData, RenderError, RenderResult};

use std::collections::HashMap;

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
