/// SQL schema definitions for the Ralphtown database
///
/// Tables:
/// - repos: Git repositories being managed
/// - sessions: Ralph sessions tied to repos
/// - messages: Chat messages within sessions
/// - output_logs: Raw output from Ralph processes
/// - config: Key-value configuration storage

/// Schema version for migrations
pub const SCHEMA_VERSION: i32 = 1;

/// SQL to create all tables
pub const CREATE_TABLES: &str = r#"
-- Repositories table
CREATE TABLE IF NOT EXISTS repos (
    id TEXT PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    repo_id TEXT NOT NULL,
    name TEXT,
    status TEXT NOT NULL DEFAULT 'idle',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (repo_id) REFERENCES repos(id) ON DELETE CASCADE
);

-- Messages table (prompts and responses in a session)
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

-- Output logs table (raw stdout/stderr from Ralph)
CREATE TABLE IF NOT EXISTS output_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    stream TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

-- Config table (key-value storage)
CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_sessions_repo_id ON sessions(repo_id);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_messages_session_id ON messages(session_id);
CREATE INDEX IF NOT EXISTS idx_output_logs_session_id ON output_logs(session_id);
"#;

/// SQL to insert or update schema version
pub const UPSERT_SCHEMA_VERSION: &str = r#"
INSERT OR REPLACE INTO schema_version (version) VALUES (?1)
"#;

/// SQL to get current schema version
pub const GET_SCHEMA_VERSION: &str = r#"
SELECT version FROM schema_version LIMIT 1
"#;
