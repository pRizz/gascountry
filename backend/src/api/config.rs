use axum::{
    extract::{Path as AxumPath, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{AppError, AppResult};

use super::AppState;

/// Response for getting all config values
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub config: HashMap<String, String>,
}

/// Request body for updating config values
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateConfigRequest {
    pub config: HashMap<String, String>,
}

/// Response for getting a single config value
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigValueResponse {
    pub key: String,
    pub value: Option<String>,
}

/// Request body for setting a single config value
#[derive(Debug, Deserialize, Serialize)]
pub struct SetConfigValueRequest {
    pub value: String,
}

/// AI Backend option for Ralph
#[derive(Debug, Serialize, Deserialize)]
pub struct AiBackend {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// Response for listing available AI backends
#[derive(Debug, Serialize, Deserialize)]
pub struct BackendsResponse {
    pub backends: Vec<AiBackend>,
}

/// Preset option for Ralph
#[derive(Debug, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// Response for listing available presets
#[derive(Debug, Serialize, Deserialize)]
pub struct PresetsResponse {
    pub presets: Vec<Preset>,
}

/// Get all config values
async fn get_all_config(State(state): State<AppState>) -> AppResult<Json<ConfigResponse>> {
    let entries = state
        .db
        .list_config()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let config: HashMap<String, String> = entries.into_iter().collect();

    Ok(Json(ConfigResponse { config }))
}

/// Update multiple config values at once
async fn update_config(
    State(state): State<AppState>,
    Json(req): Json<UpdateConfigRequest>,
) -> AppResult<Json<ConfigResponse>> {
    for (key, value) in &req.config {
        state
            .db
            .set_config(key, value)
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    // Return updated config
    get_all_config(State(state)).await
}

/// Get a single config value by key
async fn get_config_value(
    State(state): State<AppState>,
    AxumPath(key): AxumPath<String>,
) -> AppResult<Json<ConfigValueResponse>> {
    let value = state
        .db
        .get_config(&key)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(ConfigValueResponse { key, value }))
}

/// Set a single config value
async fn set_config_value(
    State(state): State<AppState>,
    AxumPath(key): AxumPath<String>,
    Json(req): Json<SetConfigValueRequest>,
) -> AppResult<Json<ConfigValueResponse>> {
    state
        .db
        .set_config(&key, &req.value)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(ConfigValueResponse {
        key,
        value: Some(req.value),
    }))
}

/// Delete a config value
async fn delete_config_value(
    State(state): State<AppState>,
    AxumPath(key): AxumPath<String>,
) -> AppResult<Json<()>> {
    state
        .db
        .delete_config(&key)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(()))
}

/// List available AI backends
async fn list_backends() -> Json<BackendsResponse> {
    // These are the supported backends for Ralph/Claude Code
    let backends = vec![
        AiBackend {
            id: "claude".to_string(),
            name: "Claude (Anthropic)".to_string(),
            description: "Anthropic's Claude models via API".to_string(),
        },
        AiBackend {
            id: "bedrock".to_string(),
            name: "AWS Bedrock".to_string(),
            description: "Claude models via AWS Bedrock".to_string(),
        },
        AiBackend {
            id: "vertex".to_string(),
            name: "Google Vertex AI".to_string(),
            description: "Claude models via Google Cloud Vertex AI".to_string(),
        },
    ];

    Json(BackendsResponse { backends })
}

/// List available presets
async fn list_presets() -> Json<PresetsResponse> {
    // These are common workflow presets
    let presets = vec![
        Preset {
            id: "default".to_string(),
            name: "Default".to_string(),
            description: "Standard autonomous mode".to_string(),
        },
        Preset {
            id: "tdd-red-green".to_string(),
            name: "TDD Red-Green".to_string(),
            description: "Test-driven development: write failing test, then implement".to_string(),
        },
        Preset {
            id: "feature".to_string(),
            name: "Feature Development".to_string(),
            description: "Implement a new feature with proper planning".to_string(),
        },
        Preset {
            id: "debug".to_string(),
            name: "Debug".to_string(),
            description: "Investigate and fix bugs".to_string(),
        },
        Preset {
            id: "refactor".to_string(),
            name: "Refactor".to_string(),
            description: "Clean up and improve code structure".to_string(),
        },
        Preset {
            id: "review".to_string(),
            name: "Code Review".to_string(),
            description: "Review code and suggest improvements".to_string(),
        },
    ];

    Json(PresetsResponse { presets })
}

