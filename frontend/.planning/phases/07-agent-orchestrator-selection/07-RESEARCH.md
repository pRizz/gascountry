# Phase 7: Agent Orchestrator Selection - Research

**Researched:** 2026-01-17
**Domain:** Session management, UI selection components, database schema evolution
**Confidence:** HIGH

## Summary

This phase adds the ability to select different agent orchestrators (Ralph, GSD, Gastown) per session. The current codebase already has well-structured session management with clear separation between backend (Rust/Axum/SQLite) and frontend (React/TanStack Query/shadcn). The implementation involves:

1. Adding an `orchestrator` column to the sessions table in SQLite
2. Extending the CreateSessionRequest API to include orchestrator selection
3. Adding an orchestrator selector UI in the PromptInput component with disabled "Coming Soon" badges

The existing `RalphManager` is already designed as a separate module, making it straightforward to later add GSD and Gastown managers. For now, only Ralph will be functional; others will be disabled in the UI.

**Primary recommendation:** Add orchestrator as a TEXT column with DEFAULT 'ralph', update API types, and add a segmented button or dropdown selector in PromptInput with Badge for disabled options.

## Standard Stack

The codebase already uses the correct stack. No new libraries needed.

### Core (Already in use)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| shadcn/ui | latest | UI components | Already used throughout app |
| @radix-ui/react-select | latest | Select primitives | Already installed via shadcn |
| TanStack Query | 5.x | API state management | Already manages session state |
| rusqlite | 0.31+ | SQLite database | Already used for sessions table |

### Supporting (Already in use)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Badge component | shadcn | Status indicators | For "Coming Soon" badges |
| Select component | shadcn | Dropdown selection | Alternative to segmented buttons |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Select dropdown | Segmented button/RadioGroup | Segmented buttons are more visible for 3 options; dropdown hides options |
| RadioGroup | ToggleGroup | ToggleGroup is better for icon-based selection |

**No new installations needed.** All components are already available.

## Architecture Patterns

### Current Session Flow (Important Context)

```
Frontend (Index.tsx)               Backend (sessions.rs)              Database
        |                                    |                           |
        |-- createSession({repo_id}) ------->|                           |
        |                                    |-- insert_session() ------>|
        |<-- Session --------------------|   |<-- Session ---------------|
        |                                    |                           |
        |-- runSession({id, prompt}) ------->|                           |
        |                                    |-- ralph_manager.run() --->| (spawns CLI)
```

### Recommended Change Points

**Backend Changes:**
1. `schema.rs` - Add migration SQL for orchestrator column
2. `models.rs` - Add `Orchestrator` enum and extend `Session` struct
3. `mod.rs` (db) - Update `insert_session()` to accept orchestrator parameter
4. `sessions.rs` (api) - Extend `CreateSessionRequest` to include orchestrator

**Frontend Changes:**
1. `api/types.ts` - Add `Orchestrator` type and extend `CreateSessionRequest`
2. `constants/index.ts` - Add orchestrators list with availability status
3. `PromptInput.tsx` - Add orchestrator selector (near model selector)
4. `Index.tsx` - Pass orchestrator to createSession call

### Pattern 1: Orchestrator Enum with Availability

**What:** Define orchestrators with enabled/disabled status in constants
**When to use:** When options have different availability states
**Example:**
```typescript
// Source: Project-specific pattern matching existing availableModels
export interface OrchestratorOption {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
}

export const orchestrators: OrchestratorOption[] = [
  { id: "ralph", name: "Ralph", description: "Default CLI orchestrator", enabled: true },
  { id: "gsd", name: "GSD", description: "Getting Stuff Done agent", enabled: false },
  { id: "gastown", name: "Gastown", description: "Multi-agent orchestration", enabled: false },
];
```

### Pattern 2: Select with Disabled Options and Badge

**What:** Use shadcn Select with disabled items showing "Coming Soon" badge
**When to use:** Options list where some items are unavailable
**Example:**
```tsx
// Source: shadcn/ui Select + Badge components
<Select value={orchestrator} onValueChange={setOrchestrator}>
  <SelectTrigger className="w-[180px]">
    <SelectValue placeholder="Select orchestrator" />
  </SelectTrigger>
  <SelectContent>
    {orchestrators.map((orch) => (
      <SelectItem key={orch.id} value={orch.id} disabled={!orch.enabled}>
        <div className="flex items-center justify-between w-full gap-2">
          <span>{orch.name}</span>
          {!orch.enabled && (
            <Badge variant="secondary" className="text-xs">Coming Soon</Badge>
          )}
        </div>
      </SelectItem>
    ))}
  </SelectContent>
</Select>
```

