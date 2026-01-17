# Phase 3: Clone Progress - Research

**Researched:** 2026-01-17
**Domain:** Real-time progress streaming for git clone operations
**Confidence:** HIGH

## Summary

Phase 3 adds real-time progress feedback to the clone operation implemented in Phase 2. The git2 library provides comprehensive progress callbacks through `RemoteCallbacks.transfer_progress()` which reports object counts, bytes received, and indexing progress. The recommended approach is to use Server-Sent Events (SSE) for streaming progress from backend to frontend, as clone progress is inherently unidirectional (server-to-client only).

The backend will need to modify `GitManager::clone()` to accept progress callbacks and stream updates via a tokio mpsc channel from the blocking context to an SSE endpoint. The frontend will use the native `EventSource` API to consume progress events and display them using the existing shadcn Progress component.

**Primary recommendation:** Use SSE for clone progress streaming. The existing WebSocket is designed for session output streaming with bidirectional subscribe/unsubscribe patterns - clone progress is a simpler unidirectional flow that SSE handles more elegantly.

## Standard Stack

The established libraries/tools for this domain:

### Core (Already in Project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| git2 | 0.20 | Git operations with progress callbacks | Already used for clone, provides RemoteCallbacks |
| axum | 0.8 | HTTP framework with native SSE support | Already used, has `axum::response::sse` module |
| tokio | 1.x | Async runtime with mpsc channels | Already used, provides sync channel for spawn_blocking |
| @radix-ui/react-progress | (installed) | Progress bar primitive | Already have shadcn Progress component |

### Supporting (No Additional Dependencies Needed)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio::sync::mpsc | (part of tokio) | Channel for spawn_blocking communication | Send progress from blocking thread to async |
| async-stream | 0.3 | Create async streams from channels | Already in tokio_stream, for SSE response |
| EventSource | (browser native) | Frontend SSE client | Consume progress events |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| SSE | Extend existing WebSocket | WebSocket adds complexity (subscribe/unsubscribe pattern) for simple unidirectional stream |
| SSE | Polling endpoint | Poor UX, inefficient, requires client-side timing |
| tokio::sync::mpsc | std::sync::mpsc | tokio's works across async boundaries, simpler integration |

**Installation:**
No new packages required - all tools already in the stack.

## Architecture Patterns

### Recommended Project Structure
```
backend/src/
├── api/
│   ├── repos.rs       # Add SSE endpoint for clone progress
│   └── mod.rs         # Register new route
├── git/
│   └── mod.rs         # Modify clone() to accept progress callback
frontend/src/
├── api/
│   ├── types.ts       # Add CloneProgress type
│   └── client.ts      # Add SSE helper (optional)
├── components/
│   └── ralphtown/
│       └── CloneDialog.tsx  # Add progress display
├── hooks/
│   └── useCloneProgress.ts  # SSE subscription hook (new)
```

### Pattern 1: Progress Callback with Channel

**What:** Pass a tokio mpsc sender to the blocking clone operation
**When to use:** Communicating from spawn_blocking to async context
**Example:**
```rust
// Source: tokio.rs/tokio/topics/bridging
use tokio::sync::mpsc;

pub struct CloneProgress {
    pub received_objects: usize,
    pub total_objects: usize,
    pub received_bytes: usize,
    pub indexed_objects: usize,
    pub total_deltas: usize,
    pub indexed_deltas: usize,
}

pub fn clone_with_progress(
    url: &str,
    dest: &Path,
    progress_tx: mpsc::Sender<CloneProgress>,
) -> GitResult<git2::Repository> {
    let mut callbacks = git2::RemoteCallbacks::new();

    callbacks.transfer_progress(|stats| {
        let progress = CloneProgress {
            received_objects: stats.received_objects(),
            total_objects: stats.total_objects(),
            received_bytes: stats.received_bytes(),
            indexed_objects: stats.indexed_objects(),
            total_deltas: stats.total_deltas(),
            indexed_deltas: stats.indexed_deltas(),
        };
        // blocking_send is safe in sync context
        let _ = progress_tx.blocking_send(progress);
        true // continue cloning
    });

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    git2::build::RepoBuilder::new()
        .fetch_options(fetch_options)
        .clone(url, dest)
        .map_err(|e| GitError::OperationFailed(format!("Clone failed: {}", e.message())))
}
```

### Pattern 2: SSE Endpoint with Axum

