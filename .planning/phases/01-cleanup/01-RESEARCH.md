# Phase 1: Cleanup - Research

**Researched:** 2026-01-17
**Domain:** Rust error handling (thiserror, rusqlite, Axum), dead code removal
**Confidence:** HIGH

## Summary

This phase involves two distinct tasks: removing dead frontend code (mockData.ts) and improving error handling in the Rust backend's database layer.

The mockData.ts file is imported by `PromptInput.tsx` for `availableModels` and `quickActions` constants. These must be relocated before the file can be deleted.

The DB layer currently uses `.unwrap()` extensively when parsing UUIDs and timestamps from database rows. The standard Rust pattern is to replace these with proper `Result` propagation using `?`, wrapping parse failures in custom error variants. The existing `DbError` enum already has an `InvalidData(String)` variant suitable for this purpose.

**Primary recommendation:** Use `?` operator with `map_err()` to convert parse errors to `DbError::InvalidData`, including the problematic value in the error message. Update `AppError` to support 409/422 status codes with structured JSON responses per user decisions.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| thiserror | 2 | Custom error type derivation | De facto standard for library error types in Rust |
| rusqlite | 0.33 | SQLite database access | Already in use, provides Result-based API |
| axum | 0.8 | HTTP framework | Already in use, IntoResponse trait for error conversion |
| serde | 1 | JSON serialization | Already in use for error response bodies |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| uuid | 1 | UUID parsing | Already in use, `Uuid::parse_str` returns Result |
| chrono | 0.4 | DateTime parsing | Already in use, `parse_from_rfc3339` returns Result |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| thiserror | anyhow | anyhow is for applications, thiserror for libraries - keep thiserror for DB layer |
| Custom IntoResponse | axum-error | Extra dependency, current approach is standard |

**Installation:**
No new dependencies needed - all libraries already in `backend/Cargo.toml`.

## Architecture Patterns

### Recommended Error Flow
```
rusqlite::Error / uuid::Error / chrono::ParseError
         |
         v (map_err)
      DbError
         |
         v (From trait or map_err)
      AppError
         |
         v (IntoResponse)
  HTTP Response (JSON)
```

### Pattern 1: Parse with Error Context
**What:** Convert parse failures to domain errors with the problematic value included
**When to use:** Parsing UUIDs, timestamps, enums from database rows
**Example:**
```rust
// Source: thiserror docs + Rust error handling best practices
fn parse_uuid_field(value: &str, field_name: &str) -> DbResult<Uuid> {
    Uuid::parse_str(value).map_err(|e| {
        DbError::ParseError(format!(
            "invalid {} '{}': {}",
            field_name, value, e
        ))
    })
}
```

### Pattern 2: Row Mapping with Error Propagation
**What:** Replace `.unwrap()` with `?` in row mapping closures
**When to use:** All `query_row` and `query_map` callbacks
**Example:**
```rust
// Source: rusqlite docs + Axum error handling examples
conn.query_row(
    "SELECT id, created_at FROM repos WHERE id = ?1",
    params![id.to_string()],
    |row| {
        let id_str: String = row.get(0)?;
        let created_str: String = row.get(1)?;

        let id = Uuid::parse_str(&id_str)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(e),
            ))?;

        let created_at = DateTime::parse_from_rfc3339(&created_str)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                Box::new(e),
            ))?
            .with_timezone(&Utc);

        Ok(Repo { id, created_at, ... })
    },
)
```

### Pattern 3: Structured JSON Error Response
**What:** Return consistent JSON error format with code, message, details
**When to use:** All API error responses
**Example:**
```rust
// Source: Axum error handling example + user decision in CONTEXT.md
#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message, details) = match &self {
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                msg.clone(),
                None,
            ),
            AppError::ParseError { message, value, field } => (
                StatusCode::UNPROCESSABLE_ENTITY,  // 422
                "PARSE_ERROR",
                message.clone(),
                Some(json!({ "field": field, "value": value })),
            ),
            AppError::Conflict(msg) => (
                StatusCode::CONFLICT,  // 409
                "CONFLICT",
                msg.clone(),
                None,
            ),
            // ... other variants
        };

        let body = ErrorResponse {
            error: ErrorBody { code: code.to_string(), message, details },
        };

        (status, Json(body)).into_response()
    }
}
```

### Anti-Patterns to Avoid
- **`.unwrap()` in production code:** Panics crash the server on malformed data
- **Generic error messages:** "InvalidData" without context is unhelpful for debugging
- **Nested error conversion chains:** Convert once at the boundary, not repeatedly

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Error type derivation | Manual Error trait impls | `#[derive(thiserror::Error)]` | Boilerplate-free, handles Display/source |
| Error context | String concatenation | `map_err` with formatted message | Preserves error chain, type-safe |
| UUID parsing | Custom parser | `Uuid::parse_str` | Handles all UUID formats, returns Result |
| Timestamp parsing | Custom parser | `DateTime::parse_from_rfc3339` | RFC 3339 compliant, returns Result |

**Key insight:** The uuid and chrono crates already return `Result` types. The problem is `.unwrap()` calls, not missing functionality.

## Common Pitfalls

### Pitfall 1: rusqlite Closure Error Types
**What goes wrong:** `query_row` closures expect `rusqlite::Result`, but parse errors are different types
**Why it happens:** Parse errors (uuid::Error, chrono::ParseError) don't implement `Into<rusqlite::Error>`
**How to avoid:** Wrap parse errors using `rusqlite::Error::FromSqlConversionFailure`
**Warning signs:** Compile errors about mismatched Result types in closures

