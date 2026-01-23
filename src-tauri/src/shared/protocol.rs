//! Shared request/response types for REST API and client mode

use serde::{Deserialize, Serialize};

/// Runtime operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationMode {
    /// Local desktop mode (default)
    Local,
    /// Server mode (exposing REST/WebSocket API)
    Server,
    /// Client mode (connecting to remote server)
    Client,
}

impl Default for OperationMode {
    fn default() -> Self {
        Self::Local
    }
}

impl std::fmt::Display for OperationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::Server => write!(f, "server"),
            Self::Client => write!(f, "client"),
        }
    }
}

/// Runtime configuration exposed to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Current operation mode
    pub mode: OperationMode,
    /// Server URL (for client mode)
    pub server_url: Option<String>,
    /// WebSocket URL (for client mode)
    pub ws_url: Option<String>,
}

impl RuntimeConfig {
    pub fn local() -> Self {
        Self {
            mode: OperationMode::Local,
            server_url: None,
            ws_url: None,
        }
    }

    pub fn server(port: u16) -> Self {
        Self {
            mode: OperationMode::Server,
            server_url: Some(format!("http://localhost:{}", port)),
            ws_url: Some(format!("ws://localhost:{}", port)),
        }
    }

    pub fn client(server_url: String) -> Self {
        // Convert HTTP URL to WebSocket URL
        let ws_url = server_url.replace("http://", "ws://").replace("https://", "wss://");

        Self {
            mode: OperationMode::Client,
            server_url: Some(server_url),
            ws_url: Some(ws_url),
        }
    }
}
