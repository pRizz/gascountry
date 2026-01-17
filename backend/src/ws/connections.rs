use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use super::messages::ServerMessage;

/// Capacity of the broadcast channel per session
const CHANNEL_CAPACITY: usize = 256;

/// Manages WebSocket connections and session subscriptions
#[derive(Clone)]
pub struct ConnectionManager {
    inner: Arc<RwLock<ConnectionManagerInner>>,
}

struct ConnectionManagerInner {
    /// Map of session_id -> broadcast sender for that session
    session_channels: HashMap<Uuid, broadcast::Sender<ServerMessage>>,
    /// Map of connection_id -> set of subscribed session_ids
    connection_subscriptions: HashMap<Uuid, HashSet<Uuid>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ConnectionManagerInner {
                session_channels: HashMap::new(),
                connection_subscriptions: HashMap::new(),
            })),
        }
    }

    /// Register a new connection
    pub async fn register_connection(&self, connection_id: Uuid) {
        let mut inner = self.inner.write().await;
        inner
            .connection_subscriptions
            .insert(connection_id, HashSet::new());
    }

    /// Unregister a connection and clean up its subscriptions
    pub async fn unregister_connection(&self, connection_id: Uuid) {
        let mut inner = self.inner.write().await;
        if let Some(subscriptions) = inner.connection_subscriptions.remove(&connection_id) {
            // Clean up empty channels
            for session_id in subscriptions {
                if let Some(sender) = inner.session_channels.get(&session_id) {
                    // If no receivers left, remove the channel
                    if sender.receiver_count() == 0 {
                        inner.session_channels.remove(&session_id);
                    }
                }
            }
        }
    }

    /// Subscribe a connection to a session's output
    /// Returns a receiver for the session's broadcast channel
    pub async fn subscribe(
        &self,
        connection_id: Uuid,
        session_id: Uuid,
    ) -> broadcast::Receiver<ServerMessage> {
        let mut inner = self.inner.write().await;

        // Track subscription for this connection
        if let Some(subs) = inner.connection_subscriptions.get_mut(&connection_id) {
            subs.insert(session_id);
        }

        // Get or create the broadcast channel for this session
        let sender = inner
            .session_channels
            .entry(session_id)
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0);

        sender.subscribe()
    }

    /// Unsubscribe a connection from a session
    pub async fn unsubscribe(&self, connection_id: Uuid, session_id: Uuid) {
        let mut inner = self.inner.write().await;

        // Remove from connection's subscription set
        if let Some(subs) = inner.connection_subscriptions.get_mut(&connection_id) {
            subs.remove(&session_id);
        }
    }

    /// Broadcast a message to all subscribers of a session
    pub async fn broadcast(&self, session_id: Uuid, message: ServerMessage) {
        let inner = self.inner.read().await;
        if let Some(sender) = inner.session_channels.get(&session_id) {
            // Ignore send errors (no receivers)
            let _ = sender.send(message);
        }
    }

    /// Get or create a broadcast sender for a session
    /// Used by RalphManager to send output to subscribers
    pub async fn get_session_sender(&self, session_id: Uuid) -> broadcast::Sender<ServerMessage> {
        let mut inner = self.inner.write().await;
        inner
            .session_channels
            .entry(session_id)
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0)
            .clone()
    }

    /// Check if a session has any subscribers
    pub async fn has_subscribers(&self, session_id: Uuid) -> bool {
        let inner = self.inner.read().await;
        inner
            .session_channels
            .get(&session_id)
            .is_some_and(|s| s.receiver_count() > 0)
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws::messages::OutputStream;

    #[tokio::test]
    async fn test_subscribe_and_broadcast() {
        let manager = ConnectionManager::new();
        let connection_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        manager.register_connection(connection_id).await;
        let mut receiver = manager.subscribe(connection_id, session_id).await;

        let msg = ServerMessage::Output {
            session_id,
            stream: OutputStream::Stdout,
            content: "Hello".to_string(),
        };

        manager.broadcast(session_id, msg.clone()).await;

        let received = receiver.recv().await.unwrap();
        match received {
            ServerMessage::Output { content, .. } => assert_eq!(content, "Hello"),
            _ => panic!("Unexpected message type"),
        }
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let manager = ConnectionManager::new();
        let connection_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        manager.register_connection(connection_id).await;
        let _receiver = manager.subscribe(connection_id, session_id).await;

        assert!(manager.has_subscribers(session_id).await);

        manager.unsubscribe(connection_id, session_id).await;
        // Note: receiver is still held, so channel still has receivers
        // The actual cleanup happens when the receiver is dropped
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let manager = ConnectionManager::new();
        let conn1 = Uuid::new_v4();
        let conn2 = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        manager.register_connection(conn1).await;
        manager.register_connection(conn2).await;

        let mut receiver1 = manager.subscribe(conn1, session_id).await;
        let mut receiver2 = manager.subscribe(conn2, session_id).await;

        let msg = ServerMessage::Output {
            session_id,
            stream: OutputStream::Stdout,
            content: "Hello both".to_string(),
        };

        manager.broadcast(session_id, msg).await;

        // Both receivers should get the message
        let r1 = receiver1.recv().await.unwrap();
        let r2 = receiver2.recv().await.unwrap();

        match (r1, r2) {
            (ServerMessage::Output { content: c1, .. }, ServerMessage::Output { content: c2, .. }) => {
                assert_eq!(c1, "Hello both");
                assert_eq!(c2, "Hello both");
            }
            _ => panic!("Unexpected message types"),
        }
    }

    #[tokio::test]
    async fn test_connection_cleanup() {
        let manager = ConnectionManager::new();
        let connection_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        manager.register_connection(connection_id).await;
        let receiver = manager.subscribe(connection_id, session_id).await;

        assert!(manager.has_subscribers(session_id).await);

        // Drop receiver before unregistering
        drop(receiver);

        manager.unregister_connection(connection_id).await;

        // Channel should be cleaned up since no receivers
        assert!(!manager.has_subscribers(session_id).await);
    }
}
