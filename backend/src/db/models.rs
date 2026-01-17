use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Repository model representing a git repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo {
    pub id: Uuid,
    pub path: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Session status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Idle,
    Running,
    Completed,
    Error,
    Cancelled,
}

impl SessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionStatus::Idle => "idle",
            SessionStatus::Running => "running",
            SessionStatus::Completed => "completed",
            SessionStatus::Error => "error",
            SessionStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "idle" => Ok(SessionStatus::Idle),
            "running" => Ok(SessionStatus::Running),
            "completed" => Ok(SessionStatus::Completed),
            "error" => Ok(SessionStatus::Error),
            "cancelled" => Ok(SessionStatus::Cancelled),
            _ => Err(format!("invalid session status: '{}'", s)),
        }
    }
}

/// Session model representing a Ralph session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: Option<String>,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Message role enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl MessageRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "user" => Ok(MessageRole::User),
            "assistant" => Ok(MessageRole::Assistant),
            "system" => Ok(MessageRole::System),
            _ => Err(format!("invalid message role: '{}'", s)),
        }
    }
}

/// Message model representing a chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// Output stream type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputStream {
    Stdout,
    Stderr,
}

impl OutputStream {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutputStream::Stdout => "stdout",
            OutputStream::Stderr => "stderr",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "stdout" => Ok(OutputStream::Stdout),
            "stderr" => Ok(OutputStream::Stderr),
            _ => Err(format!("invalid output stream: '{}'", s)),
        }
    }
}

/// Output log entry from Ralph process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputLog {
    pub id: i64,
    pub session_id: Uuid,
    pub stream: OutputStream,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// Configuration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}
