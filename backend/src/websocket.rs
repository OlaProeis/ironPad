use std::collections::HashSet;
use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};

use crate::services::locks::{FileLockManager, LockType};

/// WebSocket message types sent to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum WsMessage {
    /// A file was created
    FileCreated { path: String },
    /// A file was modified
    FileModified { path: String },
    /// A file was deleted
    FileDeleted { path: String },
    /// A file was renamed
    FileRenamed { from: String, to: String },
    /// A file was locked
    FileLocked {
        path: String,
        client_id: String,
        lock_type: String,
    },
    /// A file was unlocked
    FileUnlocked { path: String },
    /// Git conflict detected
    GitConflict { files: Vec<String> },
    /// Server is sending a ping
    Ping,
    /// Client connection confirmed
    Connected { client_id: String },
    /// Error message
    Error { message: String },
}

/// Client message types received from clients
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Request to lock a file
    #[serde(rename = "lock_file")]
    LockFile { path: String, lock_type: String },
    /// Request to unlock a file
    #[serde(rename = "unlock_file")]
    UnlockFile { path: String },
    /// Ping response
    #[serde(rename = "pong")]
    Pong,
}

/// Shared state for WebSocket connections
#[derive(Debug, Clone)]
pub struct WsState {
    /// Broadcast channel for sending messages to all clients
    pub tx: broadcast::Sender<WsMessage>,
    /// Set of connected client IDs
    pub clients: Arc<RwLock<HashSet<String>>>,
    /// File lock manager
    pub lock_manager: FileLockManager,
}

impl WsState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            tx,
            clients: Arc::new(RwLock::new(HashSet::new())),
            lock_manager: FileLockManager::new(),
        }
    }

    /// Broadcast a message to all connected clients
    pub fn broadcast(&self, msg: WsMessage) {
        // Ignore send errors (no receivers)
        let _ = self.tx.send(msg);
    }
}

impl Default for WsState {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<WsState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<WsState>) {
    let client_id = uuid::Uuid::new_v4().to_string();

    // Add client to set
    {
        let mut clients = state.clients.write().await;
        clients.insert(client_id.clone());
    }

    tracing::info!("WebSocket client connected: {}", client_id);

    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast channel
    let mut rx = state.tx.subscribe();

    // Send connected message
    let connected_msg = WsMessage::Connected {
        client_id: client_id.clone(),
    };
    if let Ok(json) = serde_json::to_string(&connected_msg) {
        let _ = sender.send(Message::Text(json.into())).await;
    }

    // Spawn task to forward broadcast messages to this client
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages from client
    let state_clone = state.clone();
    let client_id_clone = client_id.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        handle_client_message(&state_clone, &client_id_clone, client_msg).await;
                    } else {
                        tracing::debug!("Unknown message from {}: {}", client_id_clone, text);
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    // Clean up on disconnect
    // Release all locks held by this client
    let released_paths = state.lock_manager.release_all_for_client(&client_id).await;
    for path in released_paths {
        state.broadcast(WsMessage::FileUnlocked { path });
    }

    // Remove client from set
    {
        let mut clients = state.clients.write().await;
        clients.remove(&client_id);
    }

    tracing::info!("WebSocket client disconnected: {}", client_id);
}

/// Handle a message from a client
async fn handle_client_message(state: &Arc<WsState>, client_id: &str, msg: ClientMessage) {
    match msg {
        ClientMessage::LockFile { path, lock_type } => {
            let lock_type = match lock_type.as_str() {
                "editor" => LockType::Editor,
                "task_view" => LockType::TaskView,
                _ => {
                    tracing::warn!("Unknown lock type: {}", lock_type);
                    return;
                }
            };

            match state
                .lock_manager
                .acquire(&path, client_id, lock_type)
                .await
            {
                Ok(lock_info) => {
                    let lock_type_str = match lock_info.lock_type {
                        LockType::Editor => "editor",
                        LockType::TaskView => "task_view",
                    };
                    state.broadcast(WsMessage::FileLocked {
                        path: lock_info.path,
                        client_id: lock_info.client_id,
                        lock_type: lock_type_str.to_string(),
                    });
                }
                Err(e) => {
                    tracing::warn!("Failed to acquire lock: {}", e);
                    // Could send error back to specific client if needed
                }
            }
        }
        ClientMessage::UnlockFile { path } => {
            match state.lock_manager.release(&path, client_id).await {
                Ok(()) => {
                    state.broadcast(WsMessage::FileUnlocked { path });
                }
                Err(e) => {
                    tracing::warn!("Failed to release lock: {}", e);
                }
            }
        }
        ClientMessage::Pong => {
            // Heartbeat response, no action needed
        }
    }
}
