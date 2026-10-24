# Explicit owned-heap live concurrent marker — 2026-10-24

This candidate connects the [mutation gate](../concurrent-mutation-gate/README.md)
to actual native allocations, background tracing and owner-thread reclamation.
Unlike the full-copy reference, capture copies compact allocation headers/offsets
and root values, not payloads. The marker reads payloads through the gate in at most
256-byte chunks and uses detached mark/index/worklist state. It never reads owner
TLS, live allocation headers or changing frame slots. Shared heaps are rejected.

This is an explicit native admission experiment. Every mutation of an existing
traced payload must go through `experiment_gc_write`; generated code and native
providers require explicit admission. A restricted internal managed compiler path is
[qualified separately](../concurrent-store/README.md#managed-value-compiler-integration);
application/native service providers are **not admitted yet**. The production collector/default is unchanged.
Do not substitute this generated collector into HTTP hosts using unmodified code.

## Reachability and lifetime

The gate logs overwritten and inserted complete candidate words, including lanes
changed by narrow writes. Worker passes drain that bounded queue after tracing the
pending graph. Finish waits for a pass to stop, drains barriers that arrived after
it went idle, and only requests another pass if they discover unscanned old objects.
Current roots and conditional relationships are processed by the existing owner
collector before sweeping. Failures remain sticky across worker passes.

All old allocated objects retain stable base/span/size/kind until remark, checked
against the detached identities. Published array shapes are immutable; the mutation
adapter rejects writes to their shape headers. Payload/initialization-marker stores
must be grouped. The gate is not a complete admission verifier: pointer lifetimes,
source aliases, array marker-only exposure and adherence to the mutation contract
remain the trusted native caller's responsibility.

New allocations are absent from the detached index. They can be initialized by the
owner without concurrent marker reads; every new allocation is retained and traced
at remark, even if unreachable. This deliberately permits floating garbage and
makes construction safe without claiming a complete publication protocol. It also
preserves conservative words that point into storage allocated later: all such new
objects are traced. The previous reachable-only snapshot candidate needed a separate
future-address vector because it did not retain all new allocations. Removing that
retention here would require that protection or a stronger publication contract.

Overflow rejects the write before it changes any bytes. Finish discards the failed
cycle without sweeping; the caller can then run synchronous collection with current
roots. No synchronous GC runs inside the store helper. Descriptor retirement, release
and arbitrary native writes are not silently admitted. Cleanup and reuse stay on the
owner after marker quiescence. Final disposal destroys the gate only after the worker
has stopped using it. Prototype metadata allocation failure still aborts.

## Qualification

Optimized and undefined/bounds builds exercise:

- 100 delayed deletion/insertion cycles with actual reclamation. Removing a sole
  edge before the marker runs still retains the deleted referent for the epoch.
- Insertion after the initial worker pass, followed by transitive closure at finish.
- A reserved array's grouped payload/marker publication and shape-write rejection.
- Free-hole and appended-storage conservative addresses followed by new allocation.
- Queue exhaustion, unchanged rejected writes, no sweep on failure and synchronous
  recovery.
- 12,000 randomized block-liveness comparisons. The independent oracle includes
  initial reachability, all logged old/new referents, current roots and all new
  allocations, closed over current edges. This tests the candidate's conservative
  retention contract, not exact current-only liveness.
- Original collector, conditional/parked, dense, seed and 142,272 scanner configurations
  per build, wrapped in explicit begin/finish so the live path actually executes.

A negative control replaces deletion candidates with zero. The delayed sole-edge
fixture aborts at its liveness assertion, establishing that the barrier test can
catch premature reclamation. UBSan does not replace race-detector qualification;
the previously recorded host sanitizer startup failure remains unresolved.

## Cost experiment and next integration

The standalone native graph workload uses 4,096 live nodes, 64 new allocations and
64 guarded edge replacements per cycle, scalar work between begin/finish, and 10,000
cycles. Synchronous and concurrent modes use the same binary/managed graph checks;
process CPU includes the worker. It is not a managed JIT/AOT application or an HTTP
capacity result. Raw reports retain command lines, hashes, checksums and counters.

The first always-resubmit finish implementation is retained separately. The current
finish avoids waking the worker when the final barrier drain finds no unscanned old
object. No claim about the effect of that change should be made from historical
cohorts alone; compare each candidate to its paired synchronous runs.

The current paired means are **2.59× process CPU and 2.31× wall time** versus
synchronous GC: concurrent CPU 2.186–2.219 s, synchronous 0.832–0.871 s. This is
not a performance improvement. Arena peaks are 236,600 versus 233,016 bytes,
excluding detached metadata, worker stack, gate and queues. Each concurrent trial
completes 10,000 valid epochs with zero cancelled epochs, copies 1,331,997,760 header
bytes across all captures, reads 655,519,984 candidate payload bytes and drains
1,280,000 barrier words. Payload counters exclude shape/marker reads and owner
remark reads; these are totals, not peak memory. Metadata capture is still substantial
for small objects. The preceding always-resubmit cohort costs 2.62× synchronous CPU;
this does not establish a statistically meaningful before/after gain.

The next required integration is compiler/native writer admission and construction/
publication coverage, with explicit unsupported-provider rejection. The broader
standalone/HTTP matrix, paused-task handoff, multi-owner/platform testing and total
memory accounting remain open. Per-object mutex traffic, allocation-index lookup,
metadata capture/validation and finite-buffer fallback must be measured and reduced;
removing full payload copies alone does not establish an efficient collector.

The [.NET GC design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/garbage-collection.md)
provides the root/allocator/collector baseline, while [Go's barrier implementation](https://go.dev/src/runtime/mbarrier.go)
explains insertion/deletion reachability obligations. This prototype deliberately
uses gated conservative payload reads instead of assuming precise atomic pointer
layouts. The benefit is a checkable read protocol with existing native layouts;
the costs include synchronization, conservative retention and owner remark work.

Run `validate.py --sparse-index --output <fresh-directory>` and
`measure.py --sparse-index --output <fresh-directory>`. The generated source links
with concurrent-mutation-gate/gate.c and current native root helpers. This private
experiment changes neither Raven emission nor the managed/native bridge contract.


## Gate granularity follow-up

The [bounded-batch comparison](batch-results.md) separates marker CPU, owner pauses,
guarded writes and uninstrumented process costs. Larger batches reduce lock calls
but increase process CPU 20.8% and move more blocking into mutations in this native
cohort. The macro remains off; these findings do not replace managed/HTTP qualification.


## Generated-store boundary follow-up

A [private scalar-store adapter](store-boundary.md) now handles narrow/interior
writes and cancels an overflowing epoch without collecting unpublished temporaries.
Native tests verify bounds, construction and deferred cleanup. It is not yet emitted
by the compiler; complete writer admission and JIT/AOT consumers remain next.


## Automatic owned-heap scheduling qualification

`automatic.py` derives a separate scheduled live candidate from `generate.py`.
It reuses the snapshot integration's policy: allocation/publication budgets alternate
capture and completion; idle collects synchronously before capturing survivors.
Explicit/pressure collections finish outstanding work and recheck synchronously,
so new floating garbage cannot defeat the existing one-retry allocation contract.
Foreign-context collection, intern retirement and heap release cancel/join detached
work before changing the owning heap's lifetime. Scalar-store overflow cancellation
also contributes to scheduling cancellation counts. This prototype requires profiling
and rejects shared-worker builds. Production source/defaults are unchanged.

The policy is provisional, with the existing 256-allocation/4096-publication budgets.
It retains idle cleanup semantics and can perform extra synchronous scans. Capture
still copies compact allocation headers and roots, and each epoch still allocates
native bookkeeping. Scheduling alone does not demonstrate a throughput benefit.
The current roots and barrier protocol remain those of the explicit live candidate;
this does not admit arbitrary native writers or HTTP hosts.

Reproduce with a current codegen test runner:

```sh
python3 benchmarks/execution/concurrent-live/validate-automatic.py \
  --runner target/concurrent-store/test-runner-managed-final \
  --output target/concurrent-live/automatic-qualified-new
```

The harness checks pressure recovery, idle resource cleanup, foreign contexts,
intern retirement and release; both allocation and publication budgets start/finish
live epochs. Optimized/UBSan builds each run 100 guarded graph-mutation cycles and
existing native collection/root-seeding fixtures. It also reuses the archived managed
AOT object and current JIT fixture against the scheduled host, replacing explicit
begin with the normal idle boundary in the native driver. Optimized/UBSan AOT and
JIT each run 100 managed cycles, retaining the exact two guarded-write checks and
fault/root restoration assertions. Managed constructors and the full entry wrapper
remain outside this fixture's scope.

`automatic-validation.json.gz` contains commands/results and input hashes;
`automatic-native-gc.c.gz` preserves generated source. The initial managed link
omitted `NEOCLR_SHADOW_TEST`, leaving the fixture's marker-gate symbol unresolved;
`automatic-initial-link-failure.json.gz` retains it. Correcting the test build flags
resolved the link without changing runtime behavior. No timing measurements were
made. Next qualify native allocation/service writers and the full host entry before
running ordinary standalone and HTTP comparisons. The existing CLR/Go barrier and
collector research remains applicable; this is reuse of an experimental scheduling
policy, not a semantic divergence or performance claim.


## Native-call synchronous fallback prototype (2026-10-24)

`native-scopes.py` extends the automatic candidate with a private, owned-thread
fallback scope. Entry requires a matching published caller frame and cancels/joins
an active marker without collecting; this preserves unpublished native inputs.
While nested, explicit, pressure, idle and publication collections use synchronous
GC and cannot start a live epoch. Exit does not collect or restart the marker before
the caller publishes results. Normal scheduling resumes at subsequent boundaries.
Monotonic tokens enforce LIFO nesting, capped at 64 without heap allocation (512 bytes
of thread-local token storage plus counters/state). Wrong-thread/context, unframed,
overflow, stale-token and out-of-order entry/exit requests reject; heap release is
rejected while scoped. Existing synchronous rooting requirements still apply.

This is a proposed integration mechanism, not general native-provider admission.
The compiler now selects an exact scoped UTF-8 slicing adapter (see
[generated qualification](../concurrent-store/README.md#scoped-utf-8-slicing-2026-10-24)).
Other calls remain unadmitted. Nonlocal unwinding, asynchronous retained writes,
shared heaps and callback/fiber transitions remain unqualified. Wrapping an entire
async pump would suppress concurrent collection for that whole native call and is
not an acceptable substitute for qualifying its individual transitions. Frequent
cancellation could erase useful marking work; measure fallback counts and throughput
before adopting this mechanism for hot providers.

The comparison is with the CLR's explicit GC modes and published frame/root state:
[CLR threading design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/threading.md),
reviewed 2026-10-24. CLR preemptive mode permits collection while native code that does
not access managed objects runs; native VM code accessing objects uses cooperative
mode. This prototype instead cancels concurrent marking and retains synchronous
collection for a scoped owned heap. It is a conservative experimental implementation
tradeoff, not a reproduction of CLR transitions or a new managed API contract.

Run `python3 benchmarks/execution/concurrent-live/validate-native-scopes.py --output
 target/concurrent-live/native-scopes-new` (one shell line, fresh output directory).
`native-scopes-validation.json.gz` records four optimized/undefined-bounds builds
across forced/paced scheduling, 16 lifecycle rounds each, existing collector/root
seeding/automatic regression tests and 120 previously generated managed AOT array
cycles. All pass. Removing entry cancellation fails the active-marker assertion as
expected. Generated positive/negative sources are archived alongside the report.
The initial harness incorrectly ran the forced-collection baseline with pacing;
its exit 58 is preserved in `native-scopes-initial-harness-failure.json.gz`. Correcting
that configuration made the baseline pass. There is no new performance result,
compiler admission, HTTP qualification or production-default change in this slice.


## Async ownership and socket transfers with live epochs (2026-10-24)

Two native integration harnesses now compare synchronous GC with the automatic live
candidate across forced/paced and optimized/undefined-bounds builds on macOS ARM64.
They use the real cooperative async carrier and real loopback socket implementation;
the async fixture stubs the idle scheduler to isolate ownership, while the socket
fixture exercises actual polling and transfer. No fallback scope surrounds the pump.

`validate-async-ownership.py --output <fresh-directory>` derives a checked extension
of the existing native-owned-body fixture. Each of eight runs starts 321 callbacks,
resumes 160 and abandons 161, including owner expiry inside scheduler acquire and
host-root capacity exhaustion. All return zero reserved stack bytes after shutdown.
Each run exercises 1,121 explicit collection/start points and 640 mutations. In the
four live runs, begin asserts an active epoch; a new child is attached through a
suspended local's heap object, and guarded scalar mutations run after resumption.
The callback root frame is parked/restored by the production carrier. Collection
retains the child through that conditional frame, then reclaims the graph when its
owner disappears. These tests also cover late completion after abandonment, managed
and ordinary owned bodies, cached contexts, release and shutdown. This is controlled
native callback evidence, not generated callback-wrapper admission or an HTTP test.

`validate-socket-transfer.py --output <fresh-directory>` extends the existing real
loopback transfer fixture to start an epoch after collection and retain ownership
until shutdown. All eight builds pass accept/delivery root lifetime, partial receive,
reserved-byte initialization, snapshot send despite later buffer mutation, deadline
and cancellation errors, EOF, pending-operation fault teardown and zero retained
heap. The live marker can remain active while native receive copies modify BYTES
payloads, which the collector intentionally does not trace. Result outputs are C
stack storage; generated heap-output aliases still require guarded wrappers.

Reports and generated fixtures are archived in `async-ownership-*` and
`socket-transfer-*`. The first async extension mistakenly placed the child pointer
in OBJECT's untraced type-header word; the synchronous baseline correctly failed.
The corrected fixture uses a separate fourth payload word, retaining its original
owner back-reference. The initial failure is archived, not counted as a runtime bug.

This advances carrier/socket lifetime qualification without changing production
code, compiler admission or collector defaults. Existing CLR frame/suspension and
conditional-root research still applies. Full generated callback/entry integration,
remaining native services, shared heaps, race-detector coverage and HTTP/standalone
performance comparisons remain open. These runs measure correctness, not throughput
or CPU; do not extrapolate their small fixture timings to server performance.