**What:** Stream progress events to client using Server-Sent Events
**When to use:** Real-time unidirectional server-to-client updates
**Example:**
```rust
// Source: docs.rs/axum/latest/axum/response/sse/
use axum::{
    response::sse::{Event, KeepAlive, Sse},
    extract::State,
    Json,
};
use futures::stream::Stream;
use std::convert::Infallible;

async fn clone_with_progress_sse(
    State(state): State<AppState>,
    Json(req): Json<CloneRepoRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<CloneProgress>(32);

    // Spawn the blocking clone operation
    let url = req.url.clone();
    let dest = build_dest_path(&req.url);
    tokio::task::spawn_blocking(move || {
        clone_with_progress(&url, &dest, tx)
    });

    // Convert channel to SSE stream
    let stream = async_stream::stream! {
        while let Some(progress) = rx.recv().await {
            let data = serde_json::to_string(&progress).unwrap();
            yield Ok(Event::default().data(data));
        }
        // Send completion event
        yield Ok(Event::default().event("complete").data("{}"));
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
```

### Pattern 3: Frontend SSE Hook

**What:** React hook to consume SSE events
**When to use:** Subscribing to server-sent progress updates
**Example:**
```typescript
// Source: MDN EventSource documentation
export interface CloneProgress {
  received_objects: number;
  total_objects: number;
  received_bytes: number;
  indexed_objects: number;
  total_deltas: number;
  indexed_deltas: number;
}

export function useCloneProgress(
  onProgress: (progress: CloneProgress) => void,
  onComplete: () => void,
  onError: (error: string) => void
) {
  const eventSourceRef = useRef<EventSource | null>(null);

  const startClone = useCallback((url: string) => {
    // Close any existing connection
    eventSourceRef.current?.close();

    const eventSource = new EventSource(
      `/api/repos/clone-progress?url=${encodeURIComponent(url)}`
    );

    eventSource.onmessage = (event) => {
      const progress = JSON.parse(event.data) as CloneProgress;
      onProgress(progress);
    };

    eventSource.addEventListener('complete', () => {
      eventSource.close();
      onComplete();
    });

    eventSource.onerror = () => {
      eventSource.close();
      onError('Clone failed');
    };

    eventSourceRef.current = eventSource;
  }, [onProgress, onComplete, onError]);

  useEffect(() => {
    return () => eventSourceRef.current?.close();
  }, []);

  return { startClone };
}
```

### Anti-Patterns to Avoid
- **Polling for progress:** Inefficient and poor UX - use SSE instead
- **Sending progress on every callback:** The git2 callback fires frequently - throttle to avoid flooding
- **Blocking the async runtime:** Always use spawn_blocking for git2 operations
- **Using WebSocket for simple unidirectional streams:** Adds unnecessary complexity

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Progress streaming | Custom TCP socket | Axum SSE (`axum::response::sse`) | Built-in, handles keep-alive, proper headers |
| Channel communication | Arc<Mutex<Vec>> with polling | `tokio::sync::mpsc` | Designed for this, handles backpressure |
| Progress percentage | Manual calculation | Calculate from git2 Progress fields | `received_objects / total_objects * 100` |
| Throttling callbacks | Custom timing logic | Check if channel has capacity | `tx.try_send()` drops if full |

**Key insight:** The git2 progress callback can fire thousands of times per second on fast networks. Use channel capacity to naturally throttle - if the channel is full, skip the update. The frontend only needs ~10-30 updates per second for smooth UI.

## Common Pitfalls

### Pitfall 1: Progress Callback Performance
**What goes wrong:** UI becomes unresponsive due to high-frequency progress updates
**Why it happens:** git2's transfer_progress callback fires very frequently
**How to avoid:** Use bounded channel with `try_send()` to drop excess updates
**Warning signs:** Frontend lagging, browser memory growing

### Pitfall 2: SSE Connection Not Closing
**What goes wrong:** SSE connection stays open after clone completes
**Why it happens:** Missing completion event or error handling
**How to avoid:** Send explicit "complete" event, handle all termination cases
**Warning signs:** Open connections accumulating, memory leaks

### Pitfall 3: Progress During Object Resolution
**What goes wrong:** Progress jumps from ~50% to 100% suddenly
**Why it happens:** git2 has two phases: object transfer, then delta resolution
**How to avoid:** Show both phases in UI, or calculate combined percentage
**Warning signs:** Progress bar appears stuck, then jumps

### Pitfall 4: Authentication Blocking Progress
**What goes wrong:** SSH repos require auth, blocking before any progress
**Why it happens:** git2 needs credentials callback for private repos
**How to avoid:** Consider credential prompting in Phase 4, or document HTTPS-only for now
**Warning signs:** Clone hangs with no progress for SSH URLs

