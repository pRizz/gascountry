# Phase 4: Error Handling - Research

**Researched:** 2026-01-17
**Domain:** Git authentication errors, CLI detection, user-friendly error messaging
**Confidence:** MEDIUM-HIGH

## Summary

This phase implements helpful, actionable error messages for common failure scenarios during git clone operations and ralph CLI execution. The research focused on three main areas:

1. **Git2 error detection patterns** - How to differentiate SSH vs HTTPS authentication failures using git2's `ErrorCode` and `ErrorClass` enums
2. **Ralph CLI detection** - Using the `which` crate to detect if the ralph binary is available in PATH
3. **Frontend error display** - Leveraging existing toast notification patterns with expanded error details

**Primary recommendation:** Extend the backend's error handling to categorize git2 errors by `ErrorClass` (Ssh vs Http) and provide structured error responses with user-friendly messages and troubleshooting steps. Use the existing toast system on the frontend, enhanced with a collapsible details section for longer troubleshooting content.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| git2 | 0.20 | Git operations and error types | Already in use, provides ErrorCode and ErrorClass |
| thiserror | 2 | Custom error types | Already in use for error definitions |
| which | latest | Binary detection in PATH | Standard Rust crate for `which(1)` equivalent |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde_json | 1 | Structured error details | For optional error metadata |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| which crate | std::process::Command::new("ralph").spawn() | `which` is cleaner, doesn't spawn process |
| Inline error strings | Error catalog/i18n | Catalog adds complexity; inline is fine for single-language app |

**Installation:**
```bash
cargo add which
```

## Architecture Patterns

### Recommended Error Categorization

```
GitError (current)
  |-- NotARepo
  |-- OperationFailed
  |-- CommandFailed
  |-- InvalidBranch

CloneError (new - extends error types)
  |-- SshAuthFailed { message, help_steps }
  |-- HttpsAuthFailed { message, help_steps }
  |-- NetworkError { message }
  |-- DirectoryExists { path }
  |-- InvalidUrl { url }

RalphError (current)
  |-- RepoBusy
  |-- SessionAlreadyRunning
  |-- SpawnFailed  <-- Enhanced with NOT_FOUND detection
  |-- NotRunning

AppError (current)
  |-- Add: UserActionRequired { code, message, details, help_steps }
```

### Pattern 1: Error Classification by git2 ErrorClass
**What:** Use `error.class()` to differentiate SSH vs HTTP errors
**When to use:** When handling git2::Error in clone operations
**Example:**
```rust
// Source: https://docs.rs/git2/latest/git2/enum.ErrorClass.html
fn classify_git_error(err: git2::Error) -> CloneError {
    match err.class() {
        git2::ErrorClass::Ssh => {
            // SSH authentication failure
            CloneError::SshAuthFailed {
                message: err.message().to_string(),
                help_steps: vec![
                    "Ensure your SSH key is added to ssh-agent: ssh-add ~/.ssh/id_ed25519".into(),
                    "Verify your key is added to GitHub/GitLab: ssh -T git@github.com".into(),
                    "Check SSH key format (ed25519 or RSA)".into(),
                ],
            }
        }
        git2::ErrorClass::Http => {
            // HTTPS authentication failure
            CloneError::HttpsAuthFailed {
                message: err.message().to_string(),
                help_steps: vec![
                    "HTTPS cloning requires a Personal Access Token (PAT)".into(),
                    "Create a PAT at GitHub Settings > Developer Settings > Tokens".into(),
                    "Use the PAT as password when prompted".into(),
                ],
            }
        }
        git2::ErrorClass::Net => {
            CloneError::NetworkError {
                message: err.message().to_string(),
            }
        }
        _ => CloneError::Generic(err.message().to_string()),
    }
}
```

