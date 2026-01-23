//! Server mode implementation for GPU sharing
//!
//! This module enables the application to run as a server, exposing REST and WebSocket APIs
//! for remote clients to access inference capabilities.

pub mod router;
pub mod state;
pub mod handlers;
pub mod websocket;
pub mod ws_state;

use std::net::SocketAddr;
use std::sync::Arc;
use axum::Router;
use tracing::{info, error};

use crate::AppState as TauriAppState;

pub use state::ServerState;

/// Start the HTTP/WebSocket server
///
/// # Arguments
/// * `app_state` - The full application state (includes WsState)
/// * `port` - Port to bind the server to
/// * `headless` - Whether to run without desktop UI
///
/// # Returns
/// Server address if successful
pub async fn start_server(
    app_state: Arc<TauriAppState>,
    port: u16,
    headless: bool,
) -> Result<SocketAddr, Box<dyn std::error::Error + Send + Sync>> {
    info!(
        port = port,
        headless = headless,
        "Starting server mode"
    );

    // Get WebSocket state from app state
    let ws_state = app_state.ws_state.as_ref()
        .expect("WsState must be initialized in server mode")
        .as_ref()
        .clone();

    // Create server state
    let server_state = Arc::new(ServerState::new(app_state, ws_state));

    // Build router
    let app = router::create_router(server_state);

    // Bind to address
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let actual_addr = listener.local_addr()?;

    info!(
        address = %actual_addr,
        "Server listening"
    );

    // Start server in background
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!(error = %e, "Server error");
        }
    });

    Ok(actual_addr)
}
