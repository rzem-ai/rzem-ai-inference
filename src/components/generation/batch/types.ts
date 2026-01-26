/**
 * TypeScript types for batch scripting
 * Must match Rust types in src-tauri/src/batch/types.rs
 */

/**
 * Parsed data from CSV or JSON file
 * Rust: BatchData
 */
export interface BatchData {
  /** Column names (from CSV headers or JSON object keys) */
  columns: string[];
  /** Rows of data, each row is a map of column_name -> value */
  rows: Record<string, string>[];
}

/**
 * Result of template rendering
 * Rust: RenderResult
 */
export interface RenderResult {
  /** Successfully rendered prompts (empty string for error rows) */
  rendered: string[];
  /** Errors that occurred during rendering (row index, error message) */
  errors: RenderError[];
}

/**
 * Template rendering error for a specific row
 * Rust: RenderError
 */
export interface RenderError {
  /** Zero-based row index */
  row: number;
  /** Error message */
  error: string;
}

/**
 * Preview row for display in table
 */
export interface PreviewRow {
  /** Row number (1-indexed for display) */
  rowNumber: number;
  /** Rendered prompt (empty if error) */
  prompt: string;
  /** Original data values */
  data: Record<string, string>;
  /** Error message if rendering failed */
  error?: string;
}
