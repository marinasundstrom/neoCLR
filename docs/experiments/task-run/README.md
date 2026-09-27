# Task.Run prerequisites — shared managed storage

**Development, 2026-09-27.** The public Task.Run API is not implemented by this
slice. The [concurrency design](../../concurrency-direction.md#taskrun-with-shared-captures--author-direction-2026-09-27)
owns its selected sharing/progress contract, .NET comparison and future green-thread
direction. [Runtime tracking](../../tracking/runtime-language.md#author-selected-taskrun-work--2026-09-27)
owns implementation status.

## Implemented boundary

Managed storage changes from Rc/RefCell to Arc and a mutex per slot. Cloning a
reference retains location identity; heap references remain weak capabilities whose
lifetime is governed by tracing. Storage access and its write counters/payload cache
are protected together. A whole-value read returns a coherent copy. A guest
read/modify/write sequence is still multiple operations, not an atomic increment.
No unsafe Send/Sync implementations are introduced.

Debugger snapshots copy the slot payload before following its references, avoiding
holding storage locks during graph traversal. Existing frame-escape, readonly,
construction and output-assignment rules remain enforced. Native tests access the
same object through interior aliases on two threads; no serialization or object
copy is used to substitute for sharing.

This is a storage prerequisite only. The interpreter still runs one execution per
invocation. The coordination prerequisite below now protects its heap access, but
it does not yet interleave multiple guest VMs. The new
native tests keep the heap alive while accessing it, and only collect after the
threads join. They **do not** establish safe concurrent collection. Promise,
TaskQueue, general collections and host-resource ownership still need their own
concurrency contracts and implementation. Arc alone does not supply GC roots.

.NET's shared-object Task.Run behavior motivates preserving identity, but this
per-slot lock is a neoCLR implementation choice, not a claim about CLR internals.
It trades atomic reference-count operations and lock overhead for safe native
storage access. A purpose-built atomic/borrow protocol could cost less but would
introduce more correctness obligations; it is not required for this slice.

## Focused validation

```sh
cargo test --lib slots::tests -- --test-threads=1
cargo test --test reference_slots --test heap_references --test delegates \
  --test debugger --test class_constructors --test runtime_gc
```

Results: 10 slot tests and 99 selected integration tests pass. Coverage includes
cross-thread alias mutation, whole-record replacement/read consistency, payload
cache invalidation, weak-handle expiration, captured receiver retention, explicit
GC, construction roots, frame provenance and debugger inspection. No full suite,
website build or Raven/API snapshot regeneration is needed for this internal
storage change; no Raven signatures changed.

## Single-thread cost check

[measure-storage.rs](measure-storage.rs) runs 200,000 class-field reads/writes with
local counter updates per observation. Module loading and verification occur
outside timing; each execution checks its result. This specifically exercises the
ordinary-code path affected by slot locks. It is not a Task.Run throughput test,
HTTP qualification or comparison with .NET performance.

Build the probe against separate baseline and candidate release libraries:

```sh
cargo build --release --lib
rustc --edition=2024 -O docs/experiments/task-run/measure-storage.rs \
  -L dependency=target/release/deps \
  --extern neoclr=target/release/libneoclr.rlib -o /absolute/path/to/runner
python3 docs/experiments/task-run/compare-storage.py \
  --baseline /absolute/path/to/baseline-runner \
  --candidate /absolute/path/to/candidate-runner \
  --baseline-commit 5522cf1a --output /absolute/path/to/results.json
```

On this machine, use SDKROOT pointing to the installed Xcode macOS 26.2 SDK if the
Command Line Tools SDK cannot be linked by the selected compiler. The checked-in
[observations](storage-cost.json) retain runner/source hashes, compiler/platform,
all observations and medians. The order is baseline/candidate/candidate/baseline,
with the first of six observations per process excluded as warmup. This is a small
local regression check with background machine activity, not a release threshold.
The recorded medians are 0.4331 seconds for the baseline and 0.4497 seconds for the
candidate (1.0383×, about 4% slower). This observed cost does not justify a separate
optimization project; repeat only if subsequent shared-execution changes or a
supported workload raise a material regression question.

## Shared heap/root coordination

The runtime now creates an invocation-owned heap coordinator. A participant has a
registered root snapshot and acquires exclusive graph access. Root publication,
new participant registration and collection happen under that same gate. Collection
combines the current participant's roots with every retained participant snapshot;
transitive references are still traced by the existing collector. Registered roots
are invocation-local VM allocation IDs, not guest handles or a cross-heap import API.

Registration precedes transfer of queued captures. A completion can retain its
participant until the receiving owner publishes the result. Registrations are weak
in the coordinator, so dropping an abandoned submission never reacquires the gate
and cannot deadlock a submitter holding it. Expired entries are pruned on admission
and collection. Admission has an explicit participant bound; exporting the final
heap requires all participants to have been released. Invalid root publication
preserves the previous snapshot and does not partially sweep.

The VM's automatic, explicit and final collections now use this coordinator. The
current VM holds one access guard for its execution, with a one-participant budget;
this is deliberately **not** concurrent guest execution yet. Before native guest
callbacks can run, the interpreter must publish complete roots and release the
guard at appropriate safepoints and blocking boundaries. No GC can make progress
while a participant holds access, so holding that guard during a blocking host call
is not a viable final Task.Run scheduling policy. Slot locks alone remain insufficient.
All VM graph access must obey the gate; the internal ManagedHeap helper interface
must not be used to bypass registered-root collection.

Focused checks for this slice:

```sh
cargo test --lib shared_heap::tests
cargo test --test runtime_gc --test gc_diagnostics --test heap_references --test entry_results
```

Six coordinator tests and 25 integration tests pass. They cover queued capture
retention, native mutation/result handoff, exclusion of collection during mutation,
transitive graph retention, abandoned admission, invalid publication, bounded
participants and export ordering. Existing VM collection/entry-dispatch behavior
passes. No new per-instruction scheduling or slot operations were introduced; the
previous storage cost check is retained, not rerun as a scheduler benchmark.

## Next prerequisite

Establish bounded native submission, rooted completion ownership, cancellation and
join-before-disposal. Then connect guest execution safepoints and shared services,
and establish concurrent Promise/queue publication before exposing Run overloads.
Native blocking-work progress and async callback unwrapping need executable
acceptance at that later API boundary. Retaining every object until invocation exit
is not a substitute for bounded live-object accounting.
