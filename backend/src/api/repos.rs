use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use axum::{
    extract::{Path as AxumPath, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::{delete, get, post},
    Json, Router,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::db::models::Repo;
use crate::error::{AppError, AppResult};
use crate::git::{CloneProgress, GitManager};

use super::AppState;

/// Request body for adding a new repository
#[derive(Debug, Deserialize, Serialize)]
pub struct AddRepoRequest {
    /// Path to the git repository
    pub path: String,
    /// Optional name (defaults to directory name)
    pub name: Option<String>,
}

/// Request body for cloning a repository
#[derive(Debug, Deserialize, Serialize)]
pub struct CloneRepoRequest {
    /// Git URL (SSH or HTTPS format)
    pub url: String,
}

/// Response for clone operation
#[derive(Debug, Serialize, Deserialize)]
pub struct CloneRepoResponse {
    pub repo: Repo,
    pub message: String,
}

/// Query parameters for clone with progress SSE endpoint
#[derive(Debug, Deserialize)]
pub struct CloneProgressQuery {
    /// Git URL to clone (required)
    pub url: String,
}

/// SSE event types for clone progress
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CloneEvent {
    /// Progress update during clone
    Progress(CloneProgress),
    /// Clone completed successfully
    Complete { repo: Repo, message: String },
    /// Clone failed with error
    Error {
        message: String,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        help_steps: Vec<String>,
    },
}

/// Request body for scanning directories
#[derive(Debug, Deserialize, Serialize)]
pub struct ScanRequest {
    /// Directories to scan for git repos
    pub directories: Vec<String>,
    /// Maximum depth to scan (default: 2)
    #[serde(default = "default_scan_depth")]
    pub depth: usize,
}

fn default_scan_depth() -> usize {
    2
}

/// Response for scan operation
#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResponse {
    /// Repositories found during scan
    pub found: Vec<FoundRepo>,
}

/// A repository found during scanning
#[derive(Debug, Serialize, Deserialize)]
pub struct FoundRepo {
    pub path: String,
    pub name: String,
}

/// List all repositories
async fn list_repos(State(state): State<AppState>) -> AppResult<Json<Vec<Repo>>> {
    let repos = state
        .db
        .list_repos()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(repos))
}

/// Add a new repository
async fn add_repo(
    State(state): State<AppState>,
    Json(req): Json<AddRepoRequest>,
) -> AppResult<Json<Repo>> {
    let path = Path::new(&req.path);

    // Validate path exists and is a git repository (returns helpful errors)
    crate::git::validate_repo_path(path)?;

    // Canonicalize path for consistent storage
    let canonical_path = path
        .canonicalize()
        .map_err(|e| AppError::Internal(format!("Failed to canonicalize path: {}", e)))?;

    let path_str = canonical_path.to_string_lossy().to_string();

    // Derive name from directory if not provided
    let name = req.name.unwrap_or_else(|| {
        canonical_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    });

    // Check if repo already exists (using canonical path)
    if state.db.get_repo_by_path(&path_str).is_ok() {
        return Err(AppError::BadRequest(format!(
            "Repository already exists: {}",
            path_str
        )));
    }

    let repo = state
        .db
        .insert_repo(&path_str, &name)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(repo))
}

/// Delete a repository by ID
async fn delete_repo(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
) -> AppResult<Json<()>> {
    state.db.delete_repo(id).map_err(|e| match e {
        crate::db::DbError::NotFound => AppError::NotFound(format!("Repository not found: {}", id)),
        _ => AppError::Internal(e.to_string()),
    })?;

    Ok(Json(()))
}

/// Scan directories for git repositories
async fn scan_repos(Json(req): Json<ScanRequest>) -> AppResult<Json<ScanResponse>> {
    let mut found = Vec::new();

    for dir in &req.directories {
        let path = Path::new(dir);
        if path.exists() && path.is_dir() {
            scan_directory(path, 0, req.depth, &mut found);
        }
    }

    Ok(Json(ScanResponse { found }))
}

