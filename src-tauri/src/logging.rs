//! Logging configuration using tracing
//!
//! Provides structured, async-aware logging with configurable levels.
//!
//! # Log Levels
//! - `error`: Critical errors that prevent operation
//! - `warn`: Unexpected situations that don't prevent operation
//! - `info`: High-level operational messages (default)
//! - `debug`: Detailed debugging information
//! - `trace`: Very detailed tracing information
//!
//! # Configuration
//! Set `RUST_LOG` environment variable to control log levels:
//! - `RUST_LOG=debug` - Enable debug for all modules
//! - `RUST_LOG=rzem_ai_inference=debug` - Enable debug for this crate only
//! - `RUST_LOG=rzem_ai_inference::inference=trace` - Trace for inference module

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize the logging system
///
/// Call this once at application startup. Safe to call multiple times
/// (subsequent calls are no-ops).
///
/// # Arguments
/// * `default_level` - Default log level if RUST_LOG is not set (e.g., "info", "debug")
pub fn init_logging(default_level: &str) {
    // Build filter from RUST_LOG env var, falling back to default
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_level));

    // Build subscriber with formatting
    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_target(true)           // Show module path
                .with_thread_ids(false)      // Hide thread IDs
                .with_thread_names(false)    // Hide thread names
                .with_file(false)            // Hide file names
                .with_line_number(false)     // Hide line numbers
                .compact()                   // Compact format
        );

    // Try to set global subscriber (ignore error if already set)
    let _ = tracing::subscriber::set_global_default(subscriber);
}

/// Initialize logging with verbose output (for CLI)
///
/// Includes file and line numbers for easier debugging.
#[allow(dead_code)]
pub fn init_logging_verbose(default_level: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_level));

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(true)             // Show file names
                .with_line_number(true)      // Show line numbers
        );

    let _ = tracing::subscriber::set_global_default(subscriber);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logging() {
        // Should not panic even when called multiple times
        init_logging("info");
        init_logging("debug");
    }
}
