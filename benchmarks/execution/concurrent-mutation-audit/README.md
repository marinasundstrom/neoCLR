# Live-heap concurrent marking: mutation audit — 2026-10-24

This audit identifies implementation work needed to replace the measured full-copy
candidate. It does not enable concurrent access to production heaps. The preceding
[reachable-closure candidate](../concurrent-reachable/README.md) remains the safety
reference and synchronous GC remains the default.

## Findings that change the implementation plan

A pointer-only barrier is insufficient with the current native scanner. OBJECT
payloads are conservatively scanned as eight-byte words after the identity lane;
RECORDS scan all initialized lanes. `write_typed` can emit I8/I16 stores into these
lanes. Reading them concurrently with ordinary `memcpy` is a data race even when
that field is semantically a scalar. Neither an atomic marker flag nor a retrying
sequence counter makes non-atomic concurrent payload access safe.

The generated `write` helper also handles stack/output storage. Wrapping it blindly
would impose heap synchronization on local assignments and output buffers, while
still missing direct stores and native helpers. Heap provenance must be explicit
or dynamically checked for byrefs; source-level helper counts are not runtime costs.

Reserved record-array stores write payload before their initialization marker.
Reserved string/function-array stores currently set the marker before the payload
store. Both are safe under today's no-collection-between-instructions rule; neither
is a safe live concurrent publication protocol. A mutation region must cover both
writes, including narrow lanes and marker bytes. Merely reversing the second order
would not solve subsequent writes to an already initialized element.

## Reviewed surfaces

Paths below are relative to the repository. Exact source identities and lexical
candidate locations are retained in `inventory.json.gz`.

| Surface | Current implementation evidence | Required boundary |
| --- | --- | --- |
| Instance/static fields | `runtime/codegen/src/values.rs`, `Op::SetField`, `Op::SetStaticField`, `write_typed` | Entire field snapshot, including scalar lanes scanned conservatively |
| Borrowed writes/initialization | `values.rs`, `Op::StoreObject`, `Op::InitializeObject`, `FieldAddress`/`ArrayAddress` | Resolve containing allocation for interior destinations; distinguish stack from heap; cover zeroing as well as assignment |
| Arrays | `values.rs`, both `Op::StoreArrayElement` branches | Payload and initialization marker together; reference-bearing bulk replacement must use same protocol |
| Boxing/construction/function descriptors | `values.rs` constructor paths and direct descriptor stores; `enum_display.rs` | Explicit unpublished-allocation state or guarded initialization; enqueue outgoing edges on publication |
| Native object/array/static/intern initialization | `runtime/native/memory/text-arena.c`, `runtime/native/gc/native-gc.c` | Allocator zeroing alone does not publish a complete object; table/header/payload initialization must finish before scanner admission |
| Byte views and text | `text-arena.c` setters/copies and `memory-views.h` provider contract | BYTES/TEXT payloads are not traced, but their descriptors/publication and reference-bearing view owners still need coordination; prove exemption by allocation kind |
| Context/output/fault writes | `fault_details.rs`, `compiler.rs`, `native_imports.rs`, `values.rs` | Root handshake captures changing values; output pointers can alias managed storage and must not be assumed stack without proof |
| Conditional handles and parked tasks | `root-probe.c` parking lifecycle and native GC host-root services | Snapshot owner/value relationships and active roots; promotion to a strong root and release/expiry must participate in epoch completion |
| Worker/provider/native imports | `native-workers.c`, native import lowering, native host adapters | Separate heap ownership from shared references; audit aliases and provider writes, including callbacks; unsupported paths must reject concurrent admission explicitly |
| Retirement/sweep/reuse | `native-gc.c` intern retirement, poisoning, free-list rebuild, resource cleanup | Marker quiescent before descriptor changes/reuse; revoke base caches before reuse; keep cleanup on the owner |

`inventory.py` records 57 generated `.store(` lines, 42 generated write-helper
lines, 117 native bulk-memory lines and 67 root-lifecycle lines at this revision.
These include declarations, reads and tests; helper calls represent variable numbers
of emitted stores. Ordinary C assignments and externally supplied providers are not
exhaustively enumerated. This is a review index, **not coverage proof** or a count of
57 barrier sites. Rerun against current sources before integration.

## Proposed first live-heap protocol

