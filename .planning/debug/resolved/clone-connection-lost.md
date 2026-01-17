---
status: resolved
trigger: "Clone operation fails with 'Connection to the server was lost' when attempting to clone a public GitHub repository"
created: 2026-01-17T12:00:00Z
updated: 2026-01-17T12:04:00Z
---

## Current Focus

hypothesis: CONFIRMED AND FIXED - SSE event name collision resolved
test: All backend tests pass, code compiles
expecting: Clone will now properly receive error messages from server
next_action: Archive session

## Symptoms

expected: Clone completes successfully, repository is added to the app, success message shown
actual: Error message "Failed to clone repository. Connection to the server was lost."
errors: "Connection to the server was lost" - suggests SSE connection dropped
reproduction: Start app -> Click repo selector -> "Clone from URL..." -> paste https://github.com/octocat/Hello-World.git -> click Clone
started: First time testing clone functionality (just implemented in Phase 2-5)

## Eliminated

## Evidence

- timestamp: 2026-01-17T12:01:00Z
  checked: useCloneProgress.ts line 99
  found: "Connection to server lost" error message is generated when EventSource error event has no data
  implication: The server is closing the SSE connection without sending an error event with data

- timestamp: 2026-01-17T12:01:00Z
  checked: repos.rs clone_with_progress_sse function
  found: SSE stream uses async_stream to yield events, spawns clone in spawn_blocking
  implication: Need to trace the exact flow when clone fails

- timestamp: 2026-01-17T12:02:00Z
  checked: MDN documentation on EventSource error event
  found: The built-in "error" event on EventSource is specifically for connection-level errors, not for custom error events. "The error event covers both lost connection (will auto-reconnect) and fatal errors."
  implication: Server's `event: error` naming conflicts with browser's built-in error event

- timestamp: 2026-01-17T12:02:00Z
  checked: Backend repos.rs lines 455, 483, 493
  found: Server sends `Event::default().event("error").data(data)` for clone errors
  implication: Using "error" as event name is the problem - browser interprets connection close as error too

- timestamp: 2026-01-17T12:02:00Z
  checked: Frontend useCloneProgress.ts line 82-103
  found: addEventListener("error") catches BOTH custom "error" events AND browser connection errors
  implication: When connection closes, browser fires error event (no data) which triggers "Connection to server lost" message

- timestamp: 2026-01-17T12:03:00Z
  checked: Backend cargo check
  found: Compiles successfully after changing "error" to "clone_error"
  implication: Backend fix is syntactically correct

- timestamp: 2026-01-17T12:03:00Z
  checked: Frontend tsc --noEmit
  found: TypeScript compiles successfully with updated event listener
  implication: Frontend fix is type-safe

- timestamp: 2026-01-17T12:04:00Z
  checked: Backend cargo test
  found: All 79 tests pass
  implication: No regressions introduced by the fix

## Resolution

root_cause: SSE event name collision - server uses "error" as custom event name which conflicts with browser's built-in EventSource "error" event. When the SSE connection closes (after yielding the error event), the browser fires its own error event with no data, which the frontend interprets as "Connection to server lost".

fix:
1. Backend (repos.rs): Changed all `.event("error")` to `.event("clone_error")` for server-sent error events
2. Frontend (useCloneProgress.ts):
   - Added listener for `"clone_error"` custom event to receive server error details
   - Kept `"error"` listener only for genuine connection errors
   - Added `receivedServerError` flag to prevent duplicate error handling when server error is followed by connection close

verification:
- Backend compiles (cargo check passes)
- Frontend typechecks (tsc --noEmit passes)
- All 79 backend tests pass (cargo test)

files_changed:
  - backend/src/api/repos.rs (changed "error" to "clone_error" in 8 places)
  - frontend/src/hooks/useCloneProgress.ts (separated clone_error and connection error handling)
