# Phase 1: Cleanup - Context

**Gathered:** 2026-01-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Remove dead code (mockData.ts) and replace `.unwrap()` calls in the DB layer with proper Result-based error handling. This is technical cleanup work to reduce panic points and improve error surfaces.

</domain>

<decisions>
## Implementation Decisions

### Error Detail Level
- API error messages should include technical details (root cause, field names, problematic values)
- Same verbosity in all environments — this is a local-only app, no need to hide details
- Only log unexpected errors (internal errors) server-side; NotFound is normal flow, don't log

### Error Type Granularity
- Moderate granularity for DbError variants: ParseError, ConstraintViolation, ConnectionError, NotFound (not fine-grained per-field types)
- Error types should include the problematic value for debugging (e.g., `ParseError("invalid uuid: 'not-a-uuid'"`)

### HTTP Status Codes
- Use detailed codes where appropriate: 409 (conflict), 422 (unprocessable entity), not just 400/404/500
- Map error types to specific status codes

### Error Response Format
- Structured JSON with error code: `{"error": {"code": "PARSE_ERROR", "message": "...", "details": {...}}}`
- Include full debug info in details: file/line info, field names, values — helpful for development

### Claude's Discretion
- Exact DbError enum structure
- Which `.unwrap()` calls to replace vs which are safe (e.g., in tests)
- Internal error logging format

</decisions>

<specifics>
## Specific Ideas

No specific requirements — standard cleanup following the decisions above.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-cleanup*
*Context gathered: 2026-01-17*
