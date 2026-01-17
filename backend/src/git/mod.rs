//! Git operations module
//!
//! Provides git functionality for repository management:
//! - Read operations (status, log, branches, diff_stats) using git2 library
//! - Write operations (pull, push, commit, reset, checkout) using CLI subprocess

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use thiserror::Error;

/// Git operation errors
#[derive(Debug, Error)]
pub enum GitError {
    #[error("Not a git repository: {0}")]
    NotARepo(String),

    #[error("Git operation failed: {0}")]
    OperationFailed(String),

    #[error("Git command failed: {0}")]
    CommandFailed(String),

    #[error("Invalid branch name: {0}")]
    InvalidBranch(String),
}

pub type GitResult<T> = Result<T, GitError>;

/// File status in git working tree
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileStatusType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Untracked,
}

/// Status of a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatus {
    pub path: String,
    pub status: FileStatusType,
    /// For renamed files, the original path
    pub old_path: Option<String>,
}

/// Git repository status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub staged: Vec<FileStatus>,
    pub unstaged: Vec<FileStatus>,
    pub untracked: Vec<String>,
}

/// A git commit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub id: String,
    pub short_id: String,
    pub message: String,
    pub author: String,
    pub email: String,
    pub timestamp: String,
}

/// A git branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
}

/// File change statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDelta {
    pub path: String,
    pub added: usize,
    pub removed: usize,
}

/// Result of a git command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Git operations manager
pub struct GitManager;

impl GitManager {
    /// Get repository status using git2
    pub fn status(repo_path: &Path) -> GitResult<GitStatus> {
        let repo = git2::Repository::open(repo_path)
            .map_err(|e| GitError::NotARepo(e.message().to_string()))?;

        // Get current branch
        let branch = Self::get_current_branch(&repo)?;

        // Get ahead/behind counts
        let (ahead, behind) = Self::get_ahead_behind(&repo).unwrap_or((0, 0));

        // Get status
        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        let mut untracked = Vec::new();

        let statuses = repo
            .statuses(Some(
                git2::StatusOptions::new()
                    .include_untracked(true)
                    .recurse_untracked_dirs(true),
            ))
            .map_err(|e| GitError::OperationFailed(e.message().to_string()))?;

        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status = entry.status();

            // Index (staged) changes
            if status.is_index_new() {
                staged.push(FileStatus {
                    path: path.clone(),
                    status: FileStatusType::Added,
                    old_path: None,
                });
            } else if status.is_index_modified() {
                staged.push(FileStatus {
                    path: path.clone(),
                    status: FileStatusType::Modified,
                    old_path: None,
                });
            } else if status.is_index_deleted() {
                staged.push(FileStatus {
                    path: path.clone(),
                    status: FileStatusType::Deleted,
                    old_path: None,
                });
            } else if status.is_index_renamed() {
                staged.push(FileStatus {
                    path: path.clone(),
                    status: FileStatusType::Renamed,
                    old_path: entry.head_to_index().and_then(|d| d.old_file().path().map(|p| p.to_string_lossy().to_string())),
                });
            }

            // Working tree (unstaged) changes
            if status.is_wt_new() {
                untracked.push(path.clone());
            } else if status.is_wt_modified() {
                unstaged.push(FileStatus {
                    path: path.clone(),
                    status: FileStatusType::Modified,
                    old_path: None,
                });
            } else if status.is_wt_deleted() {
                unstaged.push(FileStatus {
                    path: path.clone(),
                    status: FileStatusType::Deleted,
                    old_path: None,
                });
            } else if status.is_wt_renamed() {
                unstaged.push(FileStatus {
                    path: path.clone(),
                    status: FileStatusType::Renamed,
                    old_path: entry.index_to_workdir().and_then(|d| d.old_file().path().map(|p| p.to_string_lossy().to_string())),
                });
            }
        }