### Pattern 3: Database Migration with Default Value

**What:** ALTER TABLE to add column with default value
**When to use:** Adding optional field to existing table
**Example:**
```rust
// Source: SQLite ALTER TABLE documentation
// In schema.rs, add to schema version 2 migration:
pub const MIGRATION_V2: &str = r#"
ALTER TABLE sessions ADD COLUMN orchestrator TEXT NOT NULL DEFAULT 'ralph';
"#;
```

### Anti-Patterns to Avoid
- **Hardcoding orchestrator in RalphManager:** Keep it in session data, pass to manager at runtime
- **Creating separate tables for each orchestrator's sessions:** Use single sessions table with orchestrator column
- **Frontend-only orchestrator state:** Always persist in database for session continuity

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Disabled dropdown option styling | Custom disabled styles | Radix SelectItem `disabled` prop | Built-in accessibility and styling |
| Badge positioning in select | Absolute positioning hacks | Flex layout in SelectItem content | Cleaner, responsive |
| Database migrations | Manual version checks | rusqlite_migration crate OR existing schema version pattern | Already has schema_version table |

**Key insight:** The codebase already has a schema versioning pattern in `schema.rs`. Use the existing `SCHEMA_VERSION` constant and add conditional migration logic.

## Common Pitfalls

### Pitfall 1: Forgetting to Handle Existing Sessions
**What goes wrong:** Existing sessions in database lack orchestrator field
**Why it happens:** Migration adds column but existing rows need default value
**How to avoid:** Use `DEFAULT 'ralph'` in ALTER TABLE statement
**Warning signs:** Errors when loading old sessions, null orchestrator values

### Pitfall 2: Breaking the Run Endpoint
**What goes wrong:** `run_session` always uses `ralph_manager` regardless of orchestrator
**Why it happens:** Not checking session.orchestrator before selecting manager
**How to avoid:** Add orchestrator lookup in run_session handler (even if only ralph works now)
**Warning signs:** Wrong orchestrator runs, no error for unsupported orchestrator

### Pitfall 3: SelectItem Content Styling
**What goes wrong:** Badge doesn't appear or overflows in SelectItem
**Why it happens:** SelectItem has specific content constraints
**How to avoid:** Wrap content in flex container, ensure proper spacing
**Warning signs:** Badge clipped, weird spacing in dropdown

### Pitfall 4: Not Validating Orchestrator on Backend
**What goes wrong:** Frontend could send invalid orchestrator value
**Why it happens:** Only frontend validation
**How to avoid:** Add enum validation in CreateSessionRequest deserialization
**Warning signs:** Invalid orchestrator saved to database

## Code Examples

### Backend: Orchestrator Enum (models.rs)
```rust
// Source: Pattern matching existing SessionStatus enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orchestrator {
    Ralph,
    Gsd,
    Gastown,
}

impl Orchestrator {
    pub fn as_str(&self) -> &'static str {
        match self {
            Orchestrator::Ralph => "ralph",
            Orchestrator::Gsd => "gsd",
            Orchestrator::Gastown => "gastown",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "ralph" => Ok(Orchestrator::Ralph),
            "gsd" => Ok(Orchestrator::Gsd),
            "gastown" => Ok(Orchestrator::Gastown),
            _ => Err(format!("invalid orchestrator: '{}'", s)),
        }
    }
}
```

### Backend: Session Struct Extension
```rust
// Source: Extending existing Session struct
pub struct Session {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: Option<String>,
    pub orchestrator: Orchestrator,  // NEW
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Backend: Updated CreateSessionRequest (sessions.rs)
```rust
// Source: Extending existing CreateSessionRequest
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSessionRequest {
    pub repo_id: Uuid,
    pub name: Option<String>,
    #[serde(default = "default_orchestrator")]
    pub orchestrator: Orchestrator,  // NEW - defaults to Ralph
}

fn default_orchestrator() -> Orchestrator {
    Orchestrator::Ralph
}
```

### Backend: Database Migration
```rust
// Source: SQLite ALTER TABLE + existing schema pattern
pub const SCHEMA_VERSION: i32 = 2;  // Bump from 1

