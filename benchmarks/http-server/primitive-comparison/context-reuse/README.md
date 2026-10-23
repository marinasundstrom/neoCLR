# Completed async context reuse — 2026-10-23

The optional `NEOCLR_ASYNC_CONTEXT_REUSE` macOS ARM64 experiment reuses successfully
completed async execution contexts, in addition to their guarded stack mappings.
It is implemented in the main native runtime source but **not enabled by default**.
Windows rejects this experimental define; its fiber path remains unchanged. Ordinary
builds produce a byte-identical optimized runtime object to the preceding source.

## Why and what changes

The preceding [stack profile](../stack-profile/README.md) located almost all sampled
memory clearing beneath makecontext. Stack mapping caching alone still prepared a
fresh context for every async body. The new entry loop yields after completion and
can resume to invoke a fresh callback without repeating getcontext/makecontext.

The runtime keeps cached contexts and their fiber records at stable addresses. It
never copies a ucontext or relocates the C frames which refer to it. Before caching,
the prior callback root and parked-root record are released/discarded; owner, active
handles, operation/wait IDs, readiness, status, drain/managed flags and caller pointer
are cleared. A new invocation gets fresh callback/operation state. Signal masks and
floating-point environment are inherited from its current caller, not the previous
callback. Suspended or abandoned bodies are not recycled; faulting completions are
unmapped. Existing owned-body expiration and failed-admission rollback still apply.

Cached contexts are excluded from pending-work scans and operation-release checks.
Shutdown unmaps cached contexts without treating them as live work. The existing
cache bound (eight by default) remains; excess completions are unmapped. The context
record gains reuse/FP-environment metadata and preserves native stack bytes longer,
while guarded mapping size and cache count remain bounded. No managed allocation
zeroing, object header, GC pacing, queue semantics or generated ABI changes.
Explicit roots, not residual native stack bytes, determine managed reachability.

.NET's [async implementation](https://devblogs.microsoft.com/dotnet/how-async-await-really-works/)
uses compiler-generated state machines and supports synchronous completion without
suspension. neoCLR's runtime-owned retained-stack carrier is a different existing
implementation strategy. This experiment reduces the carrier's preparation cost;
it is not .NET state-machine/ExecutionContext compatibility, a new green-thread API
or a decision to avoid future compiler-level async improvements. Broader async and
platform qualification remains necessary before selecting a default policy.

## Validation

Twelve executions cover baseline/reuse with optimized and UBSan/bounds builds of:

- Existing owned-body GC lifecycle: 321 entries, 160 resumptions, abandonment,
  collection at idle, dependent roots and activation-capacity rollback.
- Existing idle/drain scheduling, including a suspended drain and exact wait counts.
- A new lifecycle fixture: 626 callback executions over two complete runtime
  invocations, changing callback data, alternating FP rounding modes and signal masks,
  collection between bodies, twelve simultaneously parked bodies (more than eight
  cache slots), forced collection before resumption, faulting completion, recovery
  through fresh invocation, and balanced mappings after shutdown.

Two further sanitized runs validate cache limits zero and one. The forced-GC HTTP
fixture checks eight responses across two invocations with sanitizer checks, without
GC pacing. Eight uninstrumented capacity trials exercise one/four isolated owners.
Shared-heap async HTTP and Windows reuse are not qualified by these tests. UBSan is
not a race detector. The disabled-feature object is byte-identical to the old source.

A separate candidate four-owner capture validates 38,655 responses. Its 11,904 worker
samples contain eight bzero self samples, all called from GC allocation, and none
beneath makecontext. The preceding baseline profile had 1,325 bzero samples, 1,316
beneath makecontext. These wall-clock samples corroborate the mechanism, not an exact
CPU reduction or controlled percentage comparison. Allocation/root/TLS work remains
prominent. The profiler now uses the verified executable name for worker identification;
a seventh parser regression covers names such as candidate-w4 instead of only server.

## Paired HTTP result

Same application object, native sources, owned heaps, 256/1,024 GC pacing, EOF-aware
Go client, 32 connections, ten x response bytes and Connection: close. Only the
context-reuse define differs. One/four isolated application owners run forward/reverse,
two seconds warmup and six measured; builds finish before serial trials. CPU per
response is server lifetime CPU minus client CPU, divided by all warmup/window/drain
responses. Every response and process exit is checked. This is a server before/after
comparison using the same external Go driver, not a neoCLR HttpClient benchmark.

| Variant | Owners | Requests/s (two trials) | CPU µs/response (two trials) | p99 ms (two trials) |
| --- | ---: | --- | --- | --- |
| Baseline | 1 | 1,655.0; 1,639.3 | 605.5; 604.3 | 20.72; 38.70 |
| Reuse | 1 | 1,850.5; 1,829.5 | 542.1; 539.3 | 19.60; 39.78 |
| Baseline | 4 | 4,772.2; 4,949.5 | 773.0; 741.2 | 9.72; 8.73 |
| Reuse | 4 | 5,545.8; 5,583.0 | 654.9; 656.9 | 7.66; 8.51 |

Means improve **11.7%/14.5% throughput** and **10.6%/13.4% CPU per response** at one/four
owners. Four-owner mean rises from 4,860.8 to 5,564.4 requests/s. One-owner tails remain
variable; short desktop runs do not establish sustained capacity or universal speedups.
Kestrel was not rerun. Its earlier result is a target, not a current paired control.
Retain the useful opt-in experiment and qualify broader consumers/platform behavior
before enabling it by default. This does not complete the comparable-throughput goal.

## Reproduce

Use the existing SDK and matching compiler/application bundle. Output paths must be new.

```sh
python3 benchmarks/http-server/primitive-comparison/context-reuse/validate.py \
  --output target/context-reuse-validation
python3 benchmarks/http-server/primitive-comparison/context-reuse/forced.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-owned-root/candidate.o --output target/context-reuse-forced
python3 benchmarks/http-server/primitive-comparison/context-reuse/measure.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-owned-root/candidate.o \
  --client target/primitive-comparison-refresh/client --output target/context-reuse-measure
```

Additional cache-bound and disabled-object compile commands are retained in their
JSON reports. The profile build manifest selects the measured candidate-w4 command;
use it with the stack-profile capture/analyze tools. Raw sample and full attribution
are retained as gzip files. All reports retain source/object hashes and commands.
No public Raven API, compiler representation or website capability changes here.