        Ok(GitStatus {
            branch,
            ahead,
            behind,
            staged,
            unstaged,
            untracked,
        })
    }

    /// Get recent commit log using git2
    pub fn log(repo_path: &Path, limit: usize) -> GitResult<Vec<Commit>> {
        let repo = git2::Repository::open(repo_path)
            .map_err(|e| GitError::NotARepo(e.message().to_string()))?;

        let mut revwalk = repo
            .revwalk()
            .map_err(|e| GitError::OperationFailed(e.message().to_string()))?;

        // Start from HEAD
        revwalk
            .push_head()
            .map_err(|e| GitError::OperationFailed(e.message().to_string()))?;

        let mut commits = Vec::new();
        for oid in revwalk.take(limit) {
            let oid = oid.map_err(|e| GitError::OperationFailed(e.message().to_string()))?;
            let commit = repo
                .find_commit(oid)
                .map_err(|e| GitError::OperationFailed(e.message().to_string()))?;

            let author = commit.author();
            let time = commit.time();
            let timestamp = chrono::DateTime::from_timestamp(time.seconds(), 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default();

            commits.push(Commit {
                id: oid.to_string(),
                short_id: oid.to_string()[..7.min(oid.to_string().len())].to_string(),
                message: commit.message().unwrap_or("").trim().to_string(),
                author: author.name().unwrap_or("").to_string(),
                email: author.email().unwrap_or("").to_string(),
                timestamp,
            });
        }

        Ok(commits)
    }

    /// List branches using git2
    pub fn branches(repo_path: &Path) -> GitResult<Vec<Branch>> {
        let repo = git2::Repository::open(repo_path)
            .map_err(|e| GitError::NotARepo(e.message().to_string()))?;

        let current_branch = Self::get_current_branch(&repo).unwrap_or_default();

        let mut branches = Vec::new();

        // Local branches
        let local_branches = repo
            .branches(Some(git2::BranchType::Local))
            .map_err(|e| GitError::OperationFailed(e.message().to_string()))?;

        for branch in local_branches {
            let (branch, _) = branch.map_err(|e| GitError::OperationFailed(e.message().to_string()))?;
            let name = branch.name().ok().flatten().unwrap_or("").to_string();

            let upstream = branch
                .upstream()
                .ok()
                .and_then(|u| u.name().ok().flatten().map(|s| s.to_string()));

            branches.push(Branch {
                name: name.clone(),
                is_current: name == current_branch,
                is_remote: false,
                upstream,
            });
        }

        // Remote branches
        let remote_branches = repo
            .branches(Some(git2::BranchType::Remote))
            .map_err(|e| GitError::OperationFailed(e.message().to_string()))?;

        for branch in remote_branches {
            let (branch, _) = branch.map_err(|e| GitError::OperationFailed(e.message().to_string()))?;
            let name = branch.name().ok().flatten().unwrap_or("").to_string();

            // Skip HEAD references
            if name.ends_with("/HEAD") {
                continue;
            }

            branches.push(Branch {
                name,
                is_current: false,
                is_remote: true,
                upstream: None,
            });
        }

        Ok(branches)
    }

    /// Get diff statistics for uncommitted changes
    pub fn diff_stats(repo_path: &Path) -> GitResult<Vec<FileDelta>> {
        let repo = git2::Repository::open(repo_path)
            .map_err(|e| GitError::NotARepo(e.message().to_string()))?;

        let mut deltas = Vec::new();

        // Get HEAD tree
        let head = repo.head().ok();
        let head_tree = head.as_ref().and_then(|h| h.peel_to_tree().ok());

        // Diff against HEAD (includes both staged and unstaged)
        let diff = repo
            .diff_tree_to_workdir_with_index(head_tree.as_ref(), None)
            .map_err(|e| GitError::OperationFailed(e.message().to_string()))?;

        let stats = diff
            .stats()
            .map_err(|e| GitError::OperationFailed(e.message().to_string()))?;

        // Get per-file stats
        for i in 0..diff.deltas().len() {
            if let Some(delta) = diff.get_delta(i) {
                let path = delta
                    .new_file()
                    .path()
                    .or_else(|| delta.old_file().path())
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Get patch for line counts
                if let Ok(patch) = git2::Patch::from_diff(&diff, i) {
                    if let Some(patch) = patch {
                        let (_, additions, deletions) = patch.line_stats().unwrap_or((0, 0, 0));
                        deltas.push(FileDelta {
                            path,
                            added: additions,
                            removed: deletions,
                        });
                    }
                }
            }
        }

        // Handle case where stats shows changes but no deltas (binary files, etc.)
        if deltas.is_empty() && stats.files_changed() > 0 {
            // Fall back to just reporting overall stats
            deltas.push(FileDelta {
                path: "(binary or unreadable files)".to_string(),
                added: stats.insertions(),
                removed: stats.deletions(),
            });
        }

        Ok(deltas)
    }

    // --- Write operations using CLI subprocess ---

    /// Execute git pull
    pub fn pull(repo_path: &Path) -> GitResult<CommandOutput> {
        Self::run_git_command(repo_path, &["pull"])
    }

    /// Execute git push
    pub fn push(repo_path: &Path) -> GitResult<CommandOutput> {
        Self::run_git_command(repo_path, &["push"])
    }

    /// Execute git commit with message
    pub fn commit(repo_path: &Path, message: &str) -> GitResult<CommandOutput> {
        Self::run_git_command(repo_path, &["commit", "-m", message])
    }

    /// Execute git reset --hard
    pub fn reset_hard(repo_path: &Path) -> GitResult<CommandOutput> {
        Self::run_git_command(repo_path, &["reset", "--hard"])
    }

    /// Execute git checkout to switch branch
    pub fn checkout(repo_path: &Path, branch: &str) -> GitResult<CommandOutput> {
        // Validate branch name (basic sanity check)
        if branch.contains("..") || branch.starts_with('-') || branch.contains('\0') {
            return Err(GitError::InvalidBranch(branch.to_string()));
        }
        Self::run_git_command(repo_path, &["checkout", branch])
    }

    /// Stage all changes (git add -A)
    pub fn add_all(repo_path: &Path) -> GitResult<CommandOutput> {
        Self::run_git_command(repo_path, &["add", "-A"])
    }

    // --- Helper methods ---

    fn get_current_branch(repo: &git2::Repository) -> GitResult<String> {
        let head = repo
            .head()
            .map_err(|e| GitError::OperationFailed(e.message().to_string()))?;

        if head.is_branch() {
            Ok(head
                .shorthand()
                .unwrap_or("HEAD")
                .to_string())
        } else {
            // Detached HEAD
            Ok(head
                .target()
                .map(|oid| oid.to_string()[..7].to_string())
                .unwrap_or_else(|| "HEAD".to_string()))
        }
    }

    fn get_ahead_behind(repo: &git2::Repository) -> GitResult<(usize, usize)> {
        let head = repo.head().ok();
        let head_ref = head.as_ref().and_then(|h| h.shorthand());

        if let Some(branch_name) = head_ref {
            // Try to find upstream
            if let Ok(branch) = repo.find_branch(branch_name, git2::BranchType::Local) {
                if let Ok(upstream) = branch.upstream() {
                    let local_oid = repo.head().ok().and_then(|h| h.target());
                    let upstream_oid = upstream.get().target();

                    if let (Some(local), Some(upstream)) = (local_oid, upstream_oid) {
                        if let Ok((ahead, behind)) = repo.graph_ahead_behind(local, upstream) {
                            return Ok((ahead, behind));
                        }
                    }
                }
            }
        }

        Ok((0, 0))
    }

    fn run_git_command(repo_path: &Path, args: &[&str]) -> GitResult<CommandOutput> {
        let output = Command::new("git")
            .current_dir(repo_path)
            .args(args)
            .output()
            .map_err(|e| GitError::CommandFailed(format!("Failed to run git: {}", e)))?;

        Ok(CommandOutput {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, git2::Repository) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo = git2::Repository::init(temp_dir.path()).expect("Failed to init repo");

        // Configure user for commits
        {
            let mut config = repo.config().expect("Failed to get config");
            config.set_str("user.name", "Test User").expect("Failed to set user.name");
            config.set_str("user.email", "test@example.com").expect("Failed to set user.email");
        }

        // Create initial commit
        {
            let sig = repo.signature().expect("Failed to create signature");
            let tree_id = repo.index().expect("Failed to get index").write_tree().expect("Failed to write tree");
            let tree = repo.find_tree(tree_id).expect("Failed to find tree");
            repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
                .expect("Failed to create initial commit");
        }

        (temp_dir, repo)
    }

    #[test]
    fn test_status_clean_repo() {
        let (temp_dir, _repo) = create_test_repo();

        let status = GitManager::status(temp_dir.path()).expect("Failed to get status");

        assert!(!status.branch.is_empty());
        assert!(status.staged.is_empty());
        assert!(status.unstaged.is_empty());
        assert!(status.untracked.is_empty());
    }

    #[test]
    fn test_status_with_untracked() {
        let (temp_dir, _repo) = create_test_repo();

        // Create an untracked file
        fs::write(temp_dir.path().join("new_file.txt"), "content").expect("Failed to write file");

        let status = GitManager::status(temp_dir.path()).expect("Failed to get status");

        assert_eq!(status.untracked.len(), 1);
        assert!(status.untracked.contains(&"new_file.txt".to_string()));
    }

    #[test]
    fn test_status_with_modified() {
        let (temp_dir, repo) = create_test_repo();

        // Create and commit a file
        let file_path = temp_dir.path().join("tracked.txt");
        fs::write(&file_path, "initial").expect("Failed to write file");

        // Stage and commit
        let mut index = repo.index().expect("Failed to get index");
        index.add_path(Path::new("tracked.txt")).expect("Failed to add file");
        index.write().expect("Failed to write index");
        let tree_id = index.write_tree().expect("Failed to write tree");
        let tree = repo.find_tree(tree_id).expect("Failed to find tree");
        let sig = repo.signature().expect("Failed to create signature");
        let parent = repo.head().expect("Failed to get HEAD").peel_to_commit().expect("Failed to peel to commit");
        repo.commit(Some("HEAD"), &sig, &sig, "Add file", &tree, &[&parent])
            .expect("Failed to commit");

        // Now modify the file
        fs::write(&file_path, "modified").expect("Failed to modify file");

        let status = GitManager::status(temp_dir.path()).expect("Failed to get status");

        assert_eq!(status.unstaged.len(), 1);
        assert_eq!(status.unstaged[0].status, FileStatusType::Modified);
    }

    #[test]
    fn test_log() {
        let (temp_dir, _repo) = create_test_repo();

        let commits = GitManager::log(temp_dir.path(), 10).expect("Failed to get log");

        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].message, "Initial commit");
    }

    #[test]
    fn test_branches() {
        let (temp_dir, _repo) = create_test_repo();

        let branches = GitManager::branches(temp_dir.path()).expect("Failed to get branches");

        // Should have at least one local branch
        let local_branches: Vec<_> = branches.iter().filter(|b| !b.is_remote).collect();
        assert!(!local_branches.is_empty());

        // Current branch should be marked
        let current = branches.iter().find(|b| b.is_current);
        assert!(current.is_some());
    }

    #[test]
    fn test_diff_stats_no_changes() {
        let (temp_dir, _repo) = create_test_repo();

        let deltas = GitManager::diff_stats(temp_dir.path()).expect("Failed to get diff stats");

        assert!(deltas.is_empty());
    }

    #[test]
    fn test_diff_stats_with_changes() {
        let (temp_dir, repo) = create_test_repo();

        // Create and commit a file
        let file_path = temp_dir.path().join("tracked.txt");
        fs::write(&file_path, "line1\nline2\nline3\n").expect("Failed to write file");

        let mut index = repo.index().expect("Failed to get index");
        index.add_path(Path::new("tracked.txt")).expect("Failed to add file");
        index.write().expect("Failed to write index");
        let tree_id = index.write_tree().expect("Failed to write tree");
        let tree = repo.find_tree(tree_id).expect("Failed to find tree");
        let sig = repo.signature().expect("Failed to create signature");
        let parent = repo.head().expect("Failed to get HEAD").peel_to_commit().expect("Failed to peel to commit");
        repo.commit(Some("HEAD"), &sig, &sig, "Add file", &tree, &[&parent])
            .expect("Failed to commit");

        // Modify the file
        fs::write(&file_path, "line1\nmodified\nline3\nnew line\n").expect("Failed to modify file");

        let deltas = GitManager::diff_stats(temp_dir.path()).expect("Failed to get diff stats");

        assert!(!deltas.is_empty());
        let delta = &deltas[0];
        assert_eq!(delta.path, "tracked.txt");
        assert!(delta.added > 0 || delta.removed > 0);
    }

    #[test]
    fn test_not_a_repo() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        // Don't initialize as git repo

        let result = GitManager::status(temp_dir.path());
        assert!(matches!(result, Err(GitError::NotARepo(_))));
    }

    #[test]
    fn test_checkout_invalid_branch() {
        let result = GitManager::checkout(Path::new("/tmp"), "--invalid");
        assert!(matches!(result, Err(GitError::InvalidBranch(_))));

        let result = GitManager::checkout(Path::new("/tmp"), "foo..bar");
        assert!(matches!(result, Err(GitError::InvalidBranch(_))));
    }
}