/// Create the config router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/config", get(get_all_config).put(update_config))
        .route(
            "/config/{key}",
            get(get_config_value)
                .put(set_config_value)
                .delete(delete_config_value),
        )
        .route("/config/presets", get(list_presets))
        .route("/config/backends", get(list_backends))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::AppState;
    use crate::db::Database;
    use axum_test::TestServer;

    fn create_test_state() -> AppState {
        let db = Database::in_memory().expect("Failed to create test database");
        AppState::new(db)
    }

    fn create_test_server(state: AppState) -> TestServer {
        let app = Router::new().merge(router()).with_state(state);
        TestServer::new(app).expect("Failed to create test server")
    }

    #[tokio::test]
    async fn test_get_all_config_empty() {
        let state = create_test_state();
        let server = create_test_server(state);

        let response = server.get("/config").await;
        response.assert_status_ok();

        let config: ConfigResponse = response.json();
        assert!(config.config.is_empty());
    }

    #[tokio::test]
    async fn test_set_and_get_config_value() {
        let state = create_test_state();
        let server = create_test_server(state);

        // Set a value
        let response = server
            .put("/config/test_key")
            .json(&SetConfigValueRequest {
                value: "test_value".to_string(),
            })
            .await;
        response.assert_status_ok();

        let result: ConfigValueResponse = response.json();
        assert_eq!(result.key, "test_key");
        assert_eq!(result.value, Some("test_value".to_string()));

        // Get the value
        let response = server.get("/config/test_key").await;
        response.assert_status_ok();

        let result: ConfigValueResponse = response.json();
        assert_eq!(result.key, "test_key");
        assert_eq!(result.value, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_get_nonexistent_config_value() {
        let state = create_test_state();
        let server = create_test_server(state);

        let response = server.get("/config/nonexistent").await;
        response.assert_status_ok();

        let result: ConfigValueResponse = response.json();
        assert_eq!(result.key, "nonexistent");
        assert!(result.value.is_none());
    }

    #[tokio::test]
    async fn test_update_multiple_config_values() {
        let state = create_test_state();
        let server = create_test_server(state);

        let mut config = HashMap::new();
        config.insert("key1".to_string(), "value1".to_string());
        config.insert("key2".to_string(), "value2".to_string());
        config.insert("key3".to_string(), "value3".to_string());

        let response = server
            .put("/config")
            .json(&UpdateConfigRequest { config })
            .await;
        response.assert_status_ok();

        let result: ConfigResponse = response.json();
        assert_eq!(result.config.len(), 3);
        assert_eq!(result.config.get("key1"), Some(&"value1".to_string()));
        assert_eq!(result.config.get("key2"), Some(&"value2".to_string()));
        assert_eq!(result.config.get("key3"), Some(&"value3".to_string()));
    }

    #[tokio::test]
    async fn test_delete_config_value() {
        let state = create_test_state();
        let server = create_test_server(state);

        // Set a value first
        server
            .put("/config/to_delete")
            .json(&SetConfigValueRequest {
                value: "will be deleted".to_string(),
            })
            .await;

        // Delete it
        let response = server.delete("/config/to_delete").await;
        response.assert_status_ok();

        // Verify it's gone
        let response = server.get("/config/to_delete").await;
        response.assert_status_ok();

        let result: ConfigValueResponse = response.json();
        assert!(result.value.is_none());
    }

    #[tokio::test]
    async fn test_list_backends() {
        let state = create_test_state();
        let server = create_test_server(state);

        let response = server.get("/config/backends").await;
        response.assert_status_ok();

        let result: BackendsResponse = response.json();
        assert!(!result.backends.is_empty());

        // Verify claude backend exists
        let claude = result.backends.iter().find(|b| b.id == "claude");
        assert!(claude.is_some());
    }

    #[tokio::test]
    async fn test_list_presets() {
        let state = create_test_state();
        let server = create_test_server(state);

        let response = server.get("/config/presets").await;
        response.assert_status_ok();

        let result: PresetsResponse = response.json();
        assert!(!result.presets.is_empty());

        // Verify default preset exists
        let default = result.presets.iter().find(|p| p.id == "default");
        assert!(default.is_some());

        // Verify tdd preset exists
        let tdd = result.presets.iter().find(|p| p.id == "tdd-red-green");
        assert!(tdd.is_some());
    }

    #[tokio::test]
    async fn test_update_existing_config_value() {
        let state = create_test_state();
        let server = create_test_server(state);

        // Set initial value
        server
            .put("/config/update_test")
            .json(&SetConfigValueRequest {
                value: "initial".to_string(),
            })
            .await;

        // Update the value
        let response = server
            .put("/config/update_test")
            .json(&SetConfigValueRequest {
                value: "updated".to_string(),
            })
            .await;
        response.assert_status_ok();

        // Verify update
        let response = server.get("/config/update_test").await;
        response.assert_status_ok();

        let result: ConfigValueResponse = response.json();
        assert_eq!(result.value, Some("updated".to_string()));
    }
}
