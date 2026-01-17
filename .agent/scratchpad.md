# Ralphtown Implementation Scratchpad

## Current Focus: Step 11 Complete - Moving to Step 12

### Progress Checklist (from plan.md)
- [x] Step 0: Rename project from Gascountry to Ralphtown
- [x] Step 1: Project restructure to monorepo
- [x] Step 2: Backend scaffold with Axum
- [x] Step 3: Database layer with SQLite
- [x] Step 4: Repository management API
- [x] Step 5: Session management API
- [x] Step 6: WebSocket infrastructure
- [x] Step 7: Ralph process spawning
- [x] Step 8: Output streaming to WebSocket
- [x] Step 9: Interrupt/cancel functionality
- [x] Step 10: Git operations
- [x] Step 11: Frontend API integration
- [ ] Step 12: Frontend WebSocket integration
- [ ] Step 13: Configuration management
- [ ] Step 14: Service installation
- [ ] Step 15: Cargo install packaging
- [ ] Step 16: Polish and integration testing

---

## Step 0 - COMPLETED

### Changes Made
- [x] Renamed `src/types/gastown.ts` to `src/types/ralphtown.ts`
- [x] Renamed `GastownInstance` type to `RalphtownInstance`
- [x] Renamed `src/components/gastown/` directory to `src/components/ralphtown/`
- [x] Updated all imports across codebase (gastown -> ralphtown)
- [x] Updated `mockGastownInstances` to `mockRalphtownInstances`
- [x] Updated `package.json` name to "ralphtown"
- [x] Updated `index.html` title and meta tags
- [x] Updated `README.md` with Ralphtown branding
- [x] Updated CSS comment "Gastown specific tokens" -> "Ralphtown specific tokens"
- [x] Updated UI text:
  - "Gascountry" header -> "Ralphtown"
  - "Search gastowns..." -> "Search sessions..."
  - "New gascountry" -> "New session"
  - "gascountry source code" -> "ralphtown source code"
  - "Gastown spawned" toast -> "Session started"
  - "Ask gastown to build..." -> "Ask Ralph to build..."
- [x] Updated GitHub URL to pRizz/ralphtown
- [x] Renamed callback props: onNewGastown -> onNewSession, onSpawnGastown -> onStartSession

### Verification
- Build: ✅ PASS
- Tests: ✅ PASS
- Grep for gastown/gascountry in src/: ✅ No matches

---

## Step 1 - COMPLETED

### Changes Made
- [x] Created `/frontend` directory and moved all React code into it
- [x] Created `/backend` directory with Rust project scaffold
- [x] Created workspace `Cargo.toml` at root with `members = ["backend"]`
- [x] Frontend paths already use relative references - no changes needed
- [x] Updated `.gitignore` for Rust artifacts (target/, Cargo.lock)
- [x] Created `backend/src/main.rs` with minimal Axum server and health endpoint

### Directory Structure
```
ralphtown/
├── Cargo.toml              # Workspace manifest
├── frontend/
│   ├── package.json
│   ├── vite.config.ts
│   ├── src/
│   └── ...
├── backend/
│   ├── Cargo.toml
│   └── src/main.rs
└── README.md
```

### Verification
- Frontend build: ✅ PASS (`npm run build` from frontend/)
- Frontend tests: ✅ PASS (1/1)
- Backend check: ✅ PASS (`cargo check` from root)

---

## Step 2 - COMPLETED

### Changes Made
- [x] Added `backend/src/error.rs` with AppError enum (Internal, NotFound, BadRequest)
- [x] Error types implement IntoResponse for Axum HTTP responses
- [x] Added AppResult<T> type alias for Result<T, AppError>
- [x] Refactored main.rs to export `create_app()` function for testing
- [x] Added integration test for health endpoint using axum-test v18
- [x] Made HealthResponse public and derive Deserialize for test assertions

### Files Added/Modified
- `backend/src/error.rs` (new) - Error handling types
- `backend/src/main.rs` - Added error module, create_app(), and test
- `backend/Cargo.toml` - Added axum-test v18 dev dependency

### Verification
- Backend cargo check: ✅ PASS
- Backend cargo test: ✅ PASS (1 test)
- Frontend build: ✅ PASS
- Frontend tests: ✅ PASS (1 test)

---

## Step 3 - COMPLETED

