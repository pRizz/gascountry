use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Subscribe to output from a session
    Subscribe { session_id: Uuid },
    /// Unsubscribe from a session
    Unsubscribe { session_id: Uuid },
    /// Cancel a running session
    Cancel { session_id: Uuid },
    /// Ping to keep connection alive
    Ping,
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Acknowledgment of subscription
    Subscribed { session_id: Uuid },
    /// Acknowledgment of unsubscription
    Unsubscribed { session_id: Uuid },
    /// Output line from a session (stdout or stderr)
    Output {
        session_id: Uuid,
        stream: OutputStream,
        content: String,
    },
    /// Session status changed
    Status {
        session_id: Uuid,
        status: SessionStatus,
    },
    /// Error message
    Error { message: String },
    /// Pong response to ping
    Pong,
}

/// Output stream type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputStream {
    Stdout,
    Stderr,
}

/// Session status for WebSocket updates
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Idle,
    Running,
    Completed,
    Error,
    Cancelled,
}

impl From<crate::db::models::SessionStatus> for SessionStatus {
    fn from(status: crate::db::models::SessionStatus) -> Self {
        match status {
            crate::db::models::SessionStatus::Idle => SessionStatus::Idle,
            crate::db::models::SessionStatus::Running => SessionStatus::Running,
            crate::db::models::SessionStatus::Completed => SessionStatus::Completed,
            crate::db::models::SessionStatus::Error => SessionStatus::Error,
            crate::db::models::SessionStatus::Cancelled => SessionStatus::Cancelled,
        }
    }
}

impl From<crate::db::models::OutputStream> for OutputStream {
    fn from(stream: crate::db::models::OutputStream) -> Self {
        match stream {
            crate::db::models::OutputStream::Stdout => OutputStream::Stdout,
            crate::db::models::OutputStream::Stderr => OutputStream::Stderr,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_serialize() {
        let msg = ClientMessage::Subscribe {
            session_id: Uuid::nil(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"subscribe\""));
        assert!(json.contains("\"session_id\""));
    }

    #[test]
    fn test_server_message_serialize() {
        let msg = ServerMessage::Output {
            session_id: Uuid::nil(),
            stream: OutputStream::Stdout,
            content: "Hello".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"output\""));
        assert!(json.contains("\"stream\":\"stdout\""));
    }

    #[test]
    fn test_client_message_deserialize() {
        let json = r#"{"type":"subscribe","session_id":"00000000-0000-0000-0000-000000000000"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        matches!(msg, ClientMessage::Subscribe { .. });
    }
}