### Pattern 2: Binary Detection with `which` Crate
**What:** Check if ralph is in PATH before attempting to spawn
**When to use:** When starting a ralph session
**Example:**
```rust
// Source: https://docs.rs/which/
use which::which;

fn check_ralph_available() -> Result<(), RalphError> {
    match which("ralph") {
        Ok(_path) => Ok(()),
        Err(_) => Err(RalphError::NotFound {
            message: "ralph CLI not found in PATH".into(),
            help_steps: vec![
                "Install ralph: cargo install ralph".into(),
                "Or download from: https://github.com/example/ralph/releases".into(),
                "Ensure ~/.cargo/bin is in your PATH".into(),
                "Restart your terminal after installation".into(),
            ],
        }),
    }
}
```

### Pattern 3: Structured Error Response with Help Steps
**What:** Include actionable help steps in error response
**When to use:** For all user-facing errors that require action
**Example:**
```rust
#[derive(Serialize)]
struct UserFacingError {
    error: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    help_steps: Vec<String>,
}
```

### Anti-Patterns to Avoid
- **Generic "Operation failed" messages:** Always include the underlying error message
- **Technical jargon without context:** Frame errors in terms users understand
- **Help steps that require expertise:** Provide copy-paste commands when possible
- **Swallowing error context:** Always preserve the original error message

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| PATH binary detection | Custom path traversal | `which` crate | Cross-platform, handles edge cases |
| Error classification | Message string parsing | `git2::Error::class()` | Semantic, not text-based |
| Toast notifications | Custom alert system | Existing shadcn toast | Already integrated and styled |

**Key insight:** git2 provides structured error information via `ErrorClass` and `ErrorCode` that should be used programmatically rather than parsing error message strings.

## Common Pitfalls

### Pitfall 1: Relying on Error Message Text
**What goes wrong:** Error messages from libgit2 vary by version and platform
**Why it happens:** Developers parse message strings like "authentication failed"
**How to avoid:** Use `error.class()` and `error.code()` for classification
**Warning signs:** Code that does `if message.contains("auth")`

### Pitfall 2: Missing Credential Callback
**What goes wrong:** Clone fails with "authentication required but no callback set"
**Why it happens:** git2 requires explicit credential configuration for auth
**How to avoid:** For now, document that users need SSH agent or git credential helper configured. Future enhancement could add credential callback.
**Warning signs:** Works on command line but fails in app

### Pitfall 3: SSH Key Format Incompatibility
**What goes wrong:** SSH auth fails on Windows with certain key types
**Why it happens:** libssh2's WinCNG backend doesn't support modern key formats
**How to avoid:** Document that ed25519 keys may have issues on Windows
**Warning signs:** "Failed to authenticate SSH session" on Windows only

### Pitfall 4: Repo Path Validation After Add
**What goes wrong:** User moves/deletes repo folder, app crashes on operations
**Why it happens:** Path was valid at add time but is now stale
**How to avoid:** Check `Path::exists()` before git operations, return helpful error
**Warning signs:** Panic or unclear error when repo folder missing

## Code Examples

Verified patterns from official sources:

### Detecting Error Class from git2
```rust
// Source: https://docs.rs/git2/latest/git2/struct.Error.html
fn handle_clone_error(err: git2::Error) {
    let code = err.code();      // ErrorCode - programmatically actionable
    let class = err.class();    // ErrorClass - informative category
    let message = err.message(); // Human-readable message

    match class {
        git2::ErrorClass::Ssh => {
            // SSH-related failure
            tracing::error!("SSH error: {} (code: {:?})", message, code);
        }
        git2::ErrorClass::Http => {
            // HTTP/HTTPS-related failure
            tracing::error!("HTTP error: {} (code: {:?})", message, code);
        }
        git2::ErrorClass::Net => {
            // Network connectivity issue
            tracing::error!("Network error: {}", message);
        }
        _ => {
            tracing::error!("Git error: {}", message);
        }
    }
}
```

### Checking Binary in PATH
```rust
// Source: https://docs.rs/which/
use which::which;

pub fn ensure_ralph_installed() -> Result<std::path::PathBuf, String> {
    which("ralph").map_err(|_| {
        "ralph CLI not found. Please install ralph and ensure it's in your PATH."
            .to_string()
    })
}
```