### Changes Made
- [x] Added dependencies: rusqlite 0.33 (bundled), dirs 6, uuid 1 (v4, serde), chrono 0.4 (serde), thiserror 2, tempfile 3 (dev)
- [x] Created `backend/src/db/schema.rs` - Schema SQL for repos, sessions, messages, output_logs, config tables with indexes
- [x] Created `backend/src/db/models.rs` - Rust structs: Repo, Session, SessionStatus, Message, MessageRole, OutputLog, OutputStream, ConfigEntry
- [x] Created `backend/src/db/mod.rs` - Database struct with connection pool (Arc<Mutex<Connection>>)
- [x] Implemented `Database::new(path)` and `Database::in_memory()` for testing
- [x] Implemented `Database::default_path()` using dirs crate → `~/.local/share/ralphtown/ralphtown.db`
- [x] Full CRUD operations: repos, sessions, messages, config
- [x] Session status updates with timestamps
- [x] Foreign key cascade deletes enabled

### Files Added/Modified
- `backend/Cargo.toml` - Added new dependencies
- `backend/src/db/mod.rs` (new) - Database struct, connection management, CRUD operations
- `backend/src/db/schema.rs` (new) - Schema creation SQL
- `backend/src/db/models.rs` (new) - Rust structs matching tables
- `backend/src/main.rs` - Added `pub mod db;`

### Verification
- Backend cargo check: ✅ PASS
- Backend cargo test: ✅ PASS (7 tests - 1 health + 6 db tests)
- Frontend tests: ✅ PASS (1 test)

---

## Step 4 - COMPLETED

### Changes Made
- [x] Added git2 v0.20 dependency to Cargo.toml
- [x] Created `backend/src/api/mod.rs` - API module with AppState struct
- [x] Created `backend/src/api/repos.rs` - Repository management endpoints
- [x] Injected Database into Axum state via AppState wrapper
- [x] `GET /api/repos` - List all repositories
- [x] `POST /api/repos` - Add repo with git validation
- [x] `DELETE /api/repos/{id}` - Remove repo by ID
- [x] `POST /api/repos/scan` - Scan directories for git repos (recursive with depth limit)
- [x] Canonicalize paths before storage for consistency
- [x] Validate paths exist and are valid git repositories
- [x] Added 8 integration tests for all endpoints

### Files Added/Modified
- `backend/Cargo.toml` - Added git2 dependency
- `backend/src/api/mod.rs` (new) - AppState struct, module exports
- `backend/src/api/repos.rs` (new) - Repository endpoints and tests
- `backend/src/main.rs` - Added api module, create_app with state, create_test_app helper

### Verification
- Backend cargo test: ✅ PASS (15 tests - 7 db + 1 health + 7 repo API)
- Frontend tests: ✅ PASS (1 test)

---

## Step 5 - COMPLETED

### Changes Made
- [x] Created `backend/src/api/sessions.rs` - Session management endpoints
- [x] `POST /api/sessions` - Create session for a repo (validates repo exists)
- [x] `GET /api/sessions` - List all sessions
- [x] `GET /api/sessions/{id}` - Get session details with messages (SessionDetails response)
- [x] `DELETE /api/sessions/{id}` - Delete session
- [x] Added sessions module to `backend/src/api/mod.rs`
- [x] Wired sessions router into `backend/src/main.rs`
- [x] Added 7 integration tests covering all endpoints and edge cases

### Files Added/Modified
- `backend/src/api/sessions.rs` (new) - Session endpoints with tests
- `backend/src/api/mod.rs` - Added sessions module export
- `backend/src/main.rs` - Nested sessions router

### Verification
- Backend cargo test: ✅ PASS (22 tests - 6 db + 1 health + 8 repo API + 7 session API)
- Frontend tests: ✅ PASS (1 test)

---

## Step 6 - COMPLETED

### Changes Made
- [x] Added axum `ws` feature and `futures` 0.3 dependency to Cargo.toml
- [x] Created `backend/src/ws/messages.rs` - Client/Server message types with serde serialization
  - ClientMessage: Subscribe, Unsubscribe, Cancel, Ping
  - ServerMessage: Subscribed, Unsubscribed, Output, Status, Error, Pong
  - OutputStream enum: Stdout, Stderr
  - SessionStatus enum: Idle, Running, Completed, Error, Cancelled
- [x] Created `backend/src/ws/connections.rs` - ConnectionManager with broadcast channels
  - Uses tokio broadcast channels for pub/sub per session
  - Tracks connections and their subscriptions
  - Supports multiple subscribers per session
  - Automatic cleanup on disconnect
- [x] Created `backend/src/ws/mod.rs` - WebSocket handler
  - Handles WebSocket upgrade at `/api/ws`
  - Processes client messages (subscribe, unsubscribe, cancel, ping)
  - Spawns tasks to forward broadcast messages to connected clients
- [x] Updated `backend/src/api/mod.rs` - Added ConnectionManager to AppState
- [x] Updated `backend/src/main.rs` - Added ws module and nested ws router

