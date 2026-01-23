//! Shared server state
//!
//! Wraps the Tauri AppState for use in Axum handlers

use std::sync::Arc;
use crate::AppState as TauriAppState;
use super::ws_state::WsState;

/// Server state that wraps Tauri's AppState
///
/// This allows Axum handlers to access the same state as Tauri commands
#[derive(Clone)]
pub struct ServerState {
    tauri_state: Arc<TauriAppState>,
    ws_state: WsState,
}

impl ServerState {
    pub fn new(tauri_state: Arc<TauriAppState>, ws_state: WsState) -> Self {
        Self {
            tauri_state,
            ws_state,
        }
    }

    /// Get a reference to the underlying Tauri state
    pub fn tauri_state(&self) -> &Arc<TauriAppState> {
        &self.tauri_state
    }

    /// Get a reference to the WebSocket state
    pub fn ws_state(&self) -> &WsState {
        &self.ws_state
    }
}
