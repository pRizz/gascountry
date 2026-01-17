pub mod models;
pub mod schema;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use rusqlite::{params, Connection};
use thiserror::Error;
use uuid::Uuid;

use models::{Message, MessageRole, Repo, Session, SessionStatus};
use schema::{CREATE_TABLES, GET_SCHEMA_VERSION, SCHEMA_VERSION, UPSERT_SCHEMA_VERSION};

/// Database error types
#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Failed to determine data directory")]
    NoDataDir,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Record not found")]
    NotFound,

    #[error("Invalid data: {0}")]
    InvalidData(String),
}

pub type DbResult<T> = Result<T, DbError>;

/// Database wrapper with connection management
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Create a new database connection, initializing schema if needed
    pub fn new(path: PathBuf) -> DbResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.init_schema()?;
        Ok(db)
    }

    /// Create an in-memory database (for testing)
    pub fn in_memory() -> DbResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.init_schema()?;
        Ok(db)
    }

    /// Get the default database path based on platform
    pub fn default_path() -> DbResult<PathBuf> {
        let data_dir = dirs::data_dir().ok_or(DbError::NoDataDir)?;
        Ok(data_dir.join("ralphtown").join("ralphtown.db"))
    }

    /// Initialize database schema
    fn init_schema(&self) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();

        // Create tables
        conn.execute_batch(CREATE_TABLES)?;

        // Check and update schema version
        let current_version: Option<i32> = conn
            .query_row(GET_SCHEMA_VERSION, [], |row| row.get(0))
            .ok();

        if current_version.unwrap_or(0) < SCHEMA_VERSION {
            conn.execute(UPSERT_SCHEMA_VERSION, params![SCHEMA_VERSION])?;
        }

        Ok(())
    }

    // ==================== Repo Operations ====================

    /// Insert a new repository
    pub fn insert_repo(&self, path: &str, name: &str) -> DbResult<Repo> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now();
        let id = Uuid::new_v4();

        conn.execute(
            "INSERT INTO repos (id, path, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id.to_string(),
                path,
                name,
                now.to_rfc3339(),
                now.to_rfc3339()
            ],
        )?;

        Ok(Repo {
            id,
            path: path.to_string(),
            name: name.to_string(),
            created_at: now,
            updated_at: now,
        })
    }

    /// Get a repository by ID
    pub fn get_repo(&self, id: Uuid) -> DbResult<Repo> {
        let conn = self.conn.lock().unwrap();

        conn.query_row(
            "SELECT id, path, name, created_at, updated_at FROM repos WHERE id = ?1",
            params![id.to_string()],
            |row| {
                Ok(Repo {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    path: row.get(1)?,
                    name: row.get(2)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::NotFound,
            _ => DbError::Sqlite(e),
        })
    }

    /// Get a repository by path
    pub fn get_repo_by_path(&self, path: &str) -> DbResult<Repo> {
        let conn = self.conn.lock().unwrap();

        conn.query_row(
            "SELECT id, path, name, created_at, updated_at FROM repos WHERE path = ?1",
            params![path],
            |row| {
                Ok(Repo {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    path: row.get(1)?,
                    name: row.get(2)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::NotFound,
            _ => DbError::Sqlite(e),
        })
    }

    /// List all repositories
    pub fn list_repos(&self) -> DbResult<Vec<Repo>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT id, path, name, created_at, updated_at FROM repos ORDER BY name")?;

        let repos = stmt
            .query_map([], |row| {
                Ok(Repo {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    path: row.get(1)?,
                    name: row.get(2)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(repos)
    }

    /// Delete a repository by ID
    pub fn delete_repo(&self, id: Uuid) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute("DELETE FROM repos WHERE id = ?1", params![id.to_string()])?;

        if affected == 0 {
            return Err(DbError::NotFound);
        }
        Ok(())
    }

    // ==================== Session Operations ====================

    /// Insert a new session
    pub fn insert_session(&self, repo_id: Uuid, name: Option<&str>) -> DbResult<Session> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now();
        let id = Uuid::new_v4();

        conn.execute(
            "INSERT INTO sessions (id, repo_id, name, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id.to_string(),
                repo_id.to_string(),
                name,
                SessionStatus::Idle.as_str(),
                now.to_rfc3339(),
                now.to_rfc3339()
            ],
        )?;

        Ok(Session {
            id,
            repo_id,
            name: name.map(String::from),
            status: SessionStatus::Idle,
            created_at: now,
            updated_at: now,
        })
    }

    /// Get a session by ID
    pub fn get_session(&self, id: Uuid) -> DbResult<Session> {
        let conn = self.conn.lock().unwrap();

        conn.query_row(
            "SELECT id, repo_id, name, status, created_at, updated_at FROM sessions WHERE id = ?1",
            params![id.to_string()],
            |row| {
                Ok(Session {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    repo_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                    name: row.get(2)?,
                    status: SessionStatus::from_str(&row.get::<_, String>(3)?).unwrap(),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::NotFound,
            _ => DbError::Sqlite(e),
        })
    }

    /// List all sessions
    pub fn list_sessions(&self) -> DbResult<Vec<Session>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, repo_id, name, status, created_at, updated_at FROM sessions ORDER BY updated_at DESC",
        )?;

        let sessions = stmt
            .query_map([], |row| {
                Ok(Session {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    repo_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                    name: row.get(2)?,
                    status: SessionStatus::from_str(&row.get::<_, String>(3)?).unwrap(),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// List sessions for a specific repository
    pub fn list_sessions_by_repo(&self, repo_id: Uuid) -> DbResult<Vec<Session>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, repo_id, name, status, created_at, updated_at FROM sessions WHERE repo_id = ?1 ORDER BY updated_at DESC",
        )?;

        let sessions = stmt
            .query_map(params![repo_id.to_string()], |row| {
                Ok(Session {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    repo_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                    name: row.get(2)?,
                    status: SessionStatus::from_str(&row.get::<_, String>(3)?).unwrap(),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Update session status
    pub fn update_session_status(&self, id: Uuid, status: SessionStatus) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now();

        let affected = conn.execute(
            "UPDATE sessions SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), now.to_rfc3339(), id.to_string()],
        )?;

        if affected == 0 {
            return Err(DbError::NotFound);
        }
        Ok(())
    }

    /// Delete a session by ID
    pub fn delete_session(&self, id: Uuid) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        let affected =
            conn.execute("DELETE FROM sessions WHERE id = ?1", params![id.to_string()])?;

        if affected == 0 {
            return Err(DbError::NotFound);
        }
        Ok(())
    }

    // ==================== Message Operations ====================

    /// Insert a new message
    pub fn insert_message(
        &self,
        session_id: Uuid,
        role: MessageRole,
        content: &str,
    ) -> DbResult<Message> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now();
        let id = Uuid::new_v4();

        conn.execute(
            "INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id.to_string(),
                session_id.to_string(),
                role.as_str(),
                content,
                now.to_rfc3339()
            ],
        )?;

        Ok(Message {
            id,
            session_id,
            role,
            content: content.to_string(),
            created_at: now,
        })
    }

    /// List messages for a session
    pub fn list_messages(&self, session_id: Uuid) -> DbResult<Vec<Message>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, created_at FROM messages WHERE session_id = ?1 ORDER BY created_at",
        )?;

        let messages = stmt
            .query_map(params![session_id.to_string()], |row| {
                Ok(Message {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    session_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                    role: MessageRole::from_str(&row.get::<_, String>(2)?).unwrap(),
                    content: row.get(3)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    // ==================== Config Operations ====================

    /// Get a config value
    pub fn get_config(&self, key: &str) -> DbResult<Option<String>> {
        let conn = self.conn.lock().unwrap();

        match conn.query_row(
            "SELECT value FROM config WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        ) {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// Set a config value
    pub fn set_config(&self, key: &str, value: &str) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now();

        conn.execute(
            "INSERT OR REPLACE INTO config (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![key, value, now.to_rfc3339()],
        )?;

        Ok(())
    }

    /// Delete a config value
    pub fn delete_config(&self, key: &str) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM config WHERE key = ?1", params![key])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_creation() {
        let db = Database::in_memory().expect("Failed to create in-memory database");

        // Verify tables exist by querying them
        let repos = db.list_repos().expect("Failed to list repos");
        assert!(repos.is_empty());

        let sessions = db.list_sessions().expect("Failed to list sessions");
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_repo_crud() {
        let db = Database::in_memory().expect("Failed to create in-memory database");

        // Insert
        let repo = db
            .insert_repo("/path/to/repo", "my-repo")
            .expect("Failed to insert repo");
        assert_eq!(repo.path, "/path/to/repo");
        assert_eq!(repo.name, "my-repo");

        // Get by ID
        let fetched = db.get_repo(repo.id).expect("Failed to get repo");
        assert_eq!(fetched.id, repo.id);
        assert_eq!(fetched.path, repo.path);

        // Get by path
        let fetched_by_path = db
            .get_repo_by_path("/path/to/repo")
            .expect("Failed to get repo by path");
        assert_eq!(fetched_by_path.id, repo.id);

        // List
        let repos = db.list_repos().expect("Failed to list repos");
        assert_eq!(repos.len(), 1);

        // Delete
        db.delete_repo(repo.id).expect("Failed to delete repo");
        let repos = db.list_repos().expect("Failed to list repos");
        assert!(repos.is_empty());
    }

    #[test]
    fn test_session_crud() {
        let db = Database::in_memory().expect("Failed to create in-memory database");

        // Create a repo first
        let repo = db
            .insert_repo("/path/to/repo", "my-repo")
            .expect("Failed to insert repo");

        // Insert session
        let session = db
            .insert_session(repo.id, Some("Test Session"))
            .expect("Failed to insert session");
        assert_eq!(session.repo_id, repo.id);
        assert_eq!(session.name, Some("Test Session".to_string()));
        assert_eq!(session.status, SessionStatus::Idle);

        // Get by ID
        let fetched = db.get_session(session.id).expect("Failed to get session");
        assert_eq!(fetched.id, session.id);

        // Update status
        db.update_session_status(session.id, SessionStatus::Running)
            .expect("Failed to update status");
        let updated = db.get_session(session.id).expect("Failed to get session");
        assert_eq!(updated.status, SessionStatus::Running);

        // List
        let sessions = db.list_sessions().expect("Failed to list sessions");
        assert_eq!(sessions.len(), 1);

        // List by repo
        let sessions_by_repo = db
            .list_sessions_by_repo(repo.id)
            .expect("Failed to list sessions by repo");
        assert_eq!(sessions_by_repo.len(), 1);

        // Delete
        db.delete_session(session.id)
            .expect("Failed to delete session");
        let sessions = db.list_sessions().expect("Failed to list sessions");
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_message_crud() {
        let db = Database::in_memory().expect("Failed to create in-memory database");

        // Create repo and session
        let repo = db
            .insert_repo("/path/to/repo", "my-repo")
            .expect("Failed to insert repo");
        let session = db
            .insert_session(repo.id, None)
            .expect("Failed to insert session");

        // Insert messages
        let msg1 = db
            .insert_message(session.id, MessageRole::User, "Hello!")
            .expect("Failed to insert message");
        let msg2 = db
            .insert_message(session.id, MessageRole::Assistant, "Hi there!")
            .expect("Failed to insert message");

        assert_eq!(msg1.role, MessageRole::User);
        assert_eq!(msg2.role, MessageRole::Assistant);

        // List messages
        let messages = db
            .list_messages(session.id)
            .expect("Failed to list messages");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].content, "Hello!");
        assert_eq!(messages[1].content, "Hi there!");
    }

    #[test]
    fn test_config_crud() {
        let db = Database::in_memory().expect("Failed to create in-memory database");

        // Get non-existent
        let value = db.get_config("backend").expect("Failed to get config");
        assert!(value.is_none());

        // Set
        db.set_config("backend", "claude")
            .expect("Failed to set config");
        let value = db.get_config("backend").expect("Failed to get config");
        assert_eq!(value, Some("claude".to_string()));

        // Update
        db.set_config("backend", "gemini")
            .expect("Failed to set config");
        let value = db.get_config("backend").expect("Failed to get config");
        assert_eq!(value, Some("gemini".to_string()));

        // Delete
        db.delete_config("backend")
            .expect("Failed to delete config");
        let value = db.get_config("backend").expect("Failed to get config");
        assert!(value.is_none());
    }

    #[test]
    fn test_cascade_delete() {
        let db = Database::in_memory().expect("Failed to create in-memory database");

        // Create repo, session, and messages
        let repo = db
            .insert_repo("/path/to/repo", "my-repo")
            .expect("Failed to insert repo");
        let session = db
            .insert_session(repo.id, None)
            .expect("Failed to insert session");
        db.insert_message(session.id, MessageRole::User, "Hello!")
            .expect("Failed to insert message");

        // Delete repo should cascade to sessions and messages
        db.delete_repo(repo.id).expect("Failed to delete repo");

        // Session should be gone
        let result = db.get_session(session.id);
        assert!(matches!(result, Err(DbError::NotFound)));
    }
}
