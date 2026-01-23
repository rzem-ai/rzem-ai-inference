//! HTTP router configuration
//!
//! Sets up all REST API routes and middleware

use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post, delete},
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use super::state::ServerState;
use super::handlers;

/// Create the main Axum router with all routes and middleware
pub fn create_router(state: Arc<ServerState>) -> Router {
    Router::new()
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