### Files Added/Modified
- `backend/Cargo.toml` - Added axum ws feature, futures
- `backend/src/ws/mod.rs` (new) - WebSocket handler and router
- `backend/src/ws/messages.rs` (new) - Message type definitions
- `backend/src/ws/connections.rs` (new) - Connection manager with broadcast
- `backend/src/api/mod.rs` - Added ConnectionManager to AppState
- `backend/src/main.rs` - Added ws module export and router

### Verification
- Backend cargo check: ✅ PASS
- Backend cargo test: ✅ PASS (29 tests - 6 db + 1 health + 8 repo API + 7 session API + 4 ws connections + 3 ws messages)
- Frontend tests: ✅ PASS (1 test)

---

## Step 7 - COMPLETED

### Changes Made
- [x] Created `backend/src/ralph/mod.rs` - RalphManager struct for process spawning and tracking
- [x] Implemented `run()` method that spawns `ralph run --autonomous --prompt "..."`
- [x] Process group setup on Unix via `libc::setpgid(0, 0)` in `pre_exec`
- [x] Track running processes by session_id in `HashMap<Uuid, ProcessHandle>`
- [x] Enforce 1 instance per repo constraint via `active_repos: HashMap<Uuid, Uuid>` (repo_id -> session_id)
- [x] Capture exit code and update session status (Completed/Error based on exit code)
- [x] Added `POST /api/sessions/{id}/run` endpoint with RunSessionRequest/RunSessionResponse
- [x] Integrated RalphManager into AppState
- [x] Updated WebSocket cancel handler to use RalphManager.cancel()
- [x] Added nix 0.29 and libc 0.2 dependencies for signal handling

### Files Added/Modified
- `backend/Cargo.toml` - Added nix, libc dependencies
- `backend/src/ralph/mod.rs` (new) - RalphManager struct with run/cancel methods
- `backend/src/main.rs` - Added ralph module export
- `backend/src/api/mod.rs` - Added RalphManager to AppState
- `backend/src/api/sessions.rs` - Added run_session endpoint
- `backend/src/ws/mod.rs` - Updated Cancel handler to use RalphManager

### Verification
- Backend cargo test: ✅ PASS (32 tests - 6 db + 1 health + 8 repo API + 7 session API + 4 ws connections + 3 ws messages + 3 ralph)
- Frontend tests: ✅ PASS (1 test)

---

## Step 8 - COMPLETED

### Changes Made
- [x] Added `insert_output_log(session_id, stream, content)` to Database
- [x] Added `list_output_logs(session_id, stream_filter, limit, offset)` to Database
- [x] Added `delete_output_logs(session_id)` to Database
- [x] Updated RalphManager stdout/stderr readers to persist output alongside WebSocket broadcast
- [x] Added `GET /api/sessions/{id}/output` endpoint with query params:
  - `?stream=stdout|stderr` - filter by stream type
  - `?limit=N` - limit results
  - `?offset=N` - pagination offset
- [x] Added OutputQueryParams, OutputResponse DTOs
- [x] Added 10 new tests (2 db tests + 3 api tests for output logs)

### Files Modified
- `backend/src/db/mod.rs` - Added output_logs CRUD operations + tests
- `backend/src/ralph/mod.rs` - Added db persistence alongside broadcast
- `backend/src/api/sessions.rs` - Added output endpoint + tests

### Verification
- Backend cargo test: ✅ PASS (37 tests)
- Frontend tests: ✅ PASS (1 test)

---

## Step 9 - COMPLETED

### Changes Made
- [x] Reviewed existing cancel implementation in RalphManager - already has SIGTERM → wait 5s → SIGKILL
- [x] WebSocket cancel handler already routes to RalphManager.cancel()
- [x] Session status transitions to Cancelled implemented
- [x] WebSocket status broadcast on cancel implemented
- [x] Added `POST /api/sessions/{id}/cancel` REST endpoint for API parity
- [x] Added 2 integration tests for cancel endpoint (nonexistent session, not running)

### Files Modified
- `backend/src/api/sessions.rs` - Added cancel_session endpoint, CancelSessionResponse struct, route, and tests

### Verification
- Backend cargo test: ✅ PASS (39 tests - 6 db + 1 health + 8 repo API + 9 session API + 4 ws connections + 3 ws messages + 3 ralph)
- Frontend tests: ✅ PASS (1 test)

---

## Step 10 - COMPLETED

### Changes Made
- [x] Created `backend/src/git/mod.rs` - GitManager struct for git operations
- [x] Implemented read operations using git2:
  - `status()` - Get repo status (branch, ahead/behind, staged, unstaged, untracked)
  - `log()` - Get commit history with limit
  - `branches()` - List local and remote branches
  - `diff_stats()` - Get file changes with added/removed line counts
