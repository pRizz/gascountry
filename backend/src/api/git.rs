//! Git operations REST API endpoints
//!
//! Provides endpoints for git operations on session repositories:
//! - Read operations: status, log, branches, diff
//! - Write operations: pull, push, commit, reset, checkout

use axum::{
    extract::{Path as AxumPath, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::git::{Branch, Commit, CommandOutput, FileDelta, GitError, GitManager, GitStatus};

use super::AppState;

/// Query parameters for git log
#[derive(Debug, Deserialize)]
pub struct LogQueryParams {
    /// Maximum number of commits to return (default: 20)
    pub limit: Option<usize>,
}

/// Request body for git commit
#[derive(Debug, Deserialize, Serialize)]
pub struct CommitRequest {
    /// Commit message
    pub message: String,
    /// Whether to stage all changes first (git add -A)
    #[serde(default)]
    pub stage_all: bool,
}

/// Request body for git reset
#[derive(Debug, Deserialize, Serialize)]
pub struct ResetRequest {
    /// Must be true to confirm destructive operation
    pub confirm: bool,
}

/// Request body for git checkout
#[derive(Debug, Deserialize, Serialize)]
pub struct CheckoutRequest {
    /// Branch name to checkout
    pub branch: String,
}

/// Response wrapper for git status
#[derive(Debug, Serialize, Deserialize)]
pub struct GitStatusResponse {
    pub session_id: Uuid,
    #[serde(flatten)]
    pub status: GitStatus,
}

/// Response wrapper for git log
#[derive(Debug, Serialize, Deserialize)]
pub struct GitLogResponse {
    pub session_id: Uuid,
    pub commits: Vec<Commit>,
}

/// Response wrapper for branches
#[derive(Debug, Serialize, Deserialize)]
pub struct GitBranchesResponse {
    pub session_id: Uuid,
    pub branches: Vec<Branch>,
}

/// Response wrapper for diff stats
#[derive(Debug, Serialize, Deserialize)]
pub struct GitDiffResponse {
    pub session_id: Uuid,
    pub files: Vec<FileDelta>,
    pub total_added: usize,
    pub total_removed: usize,
}

/// Response wrapper for git command output
#[derive(Debug, Serialize, Deserialize)]
pub struct GitCommandResponse {
    pub session_id: Uuid,
    #[serde(flatten)]
    pub output: CommandOutput,
}

/// Helper to get the repo path for a session
async fn get_session_repo_path(state: &AppState, session_id: Uuid) -> AppResult<std::path::PathBuf> {
    let session = state.db.get_session(session_id).map_err(|e| match e {
        crate::db::DbError::NotFound => AppError::NotFound(format!("Session not found: {}", session_id)),
        _ => AppError::Internal(e.to_string()),
    })?;

    let repo = state.db.get_repo(session.repo_id).map_err(|e| match e {
        crate::db::DbError::NotFound => {
            AppError::Internal(format!("Repository not found for session: {}", session_id))
        }
        _ => AppError::Internal(e.to_string()),
    })?;

    Ok(std::path::PathBuf::from(&repo.path))
}

/// Map GitError to AppError
fn map_git_error(e: GitError) -> AppError {
    match e {
        GitError::NotARepo(msg) => AppError::BadRequest(format!("Not a git repository: {}", msg)),
        GitError::InvalidBranch(msg) => AppError::BadRequest(format!("Invalid branch: {}", msg)),
        GitError::OperationFailed(msg) => AppError::Internal(format!("Git operation failed: {}", msg)),
        GitError::CommandFailed(msg) => AppError::Internal(format!("Git command failed: {}", msg)),
    }
}

/// GET /api/sessions/{id}/git/status - Get git status
async fn get_status(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
) -> AppResult<Json<GitStatusResponse>> {
    let repo_path = get_session_repo_path(&state, id).await?;
    let status = GitManager::status(&repo_path).map_err(map_git_error)?;

    Ok(Json(GitStatusResponse {
        session_id: id,
        status,
    }))
}

/// GET /api/sessions/{id}/git/log - Get recent commits
async fn get_log(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
    Query(params): Query<LogQueryParams>,
) -> AppResult<Json<GitLogResponse>> {
    let repo_path = get_session_repo_path(&state, id).await?;
    let limit = params.limit.unwrap_or(20);
    let commits = GitManager::log(&repo_path, limit).map_err(map_git_error)?;

    Ok(Json(GitLogResponse {
        session_id: id,
        commits,
    }))
}

/// GET /api/sessions/{id}/git/branches - List branches
async fn get_branches(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
) -> AppResult<Json<GitBranchesResponse>> {
    let repo_path = get_session_repo_path(&state, id).await?;
    let branches = GitManager::branches(&repo_path).map_err(map_git_error)?;

    Ok(Json(GitBranchesResponse {
        session_id: id,
        branches,
    }))
}

/// GET /api/sessions/{id}/git/diff - Get diff statistics
async fn get_diff(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
) -> AppResult<Json<GitDiffResponse>> {
    let repo_path = get_session_repo_path(&state, id).await?;
    let files = GitManager::diff_stats(&repo_path).map_err(map_git_error)?;

    let total_added: usize = files.iter().map(|f| f.added).sum();
    let total_removed: usize = files.iter().map(|f| f.removed).sum();

    Ok(Json(GitDiffResponse {
        session_id: id,
        files,
        total_added,
        total_removed,
    }))
}

/// POST /api/sessions/{id}/git/pull - Execute git pull
async fn post_pull(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
) -> AppResult<Json<GitCommandResponse>> {
    let repo_path = get_session_repo_path(&state, id).await?;
    let output = GitManager::pull(&repo_path).map_err(map_git_error)?;

    Ok(Json(GitCommandResponse {
        session_id: id,
        output,
    }))
}

/// POST /api/sessions/{id}/git/push - Execute git push
async fn post_push(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
) -> AppResult<Json<GitCommandResponse>> {
    let repo_path = get_session_repo_path(&state, id).await?;
    let output = GitManager::push(&repo_path).map_err(map_git_error)?;

    Ok(Json(GitCommandResponse {
        session_id: id,
        output,
    }))
}

/// POST /api/sessions/{id}/git/commit - Execute git commit
async fn post_commit(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
    Json(req): Json<CommitRequest>,
) -> AppResult<Json<GitCommandResponse>> {
    let repo_path = get_session_repo_path(&state, id).await?;

    // Validate message
    if req.message.trim().is_empty() {
        return Err(AppError::BadRequest("Commit message cannot be empty".to_string()));
    }

    // Stage all changes if requested
    if req.stage_all {
        GitManager::add_all(&repo_path).map_err(map_git_error)?;
    }

    let output = GitManager::commit(&repo_path, &req.message).map_err(map_git_error)?;

    Ok(Json(GitCommandResponse {
        session_id: id,
        output,
    }))
}

/// POST /api/sessions/{id}/git/reset - Execute git reset --hard
async fn post_reset(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
    Json(req): Json<ResetRequest>,
) -> AppResult<Json<GitCommandResponse>> {
    // Require explicit confirmation for destructive operation
    if !req.confirm {
        return Err(AppError::BadRequest(
            "Reset requires confirmation. Set confirm: true to proceed.".to_string(),
        ));
    }

    let repo_path = get_session_repo_path(&state, id).await?;
    let output = GitManager::reset_hard(&repo_path).map_err(map_git_error)?;

    Ok(Json(GitCommandResponse {
        session_id: id,
        output,
    }))
}

/// POST /api/sessions/{id}/git/checkout - Switch branch
async fn post_checkout(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
    Json(req): Json<CheckoutRequest>,
) -> AppResult<Json<GitCommandResponse>> {
    let repo_path = get_session_repo_path(&state, id).await?;

    // Validate branch name
    if req.branch.trim().is_empty() {
        return Err(AppError::BadRequest("Branch name cannot be empty".to_string()));
    }

    let output = GitManager::checkout(&repo_path, &req.branch).map_err(map_git_error)?;

    Ok(Json(GitCommandResponse {
        session_id: id,
        output,
    }))
}

/// Create the git router (nested under sessions)
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sessions/{id}/git/status", get(get_status))
        .route("/sessions/{id}/git/log", get(get_log))
        .route("/sessions/{id}/git/branches", get(get_branches))
        .route("/sessions/{id}/git/diff", get(get_diff))
        .route("/sessions/{id}/git/pull", post(post_pull))
        .route("/sessions/{id}/git/push", post(post_push))
        .route("/sessions/{id}/git/commit", post(post_commit))
        .route("/sessions/{id}/git/reset", post(post_reset))
        .route("/sessions/{id}/git/checkout", post(post_checkout))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::repos::{router as repos_router, AddRepoRequest};
    use crate::api::sessions::{router as sessions_router, CreateSessionRequest};
    use crate::db::models::{Repo, Session};
    use crate::db::Database;
    use axum_test::TestServer;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_state() -> AppState {
        let db = Database::in_memory().expect("Failed to create test database");
        AppState::new(db)
    }

    fn create_test_server(state: AppState) -> TestServer {
        let app = Router::new()
            .merge(repos_router())
            .merge(sessions_router())
            .merge(router())
            .with_state(state);
        TestServer::new(app).expect("Failed to create test server")
    }

    fn create_git_repo_with_commit() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo = git2::Repository::init(temp_dir.path()).expect("Failed to init git repo");

        // Configure user
        let mut config = repo.config().expect("Failed to get config");
        config.set_str("user.name", "Test User").expect("Failed to set user.name");
        config.set_str("user.email", "test@example.com").expect("Failed to set user.email");

        // Create initial commit
        let sig = repo.signature().expect("Failed to create signature");
        let tree_id = repo.index().expect("Failed to get index").write_tree().expect("Failed to write tree");
        let tree = repo.find_tree(tree_id).expect("Failed to find tree");
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .expect("Failed to create initial commit");

        temp_dir
    }

    async fn create_test_session(server: &TestServer) -> (Session, TempDir) {
        let temp_dir = create_git_repo_with_commit();
        let path = temp_dir.path().to_string_lossy().to_string();

        // Add repo
        let response = server
            .post("/repos")
            .json(&AddRepoRequest {
                path,
                name: Some("test-repo".to_string()),
            })
            .await;
        response.assert_status_ok();
        let repo: Repo = response.json();

        // Create session
        let response = server
            .post("/sessions")
            .json(&CreateSessionRequest {
                repo_id: repo.id,
                name: Some("Test Session".to_string()),
            })
            .await;
        response.assert_status_ok();
        let session: Session = response.json();

        (session, temp_dir)
    }

    #[tokio::test]
    async fn test_get_status() {
        let state = create_test_state();
        let server = create_test_server(state);
        let (session, _temp_dir) = create_test_session(&server).await;

        let response = server
            .get(&format!("/sessions/{}/git/status", session.id))
            .await;
        response.assert_status_ok();

        let status: GitStatusResponse = response.json();
        assert_eq!(status.session_id, session.id);
        assert!(!status.status.branch.is_empty());
    }

    #[tokio::test]
    async fn test_get_status_with_untracked() {
        let state = create_test_state();
        let server = create_test_server(state);
        let (session, temp_dir) = create_test_session(&server).await;

        // Create an untracked file
        fs::write(temp_dir.path().join("new_file.txt"), "content").expect("Failed to write file");

        let response = server
            .get(&format!("/sessions/{}/git/status", session.id))
            .await;
        response.assert_status_ok();

        let status: GitStatusResponse = response.json();
        assert!(status.status.untracked.contains(&"new_file.txt".to_string()));
    }

    #[tokio::test]
    async fn test_get_log() {
        let state = create_test_state();
        let server = create_test_server(state);
        let (session, _temp_dir) = create_test_session(&server).await;

        let response = server
            .get(&format!("/sessions/{}/git/log", session.id))
            .await;
        response.assert_status_ok();

        let log: GitLogResponse = response.json();
        assert_eq!(log.session_id, session.id);
        assert!(!log.commits.is_empty());
        assert_eq!(log.commits[0].message, "Initial commit");
    }

    #[tokio::test]
    async fn test_get_log_with_limit() {
        let state = create_test_state();
        let server = create_test_server(state);
        let (session, _temp_dir) = create_test_session(&server).await;

        let response = server
            .get(&format!("/sessions/{}/git/log?limit=5", session.id))
            .await;
        response.assert_status_ok();

        let log: GitLogResponse = response.json();
        assert!(log.commits.len() <= 5);
    }

    #[tokio::test]
    async fn test_get_branches() {
        let state = create_test_state();
        let server = create_test_server(state);
        let (session, _temp_dir) = create_test_session(&server).await;

        let response = server
            .get(&format!("/sessions/{}/git/branches", session.id))
            .await;
        response.assert_status_ok();

        let branches: GitBranchesResponse = response.json();
        assert_eq!(branches.session_id, session.id);
        assert!(!branches.branches.is_empty());

        // Should have current branch marked
        let current = branches.branches.iter().find(|b| b.is_current);
        assert!(current.is_some());
    }

    #[tokio::test]
    async fn test_get_diff_no_changes() {
        let state = create_test_state();
        let server = create_test_server(state);
        let (session, _temp_dir) = create_test_session(&server).await;

        let response = server
            .get(&format!("/sessions/{}/git/diff", session.id))
            .await;
        response.assert_status_ok();

        let diff: GitDiffResponse = response.json();
        assert_eq!(diff.session_id, session.id);
        assert!(diff.files.is_empty());
        assert_eq!(diff.total_added, 0);
        assert_eq!(diff.total_removed, 0);
    }

    #[tokio::test]
    async fn test_commit_empty_message() {
        let state = create_test_state();
        let server = create_test_server(state);
        let (session, _temp_dir) = create_test_session(&server).await;

        let response = server
            .post(&format!("/sessions/{}/git/commit", session.id))
            .json(&CommitRequest {
                message: "  ".to_string(),
                stage_all: false,
            })
            .await;
        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_reset_requires_confirm() {
        let state = create_test_state();
        let server = create_test_server(state);
        let (session, _temp_dir) = create_test_session(&server).await;

        let response = server
            .post(&format!("/sessions/{}/git/reset", session.id))
            .json(&ResetRequest { confirm: false })
            .await;
        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_checkout_empty_branch() {
        let state = create_test_state();
        let server = create_test_server(state);
        let (session, _temp_dir) = create_test_session(&server).await;

        let response = server
            .post(&format!("/sessions/{}/git/checkout", session.id))
            .json(&CheckoutRequest {
                branch: "  ".to_string(),
            })
            .await;
        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_nonexistent_session() {
        let state = create_test_state();
        let server = create_test_server(state);
        let fake_id = Uuid::new_v4();

        let response = server
            .get(&format!("/sessions/{}/git/status", fake_id))
            .await;
        response.assert_status_not_found();
    }
}
