//! WebSocket handler for real-time job updates

use std::sync::Arc;
use std::time::Duration;
use axum::{
    extract::{
        State,
        ws::{WebSocket, WebSocketUpgrade, Message},
    },
    response::Response,
};
use futures_util::{StreamExt, SinkExt, stream::SplitSink};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use tokio::sync::mpsc;

use crate::inference::PipelineStage;
use crate::server::state::ServerState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Subscribe { job_id: String },
    Unsubscribe { job_id: String },
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Connected {
        connection_id: String,
    },
    Subscribed {
        job_id: String,
    },
    Unsubscribed {
        job_id: String,
    },
    JobProgress {
        job_id: String,
        status: String,
        progress: f32,
        stage: Option<PipelineStage>,
        stage_progress: Option<f32>,
        message: Option<String>,
        eta_seconds: Option<f32>,
        current_step: Option<u32>,
        total_steps: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        preview_data: Option<String>,
    },
    JobComplete {
        job_id: String,
        result_url: String,
    },
    JobFailed {
        job_id: String,
        error: String,
    },
    Pong,
    Error {
        message: String,
    },
}

/// WebSocket upgrade handler
///
/// GET /api/v1/ws
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle a WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<ServerState>) {
    info!("New WebSocket connection established");

    let (ws_sender, mut ws_receiver) = socket.split();

    // Create an mpsc channel for sending messages to this connection
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Register connection and get connection ID
    let conn_id = state.ws_state().add_connection(tx).await;
    info!(connection_id = %conn_id, "Connection registered");

    // Send connected message
    if let Ok(msg) = serde_json::to_string(&ServerMessage::Connected {
        connection_id: conn_id.clone(),
    }) {
        // We can't await here since ws_sender is moved, so we'll send it through the channel
        // Actually, we need to handle this differently
    }

    // Clone for the write task
    let conn_id_write = conn_id.clone();
    let state_write = state.clone();

    // Spawn a task to handle writing messages to the WebSocket
    let mut write_task = tokio::spawn(async move {
        handle_write(ws_sender, rx, conn_id_write, state_write).await
    });

    // Clone for the read task
    let conn_id_read = conn_id.clone();
    let state_read = state.clone();

    // Spawn a task to handle reading messages from the WebSocket
    let mut read_task = tokio::spawn(async move {
        handle_read(ws_receiver, conn_id_read, state_read).await
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut write_task => {
            read_task.abort();
        }
        _ = &mut read_task => {
            write_task.abort();
        }
    }

    // Clean up connection
    state.ws_state().remove_connection(&conn_id).await;
    info!(connection_id = %conn_id, "Connection cleaned up");
}

/// Handle writing messages to the WebSocket
async fn handle_write(
    mut ws_sender: SplitSink<WebSocket, Message>,
    mut rx: mpsc::UnboundedReceiver<String>,
    conn_id: String,
    _state: Arc<ServerState>,
) {
    // Send welcome message
    if let Ok(msg) = serde_json::to_string(&ServerMessage::Connected {
        connection_id: conn_id.clone(),
    }) {
        if ws_sender.send(Message::Text(msg)).await.is_err() {
            return;
        }
    }

    // Set up heartbeat
    let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            // Receive messages from the channel
            Some(text) = rx.recv() => {
                if ws_sender.send(Message::Text(text)).await.is_err() {
                    error!(connection_id = %conn_id, "Failed to send message");
                    break;
                }
            }

            // Send periodic pings
            _ = heartbeat_interval.tick() => {
                debug!(connection_id = %conn_id, "Sending heartbeat ping");
                if ws_sender.send(Message::Ping(vec![])).await.is_err() {
                    warn!(connection_id = %conn_id, "Failed to send heartbeat");
                    break;
                }
            }
        }
    }

    info!(connection_id = %conn_id, "Write task ended");
}

/// Handle reading messages from the WebSocket
async fn handle_read(
    mut ws_receiver: futures_util::stream::SplitStream<WebSocket>,
    conn_id: String,
    state: Arc<ServerState>,
) {
    while let Some(msg_result) = ws_receiver.next().await {
        match msg_result {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_client_message(&text, &conn_id, &state).await {
                    warn!(
                        connection_id = %conn_id,
                        error = %e,
                        "Failed to handle client message"
                    );

                    // Send error message to client
                    if let Ok(error_msg) = serde_json::to_string(&ServerMessage::Error {
                        message: e.to_string(),
                    }) {
                        let _ = state.ws_state().send_to_connection(&conn_id, &error_msg).await;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!(connection_id = %conn_id, "Client closed connection");
                break;
            }
            Ok(Message::Pong(_)) => {
                debug!(connection_id = %conn_id, "Received pong");
            }
            Err(e) => {
                error!(connection_id = %conn_id, error = %e, "WebSocket error");
                break;
            }
            _ => {}
        }
    }

    info!(connection_id = %conn_id, "Read task ended");
}

/// Handle a client message
async fn handle_client_message(
    text: &str,
    conn_id: &str,
    state: &Arc<ServerState>,
) -> Result<(), String> {
    let client_msg: ClientMessage = serde_json::from_str(text)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    match client_msg {
        ClientMessage::Ping => {
            // Send pong response
            let pong = ServerMessage::Pong;
            let json = serde_json::to_string(&pong)
                .map_err(|e| format!("Failed to serialize pong: {}", e))?;

            state.ws_state().send_to_connection(conn_id, &json).await;
            debug!(connection_id = %conn_id, "Handled ping");
        }

        ClientMessage::Subscribe { job_id } => {
            // Subscribe to job updates
            state.ws_state().subscribe(conn_id, &job_id).await;

            // Send confirmation
            let response = ServerMessage::Subscribed {
                job_id: job_id.clone(),
            };
            let json = serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize response: {}", e))?;

            state.ws_state().send_to_connection(conn_id, &json).await;
            info!(connection_id = %conn_id, job_id = %job_id, "Subscribed to job");

            // Send current job status if available
            if let Some(job) = state.tauri_state().queue_manager.get_job(&job_id).await {
                let status_msg = ServerMessage::JobProgress {
                    job_id: job.id.clone(),
                    status: format!("{:?}", job.status).to_lowercase(),
                    progress: job.progress,
                    stage: None,
                    stage_progress: None,
                    message: None,
                    eta_seconds: None,
                    current_step: None,
                    total_steps: None,
                    preview_data: None,
                };

                if let Ok(json) = serde_json::to_string(&status_msg) {
                    state.ws_state().send_to_connection(conn_id, &json).await;
                }
            }
        }

        ClientMessage::Unsubscribe { job_id } => {
            // Unsubscribe from job updates
            state.ws_state().unsubscribe(conn_id, &job_id).await;

            // Send confirmation
            let response = ServerMessage::Unsubscribed {
                job_id: job_id.clone(),
            };
            let json = serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize response: {}", e))?;

            state.ws_state().send_to_connection(conn_id, &json).await;
            info!(connection_id = %conn_id, job_id = %job_id, "Unsubscribed from job");
        }
    }

    Ok(())
}

/// Helper function to broadcast job updates to subscribed clients
///
/// This should be called by the queue processor when job status changes
pub async fn broadcast_job_update(
    ws_state: &super::ws_state::WsState,
    job_id: &str,
    message: ServerMessage,
) {
    if let Ok(json) = serde_json::to_string(&message) {
        ws_state.broadcast_to_job(job_id, &json).await;
    }
}
