# Concurrent snapshot marking with changed-block remark — 2026-10-24

This explicit native experiment consumes background mark results to perform real
reclamation on the owner thread. It extends the [immutable shadow tracer](../concurrent-shadow/README.md)
without letting a worker read mutable payloads. It is an experimental correctness
candidate, not the runtime default or an HTTP-integrated collector.

## Algorithm and safety argument

`experiment_gc_begin` validates the heap and seeds current roots using the existing
collector, captures heap/conditional-root values, starts background tracing, clears
private mark/queue bits in the original heap and returns without sweeping. The
owner may allocate and mutate the heap while tracing continues. No intervening
collection or arena release is allowed; those operations reject the active epoch.
The explicit experimental contract requires finish before allocation pressure or
context teardown. It is not yet suitable for general generated execution.

`experiment_gc_finish` waits for the owned job, validates the current heap and
requires every originally allocated block to retain its address, span, length and
kind. It seeds current roots, preserves snapshot-marked unchanged blocks, and
queues every new or bytewise-changed block for tracing. Existing root, conditional,
parked-chain, cleanup and sweep logic then runs on the owner thread. The worker
never reclaims memory, invokes resource cleanup or accesses live roots/TLS.

The retained set includes the initial reachable closure, the current root closure,
and all new/changed blocks and their current closure. An unchanged marked block's
old references already have snapshot marks; changed referents are independently
queued. A current root into a previously unmarked object triggers ordinary tracing.
This covers references introduced by generated, native, byref and bulk writes
without yet instrumenting those stores. Conditional exposure is checked through
current strong roots and the ordinary conditional fixed point. Old allocated blocks
cannot be reused before finish; originally free spans may split for new allocations.

This deliberately retains floating garbage, including changed unreachable blocks
and newly allocated garbage, until a later cycle. It uses whole-payload comparison,
not hashes: there is no collision-based correctness assumption. Invalid snapshots,
changed allocation identities or malformed current payloads fail before sweep;
private marks are reset and synchronous recovery is available after repair.

The implementation shares the existing sparse index, collector rules and cleanup
logic. A generated out-of-tree source replaces only capture/seed lifecycle points.
The shadow scanner and immutable worker are reused; shadow-comparison counters are
not presented as correctness evidence for this authoritative collector. Owned-heap
release joins the worker after the epoch completes.

## Evidence

`validation.json.gz` passes five fixtures in optimized and undefined/bounds-sanitized
configurations:

- 100 delayed cycles with a cyclic original graph, disconnected original referent,
  fresh allocations reusing an old free hole, references to initially unreachable
  graphs, and changes to unreachable objects. Finish reclaims an unchanged dead
  block while retaining the required closures; later synchronous collection releases
  the floating garbage.
- A conditional-handle value exposed into a strong root after capture survives even
  when its original owner is reclaimed.
- Nested begin, foreign-context finish, intervening collection/release and duplicate
  finish reject. An asynchronously discovered malformed array fails before sweep;
  repair and subsequent synchronous collection succeed.
- Existing conditional/parked/resource/fault, dense-index, intern-seed/stale-mark and
  142,272 scan cases run through immediate begin/finish adapters while retaining
  their original assertions. Immediate adapters establish unchanged-cycle behavior;
  the delayed fixture covers actual mutations between capture and finish.

The first run stopped at an unused adapter warning in the dense fixture. The fixed
adapter and all subsequent reports are retained. The host's ThreadSanitizer and
AddressSanitizer startup failures are documented in the shadow experiment; neither
is claimed as passing here. Shared heaps and Windows are unqualified.

## Performance experiment

`benchmark.c` runs a standalone native heap workload with a 4,096-object live chain,
64 allocations per cycle, one changing reference and 10,000 scalar iterations of
mutator work. Each process performs 10,000 cycles. Forward/reverse synchronous and
concurrent trials use the same compiled runtime, workload checksum and final graph
assertions. Process CPU includes the marker thread; the wall interval includes
worker creation but excludes setup and final teardown. Peak arena extent excludes
snapshot/index/queue memory and is not RSS.

The initial 200-cycle runs are retained separately because their intervals were
short. `measurement.json.gz` contains the longer comparison. This workload directly
exercises native heap/collector behavior; it is not a managed JIT/AOT or HTTP
throughput result. Whole-heap copies, bitmap/index allocations, binary-search
background lookup and an additional index/remark pass can cost more CPU than the
synchronous collector. A functional concurrent cycle is not evidence that this
architecture should become the default.

The longer paired runs use **3.69× process CPU** and **3.21× wall time**
for the concurrent snapshot candidate. These results argue against adopting full
snapshot/compare as the performance architecture. Retain it as a reclamation
correctness reference; pursue a cheaper protocol before broad HTTP integration.
Raw pairs and arena extents are in `summary.json` and `measurement.json.gz`.

## Remaining integration

Before general execution, automatic pacing must distinguish begin from finish;
explicit/idle/pressure collections and context release must complete or cancel an
outstanding epoch safely. Allocation failure needs a synchronous recovery policy.
The diagnostic allocation-abort policy is not a production out-of-memory contract.
Lifecycle admission must explicitly exclude shared-worker heaps until qualified.

The desired low-CPU architecture still needs mutation/read-path work. Whole-heap
copy/comparison is a correctness reference for replacing those costs with precise
layouts and barriers or another measured read-safe protocol. It does not justify
reading conservative scalar words concurrently with ordinary C writes. The full
managed standalone/HTTP matrix, pause distributions, peer comparisons and default
selection remain open under the original author-directed goal.

```sh
python3 benchmarks/execution/concurrent-remark/validate.py --output target/remark-new/validation
python3 benchmarks/execution/concurrent-remark/measure.py --output target/remark-new/measurement
```
