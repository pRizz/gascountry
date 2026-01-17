# Ralphtown

## What This Is

A desktop UI for managing AI coding agents (ralph) across multiple git repositories. Users can run autonomous coding sessions, view real-time output, clone repositories with full authentication support, and manage multiple repos from a single local-first interface. Built with React/TypeScript frontend, Rust/Axum backend, WebSocket streaming, and SQLite persistence.

## Core Value

Users can run autonomous AI coding sessions across multiple repositories from a single interface with real-time feedback.

## Current State

**Version:** v1.0 (shipped 2026-01-17)

Ralphtown v1.0 delivers complete git clone functionality with authentication, progress streaming, helpful error messages, repository management, and orchestrator selection. Users can clone public and private repos, receive credential prompts on auth failures, manage their repository list, and select orchestrators for sessions.

**Codebase:**
- 14,256 LOC (Rust + TypeScript)
- 7 phases, 12 plans executed
- 23 files modified in milestone

## Requirements

### Validated

- ✓ Add and manage local git repositories — existing
- ✓ Create ralph sessions with prompts — existing
- ✓ Run ralph in autonomous mode — existing
- ✓ Real-time WebSocket streaming of ralph output — existing
- ✓ Session history and persistence — existing
- ✓ Git operations (status, log, branches, diff, commit) — existing
- ✓ Cross-platform system service installation — existing
- ✓ Single binary distribution with embedded frontend — existing
- ✓ Clone repos from git URL (SSH or HTTPS) — v1.0
- ✓ Default clone location (~/ralphtown/) — v1.0
- ✓ Clone progress UI with status feedback — v1.0
- ✓ Auto-select cloned repo in selector — v1.0
- ✓ Credential prompts for failed auth (GitHub PAT, username/password, SSH passphrase) — v1.0
- ✓ Alternative auth instructions for users who prefer CLI setup — v1.0
- ✓ Helpful error messages explaining auth failures and how to fix — v1.0
- ✓ Delete unused mockData.ts (dead code cleanup) — v1.0
- ✓ Validate repo path exists before session creation — v1.0
- ✓ Helpful error when ralph CLI not found in PATH — v1.0
- ✓ Replace .unwrap() with proper error handling in DB layer — v1.0
- ✓ Repository manager with CRUD interface — v1.0
- ✓ Per-session orchestrator selection (Ralph active, GSD/Gastown Coming Soon) — v1.0

### Active

(None — run `/gsd:new-milestone` to define next goals)

### Out of Scope

- Multi-user authentication — local-only app, single user
- Remote deployment — localhost-bound by design
- Built-in credential storage/keychain — use system git credentials or prompt inline
- Folder picker for clone destination — defer to v2, use default location for now

## Context

**Tech stack:**
- React 18 + TypeScript frontend with shadcn/ui components
- Rust backend with Axum, Tokio, SQLite (rusqlite)
- git2 crate for clone with credential callbacks
- WebSocket for real-time output streaming
- SSE for clone progress streaming
- `ralph` CLI spawned as subprocess, tracked in RalphManager

**Platform directories:**
- App data: `dirs::data_dir()/ralphtown/` (database lives here)
- Clone destination: `~/ralphtown/` (user-visible, predictable location)

## Constraints

- **Tech stack**: Must use existing Rust/TypeScript stack — no new languages
- **Local-only**: Server binds to 127.0.0.1, no remote access
- **ralph dependency**: External CLI must be in PATH, can't bundle it
- **git credentials**: Must work with user's existing git auth setup (SSH keys, credential helpers, PATs)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Default clone to ~/ralphtown/ | Visible, predictable, follows ~/Documents pattern | ✓ Good |
| Prompt for creds on auth failure | Better UX than requiring pre-configured git credentials | ✓ Good |
| Use git2 callbacks for clone auth | Leverages existing git2 dependency, supports SSH/HTTPS | ✓ Good |
| SSE for clone progress | Real-time streaming with browser-native EventSource | ✓ Good |
| POST endpoint for credentials | Cleaner API than query params for sensitive data | ✓ Good |
| Schema migration v1→v2 | Add orchestrator column with backwards compatibility | ✓ Good |
| Only Ralph orchestrator initially | GSD/Gastown shown as Coming Soon for future work | ✓ Good |

---
*Last updated: 2026-01-17 after v1.0 milestone*
