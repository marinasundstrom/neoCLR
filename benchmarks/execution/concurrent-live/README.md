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
