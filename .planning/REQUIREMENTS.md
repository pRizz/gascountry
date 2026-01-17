# Requirements: Ralphtown

**Defined:** 2026-01-17
**Core Value:** Users can run autonomous AI coding sessions across multiple repositories from a single interface with real-time feedback.

## v1 Requirements

### Clone

- [ ] **CLONE-01**: User can open "Clone from URL" dialog from repo selector
- [ ] **CLONE-02**: User can paste git URL (SSH or HTTPS) and initiate clone
- [ ] **CLONE-03**: User sees progress UI during clone operation
- [ ] **CLONE-04**: Cloned repo is automatically added and selected on success
- [ ] **CLONE-05**: Clone destination defaults to ~/ralphtown/

### Auth

- [ ] **AUTH-01**: User is prompted for GitHub PAT when HTTPS clone fails auth
- [ ] **AUTH-02**: User is prompted for username/password for non-GitHub HTTPS URLs
- [ ] **AUTH-03**: User is prompted for SSH passphrase when SSH key is encrypted
- [ ] **AUTH-04**: User sees explanation of where/how credentials are used
- [ ] **AUTH-05**: User sees alternative instructions for CLI-based auth setup

### Errors

- [ ] **ERR-01**: User sees helpful message explaining SSH auth failure and fix steps
- [ ] **ERR-02**: User sees helpful message explaining HTTPS auth failure and fix steps
- [ ] **ERR-03**: User sees helpful message when ralph CLI not found in PATH
- [ ] **ERR-04**: User sees helpful message when repo path no longer exists

### Cleanup

- [ ] **CLEAN-01**: Delete unused mockData.ts file
- [ ] **CLEAN-02**: Replace .unwrap() with proper error handling in DB layer

## v2 Requirements

### Settings

- **SET-01**: User can configure default clone directory in settings
- **SET-02**: User can choose clone location per-clone via folder picker

### Clone Enhancements

- **CLONE-06**: User can clone to custom location (folder picker)
- **CLONE-07**: User can cancel in-progress clone

## Out of Scope

| Feature | Reason |
|---------|--------|
| Credential storage/keychain integration | Use system git credentials or prompt inline; complexity not justified for v1 |
| OAuth flows for GitHub/GitLab | PAT entry is simpler and sufficient |
| Clone from UI without URL (repo browser) | Paste URL is sufficient for v1 |
| Batch clone multiple repos | Single clone workflow first |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CLONE-01 | TBD | Pending |
| CLONE-02 | TBD | Pending |
| CLONE-03 | TBD | Pending |
| CLONE-04 | TBD | Pending |
| CLONE-05 | TBD | Pending |
| AUTH-01 | TBD | Pending |
| AUTH-02 | TBD | Pending |
| AUTH-03 | TBD | Pending |
| AUTH-04 | TBD | Pending |
| AUTH-05 | TBD | Pending |
| ERR-01 | TBD | Pending |
| ERR-02 | TBD | Pending |
| ERR-03 | TBD | Pending |
| ERR-04 | TBD | Pending |
| CLEAN-01 | TBD | Pending |
| CLEAN-02 | TBD | Pending |

**Coverage:**
- v1 requirements: 16 total
- Mapped to phases: 0
- Unmapped: 16 (pending roadmap)

---
*Requirements defined: 2026-01-17*
*Last updated: 2026-01-17 after initial definition*