pub const MIGRATION_V2: &str = r#"
ALTER TABLE sessions ADD COLUMN orchestrator TEXT NOT NULL DEFAULT 'ralph';
"#;
```

### Frontend: Types (api/types.ts)
```typescript
// Source: Extending existing Session interface
export type Orchestrator = "ralph" | "gsd" | "gastown";

export interface CreateSessionRequest {
  repo_id: string;
  name?: string;
  orchestrator?: Orchestrator;  // NEW - optional, defaults to ralph
}

export interface Session {
  id: string;
  repo_id: string;
  name: string | null;
  orchestrator: Orchestrator;  // NEW
  status: SessionStatus;
  created_at: string;
  updated_at: string;
}
```

### Frontend: Constants (constants/index.ts)
```typescript
// Source: Project pattern from availableModels
export interface OrchestratorOption {
  id: Orchestrator;
  name: string;
  description: string;
  enabled: boolean;
}

export const orchestrators: OrchestratorOption[] = [
  { id: "ralph", name: "Ralph", description: "CLI-based agent orchestrator", enabled: true },
  { id: "gsd", name: "GSD", description: "Getting Stuff Done framework", enabled: false },
  { id: "gastown", name: "Gastown", description: "Multi-agent orchestration", enabled: false },
];
```

### Frontend: OrchestratorSelector Component
```tsx
// Source: shadcn Select + Badge pattern
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { orchestrators, type OrchestratorOption } from "@/constants";

interface OrchestratorSelectorProps {
  value: string;
  onChange: (value: string) => void;
}

export function OrchestratorSelector({ value, onChange }: OrchestratorSelectorProps) {
  return (
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger className="w-[140px] h-8">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {orchestrators.map((orch) => (
          <SelectItem key={orch.id} value={orch.id} disabled={!orch.enabled}>
            <div className="flex items-center gap-2">
              <span>{orch.name}</span>
              {!orch.enabled && (
                <Badge variant="secondary" className="text-[10px] px-1.5 py-0">
                  Coming Soon
                </Badge>
              )}
            </div>
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single hardcoded orchestrator | Per-session orchestrator | This phase | Enables future multi-agent support |
| No orchestrator in session model | Orchestrator enum column | This phase | Database tracks which agent runs session |

**Deprecated/outdated:**
- None - this is a new feature, not refactoring existing functionality

## Open Questions

Things that couldn't be fully resolved:

1. **GSD and Gastown CLI interface**
   - What we know: They will be CLI-based like Ralph
   - What's unclear: Exact CLI arguments and output format
   - Recommendation: Design run_session to dispatch by orchestrator, implement ralph case only for now

2. **Orchestrator switching mid-session**
   - What we know: User requirements say "per-session"
   - What's unclear: Can user change orchestrator after session is created?
   - Recommendation: Lock orchestrator after session creation (immutable field)

3. **UI placement for orchestrator selector**
   - What we know: Should be visible when starting a session
   - What's unclear: Above prompt input, below repo selector, or inline with model selector?
   - Recommendation: Place above prompt input, similar prominence to repo selector

## Sources

### Primary (HIGH confidence)
- Codebase analysis: `/backend/src/api/sessions.rs` - current session creation flow
- Codebase analysis: `/backend/src/db/schema.rs` - existing schema version pattern
- Codebase analysis: `/frontend/src/components/ralphtown/PromptInput.tsx` - existing model selector pattern
- [SQLite ALTER TABLE](https://www.sqlite.org/lang_altertable.html) - official documentation

### Secondary (MEDIUM confidence)
- [shadcn/ui Select](https://ui.shadcn.com/docs/components/select) - component documentation
- [shadcn/ui Badge](https://ui.shadcn.com/docs/components/badge) - component documentation
- [rusqlite_migration](https://github.com/cljoly/rusqlite_migration) - migration patterns (codebase uses simpler approach)

### Tertiary (LOW confidence)
- Web search results on disabled option patterns - validated against shadcn docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - already using these libraries, no changes needed
- Architecture: HIGH - clear extension points identified in existing code
- Pitfalls: MEDIUM - based on general SQLite/React patterns
- UI patterns: HIGH - using existing shadcn components in documented way

**Research date:** 2026-01-17
**Valid until:** 90 days (stable domain, existing patterns)
