use axum::{
    extract::{Path as AxumPath, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::models::{Message, Orchestrator, OutputStream, OutputLog, Session, SessionStatus};
use crate::error::{AppError, AppResult};
use crate::ralph::RalphError;

use super::AppState;

/// Request body for creating a new session
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSessionRequest {
    /// Repository ID to create session for
    pub repo_id: Uuid,
    /// Optional session name
    pub name: Option<String>,
    /// Orchestrator to use for this session (defaults to ralph)
    #[serde(default)]
    pub orchestrator: Orchestrator,
}

/// Response for session details including messages
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionDetails {
    #[serde(flatten)]
    pub session: Session,
    pub messages: Vec<Message>,
}

/// Request body for running ralph on a session
#[derive(Debug, Deserialize, Serialize)]
pub struct RunSessionRequest {
    /// The prompt to send to ralph
    pub prompt: String,
}

/// Response for run session endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct RunSessionResponse {
    pub session_id: Uuid,
    pub status: SessionStatus,
    pub message: String,
}

/// Query parameters for fetching session output
#[derive(Debug, Deserialize)]
pub struct OutputQueryParams {
    /// Filter by stream type (stdout, stderr)
    pub stream: Option<String>,
    /// Maximum number of entries to return
    pub limit: Option<i64>,
    /// Offset for pagination
    pub offset: Option<i64>,
}

/// Response for session output
#[derive(Debug, Serialize, Deserialize)]
pub struct OutputResponse {
    pub session_id: Uuid,
    pub logs: Vec<OutputLog>,
    pub total: usize,
}

/// List all sessions
async fn list_sessions(State(state): State<AppState>) -> AppResult<Json<Vec<Session>>> {
    let sessions = state
        .db
        .list_sessions()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(sessions))
}

/// Create a new session
async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> AppResult<Json<Session>> {
    // Validate orchestrator is available
    if !req.orchestrator.is_available() {
        return Err(AppError::BadRequest(format!(
            "Orchestrator '{}' is not yet available",
            req.orchestrator.as_str()
        )));
    }

    // Verify repo exists
    state.db.get_repo(req.repo_id).map_err(|e| match e {
        crate::db::DbError::NotFound => {
            AppError::BadRequest(format!("Repository not found: {}", req.repo_id))
        }
        _ => AppError::Internal(e.to_string()),
    })?;

    let session = state
        .db
        .insert_session(req.repo_id, req.name.as_deref(), req.orchestrator)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(session))
}

/// Get a session by ID with its messages
async fn get_session(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
) -> AppResult<Json<SessionDetails>> {
    let session = state.db.get_session(id).map_err(|e| match e {
        crate::db::DbError::NotFound => AppError::NotFound(format!("Session not found: {}", id)),
        _ => AppError::Internal(e.to_string()),
    })?;

    let messages = state
        .db
        .list_messages(id)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(SessionDetails { session, messages }))
}

/// Delete a session by ID
async fn delete_session(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
) -> AppResult<Json<()>> {
    state.db.delete_session(id).map_err(|e| match e {
        crate::db::DbError::NotFound => AppError::NotFound(format!("Session not found: {}", id)),
        _ => AppError::Internal(e.to_string()),
    })?;

    Ok(Json(()))
}

/// Run ralph on a session
async fn run_session(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
    Json(req): Json<RunSessionRequest>,
) -> AppResult<Json<RunSessionResponse>> {
    // Get the session
    let session = state.db.get_session(id).map_err(|e| match e {
        crate::db::DbError::NotFound => AppError::NotFound(format!("Session not found: {}", id)),
        _ => AppError::Internal(e.to_string()),
    })?;

    // Get the repo path
    let repo = state.db.get_repo(session.repo_id).map_err(|e| match e {
        crate::db::DbError::NotFound => {
            AppError::Internal(format!("Repository not found for session: {}", id))
        }
        _ => AppError::Internal(e.to_string()),
    })?;

    // Start ralph
    state
        .ralph_manager
        .run(
            id,
            session.repo_id,
            &repo.path,
            &req.prompt,
            state.db.clone(),
            state.connections.clone(),
        )
        .await
        .map_err(|e| match e {
            RalphError::RepoBusy(repo_id) => AppError::BadRequest(format!(
                "Repository {} already has a running ralph process",
                repo_id
            )),
            RalphError::SessionAlreadyRunning(session_id) => AppError::BadRequest(format!(
                "Session {} already has a running process",
                session_id
            )),
            RalphError::SpawnFailed(msg) => AppError::Internal(format!("Failed to start ralph: {}", msg)),
            RalphError::NotFound { message, help_steps } => AppError::UserActionRequired {
                code: "RALPH_NOT_FOUND".to_string(),
                message,
                details: None,
                help_steps,
            },
            RalphError::NotRunning(_) => unreachable!(),
        })?;

    Ok(Json(RunSessionResponse {
        session_id: id,
        status: SessionStatus::Running,
        message: "Ralph process started".to_string(),
    }))
}