### Frontend Error Toast with Description
```typescript
// Source: Existing CloneDialog.tsx pattern
toast({
  title: "SSH Authentication Failed",
  description: "Could not authenticate with the remote server. Check your SSH key setup.",
  variant: "destructive",
});
```

### Validating Repo Path Exists
```rust
// Pattern for checking repo path before operations
fn validate_repo_path(path: &Path) -> AppResult<()> {
    if !path.exists() {
        return Err(AppError::NotFound(format!(
            "Repository path no longer exists: {}. The folder may have been moved or deleted.",
            path.display()
        )));
    }
    if !path.join(".git").exists() && git2::Repository::open(path).is_err() {
        return Err(AppError::BadRequest(format!(
            "Path exists but is not a git repository: {}",
            path.display()
        )));
    }
    Ok(())
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| String message parsing | ErrorClass/ErrorCode enums | git2 has always had this | More reliable error classification |
| Inline help text | Structured help_steps array | Best practice | Enables frontend flexibility |

**Deprecated/outdated:**
- None identified for this domain

## Open Questions

Things that couldn't be fully resolved:

1. **Credential Callback Implementation**
   - What we know: git2 supports credential callbacks for auth
   - What's unclear: Whether to implement in this phase or defer
   - Recommendation: Defer - document SSH agent/git credential helper requirement

2. **Error Message Localization**
   - What we know: Error messages are English-only
   - What's unclear: Whether i18n is needed
   - Recommendation: Use English for now, structure supports future i18n

3. **SSH Key Type Warnings**
   - What we know: Some key types fail on Windows
   - What's unclear: Exact failure modes and detection
   - Recommendation: Include in help text, don't try to detect programmatically

## Sources

### Primary (HIGH confidence)
- [git2 ErrorClass enum](https://docs.rs/git2/latest/git2/enum.ErrorClass.html) - All 35 error class variants
- [git2 ErrorCode enum](https://docs.rs/git2/latest/git2/enum.ErrorCode.html) - All error codes including Auth
- [git2 Error struct](https://docs.rs/git2/latest/git2/struct.Error.html) - Methods for code(), class(), message()
- [which crate](https://docs.rs/which/) - Binary detection API

### Secondary (MEDIUM confidence)
- Existing codebase patterns (error.rs, ralph/mod.rs, CloneDialog.tsx)

### Tertiary (LOW confidence)
- [git2-rs GitHub issues](https://github.com/rust-lang/git2-rs/issues) - SSH auth patterns (community solutions)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Based on official docs and existing codebase
- Architecture: MEDIUM-HIGH - Patterns derived from git2 docs and best practices
- Pitfalls: MEDIUM - Based on GitHub issues and community reports

**Research date:** 2026-01-17
**Valid until:** 30 days (stable APIs, well-documented crate)

---

## RESEARCH COMPLETE

**Phase:** 4 - Error Handling
**Confidence:** MEDIUM-HIGH

### Key Findings

- git2 provides `ErrorClass::Ssh` and `ErrorClass::Http` to distinguish authentication failure types - use `error.class()` not message parsing
- The `which` crate is the standard solution for checking if ralph is in PATH
- Current toast system is sufficient for errors; enhance with help_steps for actionable guidance
- Repo path validation should happen before git operations to catch moved/deleted folders
- git2 error handling requires checking class (informative) and code (actionable) separately

### File Created

`/Users/peterryszkiewicz/Repos/gascountry-ui/.planning/phases/04-error-handling/04-RESEARCH.md`

### Confidence Assessment

| Area | Level | Reason |
|------|-------|--------|
| Standard Stack | HIGH | Official docs, existing patterns |
| Architecture | MEDIUM-HIGH | Derived from git2 docs |
| Pitfalls | MEDIUM | GitHub issues, community reports |

### Open Questions

- Credential callback implementation (recommend: defer to future phase)
- Error message localization (recommend: English for now)
- SSH key type compatibility warnings (recommend: include in help text)

### Ready for Planning

Research complete. Planner can now create PLAN.md files.
