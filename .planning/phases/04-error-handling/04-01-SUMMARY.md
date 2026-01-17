---
phase: 04-error-handling
plan: 01
subsystem: api
tags: [git2, thiserror, error-handling, help-steps, user-actionable]

# Dependency graph
requires:
  - phase: 02-core-clone
    provides: GitManager clone operations
  - phase: 03-clone-progress
    provides: SSE clone progress streaming
provides:
  - CloneError enum with SSH/HTTPS auth classification
  - AppError::UserActionRequired with help_steps field
  - RalphError::NotFound for missing CLI detection
  - validate_repo_path function with actionable errors
  - From<CloneError> and From<RalphError> for AppError
affects: [05-frontend-polish]

# Tech tracking
tech-stack:
  added: [which@7]
  patterns: [error-classification-by-class, help-steps-in-errors]

key-files:
  created: []
  modified:
    - backend/Cargo.toml
    - backend/src/error.rs
    - backend/src/git/mod.rs
    - backend/src/ralph/mod.rs
    - backend/src/api/repos.rs
    - backend/src/api/sessions.rs

key-decisions:
  - "Use git2::ErrorClass for SSH/HTTPS classification instead of parsing error messages"
  - "UserActionRequired returns 422 status code (same as UnprocessableEntity)"
  - "help_steps is Vec<String> with skip_serializing_if empty for clean JSON"
  - "validate_repo_path returns UserActionRequired for both path-not-found and not-a-git-repo"

patterns-established:
  - "Error classification pattern: use library's semantic error types, not string parsing"
  - "Actionable errors pattern: include help_steps with troubleshooting commands"
  - "From trait impl pattern: centralize error conversion in error.rs"

# Metrics
duration: 6min
completed: 2026-01-17
---

# Phase 4 Plan 01: Backend Error Enhancement Summary

**Enhanced error handling with git auth failure classification and actionable help_steps in API responses**

## Performance

- **Duration:** 6 min
- **Started:** 2026-01-17T19:28:48Z
- **Completed:** 2026-01-17T19:34:24Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- CloneError enum classifies git2 errors by ErrorClass (SSH vs HTTPS vs Network)
- AppError::UserActionRequired variant with help_steps for actionable error guidance
- RalphError::NotFound detects missing CLI with installation help
- validate_repo_path provides helpful errors for invalid repository paths
- SSE clone progress streams include help_steps in error events

## Task Commits

Each task was committed atomically:

1. **Task 1: Add which crate and enhance error types** - `4bed6af` (feat)
2. **Task 2: Wire clone errors to API responses with help_steps** - `3a482fd` (feat)
3. **Task 3: Add repo path validation with helpful errors** - `3933cb3` (feat)

## Files Created/Modified

- `backend/Cargo.toml` - Added which = "7" dependency
- `backend/src/error.rs` - Added UserActionRequired variant, help_steps in ErrorBody, From impls
- `backend/src/git/mod.rs` - Added CloneError enum, classify_clone_error, validate_repo_path
- `backend/src/ralph/mod.rs` - Added RalphError::NotFound with help_steps
- `backend/src/api/repos.rs` - Updated handlers and tests for new error types
- `backend/src/api/sessions.rs` - Updated to handle RalphError::NotFound

## Decisions Made

1. **git2::ErrorClass for classification** - Use semantic error classification (Ssh, Http, Net) instead of parsing error message strings. More reliable across versions.

2. **422 status for UserActionRequired** - Same as UnprocessableEntity. User must take action to resolve, not a server error (500) or client format error (400).

3. **help_steps as Vec<String>** - Array of actionable steps the user can take. Skipped from JSON when empty for clean responses.

4. **validate_repo_path in git module** - Centralized validation with UserActionRequired errors for both path-not-found and not-a-git-repo cases.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Handle RalphError::NotFound in sessions API**
- **Found during:** Task 1 (adding NotFound variant to RalphError)
- **Issue:** Adding new enum variant caused non-exhaustive pattern match error in sessions.rs
- **Fix:** Added match arm converting RalphError::NotFound to AppError::UserActionRequired
- **Files modified:** backend/src/api/sessions.rs
- **Verification:** cargo check passed
- **Committed in:** 4bed6af (Task 1 commit)

**2. [Rule 1 - Bug] Fixed test_clone_invalid_url test assertion**
- **Found during:** Task 1 (changing clone return type to CloneError)
- **Issue:** Test asserted GitError::OperationFailed but clone now returns CloneError
- **Fix:** Updated test to check for CloneError::OperationFailed or CloneError::NetworkError
- **Files modified:** backend/src/git/mod.rs
- **Verification:** Test passes
- **Committed in:** 4bed6af (Task 1 commit)

**3. [Rule 1 - Bug] Updated repo validation tests for new status code**
- **Found during:** Task 3 (using validate_repo_path in add_repo)
- **Issue:** Tests expected 400 Bad Request but now returns 422 Unprocessable Entity
- **Fix:** Updated tests to assert 422 status and verify error code/help_steps
- **Files modified:** backend/src/api/repos.rs
- **Verification:** Tests pass with new assertions
- **Committed in:** 3933cb3 (Task 3 commit)

---

**Total deviations:** 3 auto-fixed (2 bugs, 1 blocking)
**Impact on plan:** All auto-fixes necessary for correctness. No scope creep.

## Issues Encountered

None - plan executed smoothly.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Backend error handling complete with actionable help_steps
- Ready for frontend to consume and display help_steps in error toasts/dialogs
- SSE clone events now include help_steps for auth failures
- All 79 tests passing

---
*Phase: 04-error-handling*
*Completed: 2026-01-17*
