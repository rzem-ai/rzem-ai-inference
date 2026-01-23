//! HTTP router configuration
//!
//! Sets up all REST API routes and middleware

use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post, delete},
    response::{IntoResponse, Html},
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use super::state::ServerState;
use super::handlers;

/// Root endpoint - shows API information
async fn root() -> impl IntoResponse {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>RZEM AI Inference Server</title>
    <style>
        body { font-family: monospace; max-width: 800px; margin: 40px auto; padding: 20px; background: #1a1a1a; color: #e0e0e0; }
        h1 { color: #4ade80; }
        h2 { color: #60a5fa; margin-top: 30px; }
        code { background: #2a2a2a; padding: 2px 6px; border-radius: 3px; color: #fbbf24; }
        .endpoint { background: #2a2a2a; padding: 10px; margin: 10px 0; border-radius: 5px; border-left: 3px solid #4ade80; }
        .method { color: #4ade80; font-weight: bold; }
        a { color: #60a5fa; }
    </style>
</head>
<body>
    <h1>🎨 RZEM AI Inference Server</h1>
    <p>Server is running in <strong>Server Mode</strong></p>

    <h2>📡 API Endpoints</h2>

    <div class="endpoint">
        <span class="method">GET</span> <code>/api/v1/health</code>
        <p>Health check endpoint</p>
    </div>

    <div class="endpoint">
        <span class="method">POST</span> <code>/api/v1/generate</code>
        <p>Submit a generation job</p>
    </div>

    <div class="endpoint">
        <span class="method">GET</span> <code>/api/v1/queue</code>
        <p>List all jobs in the queue</p>
    </div>

    <div class="endpoint">
        <span class="method">GET</span> <code>/api/v1/queue/:job_id</code>
        <p>Get specific job status</p>
    </div>

    <div class="endpoint">
        <span class="method">DELETE</span> <code>/api/v1/queue/:job_id</code>
        <p>Cancel a job</p>
    </div>

    <div class="endpoint">
        <span class="method">GET</span> <code>/api/v1/files/:filename</code>
        <p>Download generated image</p>
    </div>

    <div class="endpoint">
        <span class="method">GET</span> <code>/api/v1/system/stats</code>
        <p>System statistics (CPU, RAM, GPU)</p>
    </div>

    <div class="endpoint">
        <span class="method">GET</span> <code>/api/v1/models</code>
        <p>List available models</p>
    </div>

    <div class="endpoint">
        <span class="method">GET</span> <code>/api/v1/ws</code>
        <p>WebSocket endpoint for real-time updates</p>
    </div>

    <h2>🧪 Quick Test</h2>
    <p>Try: <a href="/api/v1/health"><code>GET /api/v1/health</code></a></p>

    <h2>📚 Documentation</h2>
    <p>See <code>API_TESTING_GUIDE.md</code> for examples</p>
</body>
</html>
"#)
}

/// Create the main Axum router with all routes and middleware
pub fn create_router(state: Arc<ServerState>) -> Router {
    Router::new()
        // Root endpoint - API info page
        .route("/", get(root))

        // Health check
        .route("/api/v1/health", get(handlers::system::health))

        // Generation endpoints
        .route("/api/v1/generate", post(handlers::generate::generate))

        // Queue endpoints
        .route("/api/v1/queue", get(handlers::queue::list_jobs))
        .route("/api/v1/queue/:job_id", get(handlers::queue::get_job))
        .route("/api/v1/queue/:job_id", delete(handlers::queue::cancel_job))

        // File serving
        .route("/api/v1/files/:filename", get(handlers::files::get_file))

        // System endpoints
        .route("/api/v1/system/stats", get(handlers::system::get_stats))
        .route("/api/v1/models", get(handlers::system::list_models))

        // WebSocket endpoint
        .route("/api/v1/ws", get(super::websocket::ws_handler))

        // Apply middleware
        .layer(CorsLayer::permissive())  // MVP: Allow all origins
        .layer(TraceLayer::new_for_http())

        // Add state
        .with_state(state)
}
