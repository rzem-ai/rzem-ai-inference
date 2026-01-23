//! WebSocket connection and subscription management

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use serde_json::Value as JsonValue;
use tracing::{info, warn, debug};

/// Unique identifier for a WebSocket connection
pub type ConnectionId = String;

/// Unique identifier for a job
pub type JobId = String;

/// Message sender for a WebSocket connection
pub type MessageSender = mpsc::UnboundedSender<String>;

/// Shared WebSocket state for managing connections and subscriptions
#[derive(Clone)]
pub struct WsState {
    /// Active WebSocket connections: connection_id -> message_sender
    connections: Arc<RwLock<HashMap<ConnectionId, MessageSender>>>,

    /// Job subscriptions: job_id -> [connection_ids]
    subscriptions: Arc<RwLock<HashMap<JobId, Vec<ConnectionId>>>>,
}

impl WsState {
    /// Create a new WebSocket state manager
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new WebSocket connection
    pub async fn add_connection(&self, sender: MessageSender) -> ConnectionId {
        let conn_id = Uuid::new_v4().to_string();

        let mut connections = self.connections.write().await;
        connections.insert(conn_id.clone(), sender);

        info!(connection_id = %conn_id, "WebSocket connection registered");
        conn_id
    }

    /// Remove a WebSocket connection and clean up its subscriptions
    pub async fn remove_connection(&self, conn_id: &str) {
        // Remove from connections
        let mut connections = self.connections.write().await;
        connections.remove(conn_id);

        // Remove from all subscriptions
        let mut subscriptions = self.subscriptions.write().await;
        for (job_id, conn_ids) in subscriptions.iter_mut() {
            conn_ids.retain(|id| id != conn_id);
        }

        // Clean up empty subscription lists
        subscriptions.retain(|_, conn_ids| !conn_ids.is_empty());

        info!(connection_id = %conn_id, "WebSocket connection removed");
    }

    /// Subscribe a connection to job updates
    pub async fn subscribe(&self, conn_id: &str, job_id: &str) {
        let mut subscriptions = self.subscriptions.write().await;

        subscriptions
            .entry(job_id.to_string())
            .or_insert_with(Vec::new)
            .push(conn_id.to_string());

        info!(
            connection_id = %conn_id,
            job_id = %job_id,
            "Subscribed to job"
        );
    }

    /// Unsubscribe a connection from job updates
    pub async fn unsubscribe(&self, conn_id: &str, job_id: &str) {
        let mut subscriptions = self.subscriptions.write().await;

        if let Some(conn_ids) = subscriptions.get_mut(job_id) {
            conn_ids.retain(|id| id != conn_id);

            // Clean up empty subscription list
            if conn_ids.is_empty() {
                subscriptions.remove(job_id);
            }
        }

        info!(
            connection_id = %conn_id,
            job_id = %job_id,
            "Unsubscribed from job"
        );
    }

    /// Broadcast a message to all connections subscribed to a job
    pub async fn broadcast_to_job(&self, job_id: &str, message: &str) {
        let subscriptions = self.subscriptions.read().await;
        let connections = self.connections.read().await;

        if let Some(conn_ids) = subscriptions.get(job_id) {
            let mut failed_connections = Vec::new();

            for conn_id in conn_ids {
                if let Some(sender) = connections.get(conn_id) {
                    if sender.send(message.to_string()).is_err() {
                        warn!(
                            connection_id = %conn_id,
                            "Failed to send message, marking for removal"
                        );
                        failed_connections.push(conn_id.clone());
                    }
                } else {
                    failed_connections.push(conn_id.clone());
                }
            }

            // Clean up failed connections
            drop(subscriptions);
            drop(connections);

            for conn_id in failed_connections {
                self.remove_connection(&conn_id).await;
            }
        }
    }

    /// Send a message to a specific connection
    pub async fn send_to_connection(&self, conn_id: &str, message: &str) -> bool {
        let connections = self.connections.read().await;

        if let Some(sender) = connections.get(conn_id) {
            sender.send(message.to_string()).is_ok()
        } else {
            false
        }
    }

    /// Get the number of active connections
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Get the number of active subscriptions
    pub async fn subscription_count(&self) -> usize {
        self.subscriptions.read().await.len()
    }

    /// Get subscriptions for a specific job
    pub async fn get_job_subscribers(&self, job_id: &str) -> Vec<ConnectionId> {
        self.subscriptions
            .read()
            .await
            .get(job_id)
            .cloned()
            .unwrap_or_default()
    }
}

impl Default for WsState {
    fn default() -> Self {
        Self::new()
    }
}
