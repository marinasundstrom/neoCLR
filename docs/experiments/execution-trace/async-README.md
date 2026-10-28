# Owner-thread async execution tracing — 2026-10-28

The second private tracing slice instruments the current macOS ARM64 async
carrier. It complements the [worker recorder](README.md); it does not change
scheduling, migrate stacks, make JIT compilation nonblocking or expose a managed
tracing API. Runtime comparisons remain in the
[Go/.NET investigation](../../design/go-inspired-scheduling.md#execution-tracer-proposed-first-slice).

## Contract and timeline

Build with `NEOCLR_ASYNC_TRACE`. Before admitting any async runtime context on the
owner thread, call `neoclr_async_trace_begin(capacity)` from
`runtime/native/async/native-async-trace.h`. Capacity is 1–65,536 fixed native
records (40 bytes each on the qualified target); allocation failure rejects the
session. Not beginning leaves recording disabled. Normal builds compile out the
recorder and extra fiber identity field entirely. Windows tracing is explicitly
unqualified; defining the flag there fails compilation rather than implying support.

The recorder is thread-local and never allocates, formats, performs I/O, acquires
a lock or retains a managed reference during emission. The APIs must be called on
the same owner thread and not from signal handlers. Finish only after runtime
shutdown; its view borrows native storage until dispose. Finish and dispose are
rejected while the runtime is admitted. Dispose is required before another
session. Full buffers retain a prefix and count dropped events, skipping further
clock reads; failed clock reads also count as loss. No live reader/export exists.

Every event has CLOCK_MONOTONIC nanoseconds, sequence, logical context ID, related
ID, kind and status/reason. Context IDs are assigned per activation and renewed
when the cached physical carrier is reused. They are not stack addresses, OS
thread IDs, operation IDs or managed object identities. Operation IDs connect a
park to completion; parent IDs connect nested entry/return. Events cover:

- Operation creation and successful completion, including automatic body completion.
- Context entry/return at actual stack-switch boundaries, with parent identity.
- Root publication before parking, and body finish with its returned status.
- Abandonment after owned-root expiry, or during explicit shutdown.
- Entry/exit around the scheduler's wait call.

Raw park publication occurs before switching back. The exporter begins the
parked interval at the subsequent return boundary, avoiding overlap with the
remaining execution/transition interval. It splits waiting for an operation from
completed work awaiting dispatch. Parent execution spans stop while nested child
contexts run, so one owner's execution is not counted twice. Abandoned contexts
end with an abandonment span, not a fabricated resumption. Missing events produce
unknown/incomplete state, not an assertion of deadlock.

The output has logical context tracks plus an owner-scheduler track. All run on
one owner thread: parallel-looking parked spans are not parallel execution. Wall
execution spans include native blocking, GC and OS descheduling. This slice does
not measure actual CPU occupancy, individual I/O latency or GC/JIT phases.

The private `async-1` schema is separate from the worker schema. Both use the
process monotonic clock, but automatic merging, shared session identity, Rust clock
interop and cross-thread causality are not implemented. The small independent
recorder avoids coupling the production carrier to the experimental pool; a
shared session/provider contract is the next integration step, not a permanent
promise of incompatible trace formats.

## Validation and use

```sh
SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk python3 docs/experiments/execution-trace/validate_async.py --output target/async-trace-check
python3 docs/experiments/execution-trace/async-export.py target/async-trace-check/sync-optimized-65536.json target/async-trace-check/sync.trace.json
```

Open the exported JSON through Perfetto's **Open trace file** action. Export
structure and chronology are checked automatically; interactive import is not.

The fixture uses a deterministic scheduler adapter, not real socket traffic. It
parks a body with a live rooted local, runs independent nested work, completes the
operation through scheduler wait, resumes it, and checks a pre-completed wait does
not suspend. Fault and forced-shutdown paths remain distinct. Repeated rounds reuse
physical carriers but retain distinct logical identities. The same fixture runs
with sync GC and the configured owned concurrent candidate, requiring actual
started/finished epochs. A separate existing owned-body fixture checks 161 expired
contexts that must never resume. Overflow, truncated captures, malformed sequence
and nonoverlapping nested execution intervals are checked.

[Results, retained captures, overhead and limitations](../../validation/async-trace-20261028/README.md).
This is diagnostic qualification, not an HTTP comparison or evidence of a winning
collector. Next unify sessions and add JIT wait/publication and GC phase events;
then correlate real I/O and cold/warm HTTP traffic with both collectors.
The [subsequent shared-session slice](session-README.md) now implements the first
async/GC correlation bridge; compiler and worker producers remain open.
