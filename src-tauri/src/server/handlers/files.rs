//! File serving endpoints with security validation

use std::path::{Path, PathBuf};
use std::sync::Arc;
use axum::{
    body::Body,
    extract::{State, Path as AxumPath},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tracing::{warn, error};

use crate::server::state::ServerState;

/// Security error types
#[derive(Debug)]
enum SecurityError {
    PathTraversal,
    InvalidExtension,
    FileNotFound,
}

/// Validate filename for security
///
/// - Rejects path traversal attempts
/// - Only allows PNG/JPG files
/// - Validates file exists
fn validate_filename(filename: &str) -> Result<PathBuf, SecurityError> {
    // Reject path traversal
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        warn!(filename = %filename, "Path traversal attempt blocked");
        return Err(SecurityError::PathTraversal);
    }

    // Only allow image files
    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    if !["png", "jpg", "jpeg"].contains(&ext) {
        warn!(filename = %filename, "Invalid file extension blocked");
        return Err(SecurityError::InvalidExtension);
    }

    // Build path to output directory
    let home = dirs::home_dir()
        .ok_or_else(|| SecurityError::FileNotFound)?;
    let output_dir = home.join(".rzem-ai-inference").join("outputs");
    let file_path = output_dir.join(filename);

    // Verify file exists
    if !file_path.exists() {
        return Err(SecurityError::FileNotFound);
    }

    Ok(file_path)
}

/// Serve a generated image file
///
/// GET /api/v1/files/:filename
pub async fn get_file(
    State(_state): State<Arc<ServerState>>,
    AxumPath(filename): AxumPath<String>,
) -> Result<Response, StatusCode> {
    // Validate and get file path
    let file_path = match validate_filename(&filename) {
        Ok(path) => path,
        Err(SecurityError::PathTraversal) => return Err(StatusCode::BAD_REQUEST),
        Err(SecurityError::InvalidExtension) => return Err(StatusCode::BAD_REQUEST),
        Err(SecurityError::FileNotFound) => return Err(StatusCode::NOT_FOUND),
    };

    // Open file
    let file = match File::open(&file_path).await {
        Ok(f) => f,
        Err(e) => {
            error!(error = %e, path = ?file_path, "Failed to open file");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Determine content type
    let content_type = if filename.ends_with(".png") {
        "image/png"
    } else {
        "image/jpeg"
    };

    // Stream file to client
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .body(body)
        .unwrap())
}
