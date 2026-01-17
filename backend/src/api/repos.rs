use std::path::Path;

use axum::{
    extract::{Path as AxumPath, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::models::Repo;
use crate::error::{AppError, AppResult};

use super::AppState;

/// Request body for adding a new repository
#[derive(Debug, Deserialize, Serialize)]
pub struct AddRepoRequest {
    /// Path to the git repository
    pub path: String,
    /// Optional name (defaults to directory name)
    pub name: Option<String>,
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

    // Verify path exists
    if !path.exists() {
        return Err(AppError::BadRequest(format!(
            "Path does not exist: {}",
            req.path
        )));
    }

    // Verify it's a git repository
    if git2::Repository::open(path).is_err() {
        return Err(AppError::BadRequest(format!(
            "Not a git repository: {}",
            req.path
        )));
    }

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

/// Create the repos router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/repos", get(list_repos).post(add_repo))
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

        response.assert_status_bad_request();
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

        response.assert_status_bad_request();
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
}