### Pitfall 5: Channel Dropped Before Clone Completes
**What goes wrong:** Progress stops mid-clone, no completion signal
**Why it happens:** SSE endpoint returns before spawn_blocking finishes
**How to avoid:** Keep receiver alive until spawn_blocking task completes
**Warning signs:** Progress gets to ~80% then nothing

## Code Examples

Verified patterns from official sources:

### git2 Progress Callback Setup
```rust
// Source: github.com/rust-lang/git2-rs/blob/master/examples/clone.rs
use git2::{FetchOptions, RemoteCallbacks, build::RepoBuilder};

let mut callbacks = RemoteCallbacks::new();
callbacks.transfer_progress(|stats| {
    // stats.received_objects() - objects downloaded
    // stats.total_objects() - total objects to download
    // stats.indexed_objects() - objects indexed locally
    // stats.received_bytes() - bytes downloaded
    // stats.total_deltas() - deltas to resolve
    // stats.indexed_deltas() - deltas resolved
    true // return true to continue, false to abort
});

let mut fetch_opts = FetchOptions::new();
fetch_opts.remote_callbacks(callbacks);

RepoBuilder::new()
    .fetch_options(fetch_opts)
    .clone(url, path)?;
```

### Axum SSE Response
```rust
// Source: docs.rs/axum/latest/axum/response/sse/
use axum::response::sse::{Event, KeepAlive, Sse};
use futures::stream::Stream;
use std::convert::Infallible;

async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        yield Ok(Event::default().data("hello"));
        yield Ok(Event::default().event("update").data(r#"{"value": 42}"#));
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
```

### Frontend Progress Display
```typescript
// Using existing shadcn Progress component
import { Progress } from "@/components/ui/progress";

function CloneProgressDisplay({ progress }: { progress: CloneProgress | null }) {
  if (!progress) return null;

  const percentage = progress.total_objects > 0
    ? Math.round((progress.received_objects / progress.total_objects) * 100)
    : 0;

  return (
    <div className="space-y-2">
      <Progress value={percentage} />
      <p className="text-sm text-muted-foreground">
        {progress.received_objects} / {progress.total_objects} objects
        ({formatBytes(progress.received_bytes)})
      </p>
    </div>
  );
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Polling endpoint | SSE for streaming | 2023+ (LLM streaming popularized SSE) | Simpler, more efficient |
| WebSocket for all real-time | SSE for unidirectional | 2024+ | Less overhead for one-way streams |
| Custom progress protocol | Native EventSource | Always available | Browser-native, auto-reconnect |

**Deprecated/outdated:**
- Long polling for progress: Replaced by SSE
- Using WebSocket for simple progress: Overkill for unidirectional updates

## Open Questions

Things that couldn't be fully resolved:

1. **SSH Repository Authentication**
   - What we know: git2 requires credentials callback for SSH URLs
   - What's unclear: Should Phase 3 handle auth, or defer to Phase 4?
   - Recommendation: Document HTTPS-only for Phase 3, add SSH auth in Phase 4

2. **Error Handling During Clone**
   - What we know: Clone can fail at any point (network, disk, auth)
   - What's unclear: How to communicate detailed errors through SSE
   - Recommendation: Send error event with message, close stream

3. **Multiple Concurrent Clones**
   - What we know: Each clone would have its own SSE connection
   - What's unclear: Should we limit concurrent clones?
   - Recommendation: Allow multiple, each dialog manages its own progress

## Sources

### Primary (HIGH confidence)
- [git2 RemoteCallbacks documentation](https://docs.rs/git2/latest/git2/struct.RemoteCallbacks.html) - transfer_progress API
- [git2 Progress struct](https://docs.rs/git2/latest/git2/struct.Progress.html) - available progress fields
- [git2-rs clone example](https://github.com/rust-lang/git2-rs/blob/master/examples/clone.rs) - official progress example
- [Axum SSE documentation](https://docs.rs/axum/latest/axum/response/sse/) - SSE response types
- [Tokio bridging with sync code](https://tokio.rs/tokio/topics/bridging) - spawn_blocking patterns

### Secondary (MEDIUM confidence)
- [shadcn Progress component](https://ui.shadcn.com/docs/components/progress) - usage patterns
- [SSE vs WebSocket comparison](https://softwaremill.com/sse-vs-websockets-comparing-real-time-communication-protocols/) - protocol selection

### Tertiary (LOW confidence)
- Various Medium/dev.to articles on SSE + Tauri/Rust patterns - implementation ideas

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries already in project, well-documented
- Architecture: HIGH - Follows established patterns (SSE, channels)
- Pitfalls: MEDIUM - Based on official docs and common patterns, not production experience

**Research date:** 2026-01-17
**Valid until:** 60 days (stable libraries, mature patterns)
