# Shared native trace sessions — 2026-10-28

The third private slice joins owner-thread async events and collector events in
one bounded native recording. It adds explicit producer registration, session
identity, lane-local event ordering and one monotonic time base. On macOS, new
captures use mach_absolute_time converted to nanoseconds, avoiding the observed
microsecond quantization of CLOCK_MONOTONIC; other POSIX targets retain that clock.
Exports identify the clock, and captures with different clocks must not be merged.
The
[worker](README.md) and [async](async-README.md) experiments remain controls.
This is not a new scheduler, a public diagnostics API or EventPipe compatibility;
the [Go/.NET comparison](../../design/go-inspired-scheduling.md#execution-tracer-proposed-first-slice)
continues to guide the diagnostics design.

The shared scheduler now emits runnable admission/dequeue, dispatch-root transfer/
release and queued-abandonment events in domain 4. The HTTP summary correlates them;
the older generic session exporter treats unsupported domains as unknown rather
than inventing intervals. See [queue profiling qualification](../../validation/shared-scheduler-queue-trace-20261028/README.md).

## Session ownership

Link `runtime/native/trace/session.c` and build selected providers with
`NEOCLR_EXECUTION_TRACE`. The private header is `session.h`; GC compilation needs
its directory on the include path. GC request events additionally require
`NEOCLR_GC_PROFILE`. The async bridge also requires `NEOCLR_ASYNC_TRACE` and an
active private async recording session. Normal builds have no shared recorder
link dependency or provider calls. The qualified carrier is macOS ARM64;
registration/recording uses POSIX pthreads and is not a Windows implementation.

The creator reserves 1–16 producer lanes, each containing 1–65,536 fixed native
records. A record is 56 bytes on the qualified target. Each producer explicitly
attaches to an unused lane before emitting and detaches on the same thread when
finished. A thread cannot attach twice or implicitly replace its session. A lane
cannot be reused, even after detach. Roles are caller-supplied numeric labels,
not OS thread IDs. Native session IDs distinguish sessions within the linked
runtime instance; lane + domain + source + subject identify recorded activity.

Registration, detach and close use a control mutex. Emission is single-writer TLS:
no shared logging lock, allocation, formatting, I/O or managed references. Record
payloads are numeric identities, never object/heap/stack pointers. Full buffers
retain a prefix, count loss and skip clocks. Sequence gaps also expose clock-read
failures. Unsupported signal-handler/reentrant emission is excluded explicitly.

Only the creating thread may close, read or destroy a session. Close fails while
any producer remains attached and prevents future attachment after success.
Snapshots are borrowed until destroy. The caller must join every prospective
producer before destroying the session, including threads that have not attached
yet; the session does not own those threads or automatically extend their lifetime.
No live export/draining or late attach after close is supported. This contract is
what a future compiler-worker bridge must honor through its existing join path.

## Current providers and interpretation

| Provider | Identity and events | Meaning / limitation |
| --- | --- | --- |
| Async | Producer + monotonic per-thread recording instance + logical context/operation IDs | Mirrors the qualified park/resume/nested/abandonment stream. Repeated sessions and cached physical stacks do not reuse logical identity. |
| GC request | Producer + monotonically increasing request ID; reason at begin, status at end | Wall interval around the profiled collection dispatch. It can include synchronous work or concurrent-epoch admission/finishing. It is not a precise stop-the-world pause. Direct conditional-collection entry points bypassing this dispatcher are not covered. |
| Concurrent epoch | Producer + epoch counter; publication, finish or cancellation | Published epoch lifetime through disposal/join, not marker CPU time. Owner capture before publication belongs to the request boundary. Marker thread phases and individual heap IDs are not recorded. |

The first bridge mirrors into the shared session **and** the existing private
async buffer. This deliberate temporary duplication preserves the prior control
and allows chronological checks while introducing session ownership. The measured
incremental overhead includes its extra timestamp and stores. Replacing the private
sink is a later optimization; this is not a low-overhead production claim.

The `session-1` exporter maps logical contexts, scheduler activity, owner GC
requests and concurrent epoch lifetimes onto distinct tracks on the same time
axis. Cancellation is an explicit terminal kind. Missing lanes or malformed
sequence/loss accounting are rejected. If any events were lost, the exporter
retains raw transition points and omits **all inferred duration spans**, rather
than trusting a potentially broken cross-provider history. Raw records remain
available for later analysis.

Actual worker-pool registration, Rust/compiler hooks, marker-thread phases, real
I/O correlation and cross-process clock synchronization remain open. The generic
multi-producer test is not evidence of compiler or worker-pool integration. Shared
sessions cannot yet be enabled through the production JIT/HTTP CLI.

## Reproduction and evidence

```sh
SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk python3 docs/experiments/execution-trace/validate_session.py --output target/shared-trace-check
python3 docs/experiments/execution-trace/session-export.py target/shared-trace-check/concurrent-optimized-65536.json target/shared-trace-check/concurrent.trace.json
```

Open the resulting JSON with Perfetto's **Open trace file** action. The automated
checks validate structure/chronology, not interactive viewer behavior.

[Qualification, captures and incremental overhead](../../validation/shared-trace-20261028/README.md)
include sync and genuinely active concurrent GC, cancellation, incomplete captures,
concurrent producer lifetime tests, UBSan and compiled-out assembly comparisons.
The author now prioritizes a bounded queue-policy comparison, using the current
recorders and adding providers where needed. Compiler-worker request/wait, lowering,
code generation and publication remain open. Qualify real cold/warm JIT development
cases and Native AOT publication workloads under both collectors. CPU profiling
is still needed to attribute work within intervals.
