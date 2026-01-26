use minijinja::Environment;
use std::collections::HashMap;

use super::types::{RenderError, RenderResult};

/// Render template with minijinja for each row
/// Uses {{ variable }} syntax for substitution
/// Supports filters like {{ var | upper }}, {{ var | default("fallback") }}
pub fn render_template(
    template_string: &str,
    rows: Vec<HashMap<String, String>>,
) -> RenderResult {
    let mut rendered = Vec::new();
    let mut errors = Vec::new();

    // Create minijinja environment
    let mut env = Environment::new();

    // Try to add the template
    if let Err(e) = env.add_template("prompt", template_string) {
        // Template syntax error - all rows will fail
        for row_idx in 0..rows.len() {
            rendered.push(String::new());
            errors.push(RenderError {
                row: row_idx,
                error: format!("Template syntax error: {}", e),
            });
        }
        return RenderResult { rendered, errors };
    }

    // Get the template
    let template = match env.get_template("prompt") {
        Ok(t) => t,
        Err(e) => {
            // Should not happen after successful add_template, but handle it
            for row_idx in 0..rows.len() {
                rendered.push(String::new());
                errors.push(RenderError {
                    row: row_idx,
                    error: format!("Failed to get template: {}", e),
                });
            }
            return RenderResult { rendered, errors };
        }
    };

    // Render each row
    for (row_idx, row) in rows.iter().enumerate() {
        // Convert HashMap to minijinja context
        // HashMap<String, String> implements Serialize, so we can pass it directly
        match template.render(row) {
            Ok(result) => {
                rendered.push(result);
            }
            Err(e) => {
                rendered.push(String::new());
                errors.push(RenderError {
                    row: row_idx,
                    error: format!("{}", e),
                });
            }
        }
    }

    RenderResult { rendered, errors }
}
