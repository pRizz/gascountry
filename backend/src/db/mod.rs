pub mod models;
pub mod schema;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use rusqlite::{params, Connection};
use thiserror::Error;
use uuid::Uuid;

use models::{Message, MessageRole, OutputStream, OutputLog, Repo, Session, SessionStatus};
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

    /// List all config values
    pub fn list_config(&self) -> DbResult<Vec<(String, String)>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare("SELECT key, value FROM config")?;
        let config = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(config)
    }

    // ==================== Output Log Operations ====================

    /// Insert a new output log entry
    pub fn insert_output_log(
        &self,
        session_id: Uuid,
        stream: OutputStream,
        content: &str,
    ) -> DbResult<OutputLog> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now();

        conn.execute(
            "INSERT INTO output_logs (session_id, stream, content, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                session_id.to_string(),
                stream.as_str(),
                content,
                now.to_rfc3339()
            ],
        )?;

        let id = conn.last_insert_rowid();

        Ok(OutputLog {
            id,
            session_id,
            stream,
            content: content.to_string(),
            created_at: now,
        })
    }

    /// List output logs for a session
    ///
    /// # Arguments
    /// * `session_id` - The session to get logs for
    /// * `stream_filter` - Optional filter by stream type (stdout/stderr)
    /// * `limit` - Optional limit on number of results
    /// * `offset` - Optional offset for pagination
    pub fn list_output_logs(
        &self,
        session_id: Uuid,
        stream_filter: Option<OutputStream>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> DbResult<Vec<OutputLog>> {
        let conn = self.conn.lock().unwrap();

        let base_query = "SELECT id, session_id, stream, content, created_at FROM output_logs WHERE session_id = ?1";

        // SQLite requires LIMIT when using OFFSET, so use -1 (unlimited) when only offset is provided
        let query = match (stream_filter, limit, offset) {
            (Some(_), Some(lim), Some(off)) => format!(
                "{} AND stream = ?2 ORDER BY id LIMIT {} OFFSET {}",
                base_query, lim, off
            ),
            (Some(_), Some(lim), None) => {
                format!("{} AND stream = ?2 ORDER BY id LIMIT {}", base_query, lim)
            }
            (Some(_), None, Some(off)) => {
                format!("{} AND stream = ?2 ORDER BY id LIMIT -1 OFFSET {}", base_query, off)
            }
            (Some(_), None, None) => format!("{} AND stream = ?2 ORDER BY id", base_query),
            (None, Some(lim), Some(off)) => {
                format!("{} ORDER BY id LIMIT {} OFFSET {}", base_query, lim, off)
            }
            (None, Some(lim), None) => format!("{} ORDER BY id LIMIT {}", base_query, lim),
            (None, None, Some(off)) => format!("{} ORDER BY id LIMIT -1 OFFSET {}", base_query, off),
            (None, None, None) => format!("{} ORDER BY id", base_query),
        };

        let logs = if let Some(stream) = stream_filter {
            let mut stmt = conn.prepare(&query)?;
            stmt.query_map(params![session_id.to_string(), stream.as_str()], |row| {
                Ok(OutputLog {
                    id: row.get(0)?,
                    session_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                    stream: OutputStream::from_str(&row.get::<_, String>(2)?).unwrap(),
                    content: row.get(3)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?
        } else {
            let mut stmt = conn.prepare(&query)?;
            stmt.query_map(params![session_id.to_string()], |row| {
                Ok(OutputLog {
                    id: row.get(0)?,
                    session_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                    stream: OutputStream::from_str(&row.get::<_, String>(2)?).unwrap(),
                    content: row.get(3)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?
        };

        Ok(logs)
    }

    /// Delete output logs for a session
    pub fn delete_output_logs(&self, session_id: Uuid) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM output_logs WHERE session_id = ?1",
            params![session_id.to_string()],
        )?;
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

    #[test]
    fn test_output_log_crud() {
        let db = Database::in_memory().expect("Failed to create in-memory database");

        // Create repo and session
        let repo = db
            .insert_repo("/path/to/repo", "my-repo")
            .expect("Failed to insert repo");
        let session = db
            .insert_session(repo.id, None)
            .expect("Failed to insert session");

        // Insert output logs
        let log1 = db
            .insert_output_log(session.id, OutputStream::Stdout, "Hello stdout!")
            .expect("Failed to insert output log");
        let log2 = db
            .insert_output_log(session.id, OutputStream::Stderr, "Hello stderr!")
            .expect("Failed to insert output log");
        db.insert_output_log(session.id, OutputStream::Stdout, "More stdout!")
            .expect("Failed to insert output log");

        assert_eq!(log1.stream, OutputStream::Stdout);
        assert_eq!(log2.stream, OutputStream::Stderr);

        // List all logs
        let all_logs = db
            .list_output_logs(session.id, None, None, None)
            .expect("Failed to list output logs");
        assert_eq!(all_logs.len(), 3);

        // Filter by stdout
        let stdout_logs = db
            .list_output_logs(session.id, Some(OutputStream::Stdout), None, None)
            .expect("Failed to list stdout logs");
        assert_eq!(stdout_logs.len(), 2);
        assert!(stdout_logs.iter().all(|l| l.stream == OutputStream::Stdout));

        // Filter by stderr
        let stderr_logs = db
            .list_output_logs(session.id, Some(OutputStream::Stderr), None, None)
            .expect("Failed to list stderr logs");
        assert_eq!(stderr_logs.len(), 1);
        assert_eq!(stderr_logs[0].content, "Hello stderr!");

        // Test limit
        let limited = db
            .list_output_logs(session.id, None, Some(2), None)
            .expect("Failed to list limited logs");
        assert_eq!(limited.len(), 2);

        // Test offset
        let offset = db
            .list_output_logs(session.id, None, None, Some(1))
            .expect("Failed to list offset logs");
        assert_eq!(offset.len(), 2);
        assert_eq!(offset[0].content, "Hello stderr!");

        // Test limit + offset
        let limited_offset = db
            .list_output_logs(session.id, None, Some(1), Some(1))
            .expect("Failed to list limited offset logs");
        assert_eq!(limited_offset.len(), 1);
        assert_eq!(limited_offset[0].content, "Hello stderr!");

        // Delete logs
        db.delete_output_logs(session.id)
            .expect("Failed to delete output logs");
        let empty = db
            .list_output_logs(session.id, None, None, None)
            .expect("Failed to list logs after delete");
        assert!(empty.is_empty());
    }

    #[test]
    fn test_output_log_cascade_delete() {
        let db = Database::in_memory().expect("Failed to create in-memory database");

        // Create repo, session, and output logs
        let repo = db
            .insert_repo("/path/to/repo", "my-repo")
            .expect("Failed to insert repo");
        let session = db
            .insert_session(repo.id, None)
            .expect("Failed to insert session");
        db.insert_output_log(session.id, OutputStream::Stdout, "Test output")
            .expect("Failed to insert output log");

        // Delete session should cascade to output logs
        db.delete_session(session.id)
            .expect("Failed to delete session");

        // Output logs should be gone (session cascade)
        // We can verify by checking that listing returns empty for a non-existent session
        let logs = db
            .list_output_logs(session.id, None, None, None)
            .expect("Failed to list logs");
        assert!(logs.is_empty());
    }
}
