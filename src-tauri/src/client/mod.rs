//! Client mode implementation for connecting to remote servers
//!
//! This module enables the desktop application to connect to a remote
//! RZEM AI Inference server for GPU-accelerated image generation.

pub mod api;

use std::sync::Arc;
use anyhow::Result;
use tracing::info;

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Base URL of the remote server (e.g., "http://192.168.1.100:8080")
    pub server_url: String,
    /// WebSocket URL (derived from server_url)
    pub ws_url: String,
}

impl ClientConfig {
    /// Create a new client configuration
    pub fn new(server_url: String) -> Self {
        // Convert HTTP URL to WebSocket URL
        let ws_url = server_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");

        Self {
            server_url,
            ws_url,
        }
    }

    /// Get the API base URL
    pub fn api_base(&self) -> String {
        format!("{}/api/v1", self.server_url)
    }

    /// Get the WebSocket URL
    pub fn ws_url(&self) -> String {
        format!("{}/api/v1/ws", self.ws_url)
    }
}

/// Initialize client mode
///
/// This verifies the server is reachable and returns the configuration
pub async fn init_client(server_url: String) -> Result<ClientConfig> {
    let config = ClientConfig::new(server_url.clone());

    info!(
        server_url = %server_url,
        "Initializing client mode"
    );

    // Test connection to server
    api::test_connection(&config).await?;

    info!("Client mode initialized successfully");

    Ok(config)
}
