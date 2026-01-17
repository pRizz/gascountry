# Roadmap: Ralphtown

## Overview

This roadmap delivers git clone functionality with authentication handling and improved error messaging. Starting with code cleanup to reduce tech debt, we build the core clone workflow, add progress feedback, implement comprehensive error handling, and finally add credential prompting for authentication failures. Each phase delivers a complete, verifiable capability.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Cleanup** - Remove dead code and improve DB error handling
- [ ] **Phase 2: Core Clone** - Basic clone dialog with URL input and default destination
- [ ] **Phase 3: Clone Progress** - Real-time progress UI during clone operations
- [ ] **Phase 4: Error Handling** - Helpful error messages for common failure scenarios
- [ ] **Phase 5: Authentication** - Credential prompts for auth failures

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
**Plans**: TBD

Plans:
- [ ] 02-01: TBD

### Phase 3: Clone Progress
**Goal**: Users see real-time feedback during clone operations
**Depends on**: Phase 2
**Requirements**: CLONE-03
**Success Criteria** (what must be TRUE):
  1. User sees progress indication while clone is in progress
  2. User can distinguish between "cloning" and "complete" states
  3. Progress UI updates as clone operation proceeds
**Plans**: TBD

Plans:
- [ ] 03-01: TBD

### Phase 4: Error Handling
**Goal**: Users see helpful, actionable error messages for common failure scenarios
**Depends on**: Phase 2
**Requirements**: ERR-01, ERR-02, ERR-03, ERR-04
**Success Criteria** (what must be TRUE):
  1. SSH auth failure shows explanation of SSH key setup and troubleshooting steps
  2. HTTPS auth failure shows explanation of credential requirements and fix steps
  3. Missing ralph CLI shows clear message about installing ralph and PATH setup
  4. Invalid/missing repo path shows message explaining the issue and how to fix
**Plans**: TBD

Plans:
- [ ] 04-01: TBD

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
**Plans**: TBD

Plans:
- [ ] 05-01: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Cleanup | 1/1 | Complete | 2026-01-17 |
| 2. Core Clone | 0/? | Not started | - |
| 3. Clone Progress | 0/? | Not started | - |
| 4. Error Handling | 0/? | Not started | - |
| 5. Authentication | 0/? | Not started | - |

---
*Roadmap created: 2026-01-17*
