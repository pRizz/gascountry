//! Ralph process manager - spawns and tracks ralph CLI processes

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::db::models::SessionStatus as DbSessionStatus;
use crate::db::Database;
use crate::ws::messages::{OutputStream, ServerMessage, SessionStatus as WsSessionStatus};
use crate::ws::ConnectionManager;

/// Active process handle with metadata
struct ProcessHandle {
    child: Child,
    repo_id: Uuid,
}

/// Inner state for RalphManager
struct RalphManagerInner {
    /// Map of session_id -> active process handle
    processes: HashMap<Uuid, ProcessHandle>,
    /// Set of repo_ids with running processes (for 1-instance-per-repo constraint)
    active_repos: HashMap<Uuid, Uuid>, // repo_id -> session_id
}

/// Manages spawning and tracking of ralph CLI processes
#[derive(Clone)]
pub struct RalphManager {
    inner: Arc<RwLock<RalphManagerInner>>,
}

impl RalphManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(RalphManagerInner {
                processes: HashMap::new(),
                active_repos: HashMap::new(),
            })),
        }
    }

    /// Check if a repo already has a running ralph process
    pub async fn is_repo_busy(&self, repo_id: Uuid) -> bool {
        let inner = self.inner.read().await;
        inner.active_repos.contains_key(&repo_id)
    }

    /// Get the session ID of the running process for a repo
    pub async fn get_active_session_for_repo(&self, repo_id: Uuid) -> Option<Uuid> {
        let inner = self.inner.read().await;
        inner.active_repos.get(&repo_id).copied()
    }

    /// Check if a session has a running process
    pub async fn is_session_running(&self, session_id: Uuid) -> bool {
        let inner = self.inner.read().await;
        inner.processes.contains_key(&session_id)
    }

    /// Spawn a ralph process for a session
    ///
    /// # Arguments
    /// * `session_id` - The session to run ralph for
    /// * `repo_id` - The repository ID
    /// * `repo_path` - Filesystem path to the repository
    /// * `prompt` - The prompt to send to ralph
    /// * `db` - Database for updating session status
    /// * `connections` - Connection manager for broadcasting output
    ///
    /// # Returns
    /// Ok(()) if the process started successfully, Err if it couldn't start
    pub async fn run(
        &self,
        session_id: Uuid,
        repo_id: Uuid,
        repo_path: &str,
        prompt: &str,
        db: Arc<Database>,
        connections: ConnectionManager,
    ) -> Result<(), RalphError> {
        // Check if repo already has a running process
        if self.is_repo_busy(repo_id).await {
            return Err(RalphError::RepoBusy(repo_id));
        }

        // Check if session already has a running process
        if self.is_session_running(session_id).await {
            return Err(RalphError::SessionAlreadyRunning(session_id));
        }

        // Build the command
        let mut cmd = Command::new("ralph");
        cmd.arg("run")
            .arg("--autonomous")
            .arg("--prompt")
            .arg(prompt)
            .current_dir(repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        // On Unix, set up process group for signal handling
        #[cfg(unix)]
        {
            #[allow(unused_imports)]
            use std::os::unix::process::CommandExt;
            // SAFETY: setpgid is safe to call in pre_exec, it's a standard
            // POSIX function that sets the process group for signal handling
            unsafe {
                cmd.pre_exec(|| {
                    // Set this process as the process group leader
                    // This allows us to send signals to the entire process group
                    libc::setpgid(0, 0);
                    Ok(())
                });
            }
        }

        // Spawn the process
        let mut child = cmd.spawn().map_err(|e| RalphError::SpawnFailed(e.to_string()))?;

        // Take stdout and stderr handles
        let stdout = child.stdout.take().expect("stdout was configured");
        let stderr = child.stderr.take().expect("stderr was configured");

        // Register the process
        {
            let mut inner = self.inner.write().await;
            inner.processes.insert(
                session_id,
                ProcessHandle {
                    child,
                    repo_id,
                },
            );
            inner.active_repos.insert(repo_id, session_id);
        }

        // Update session status to running
        if let Err(e) = db.update_session_status(session_id, DbSessionStatus::Running) {
            tracing::error!("Failed to update session status: {}", e);
        }

        // Broadcast status update
        connections
            .broadcast(
                session_id,
                ServerMessage::Status {
                    session_id,
                    status: WsSessionStatus::Running,
                },
            )
            .await;

        // Spawn tasks to read stdout and stderr
        let manager_clone = self.clone();
        let db_clone = db.clone();
        let connections_clone = connections.clone();

        tokio::spawn(async move {
            let stdout_connections = connections_clone.clone();
            let stderr_connections = connections_clone.clone();

            // Spawn stdout reader
            let stdout_handle = tokio::spawn({
                let session_id = session_id;
                async move {
                    let reader = BufReader::new(stdout);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        stdout_connections
                            .broadcast(
                                session_id,
                                ServerMessage::Output {
                                    session_id,
                                    stream: OutputStream::Stdout,
                                    content: line,
                                },
                            )
                            .await;
                    }
                }
            });

            // Spawn stderr reader
            let stderr_handle = tokio::spawn({
                let session_id = session_id;
                async move {
                    let reader = BufReader::new(stderr);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        stderr_connections
                            .broadcast(
                                session_id,
                                ServerMessage::Output {
                                    session_id,
                                    stream: OutputStream::Stderr,
                                    content: line,
                                },
                            )
                            .await;
                    }
                }
            });

            // Wait for both readers to finish
            let _ = tokio::join!(stdout_handle, stderr_handle);

            // Process has finished - wait for exit status and cleanup
            manager_clone
                .handle_process_exit(session_id, repo_id, db_clone, connections_clone)
                .await;
        });

        Ok(())
    }

    /// Handle process exit - cleanup and update status
    async fn handle_process_exit(
        &self,
        session_id: Uuid,
        repo_id: Uuid,
        db: Arc<Database>,
        connections: ConnectionManager,
    ) {
        // Get the exit status
        let exit_status = {
            let mut inner = self.inner.write().await;
            if let Some(mut handle) = inner.processes.remove(&session_id) {
                inner.active_repos.remove(&repo_id);
                // Wait for the child to fully exit
                handle.child.wait().await.ok()
            } else {
                None
            }
        };

        // Determine final status based on exit code
        let final_status = match exit_status {
            Some(status) if status.success() => DbSessionStatus::Completed,
            Some(_) => DbSessionStatus::Error,
            None => DbSessionStatus::Error,
        };

        // Update database
        if let Err(e) = db.update_session_status(session_id, final_status) {
            tracing::error!("Failed to update session status: {}", e);
        }

        // Broadcast final status
        connections
            .broadcast(
                session_id,
                ServerMessage::Status {
                    session_id,
                    status: final_status.into(),
                },
            )
            .await;

        tracing::info!(
            "Ralph process for session {} finished with status: {:?}",
            session_id,
            final_status
        );
    }

    /// Cancel a running ralph process
    pub async fn cancel(
        &self,
        session_id: Uuid,
        db: Arc<Database>,
        connections: ConnectionManager,
    ) -> Result<(), RalphError> {
        let (child_id, repo_id) = {
            let inner = self.inner.read().await;
            if let Some(handle) = inner.processes.get(&session_id) {
                (handle.child.id(), handle.repo_id)
            } else {
                return Err(RalphError::NotRunning(session_id));
            }
        };

        // Send SIGTERM to the process group on Unix
        #[cfg(unix)]
        {
            use nix::sys::signal::{killpg, Signal};
            use nix::unistd::Pid;

            if let Some(pid) = child_id {
                let pgid = Pid::from_raw(pid as i32);
                if let Err(e) = killpg(pgid, Signal::SIGTERM) {
                    tracing::warn!("Failed to send SIGTERM to process group: {}", e);
                }

                // Wait a bit for graceful shutdown
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                // Check if still running and send SIGKILL if needed
                let inner = self.inner.read().await;
                if inner.processes.contains_key(&session_id) {
                    drop(inner);
                    if let Err(e) = killpg(pgid, Signal::SIGKILL) {
                        tracing::warn!("Failed to send SIGKILL to process group: {}", e);
                    }
                }
            }
        }

        // On non-Unix, just kill the child directly
        #[cfg(not(unix))]
        {
            let mut inner = self.inner.write().await;
            if let Some(handle) = inner.processes.get_mut(&session_id) {
                let _ = handle.child.kill().await;
            }
        }

        // Remove from tracking and update status
        {
            let mut inner = self.inner.write().await;
            inner.processes.remove(&session_id);
            inner.active_repos.remove(&repo_id);
        }

        // Update database
        if let Err(e) = db.update_session_status(session_id, DbSessionStatus::Cancelled) {
            tracing::error!("Failed to update session status: {}", e);
        }

        // Broadcast status
        connections
            .broadcast(
                session_id,
                ServerMessage::Status {
                    session_id,
                    status: WsSessionStatus::Cancelled,
                },
            )
            .await;

        tracing::info!("Ralph process for session {} cancelled", session_id);

        Ok(())
    }

    /// Get list of active sessions
    pub async fn active_sessions(&self) -> Vec<Uuid> {
        let inner = self.inner.read().await;
        inner.processes.keys().copied().collect()
    }
}

impl Default for RalphManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur when managing ralph processes
#[derive(Debug, thiserror::Error)]
pub enum RalphError {
    #[error("Repository {0} already has a running ralph process")]
    RepoBusy(Uuid),

    #[error("Session {0} already has a running process")]
    SessionAlreadyRunning(Uuid),

    #[error("Failed to spawn ralph process: {0}")]
    SpawnFailed(String),

    #[error("Session {0} has no running process")]
    NotRunning(Uuid),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_manager_creation() {
        let manager = RalphManager::new();
        assert!(manager.active_sessions().await.is_empty());
    }

    #[tokio::test]
    async fn test_repo_busy_detection() {
        let manager = RalphManager::new();
        let repo_id = Uuid::new_v4();

        assert!(!manager.is_repo_busy(repo_id).await);
    }

    #[tokio::test]
    async fn test_session_running_detection() {
        let manager = RalphManager::new();
        let session_id = Uuid::new_v4();

        assert!(!manager.is_session_running(session_id).await);
    }
}