- [x] Implemented write operations using CLI subprocess:
  - `pull()` - Execute git pull
  - `push()` - Execute git push
  - `commit()` - Execute git commit with message
  - `reset_hard()` - Execute git reset --hard
  - `checkout()` - Switch to branch
  - `add_all()` - Stage all changes
- [x] Created `backend/src/api/git.rs` - REST endpoints for git operations:
  - `GET /api/sessions/{id}/git/status`
  - `GET /api/sessions/{id}/git/log?limit=20`
  - `GET /api/sessions/{id}/git/branches`
  - `GET /api/sessions/{id}/git/diff`
  - `POST /api/sessions/{id}/git/pull`
  - `POST /api/sessions/{id}/git/push`
  - `POST /api/sessions/{id}/git/commit`
  - `POST /api/sessions/{id}/git/reset` (requires confirm: true)
  - `POST /api/sessions/{id}/git/checkout`
- [x] Added git module export to main.rs
- [x] Added git router to api/mod.rs
- [x] Added 19 new tests (10 git unit tests + 9 API integration tests)

### Files Added/Modified
- `backend/src/git/mod.rs` (new) - GitManager with read/write operations + tests
- `backend/src/api/git.rs` (new) - Git REST endpoints + tests
- `backend/src/api/mod.rs` - Added git module export
- `backend/src/main.rs` - Added git module and router

### Verification
- Backend cargo test: ✅ PASS (58 tests)
- Frontend tests: ✅ PASS (1 test)

---

## Step 11 - Frontend API Integration - COMPLETED

### Backend API Summary (for reference)
```
GET  /api/repos              → Vec<Repo> { id, path, name, created_at, updated_at }
POST /api/repos              → AddRepoRequest { path, name? } → Repo
DEL  /api/repos/{id}         → ()

GET  /api/sessions           → Vec<Session> { id, repo_id, name?, status, created_at, updated_at }
POST /api/sessions           → CreateSessionRequest { repo_id, name? } → Session
GET  /api/sessions/{id}      → SessionDetails { session, messages }
DEL  /api/sessions/{id}      → ()
POST /api/sessions/{id}/run  → RunSessionRequest { prompt } → RunSessionResponse
POST /api/sessions/{id}/cancel → CancelSessionResponse

GET  /api/sessions/{id}/git/status   → GitStatusResponse { branch, staged, unstaged, untracked }
GET  /api/sessions/{id}/git/branches → GitBranchesResponse { branches[] }
```

### Tasks
- [x] 11.1 Create API client module (`frontend/src/api/client.ts`)
  - Base fetch wrapper with error handling
  - Type-safe API functions for all endpoints
- [x] 11.2 Create API type definitions (`frontend/src/api/types.ts`)
  - Mirror backend DTOs for type safety
- [x] 11.3 Create React Query hooks (`frontend/src/api/hooks.ts`)
  - useRepos, useSessions, useSession
  - useMutations for create/delete/run
- [x] 11.4 Update Index.tsx to use real API
  - Replace mockRalphtownInstances with useQuery
  - Wire handleStartSession to real session creation + run
- [~] 11.5 Update RepoSelector to fetch branches from git/branches endpoint
  - Deferred: Requires session to exist first to call git API; branches will come via git/branches API per-session
- [x] 11.6 Handle loading/error states in UI
  - Loading state in Index.tsx
  - Error handling via toast notifications

### Files Added/Modified
- `frontend/src/api/types.ts` (new) - API type definitions matching backend DTOs
- `frontend/src/api/client.ts` (new) - Fetch wrappers with error handling
- `frontend/src/api/hooks.ts` (new) - React Query hooks for data fetching
- `frontend/src/api/index.ts` (new) - Re-exports
- `frontend/src/types/ralphtown.ts` - Added adapter functions to map API to UI types
- `frontend/src/pages/Index.tsx` - Replaced mock data with real API calls
- `frontend/src/components/ralphtown/MainPanel.tsx` - Added repos prop, use API repos
- `frontend/src/components/ralphtown/AgentListItem.tsx` - Added idle/cancelled status
- `frontend/src/components/ralphtown/ConversationView.tsx` - Added idle/cancelled status

### Verification
- Frontend build: ✅ PASS
- Frontend tests: ✅ PASS (1 test)
- Backend tests: ✅ PASS (58 tests)

---

## Notes
- Lint has pre-existing errors in shadcn-ui components (not from rename)
- Backend uses Axum 0.8, tower-http 0.6, axum-test 18