Use an owned-heap, nonmoving marker with a per-heap mutation gate as the first
read-safe implementation. This is a provisional correctness baseline, not a claim
that a mutex per mutation will meet the performance goal. Do not expose an enable
switch before all admitted writer paths obey the protocol.

1. At an owner safepoint, capture roots and a detached allocation identity/index
   without copying all payloads. Initial allocations cannot move, retire or be
   reclaimed while the marker is active. Index lookup must not read mutating
   descriptor/free-list state. Keep new allocation bookkeeping owner-local.
2. Before changing traced payload bytes, enter the gate and preserve overwritten
   candidate words in an epoch work buffer. Cover the full eight-byte conservative
   lane for partial writes. Retain inserted candidates too until root handoff and
   black-object publication are independently proved. The mutation and queue
   publication complete before leaving the gate.
3. The marker takes the same gate while reading an object's payload into bounded
   private scratch, then releases it before address lookup/tracing. Large objects
   need bounded chunks, with barriers preserving deleted/inserted edges between
   chunks. Snapshot array shape/initialization coherently; never retain a pointer
   to mutable payload after releasing the gate.
4. Initially, treat new allocations conservatively at final remark and trace their
   completed payloads on the owner. This may retain floating garbage but avoids
   pretending unfinished object initialization is safe. Optimize this only after
   a checked publication contract exists. Preserve unresolved conservative words
   naming later-allocated storage, as the existing future-address tests require.
5. At completion, stop mutation, join/drain all marker and barrier work, capture
   current roots and conditional relationships, close the graph, then reclaim on
   the owner. No pending local buffer or parked-root update may be omitted.
6. Bound buffers and scratch. On exhaustion, cancel/join the marker and abandon its
   marks before allowing an unrecorded write; schedule synchronous recovery at a
   root-published boundary. Do not invoke GC from an arbitrary store helper while
   generated temporaries are unpublished. No callback/managed call/collection may
   occur while holding the mutation gate. Shutdown and allocation pressure use the
   same quiescence rules.

Gate entry needs a cheap inactive-epoch path. Measure both inactive and active
barrier costs, contention, copied bytes, dirty lanes and allocations, in addition to
whole-process CPU and pause distributions. Optimizing gate granularity or using
per-object copy-on-first-write comes later if this baseline's measurements justify
it. New-object publication and native aliases are integration requirements, not
permission to enable a partially covered collector.

## Alternatives and research

[Go's barrier implementation](https://go.dev/src/runtime/mbarrier.go) documents
pointer insertion/deletion shading and bulk-operation support. It supplies a useful
reachability model, but copying its pointer barriers does not solve neoCLR's
conservative scalar payload races. [.NET GC design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/garbage-collection.md)
provides the allocation/collection and root-coordination baseline; neoCLR need not
copy CLR internal object layouts to preserve familiar managed semantics.

Precise reference layouts with compatible atomic reads/writes could eliminate scalar
scan races and much of the gate traffic. That requires layout/type metadata for every
heap shape, exact handling of embedded values/arrays and native/borrowed writes.
It is broader than adding a barrier call. Copy-on-first-write can reduce initial
copying but still needs scanner/writer synchronization and a bounded publication
protocol. Cooperative incremental tracing avoids simultaneous payload reads, but
moves tracing work back to mutator boundaries. Full copying is already measured
and remains the oracle. Parallel stop-the-world tracing is a separate option, not
an obligatory prerequisite for this work.

## Integration and acceptance gates

Next implement the epoch/gate/buffer primitive with delayed-writer/marker tests,
including partial lanes, array markers, overflow, callback prohibition, cancellation
and reuse. Then integrate generated field/array/byref and native publication paths
as a single opt-in closed-world contract, with explicit rejection of unsupported
providers. Any compiler-affecting integration requires matching neoCLR/Raven bridge
documentation; this audit changes neither compiler nor bridge semantics.

Repeat existing future-address, randomized closure, conditional and GC stress tests
against the live candidate and retain a mutation-omission negative control. Qualify
on a working race-detector platform before default adoption. Run batched standalone
collections, JSON, async and object graphs as well as HTTP. Capture compilation/
startup separately from steady execution. Shared heaps and multiple owners require
separate qualification; successful owned-heap runs do not establish their safety.