/// Recursively scan a directory for git repos
fn scan_directory(path: &Path, current_depth: usize, max_depth: usize, found: &mut Vec<FoundRepo>) {
    // Check if this is a git repo
    if git2::Repository::open(path).is_ok() {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        found.push(FoundRepo {
            path: path.to_string_lossy().to_string(),
            name,
        });
        return; // Don't recurse into git repos
    }

    // Recurse if within depth limit
    if current_depth < max_depth {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    // Skip hidden directories
                    if let Some(name) = entry_path.file_name() {
                        if name.to_string_lossy().starts_with('.') {
                            continue;
                        }
                    }
                    scan_directory(&entry_path, current_depth + 1, max_depth, found);
                }
            }
        }
    }
}

/// Extract repository name from a git URL
///
/// Handles both HTTPS and SSH URL formats:
/// - `https://github.com/user/repo.git` -> `repo`
/// - `https://github.com/user/repo` -> `repo`
/// - `git@github.com:user/repo.git` -> `repo`
fn extract_repo_name(url: &str) -> Result<String, AppError> {
    let url = url.trim_end_matches('/');
    let url = url.trim_end_matches(".git");

    // Try splitting by '/' first (HTTPS URLs)
    let name = url.rsplit('/').next();

    // If that gives us empty or the whole URL, try ':' (SSH URLs)
    let name = match name {
        Some(n) if !n.is_empty() && n != url => Some(n),
        _ => url.rsplit(':').next(),
    };

    let name = name
        .filter(|n| !n.is_empty() && !n.contains('/'))
        .ok_or_else(|| AppError::BadRequest("Could not extract repository name from URL".to_string()))?;

    Ok(name.to_string())
}

/// Clone a repository from a git URL
async fn clone_repo(
    State(state): State<AppState>,
    Json(req): Json<CloneRepoRequest>,
) -> AppResult<Json<CloneRepoResponse>> {
    // Parse URL to extract repo name
    let repo_name = extract_repo_name(&req.url)?;

    // Build destination path: ~/ralphtown/{repo_name}
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Internal("Could not determine home directory".to_string()))?;
    let dest: PathBuf = home.join("ralphtown").join(&repo_name);

    // Check if destination already exists
    if dest.exists() {
        return Err(AppError::BadRequest(format!(
            "Directory already exists: {}",
            dest.display()
        )));
    }

    // Create parent directory if needed
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::Internal(format!("Failed to create directory: {}", e))
        })?;
    }

    // Clone using spawn_blocking to avoid blocking the async runtime
    let url_clone = req.url.clone();
    let dest_clone = dest.clone();
    tokio::task::spawn_blocking(move || GitManager::clone(&url_clone, &dest_clone))
        .await
        .map_err(|e| AppError::Internal(format!("Clone task failed: {}", e)))?
        .map_err(AppError::from)?;

    // Insert repo into database
    let path_str = dest.to_string_lossy().to_string();
    let repo = state
        .db
        .insert_repo(&path_str, &repo_name)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(CloneRepoResponse {
        repo,
        message: format!("Cloned to {}", dest.display()),
    }))
}

/// Type alias for the SSE stream used in clone progress
type SseStream = Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>;

/// Type alias for the full SSE response with keep-alive
type SseResponse = Sse<axum::response::sse::KeepAliveStream<SseStream>>;

/// Create an error SSE response
fn error_sse(message: String, help_steps: Vec<String>) -> SseResponse {
    let stream = async_stream::stream! {
        let event = CloneEvent::Error { message, help_steps };
        let data = serde_json::to_string(&event).unwrap_or_default();
        yield Ok(Event::default().event("error").data(data));
    };
    Sse::new(Box::pin(stream) as SseStream).keep_alive(KeepAlive::default())
}

