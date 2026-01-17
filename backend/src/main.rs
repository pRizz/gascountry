pub mod api;
pub mod db;
mod error;
pub mod git;
pub mod ralph;
pub mod ws;

use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use api::AppState;
use db::Database;

pub use error::{AppError, AppResult};
pub use ralph::RalphManager;
pub use ws::ConnectionManager;

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

pub fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/health", get(health_check))
        .nest("/api", api::repos::router())
        .nest("/api", api::sessions::router())
        .nest("/api", api::git::router())
        .nest("/api", api::config::router())
        .nest("/api", ws::router())
        .with_state(state)
        .layer(cors)
}

/// Create app with in-memory database (for testing)
pub fn create_test_app() -> Router {
    let db = Database::in_memory().expect("Failed to create test database");
    let state = AppState::new(db);
    create_app(state)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database
    let db_path = Database::default_path().expect("Failed to determine database path");
    tracing::info!("Using database at: {:?}", db_path);

    let db = Database::new(db_path).expect("Failed to initialize database");
    let state = AppState::new(db);

    let app = create_app(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::info!("Ralphtown server listening on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_health_check_returns_200() {
        let app = create_test_app();
        let server = TestServer::new(app).unwrap();

        let response = server.get("/api/health").await;

        response.assert_status_ok();
        let body: HealthResponse = response.json();
        assert_eq!(body.status, "ok");
    }
}
