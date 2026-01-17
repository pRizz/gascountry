pub mod connections;
pub mod messages;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::stream::StreamExt;
use futures::SinkExt;
use uuid::Uuid;

pub use connections::ConnectionManager;
pub use messages::{ClientMessage, OutputStream, ServerMessage, SessionStatus};

use crate::api::AppState;

/// Create the WebSocket router
pub fn router() -> Router<AppState> {
    Router::new().route("/ws", get(ws_handler))
}

/// WebSocket upgrade handler
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle an individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: AppState) {
    let connection_id = Uuid::new_v4();
    tracing::info!("WebSocket connection established: {}", connection_id);

    state.connections.register_connection(connection_id).await;

    let (mut sender, mut receiver) = socket.split();

    // Use a channel to send messages from multiple sources to the WebSocket
    let (tx, mut ws_rx) = tokio::sync::mpsc::channel::<ServerMessage>(256);

    // Task to forward from mpsc channel to WebSocket
    let sender_task = tokio::spawn(async move {
        while let Some(msg) = ws_rx.recv().await {
            let json = match serde_json::to_string(&msg) {
                Ok(j) => j,
                Err(e) => {
                    tracing::error!("Failed to serialize message: {}", e);
                    continue;
                }
            };

            if sender.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("WebSocket receive error: {}", e);
                break;
            }
        };

        match msg {
            Message::Text(text) => {
                let client_msg: ClientMessage = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(e) => {
                        let _ = tx
                            .send(ServerMessage::Error {
                                message: format!("Invalid message format: {}", e),
                            })
                            .await;
                        continue;
                    }
                };

                match client_msg {
                    ClientMessage::Subscribe { session_id } => {
                        tracing::info!(
                            "Connection {} subscribing to session {}",
                            connection_id,
                            session_id
                        );

                        // Get a receiver for this session's broadcast channel
                        let mut rx = state.connections.subscribe(connection_id, session_id).await;

                        // Spawn a task to forward messages from this subscription
                        let tx_inner = tx.clone();
                        tokio::spawn(async move {
                            while let Ok(msg) = rx.recv().await {
                                if tx_inner.send(msg).await.is_err() {
                                    break;
                                }
                            }
                        });

                        let _ = tx.send(ServerMessage::Subscribed { session_id }).await;
                    }

                    ClientMessage::Unsubscribe { session_id } => {
                        tracing::info!(
                            "Connection {} unsubscribing from session {}",
                            connection_id,
                            session_id
                        );

                        state
                            .connections
                            .unsubscribe(connection_id, session_id)
                            .await;

                        let _ = tx.send(ServerMessage::Unsubscribed { session_id }).await;
                    }

                    ClientMessage::Cancel { session_id } => {
                        tracing::info!(
                            "Connection {} requesting cancel for session {}",
                            connection_id,
                            session_id
                        );

                        // This will be implemented when RalphManager is added
                        // For now, just broadcast a status update
                        state
                            .connections
                            .broadcast(
                                session_id,
                                ServerMessage::Status {
                                    session_id,
                                    status: SessionStatus::Cancelled,
                                },
                            )
                            .await;
                    }

                    ClientMessage::Ping => {
                        let _ = tx.send(ServerMessage::Pong).await;
                    }
                }
            }

            Message::Close(_) => {
                tracing::info!("WebSocket connection closing: {}", connection_id);
                break;
            }

            _ => {}
        }
    }

    // Cleanup
    sender_task.abort();
    state.connections.unregister_connection(connection_id).await;
    tracing::info!("WebSocket connection closed: {}", connection_id);
}