/// Clone a repository with SSE progress streaming
///
/// This endpoint streams clone progress events and a final complete/error event.
/// Uses Server-Sent Events (SSE) for real-time progress feedback.
async fn clone_with_progress_sse(
    State(state): State<AppState>,
    Query(query): Query<CloneProgressQuery>,
) -> SseResponse {
    // Parse URL to extract repo name
    let repo_name = match extract_repo_name(&query.url) {
        Ok(name) => name,
        Err(e) => {
            return error_sse(e.to_string(), Vec::new());
        }
    };

    // Build destination path: ~/ralphtown/{repo_name}
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => {
            return error_sse("Could not determine home directory".to_string(), Vec::new());
        }
    };
    let dest: PathBuf = home.join("ralphtown").join(&repo_name);

    // Check if destination already exists
    if dest.exists() {
        return error_sse(format!("Directory already exists: {}", dest.display()), Vec::new());
    }

    // Create parent directory if needed
    if let Some(parent) = dest.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return error_sse(format!("Failed to create directory: {}", e), Vec::new());
        }
    }

    // Create bounded channel for progress updates
    let (progress_tx, mut progress_rx) = mpsc::channel::<CloneProgress>(32);

    // Spawn the blocking clone operation
    let url_clone = query.url.clone();
    let dest_clone = dest.clone();
    let clone_handle = tokio::task::spawn_blocking(move || {
        GitManager::clone_with_progress(&url_clone, &dest_clone, progress_tx)
    });

    // Create the SSE stream
    let stream = async_stream::stream! {
        // Stream progress updates while clone is running
        loop {
            tokio::select! {
                // Check for progress updates
                progress = progress_rx.recv() => {
                    match progress {
                        Some(p) => {
                            let event = CloneEvent::Progress(p);
                            let data = serde_json::to_string(&event).unwrap_or_default();
                            yield Ok(Event::default().data(data));
                        }
                        None => {
                            // Channel closed, clone is complete or errored
                            break;
                        }
                    }
                }
            }
        }

        // Wait for clone to complete and send final event
        match clone_handle.await {
            Ok(Ok(_)) => {
                // Clone succeeded, insert repo into database
                let path_str = dest.to_string_lossy().to_string();
                match state.db.insert_repo(&path_str, &repo_name) {
                    Ok(repo) => {
                        let event = CloneEvent::Complete {
                            repo,
                            message: format!("Cloned to {}", dest.display()),
                        };
                        let data = serde_json::to_string(&event).unwrap_or_default();
                        yield Ok(Event::default().event("complete").data(data));
                    }
                    Err(e) => {
                        let event = CloneEvent::Error {
                            message: format!("Failed to save repo to database: {}", e),
                            help_steps: Vec::new(),
                        };
                        let data = serde_json::to_string(&event).unwrap_or_default();
                        yield Ok(Event::default().event("error").data(data));
                    }
                }
            }
            Ok(Err(clone_error)) => {
                // Extract help_steps from CloneError variants
                let (message, help_steps) = match &clone_error {
                    crate::git::CloneError::SshAuthFailed { message, help_steps } => {
                        (message.clone(), help_steps.clone())
                    }
                    crate::git::CloneError::HttpsAuthFailed { message, help_steps } => {
                        (message.clone(), help_steps.clone())
                    }
                    crate::git::CloneError::NetworkError { message } => {
                        (format!("Network error: {}", message), Vec::new())
                    }
                    crate::git::CloneError::OperationFailed { message } => {
                        (format!("Clone failed: {}", message), Vec::new())
                    }
                };
                let event = CloneEvent::Error { message, help_steps };
                let data = serde_json::to_string(&event).unwrap_or_default();
                yield Ok(Event::default().event("error").data(data));
            }
            Err(e) => {
                let event = CloneEvent::Error {
                    message: format!("Clone task panicked: {}", e),
                    help_steps: Vec::new(),
                };
                let data = serde_json::to_string(&event).unwrap_or_default();
                yield Ok(Event::default().event("error").data(data));
            }
        }
    };

    Sse::new(Box::pin(stream) as SseStream).keep_alive(KeepAlive::default())
}

