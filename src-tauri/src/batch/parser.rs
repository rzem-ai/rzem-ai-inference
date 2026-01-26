use anyhow::{Context, Result};
use std::collections::HashMap;

use super::types::BatchData;

/// Parse CSV content with headers
/// First row must contain column names
pub fn parse_csv(content: &str) -> Result<BatchData> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(false) // Require same number of fields per row
        .from_reader(content.as_bytes());

    // Get headers
    let headers = reader
        .headers()
        .context("Failed to read CSV headers")?
        .clone();

    let columns: Vec<String> = headers.iter().map(|s| s.to_string()).collect();

    if columns.is_empty() {
        anyhow::bail!("CSV file has no columns");
    }

    // Parse rows
    let mut rows = Vec::new();
    for (idx, result) in reader.records().enumerate() {
        let record = result.context(format!("Failed to parse CSV row {}", idx + 1))?;

        let mut row_map = HashMap::new();
        for (col_idx, value) in record.iter().enumerate() {
            if let Some(col_name) = columns.get(col_idx) {
                row_map.insert(col_name.clone(), value.to_string());
            }
        }
        rows.push(row_map);
    }

    if rows.is_empty() {
        anyhow::bail!("CSV file has no data rows");
    }

    Ok(BatchData { columns, rows })
}

/// Parse JSON content
/// Supports two formats:
/// 1. Array of objects: [{"col1": "val1", "col2": "val2"}, ...]
/// 2. Object with arrays: {"col1": ["val1", ...], "col2": ["val2", ...]}
pub fn parse_json(content: &str) -> Result<BatchData> {
    // Try parsing as array of objects first
    if let Ok(array) = serde_json::from_str::<Vec<HashMap<String, serde_json::Value>>>(content) {
        return parse_json_array_of_objects(array);
    }

    // Try parsing as object with arrays
    if let Ok(obj) = serde_json::from_str::<HashMap<String, serde_json::Value>>(content) {
        return parse_json_object_with_arrays(obj);
    }

    anyhow::bail!("JSON must be either an array of objects or an object with arrays")
}

/// Parse JSON array of objects: [{"col1": "val1"}, {"col1": "val2"}]
fn parse_json_array_of_objects(
    array: Vec<HashMap<String, serde_json::Value>>,
) -> Result<BatchData> {
    if array.is_empty() {
        anyhow::bail!("JSON array is empty");
    }

    // Get all unique column names from all objects
    let mut columns_set = std::collections::HashSet::new();
    for obj in &array {
        for key in obj.keys() {
            columns_set.insert(key.clone());
        }
    }

    let mut columns: Vec<String> = columns_set.into_iter().collect();
    columns.sort(); // Consistent ordering

    if columns.is_empty() {
        anyhow::bail!("JSON objects have no keys");
    }

    // Convert to rows
    let rows: Vec<HashMap<String, String>> = array
        .into_iter()
        .map(|obj| {
            columns
                .iter()
                .map(|col| {
                    let value = obj
                        .get(col)
                        .map(|v| json_value_to_string(v))
                        .unwrap_or_default();
                    (col.clone(), value)
                })
                .collect()
        })
        .collect();

    Ok(BatchData { columns, rows })
}

/// Parse JSON object with arrays: {"col1": ["val1", "val2"], "col2": ["val3", "val4"]}
fn parse_json_object_with_arrays(obj: HashMap<String, serde_json::Value>) -> Result<BatchData> {
    if obj.is_empty() {
        anyhow::bail!("JSON object is empty");
    }

    // Extract arrays and check they all have the same length
    let mut arrays: Vec<(String, Vec<String>)> = Vec::new();
    let mut expected_length: Option<usize> = None;

    for (key, value) in obj {
        if let serde_json::Value::Array(arr) = value {
            let string_arr: Vec<String> = arr.iter().map(json_value_to_string).collect();

            if let Some(len) = expected_length {
                if string_arr.len() != len {
                    anyhow::bail!(
                        "All arrays must have the same length. Expected {}, got {} for key '{}'",
                        len,
                        string_arr.len(),
                        key
                    );
                }
            } else {
                expected_length = Some(string_arr.len());
            }

            arrays.push((key, string_arr));
        } else {
            anyhow::bail!("All values in JSON object must be arrays");
        }
    }

    let num_rows = expected_length.context("No arrays found")?;
    if num_rows == 0 {
        anyhow::bail!("Arrays are empty");
    }

    // Transpose: convert column-wise arrays to row-wise maps
    let mut rows = Vec::new();
    for row_idx in 0..num_rows {
        let mut row_map = HashMap::new();
        for (col_name, values) in &arrays {
            row_map.insert(col_name.clone(), values[row_idx].clone());
        }
        rows.push(row_map);
    }

    let columns: Vec<String> = arrays.into_iter().map(|(name, _)| name).collect();

    Ok(BatchData { columns, rows })
}

/// Convert JSON value to string representation
fn json_value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
        _ => value.to_string(), // For arrays/objects, use JSON representation
    }
}
