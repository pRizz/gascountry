# Roadmap: Ralphtown

## Overview

This roadmap delivers git clone functionality with authentication handling and improved error messaging. Starting with code cleanup to reduce tech debt, we build the core clone workflow, add progress feedback, implement comprehensive error handling, and finally add credential prompting for authentication failures. Each phase delivers a complete, verifiable capability.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Cleanup** - Remove dead code and improve DB error handling
- [x] **Phase 2: Core Clone** - Basic clone dialog with URL input and default destination
- [x] **Phase 3: Clone Progress** - Real-time progress UI during clone operations
- [x] **Phase 4: Error Handling** - Helpful error messages for common failure scenarios
- [x] **Phase 5: Authentication** - Credential prompts for auth failures
- [ ] **Phase 6: Repository Manager** - CRUD interface for viewing, cloning, and deleting repos
- [ ] **Phase 7: Agent Orchestrator Selection** - Per-session orchestrator selection (Ralph, GSD, Gastown)

## Phase Details

### Phase 1: Cleanup
**Goal**: Codebase is cleaner with dead code removed and DB errors handled properly
**Depends on**: Nothing (first phase)
**Requirements**: CLEAN-01, CLEAN-02
**Success Criteria** (what must be TRUE):
  1. mockData.ts file no longer exists in codebase
  2. DB layer methods return Result types instead of panicking on errors
  3. API errors from DB layer surface as proper HTTP error responses
**Plans**: 1 plan

Plans:
- [x] 01-01-PLAN.md — Delete mockData.ts and replace .unwrap() with proper error handling

### Phase 2: Core Clone
**Goal**: Users can clone repositories from git URLs to the default location
**Depends on**: Phase 1
**Requirements**: CLONE-01, CLONE-02, CLONE-04, CLONE-05
**Success Criteria** (what must be TRUE):
  1. User can open a "Clone from URL" dialog from the repo selector
  2. User can paste a git URL (SSH or HTTPS format) and initiate clone
  3. Clone destination is ~/ralphtown/ by default
  4. Cloned repo is automatically added to repo list and selected on success
**Plans**: 2 plans

Plans:
- [x] 02-01-PLAN.md — Backend clone endpoint + git clone function
- [x] 02-02-PLAN.md — Frontend CloneDialog + RepoSelector integration

### Phase 3: Clone Progress
**Goal**: Users see real-time feedback during clone operations
**Depends on**: Phase 2
**Requirements**: CLONE-03
**Success Criteria** (what must be TRUE):
  1. User sees progress indication while clone is in progress
  2. User can distinguish between "cloning" and "complete" states
  3. Progress UI updates as clone operation proceeds
**Plans**: 2 plans

Plans:
- [x] 03-01-PLAN.md — Backend SSE endpoint + git2 progress callback
- [x] 03-02-PLAN.md — Frontend useCloneProgress hook + CloneDialog progress UI

### Phase 4: Error Handling
**Goal**: Users see helpful, actionable error messages for common failure scenarios
**Depends on**: Phase 2
**Requirements**: ERR-01, ERR-02, ERR-03, ERR-04
**Success Criteria** (what must be TRUE):
  1. SSH auth failure shows explanation of SSH key setup and troubleshooting steps
  2. HTTPS auth failure shows explanation of credential requirements and fix steps
  3. Missing ralph CLI shows clear message about installing ralph and PATH setup
  4. Invalid/missing repo path shows message explaining the issue and how to fix
**Plans**: 2 plans

Plans:
- [x] 04-01-PLAN.md — Backend error classification with help_steps (CloneError, RalphError::NotFound, repo validation)
- [x] 04-02-PLAN.md — Frontend error display with help_steps in CloneDialog

### Phase 5: Authentication
**Goal**: Users can provide credentials when initial auth fails
**Depends on**: Phase 4
**Requirements**: AUTH-01, AUTH-02, AUTH-03, AUTH-04, AUTH-05
**Success Criteria** (what must be TRUE):
  1. User is prompted for GitHub PAT when HTTPS clone to GitHub fails auth
  2. User is prompted for username/password for non-GitHub HTTPS URLs that fail auth
  3. User is prompted for SSH passphrase when encrypted SSH key fails
  4. Credential prompts explain where/how credentials are used (trust messaging)
  5. User sees alternative instructions for CLI-based auth setup as fallback
**Plans**: 2 plans

Plans:
- [x] 05-01-PLAN.md — Backend credential callback support (git2 credentials, POST endpoint)
- [x] 05-02-PLAN.md — Frontend credential inputs with trust messaging and CLI alternative

### Phase 6: Repository Manager
**Goal**: Users can view, manage, and delete repositories from a dedicated settings/manager view
**Depends on**: Phase 2
**Requirements**: REPO-01, REPO-02, REPO-03, REPO-04
**Success Criteria** (what must be TRUE):
  1. User can access a repository manager view showing all repos in ~/ralphtown/
  2. User can see repo details (name, path, clone date)
  3. User can delete repos from the manager (removes from DB, optionally from disk)
  4. User can initiate clone from the manager view (reuses existing clone dialog)
**Plans**: 1 plan

Plans:
- [ ] 06-01-PLAN.md — RepositoryManagerDialog component with table, delete confirmation, and clone integration

### Phase 7: Agent Orchestrator Selection
**Goal**: Users can select different agent orchestrators per session with future options visible
**Depends on**: Phase 5
**Requirements**: ORCH-01, ORCH-02, ORCH-03, ORCH-04, ORCH-05
**Success Criteria** (what must be TRUE):
  1. User can select an orchestrator when starting a session (dropdown or selector)
  2. Ralph is available and functional as the default orchestrator
  3. Other orchestrators (GSD, Gastown) are shown but disabled with "Coming Soon" badge
  4. Selected orchestrator is persisted per-session in the database
  5. Session uses the selected orchestrator when running
**Plans**: 2 plans

Plans:
- [ ] 07-01-PLAN.md — Backend schema migration, Orchestrator enum, API update
- [ ] 07-02-PLAN.md — Frontend OrchestratorSelector component and Index.tsx integration

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Cleanup | 1/1 | Complete | 2026-01-17 |
| 2. Core Clone | 2/2 | Complete | 2026-01-17 |
| 3. Clone Progress | 2/2 | Complete | 2026-01-17 |
| 4. Error Handling | 2/2 | Complete | 2026-01-17 |
| 5. Authentication | 2/2 | Complete | 2026-01-17 |
| 6. Repository Manager | 0/1 | Planned | — |
| 7. Agent Orchestrator Selection | 0/2 | Planned | — |

---
*Roadmap created: 2026-01-17*