/// Cancel a running ralph session
async fn cancel_session(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
) -> AppResult<Json<CancelSessionResponse>> {
    // Verify session exists
    state.db.get_session(id).map_err(|e| match e {
        crate::db::DbError::NotFound => AppError::NotFound(format!("Session not found: {}", id)),
        _ => AppError::Internal(e.to_string()),
    })?;

    // Cancel the ralph process
    state
        .ralph_manager
        .cancel(id, state.db.clone(), state.connections.clone())
        .await
        .map_err(|e| match e {
            RalphError::NotRunning(session_id) => {
                AppError::BadRequest(format!("Session {} has no running process", session_id))
            }
            _ => AppError::Internal(e.to_string()),
        })?;

    Ok(Json(CancelSessionResponse {
        session_id: id,
        status: SessionStatus::Cancelled,
        message: "Ralph process cancelled".to_string(),
    }))
}

/// Response for cancel session endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct CancelSessionResponse {
    pub session_id: Uuid,
    pub status: SessionStatus,
    pub message: String,
}

/// Get session output logs (historical)
async fn get_session_output(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
    Query(params): Query<OutputQueryParams>,
) -> AppResult<Json<OutputResponse>> {
    // Verify session exists
    state.db.get_session(id).map_err(|e| match e {
        crate::db::DbError::NotFound => AppError::NotFound(format!("Session not found: {}", id)),
        _ => AppError::Internal(e.to_string()),
    })?;

    // Parse stream filter
    let stream_filter = params.stream.and_then(|s| match s.to_lowercase().as_str() {
        "stdout" => Some(OutputStream::Stdout),
        "stderr" => Some(OutputStream::Stderr),
        _ => None,
    });

    let logs = state
        .db
        .list_output_logs(id, stream_filter, params.limit, params.offset)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let total = logs.len();

    Ok(Json(OutputResponse {
        session_id: id,
        logs,
        total,
    }))
}

