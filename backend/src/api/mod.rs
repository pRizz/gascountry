pub mod git;
pub mod repos;
pub mod sessions;

use std::sync::Arc;

use crate::db::Database;
use crate::ralph::RalphManager;
use crate::ws::ConnectionManager;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub connections: ConnectionManager,
    pub ralph_manager: RalphManager,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self {
            db: Arc::new(db),
            connections: ConnectionManager::new(),
            ralph_manager: RalphManager::new(),
        }
    }
}