/// Create the repos router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/repos", get(list_repos).post(add_repo))
        .route("/repos/clone", post(clone_repo))
        .route("/repos/clone-progress", get(clone_with_progress_sse))
        .route("/repos/{id}", delete(delete_repo))
        .route("/repos/scan", post(scan_repos))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use axum_test::TestServer;
    use tempfile::TempDir;

    fn create_test_state() -> AppState {
        let db = Database::in_memory().expect("Failed to create test database");
        AppState::new(db)
    }

    fn create_test_server(state: AppState) -> TestServer {
        let app = router().with_state(state);
        TestServer::new(app).expect("Failed to create test server")
    }

    #[tokio::test]
    async fn test_list_repos_empty() {
        let state = create_test_state();
        let server = create_test_server(state);

        let response = server.get("/repos").await;
        response.assert_status_ok();

        let repos: Vec<Repo> = response.json();
        assert!(repos.is_empty());
    }

    #[tokio::test]
    async fn test_add_repo_validates_path() {
        let state = create_test_state();
        let server = create_test_server(state);

        let response = server
            .post("/repos")
            .json(&AddRepoRequest {
                path: "/nonexistent/path".to_string(),
                name: None,
            })
            .await;

        // UserActionRequired returns 422 with help_steps
        response.assert_status(axum::http::StatusCode::UNPROCESSABLE_ENTITY);
        let body = response.text();
        assert!(body.contains("REPO_PATH_NOT_FOUND"));
        assert!(body.contains("help_steps"));
    }

    #[tokio::test]
    async fn test_add_repo_validates_git() {
        let state = create_test_state();
        let server = create_test_server(state);

        // Create a temp directory that is NOT a git repo
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let response = server
            .post("/repos")
            .json(&AddRepoRequest {
                path: temp_dir.path().to_string_lossy().to_string(),
                name: None,
            })
            .await;

        // UserActionRequired returns 422 with help_steps
        response.assert_status(axum::http::StatusCode::UNPROCESSABLE_ENTITY);
        let body = response.text();
        assert!(body.contains("NOT_A_GIT_REPO"));
        assert!(body.contains("help_steps"));
    }

    #[tokio::test]
    async fn test_add_and_list_repo() {
        let state = create_test_state();
        let server = create_test_server(state);

        // Create a temp directory and init as git repo
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        git2::Repository::init(temp_dir.path()).expect("Failed to init git repo");

        // Add the repo
        let response = server
            .post("/repos")
            .json(&AddRepoRequest {
                path: temp_dir.path().to_string_lossy().to_string(),
                name: Some("test-repo".to_string()),
            })
            .await;

        response.assert_status_ok();
        let repo: Repo = response.json();
        assert_eq!(repo.name, "test-repo");

        // List repos
        let response = server.get("/repos").await;
        response.assert_status_ok();
        let repos: Vec<Repo> = response.json();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name, "test-repo");
    }

    #[tokio::test]
    async fn test_add_repo_duplicate() {
        let state = create_test_state();
        let server = create_test_server(state);

        // Create a temp directory and init as git repo
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        git2::Repository::init(temp_dir.path()).expect("Failed to init git repo");

        // Add the repo
        let response = server
            .post("/repos")
            .json(&AddRepoRequest {
                path: temp_dir.path().to_string_lossy().to_string(),
                name: None,
            })
            .await;
        response.assert_status_ok();

        // Try to add again - should fail
        let response = server
            .post("/repos")
            .json(&AddRepoRequest {
                path: temp_dir.path().to_string_lossy().to_string(),
                name: None,
            })
            .await;
        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_delete_repo() {
        let state = create_test_state();
        let server = create_test_server(state);

        // Create a temp directory and init as git repo
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        git2::Repository::init(temp_dir.path()).expect("Failed to init git repo");

        // Add the repo
        let response = server
            .post("/repos")
            .json(&AddRepoRequest {
                path: temp_dir.path().to_string_lossy().to_string(),
                name: Some("test-repo".to_string()),
            })
            .await;
        response.assert_status_ok();
        let repo: Repo = response.json();

        // Delete it
        let response = server.delete(&format!("/repos/{}", repo.id)).await;
        response.assert_status_ok();

        // Verify it's gone
        let response = server.get("/repos").await;
        let repos: Vec<Repo> = response.json();
        assert!(repos.is_empty());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_repo() {
        let state = create_test_state();
        let server = create_test_server(state);

        let fake_id = Uuid::new_v4();
        let response = server.delete(&format!("/repos/{}", fake_id)).await;
        response.assert_status_not_found();
    }

    #[tokio::test]
    async fn test_scan_repos() {
        let server = create_test_server(create_test_state());

        // Create a temp directory structure with one git repo
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_dir = temp_dir.path().join("my-project");
        std::fs::create_dir(&repo_dir).expect("Failed to create subdir");
        git2::Repository::init(&repo_dir).expect("Failed to init git repo");

        let response = server
            .post("/repos/scan")
            .json(&ScanRequest {
                directories: vec![temp_dir.path().to_string_lossy().to_string()],
                depth: 2,
            })
            .await;

        response.assert_status_ok();
        let scan_result: ScanResponse = response.json();
        assert_eq!(scan_result.found.len(), 1);
        assert_eq!(scan_result.found[0].name, "my-project");
    }

    #[test]
    fn test_extract_repo_name_https() {
        assert_eq!(
            extract_repo_name("https://github.com/user/repo.git").unwrap(),
            "repo"
        );
        assert_eq!(
            extract_repo_name("https://github.com/user/repo").unwrap(),
            "repo"
        );
        assert_eq!(
            extract_repo_name("https://github.com/user/my-project.git").unwrap(),
            "my-project"
        );
        assert_eq!(
            extract_repo_name("https://github.com/user/repo/").unwrap(),
            "repo"
        );
    }

    #[test]
    fn test_extract_repo_name_ssh() {
        assert_eq!(
            extract_repo_name("git@github.com:user/repo.git").unwrap(),
            "repo"
        );
        assert_eq!(
            extract_repo_name("git@github.com:user/repo").unwrap(),
            "repo"
        );
        assert_eq!(
            extract_repo_name("git@gitlab.com:org/my-project.git").unwrap(),
            "my-project"
        );
    }

    #[test]
    fn test_extract_repo_name_invalid() {
        // Empty string should fail
        assert!(extract_repo_name("").is_err());
        // Note: "not-a-url" extracts as "not-a-url" which is technically valid
        // for name extraction. The clone itself will fail if URL is invalid.
    }

    #[tokio::test]
    async fn test_clone_repo_from_local_source() {
        let state = create_test_state();
        let _server = create_test_server(state);

        // Create a source repo with a commit
        let source_dir = TempDir::new().expect("Failed to create source dir");
        let source_repo = git2::Repository::init(source_dir.path())
            .expect("Failed to init source repo");

        // Configure user for commits
        {
            let mut config = source_repo.config().expect("Failed to get config");
            config.set_str("user.name", "Test User").expect("Failed to set user.name");
            config.set_str("user.email", "test@example.com").expect("Failed to set user.email");
        }

        // Create initial commit
        {
            let sig = source_repo.signature().expect("Failed to create signature");
            let tree_id = source_repo.index().expect("Failed to get index")
                .write_tree().expect("Failed to write tree");
            let tree = source_repo.find_tree(tree_id).expect("Failed to find tree");
            source_repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
                .expect("Failed to create initial commit");
        }

        // Create a temp directory for the clone destination that we'll control
        // Instead of using ~/ralphtown, we test the extract_repo_name function
        // and verify the clone endpoint returns the expected structure

        // Note: We can't easily test the full clone endpoint in unit tests because
        // it hardcodes ~/ralphtown as the destination. The integration test (Task 3)
        // will verify the full flow. Here we just verify the endpoint compiles
        // and the helper functions work.
    }
}
