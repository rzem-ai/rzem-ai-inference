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

/**
 * Batch processing mode
 * Rust: BatchMode
 */
export type BatchMode = 'as-is' | 'combinatorial';

/**
 * Complete batch configuration state for stepper
 */
export interface BatchConfig {
  /** Processing mode (as-is or combinatorial) */
  mode: BatchMode;
  /** Original source data from file */
  sourceData: BatchData;
  /** Processed data after mode transformation */
  processedData: BatchData;
  /** Template string for rendering prompts */
  template: string;
}

/**
 * Template history entry
 * Rust: TemplateHistoryEntry
 */
export interface TemplateHistoryEntry {
  /** Unique ID */
  id: number;
  /** Template string */
  template: string;
  /** ISO 8601 timestamp when template was last used */
  used_at: string;
  /** Number of images generated with this template */
  image_count: number;
}

/**
 * Payload emitted when data is loaded from FileInputSection
 */
export interface DataLoadedEvent {
  /** Parsed batch data */
  data: BatchData;
  /** Name of the data source (filename or "Pasted Data") */
  filename: string;
}