### Pitfall 2: Error Chain Loss
**What goes wrong:** Converting errors loses the original cause, making debugging hard
**Why it happens:** Using `DbError::InvalidData(e.to_string())` discards the source error
**How to avoid:** Include source error via `#[source]` attribute or in message string
**Warning signs:** Error logs showing only "invalid data" without the underlying cause

### Pitfall 3: Inconsistent HTTP Status Codes
**What goes wrong:** Parse errors return 500 instead of 422, conflicts return 400 instead of 409
**Why it happens:** All errors mapped to generic Internal/BadRequest
**How to avoid:** Add specific AppError variants for ParseError (422), Conflict (409)
**Warning signs:** Frontend can't distinguish error types for user feedback

### Pitfall 4: Mock Data Partial Import
**What goes wrong:** Deleting mockData.ts breaks PromptInput component
**Why it happens:** `availableModels` and `quickActions` are still imported
**How to avoid:** Relocate constants before deleting file, verify no imports remain
**Warning signs:** TypeScript compilation errors after deletion

## Code Examples

Verified patterns from official sources:

### DbError Enum (Enhanced)
```rust
// Source: thiserror docs + CONTEXT.md decisions
#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Failed to determine data directory")]
    NoDataDir,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Record not found")]
    NotFound,

    #[error("Parse error: {message}")]
    ParseError {
        message: String,
        value: String,
        field: String,
    },

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),
}
```

### AppError Enum (Enhanced)
```rust
// Source: Axum error handling example + CONTEXT.md decisions
#[derive(Debug)]
pub enum AppError {
    Internal(String),
    NotFound(String),
    BadRequest(String),
    Conflict(String),           // 409
    UnprocessableEntity {       // 422
        message: String,
        field: Option<String>,
        value: Option<String>,
    },
}

impl From<DbError> for AppError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::NotFound => AppError::NotFound("Resource not found".to_string()),
            DbError::ParseError { message, value, field } => AppError::UnprocessableEntity {
                message,
                field: Some(field),
                value: Some(value),
            },
            DbError::ConstraintViolation(msg) => AppError::Conflict(msg),
            other => AppError::Internal(other.to_string()),
        }
    }
}
```

### Safe Row Parsing Helper
```rust
// Source: Rust error handling best practices
fn parse_uuid(row: &Row, idx: usize, field: &str) -> rusqlite::Result<Uuid> {
    let value: String = row.get(idx)?;
    Uuid::parse_str(&value).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            idx,
            rusqlite::types::Type::Text,
            Box::new(DbError::ParseError {
                message: e.to_string(),
                value,
                field: field.to_string(),
            }),
        )
    })
}

fn parse_datetime(row: &Row, idx: usize, field: &str) -> rusqlite::Result<DateTime<Utc>> {
    let value: String = row.get(idx)?;
    DateTime::parse_from_rfc3339(&value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                idx,
                rusqlite::types::Type::Text,
                Box::new(DbError::ParseError {
                    message: e.to_string(),
                    value,
                    field: field.to_string(),
                }),
            )
        })
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `.unwrap()` in DB code | `?` with `map_err` | Rust best practice since 2018 | Prevents panics, enables graceful error handling |
| `anyhow` everywhere | `thiserror` for libraries | Clarified by dtolnay ~2020 | Type-safe error matching in calling code |
| Generic 500 errors | Specific status codes | REST best practices | Clients can handle errors appropriately |
| Plain text errors | Structured JSON | API standard | Programmatic error handling possible |

**Deprecated/outdated:**
- `failure` crate: Superseded by `thiserror`/`anyhow`
- `error-chain` crate: Superseded by `thiserror`

## Open Questions

Things that couldn't be fully resolved:

1. **Where to relocate `availableModels` and `quickActions`?**
   - What we know: Currently in mockData.ts, used by PromptInput.tsx
   - What's unclear: Should they move to a constants file, the component itself, or a config module?
   - Recommendation: Create `frontend/src/constants/index.ts` for app-wide constants

2. **Should enum parsing (SessionStatus, MessageRole, OutputStream) use the same error pattern?**
   - What we know: These use `from_str()` returning `Option`, then `.unwrap()`
   - What's unclear: Whether to change signature to return Result or use helper
   - Recommendation: Change to return `Result<Self, DbError>` for consistency

## Sources

### Primary (HIGH confidence)
- [thiserror docs.rs](https://docs.rs/thiserror) - Error derive macro syntax, #[error], #[from], #[source]
- [Axum error_handling module](https://docs.rs/axum/latest/axum/error_handling/index.html) - IntoResponse pattern, Infallible requirement
- [Axum error-handling example](https://github.com/tokio-rs/axum/blob/main/examples/error-handling/src/main.rs) - Complete IntoResponse implementation
- [rusqlite docs.rs](https://docs.rs/rusqlite/latest/rusqlite/) - Result type, Error enum, FromSqlConversionFailure

### Secondary (MEDIUM confidence)
- [Rust Error Handling Guide 2025](https://markaicode.com/rust-error-handling-2025-guide/) - Current best practices, thiserror vs anyhow
- [Thiserror Rust Guide 2025](https://generalistprogrammer.com/tutorials/thiserror-rust-crate-guide) - Crate tutorial
- [Elegant Error Handling in Axum](https://leapcell.io/blog/elegant-error-handling-in-axum-actix-web-with-intoresponse) - IntoResponse patterns

### Tertiary (LOW confidence)
- WebSearch aggregation on JSON error formats - verified against Axum examples

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Using existing dependencies, well-documented patterns
- Architecture: HIGH - Standard Rust error handling idioms
- Pitfalls: HIGH - Verified against actual codebase `.unwrap()` locations

**Research date:** 2026-01-17
**Valid until:** 60 days (stable Rust ecosystem, existing dependencies)
