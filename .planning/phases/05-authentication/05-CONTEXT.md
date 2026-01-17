# Phase 5: Authentication — Discussion Context

**Captured:** 2026-01-17
**Phase Goal:** Users can provide credentials when initial auth fails

## Requirements Covered

- AUTH-01: User is prompted for GitHub PAT when HTTPS clone fails auth
- AUTH-02: User is prompted for username/password for non-GitHub HTTPS URLs
- AUTH-03: User is prompted for SSH passphrase when SSH key is encrypted
- AUTH-04: User sees explanation of where/how credentials are used
- AUTH-05: User sees alternative instructions for CLI-based auth setup

## Decisions Made

### 1. Credential UI Flow

**Decision:** Inline credential input in Clone Dialog with auto-retry

**Details:**
- When clone fails due to auth, show credential inputs directly in the Clone Dialog (no separate modal)
- After user enters credentials, automatically retry the clone (no manual "retry" button needed)
- Future: Add setting if user wants separate modal (v2 scope)

**Rationale:** Single-dialog flow is simpler and keeps user focused. Auto-retry reduces friction.

### 2. GitHub Detection

**Decision:** Host string match for GitHub, username/password for others

**Details:**
- Check if URL hostname contains `github.com` → show PAT-only input with GitHub-specific guidance
- For non-GitHub HTTPS URLs → show username + password fields
- No need for full known-hosts list (GitLab, Bitbucket can use same u/p flow)

**Rationale:** Simple string match covers the main case. GitHub's PAT requirement is the specific edge case that needs special handling.

### 3. SSH Passphrase Approach

**Decision:** git2 credential callbacks with default key paths + ssh-agent setting

**Details:**
- Use git2's credential callback API (`RemoteCallbacks::credentials()`) for SSH auth
- Try default key paths in order: `~/.ssh/id_ed25519`, `~/.ssh/id_rsa`, `~/.ssh/id_ecdsa`
- Add setting for "Use ssh-agent only" (don't try key files directly)
- When passphrase needed, prompt user inline in Clone Dialog
- Make ssh-agent preference clear in UI

**Rationale:** git2 callbacks keep everything in-process with better error handling. Default key paths cover most users. Setting provides escape hatch for advanced users.

### 4. Trust Messaging Scope

**Decision:** Multi-layered trust messaging + always-visible CLI alternative

**Trust messaging locations:**
1. **Inline below input:** Small text like "Used only for this clone, not stored"
2. **Collapsible section:** "How are my credentials used?" expandable for full explanation
3. **Tooltip:** Info icon next to password field with hover explanation

**CLI alternative:**
- Always-visible link below credential inputs: "Prefer CLI auth? Set up credentials →"
- Links to help documentation or shows inline instructions

**Rationale:** Users have different trust thresholds. Multiple explanation points ensure everyone can find the info they need. CLI alternative respects power users who prefer configuring credentials outside the app.

## Implementation Implications

### Backend Changes Needed

1. **Credential callback support in git2 clone:**
   - Add `credentials` callback to `RemoteCallbacks`
   - Support for SSH key file paths with passphrase
   - Support for HTTPS username/password
   - New endpoint or modify existing clone SSE to accept credentials

2. **GitHub detection helper:**
   - Function to check if URL is GitHub-hosted
   - Return auth type: `GitHubPat` or `UsernamePassword`

### Frontend Changes Needed

1. **CloneDialog credential inputs:**
   - Conditional credential section (shown after auth failure)
   - PAT input (for GitHub) vs username/password inputs (for others)
   - Auto-retry on credential submission

2. **Trust messaging components:**
   - Inline helper text
   - Collapsible "How credentials work" section
   - Tooltip on password/PAT field

3. **CLI alternative link:**
   - Always visible when credential section shown
   - Either inline instructions or link to docs

### API Contract

Existing SSE clone endpoint needs to handle credential retry:
- Option A: New request with credentials included
- Option B: Separate credential submission endpoint that continues paused clone

Recommendation: Option A (simpler - new clone request with credentials, treat as retry)

## Questions Resolved

| Question | Answer |
|----------|--------|
| Inline vs modal for credentials? | Inline (with future setting for modal) |
| Auto-retry or manual? | Auto-retry |
| How to detect GitHub? | Host string contains `github.com` |
| What fields for non-GitHub HTTPS? | Username + password |
| SSH: git2 callbacks or CLI fallback? | git2 credential callbacks |
| Which SSH keys to try? | Default paths (~/.ssh/id_*) with ssh-agent setting |
| Where to show trust messaging? | Inline + collapsible + tooltip (all three) |
| How prominent is CLI alternative? | Always-visible link |

## Out of Scope for Phase 5

- Credential storage/keychain integration (explicitly excluded in PROJECT.md)
- OAuth flows
- Full known-hosts list (GitLab, Bitbucket-specific messaging)
- Separate modal setting (v2)

---
*Captured from discuss-phase session: 2026-01-17*