/// Create the sessions router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sessions", get(list_sessions).post(create_session))
        .route("/sessions/{id}", get(get_session).delete(delete_session))
        .route("/sessions/{id}/run", post(run_session))
        .route("/sessions/{id}/cancel", post(cancel_session))
        .route("/sessions/{id}/output", get(get_session_output))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::repos::{router as repos_router, AddRepoRequest};
    use crate::db::models::Repo;
    use crate::db::Database;
    use axum_test::TestServer;
    use tempfile::TempDir;

    fn create_test_state() -> AppState {
        let db = Database::in_memory().expect("Failed to create test database");
        AppState::new(db)
    }

    fn create_test_server(state: AppState) -> TestServer {
        let app = Router::new()
            .merge(repos_router())
            .merge(router())
            .with_state(state);
        TestServer::new(app).expect("Failed to create test server")
    }

    async fn create_test_repo(server: &TestServer) -> Repo {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        git2::Repository::init(temp_dir.path()).expect("Failed to init git repo");

        // Need to keep temp_dir alive, so we leak it for tests
        let path = temp_dir.path().to_string_lossy().to_string();
        std::mem::forget(temp_dir);

        let response = server
            .post("/repos")
            .json(&AddRepoRequest {
                path,
                name: Some("test-repo".to_string()),
            })
            .await;

        response.assert_status_ok();
        response.json()
    }

    #[tokio::test]
    async fn test_list_sessions_empty() {
        let state = create_test_state();
        let server = create_test_server(state);

        let response = server.get("/sessions").await;
        response.assert_status_ok();

        let sessions: Vec<Session> = response.json();
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_create_session_validates_repo() {
        let state = create_test_state();
        let server = create_test_server(state);

        let fake_repo_id = Uuid::new_v4();
        let response = server
            .post("/sessions")
            .json(&CreateSessionRequest {
                repo_id: fake_repo_id,
                name: None,
                orchestrator: Orchestrator::Ralph,
            })
            .await;

        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_create_and_list_session() {
        let state = create_test_state();
        let server = create_test_server(state);

        // Create a repo first
        let repo = create_test_repo(&server).await;

        // Create a session
        let response = server
            .post("/sessions")
            .json(&CreateSessionRequest {
                repo_id: repo.id,
                name: Some("Test Session".to_string()),
                orchestrator: Orchestrator::Ralph,
            })
            .await;

        response.assert_status_ok();
        let session: Session = response.json();
        assert_eq!(session.repo_id, repo.id);
        assert_eq!(session.name, Some("Test Session".to_string()));
        assert_eq!(session.orchestrator, Orchestrator::Ralph);
        assert_eq!(session.status, crate::db::models::SessionStatus::Idle);

        // List sessions
        let response = server.get("/sessions").await;
        response.assert_status_ok();
        let sessions: Vec<Session> = response.json();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, Some("Test Session".to_string()));
        assert_eq!(sessions[0].orchestrator, Orchestrator::Ralph);
    }

    #[tokio::test]
    async fn test_get_session_with_messages() {
        let state = create_test_state();
        let server = create_test_server(state.clone());

        // Create a repo and session
        let repo = create_test_repo(&server).await;
        let response = server
            .post("/sessions")
            .json(&CreateSessionRequest {
                repo_id: repo.id,
                name: None,
                orchestrator: Orchestrator::Ralph,
            })
            .await;
        response.assert_status_ok();
        let session: Session = response.json();

        // Add some messages directly via db
        state
            .db
            .insert_message(
                session.id,
                crate::db::models::MessageRole::User,
                "Hello!",
            )
            .expect("Failed to insert message");
        state
            .db
            .insert_message(
                session.id,
                crate::db::models::MessageRole::Assistant,
                "Hi there!",
            )
            .expect("Failed to insert message");

        // Get session details
        let response = server.get(&format!("/sessions/{}", session.id)).await;
        response.assert_status_ok();
        let details: SessionDetails = response.json();
        assert_eq!(details.session.id, session.id);
        assert_eq!(details.messages.len(), 2);
        assert_eq!(details.messages[0].content, "Hello!");
        assert_eq!(details.messages[1].content, "Hi there!");
    }

    #[tokio::test]
    async fn test_get_nonexistent_session() {
        let state = create_test_state();
        let server = create_test_server(state);

        let fake_id = Uuid::new_v4();
        let response = server.get(&format!("/sessions/{}", fake_id)).await;
        response.assert_status_not_found();
    }

    #[tokio::test]
    async fn test_delete_session() {
        let state = create_test_state();
        let server = create_test_server(state);

        // Create a repo and session
        let repo = create_test_repo(&server).await;
        let response = server
            .post("/sessions")
            .json(&CreateSessionRequest {
                repo_id: repo.id,
                name: Some("To Delete".to_string()),
                orchestrator: Orchestrator::Ralph,
            })
            .await;
        response.assert_status_ok();
        let session: Session = response.json();

        // Delete it
        let response = server.delete(&format!("/sessions/{}", session.id)).await;
        response.assert_status_ok();

        // Verify it's gone
        let response = server.get(&format!("/sessions/{}", session.id)).await;
        response.assert_status_not_found();
    }

    #[tokio::test]
    async fn test_delete_nonexistent_session() {
        let state = create_test_state();
        let server = create_test_server(state);

        let fake_id = Uuid::new_v4();
        let response = server.delete(&format!("/sessions/{}", fake_id)).await;
        response.assert_status_not_found();
    }

    #[tokio::test]
    async fn test_get_session_output_empty() {
        let state = create_test_state();
        let server = create_test_server(state);

        // Create a repo and session
        let repo = create_test_repo(&server).await;
        let response = server
            .post("/sessions")
            .json(&CreateSessionRequest {
                repo_id: repo.id,
                name: None,
                orchestrator: Orchestrator::Ralph,
            })
            .await;
        response.assert_status_ok();
        let session: Session = response.json();

        // Get output (should be empty)
        let response = server
            .get(&format!("/sessions/{}/output", session.id))
            .await;
        response.assert_status_ok();
        let output: OutputResponse = response.json();
        assert_eq!(output.session_id, session.id);
        assert!(output.logs.is_empty());
        assert_eq!(output.total, 0);
    }

    #[tokio::test]
    async fn test_get_session_output_with_logs() {
        let state = create_test_state();
        let server = create_test_server(state.clone());

        // Create a repo and session
        let repo = create_test_repo(&server).await;
        let response = server
            .post("/sessions")
            .json(&CreateSessionRequest {
                repo_id: repo.id,
                name: None,
                orchestrator: Orchestrator::Ralph,
            })
            .await;
        response.assert_status_ok();
        let session: Session = response.json();

        // Add some output logs directly via db
        state
            .db
            .insert_output_log(session.id, OutputStream::Stdout, "Hello stdout!")
            .expect("Failed to insert output log");
        state
            .db
            .insert_output_log(session.id, OutputStream::Stderr, "Hello stderr!")
            .expect("Failed to insert output log");
        state
            .db
            .insert_output_log(session.id, OutputStream::Stdout, "More stdout!")
            .expect("Failed to insert output log");

        // Get all output
        let response = server
            .get(&format!("/sessions/{}/output", session.id))
            .await;
        response.assert_status_ok();
        let output: OutputResponse = response.json();
        assert_eq!(output.logs.len(), 3);

        // Get stdout only
        let response = server
            .get(&format!("/sessions/{}/output?stream=stdout", session.id))
            .await;
        response.assert_status_ok();
        let output: OutputResponse = response.json();
        assert_eq!(output.logs.len(), 2);
        assert!(output.logs.iter().all(|l| l.stream == OutputStream::Stdout));

        // Get stderr only
        let response = server
            .get(&format!("/sessions/{}/output?stream=stderr", session.id))
            .await;
        response.assert_status_ok();
        let output: OutputResponse = response.json();
        assert_eq!(output.logs.len(), 1);
        assert_eq!(output.logs[0].content, "Hello stderr!");

        // Test limit
        let response = server
            .get(&format!("/sessions/{}/output?limit=2", session.id))
            .await;
        response.assert_status_ok();
        let output: OutputResponse = response.json();
        assert_eq!(output.logs.len(), 2);

        // Test offset
        let response = server
            .get(&format!("/sessions/{}/output?offset=1", session.id))
            .await;
        response.assert_status_ok();
        let output: OutputResponse = response.json();
        assert_eq!(output.logs.len(), 2);
        assert_eq!(output.logs[0].content, "Hello stderr!");
    }

    #[tokio::test]
    async fn test_get_output_nonexistent_session() {
        let state = create_test_state();
        let server = create_test_server(state);

        let fake_id = Uuid::new_v4();
        let response = server.get(&format!("/sessions/{}/output", fake_id)).await;
        response.assert_status_not_found();
    }

    #[tokio::test]
    async fn test_cancel_nonexistent_session() {
        let state = create_test_state();
        let server = create_test_server(state);

        let fake_id = Uuid::new_v4();
        let response = server
            .post(&format!("/sessions/{}/cancel", fake_id))
            .await;
        response.assert_status_not_found();
    }

    #[tokio::test]
    async fn test_cancel_session_not_running() {
        let state = create_test_state();
        let server = create_test_server(state);

        // Create a repo and session
        let repo = create_test_repo(&server).await;
        let response = server
            .post("/sessions")
            .json(&CreateSessionRequest {
                repo_id: repo.id,
                name: Some("Test Session".to_string()),
                orchestrator: Orchestrator::Ralph,
            })
            .await;
        response.assert_status_ok();
        let session: Session = response.json();

        // Try to cancel (should fail - not running)
        let response = server
            .post(&format!("/sessions/{}/cancel", session.id))
            .await;
        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_create_session_validates_orchestrator() {
        let state = create_test_state();
        let server = create_test_server(state);

        // Create a repo first
        let repo = create_test_repo(&server).await;

        // Try to create session with unavailable orchestrator (gsd)
        let response = server
            .post("/sessions")
            .json(&CreateSessionRequest {
                repo_id: repo.id,
                name: Some("Test Session".to_string()),
                orchestrator: Orchestrator::Gsd,
            })
            .await;

        response.assert_status_bad_request();
    }
}
