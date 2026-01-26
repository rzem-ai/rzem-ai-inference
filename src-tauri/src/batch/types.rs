use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parsed data from CSV or JSON file
/// Contains column names and rows of data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchData {
    /// Column names (from CSV headers or JSON object keys)
    pub columns: Vec<String>,
    /// Rows of data, each row is a map of column_name -> value
    pub rows: Vec<HashMap<String, String>>,
}

/// Result of template rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderResult {
    /// Successfully rendered prompts (empty string for error rows)
    pub rendered: Vec<String>,
    /// Errors that occurred during rendering (row index, error message)
    pub errors: Vec<RenderError>,
}

/// Template rendering error for a specific row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderError {
    /// Zero-based row index
    pub row: usize,
    /// Error message
    pub error: String,
}
