# Noncollecting scalar-store boundary — 2026-10-24

`experiment_gc_store(context, destination, bits, width)` is a private native adapter
for the next generated-store integration. It is **not yet emitted by JIT/AOT** and
is not a managed API or production runtime import. The caller must own the supplied
heap/context and provide a valid native destination. Supported widths are 1, 2, 4
and 8 bytes; low bits are copied in native scalar representation, including unaligned
addresses. Floating values must eventually arrive as bit patterns, not numerically
converted integers. Vector/bulk stores require separate coverage.

## Implemented behavior

- Valid storage outside this owned arena is treated as trusted stack/host storage
  and written directly. This does not authorize writes into another managed heap.
  Intersections crossing the arena boundary are rejected.
- Heap destinations are resolved to their containing allocation; writes into headers,
  padding, unused arena capacity or across a payload boundary are rejected. This
  supports interior byrefs without pretending they are exact object bases.
- Inactive-epoch writes and construction of allocations absent from the detached
  index write directly. New objects remain excluded from marker reads and all are
  traced at remark under the current conservative policy.
- Existing payloads use the gate and old/new word logging. Published array shape
  headers stay immutable. Narrow writes preserve complete candidate lanes.
- Buffer exhaustion cancels and waits for the worker, discards detached marks, and
  performs the still-pending write. It does **not** walk roots, sweep, run native
  resource cleanup or call managed code. The next collection must occur at a normal
  root-published boundary. The cancelled explicit epoch can no longer be finished.

The cancellation rule resolves an integration hazard: a generated store can execute
while expression temporaries are not published as roots. Calling synchronous GC as
an overflow fallback there could collect those values. Waiting/discarding the marker
without collecting leaves them intact. It can still block, so cancellation frequency
and latency require measurement after integration. No throughput benefit is claimed.

The destination lookup currently walks allocation descriptors. That is a correctness
baseline, not an efficient store path for a large heap. Lowering must preserve known
stack/heap provenance where possible and measure any checked owner hint or indexed
lookup before deployment. Do not introduce a per-store metadata walk into production
silently. Counters distinguish outside-arena, inactive/new heap, guarded old heap,
cancelled-cycle and rejected calls; they are not allocation counters.

## Validation

Optimized and UBSan/bounds runs check 97 offset/width combinations in old live
payloads, exact low-bit results, unaffected neighboring bytes, stack writes during
and after an epoch, interior byrefs and rejected widths/context/extent/header writes.
New reserved-array headers, payload and initialization marker are constructed through
the adapter and survive remark; changing that header once published is rejected.

A deterministic overflow case first lets the worker finish, then fills its undrained
barrier queue. The next store cancels the cycle and succeeds without changing the
collection count. An object held only in an unpublished native temporary remains
valid and its attached cleanup does not run. Once put in a published reachable graph
it survives ordinary collection; final root removal reclaims it and runs cleanup once.
Existing live-marker fixtures, 12,000 differential block checks and the omitted-deletion
negative control also pass. No compiler, HTTP, shared-heap or race-detector qualification
is implied by these native tests.

## Required compiler/host integration

1. Add an explicit experimental closed-world admission contract and matching host
   capability. Do not enable concurrent epochs for unmodified generated images.
2. Lower admitted stores with context/destination, bit-preserving value and width;
   preserve fault propagation for rejected stores. Identify stack stores statically
   where proven, but handle byrefs that can target heap payloads. Cover all emitted
   scalar stores, generated adapters, native helpers and reference-bearing bulk writes.
3. Define construction completion and marker publication. Individual scalar barriers
   preserve candidate words but do not promise atomic multi-lane application values.
   Marker-only exposure of preexisting payload still requires logging/rescanning
   that payload; unsupported paths must fail admission rather than bypass the gate.
4. Keep this call noncollecting. Frame/temporary publication belongs at actual GC
   boundaries. After cancellation, restart/collect only at such a boundary, with
   pressure and shutdown recovery qualified.
5. Bind and test both JIT and AOT, including narrow/float bits, byrefs, arrays, faults,
   native returns and suspended tasks. Update both Raven and neoCLR bridge documents
   when lowering/admission actually changes. This native-only step changes neither.

This follows the compiler/runtime coordination purpose of CLR/Go write barriers
already researched in the mutation audit, while retaining neoCLR's current conservative
payload semantics. A native helper alone does not establish complete barrier coverage.
The next deliverable remains generated-store integration and executable managed
consumers, followed by the full performance/correctness matrix.
