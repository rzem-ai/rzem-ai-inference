use crate::batch::types::{BatchData, BatchRow};
use std::collections::HashMap;
use anyhow::{Result, Context};

/// Generate Cartesian product of all unique values per column
pub fn generate_combinations(data: BatchData) -> Result<BatchData> {
    if data.rows.is_empty() {
        return Ok(data);
    }

    // Extract unique values per column
    let mut column_values: HashMap<String, Vec<String>> = HashMap::new();

    for col in &data.columns {
        let mut unique_values = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for row in &data.rows {
            if let Some(value) = row.get(col) {
                if seen.insert(value.clone()) {
                    unique_values.push(value.clone());
                }
            }
        }

        column_values.insert(col.clone(), unique_values);
    }

    // Generate Cartesian product
    let combinations = cartesian_product(&data.columns, &column_values)?;

    Ok(BatchData {
        columns: data.columns.clone(),
        rows: combinations,
    })
}

fn cartesian_product(
    columns: &[String],
    column_values: &HashMap<String, Vec<String>>,
) -> Result<Vec<BatchRow>> {
    if columns.is_empty() {
        return Ok(vec![HashMap::new()]);
    }

    let first_col = &columns[0];
    let first_values = column_values
        .get(first_col)
        .context("Missing column in values map")?;

    if columns.len() == 1 {
        // Base case: single column
        return Ok(first_values
            .iter()
            .map(|v| {
                let mut row = HashMap::new();
                row.insert(first_col.clone(), v.clone());
                row
            })
            .collect());
    }

    // Recursive case: combine first column with rest
    let rest_combinations = cartesian_product(&columns[1..], column_values)?;

    let mut result = Vec::new();
    for value in first_values {
        for rest_row in &rest_combinations {
            let mut new_row = rest_row.clone();
            new_row.insert(first_col.clone(), value.clone());
            result.push(new_row);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_columns_two_values_each() {
        let data = BatchData {
            columns: vec!["style".to_string(), "subject".to_string()],
            rows: vec![
                [("style", "watercolor"), ("subject", "cat")]
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                [("style", "oil"), ("subject", "dog")]
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            ],
        };

        let result = generate_combinations(data).unwrap();
        assert_eq!(result.rows.len(), 4); // 2 × 2 = 4

        // Verify all combinations exist
        let has_combo = |style: &str, subject: &str| {
            result.rows.iter().any(|row| {
                row.get("style") == Some(&style.to_string())
                    && row.get("subject") == Some(&subject.to_string())
            })
        };

        assert!(has_combo("watercolor", "cat"));
        assert!(has_combo("watercolor", "dog"));
        assert!(has_combo("oil", "cat"));
        assert!(has_combo("oil", "dog"));
    }

    #[test]
    fn test_three_columns_varying_values() {
        let data = BatchData {
            columns: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            rows: vec![
                [("a", "1"), ("b", "x"), ("c", "red")]
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                [("a", "2"), ("b", "y"), ("c", "red")]
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            ],
        };

        let result = generate_combinations(data).unwrap();
        assert_eq!(result.rows.len(), 4); // 2 × 2 × 1 = 4
    }

    #[test]
    fn test_empty_data() {
        let data = BatchData {
            columns: vec!["col1".to_string()],
            rows: vec![],
        };

        let result = generate_combinations(data).unwrap();
        assert_eq!(result.rows.len(), 0);
    }
}
