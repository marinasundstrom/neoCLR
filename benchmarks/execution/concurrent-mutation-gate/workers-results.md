# Registered worker mutations: integration prerequisite

2026-10-25. Add an explicitly selected multiwriter protocol to the experimental
mutation gate. The native concurrent collector still rejects shared-worker heaps.
This qualifies a prerequisite, not complete shared-heap GC or HTTP worker admission.

## Why this boundary is necessary

The existing worker pool publishes participant stack frames only while paused.
`compiled_collect_locked` holds the exclusive execution lease, gathers pool and
participant roots, and invokes synchronous `neoclr_gc_collect_participants_v1`.
The concurrent candidate instead has one owning mutator, owner-thread allocation
and reclamation, and a gate that rejects writes from other threads. Its inactive
write path is deliberately lock-free because only that owner changes epochs.
Removing the shared-heap compile rejection would bypass these assumptions.

The CLR comparison remains the background-collection design: mutators may run
alongside collection, with coordinated suspension at required phases. Its managed
write barriers use mechanisms such as card/write-watch bookkeeping, not a mutex
on every store. See the primary [.NET GC design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/garbage-collection.md)
and [write-barrier implementation notes](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/jit/GC-write-barriers.md).
This mutex-based prototype trades throughput for a simple correctness boundary;
it is not a claim to match the CLR algorithm or its performance. No .NET API,
managed layout, Raven emission or Runtime Contract changes.

## Implemented protocol

`NEOCLR_GATE_WORKERS` adds sixteen bounded registration slots. Every translation
unit using this private gate layout must select the option consistently. The owner registers
an already-created live thread during an active epoch, then publishes the returned
token through caller-provided synchronization. Authority binds gate identity, epoch,
slot generation and pthread identity. Duplicate registrations are rejected; exhausted
generations retire a slot rather than wrapping. Registration output changes only on
success. Native tokens are trusted internal capabilities, not protection against
arbitrary memory corruption or forged native structs.

Registered writes always acquire the gate and validate authority before touching
payload bytes. They use the same complete-transaction old/new lane logging and
all-or-nothing overflow behavior as owner writes. They never inherit the owner's
inactive fast path, registration/revocation rights or lifecycle authority. Revocation
waits for any in-gate transaction and makes subsequent use stale. Stop invalidates
all tokens; a racing transaction either finishes before stop or is rejected. A new
epoch cannot revive an old token. Cancellation forbids writes until the owner stops
and recovers the cycle; stop also invalidates registrations after overflow.

Registration does not authorize arbitrary heap pointers or publish roots. The caller
must still validate payload/patch lifetimes, register only live threads, revoke or
stop before those threads exit/reuse, and join all gate users before destroying the
gate or reclaiming payloads. Reader/writer synchronization covers only accesses that
use this gate. No unsynchronized managed read, allocation or sweep is admitted.

The original owner-only implementation remains verbatim behind the disabled-worker
control. Final Clang ARM64 disassembly is identical to the preceding commit. The
first helper refactor changed that assembly; its comparison is retained separately,
and the control was restored before qualification. The worker-enabled path intentionally
shares patch validation and transaction application between owner and registered
writes. This temporary control duplication should be revisited when the protocol
is integrated and its performance can be measured in actual worker workloads.

## Evidence and limitations

Both ordinary and UB/bounds-sanitized builds pass the original gate tests and the
new worker tests at 64- and 4,096-word capacities. Each new test runs 4,000 transactions
from four workers while a marker reader checks coherent payload pairs, then verifies
16,000 logged words with every new edge represented. It also checks revocation,
slot reuse, cross-thread and cross-gate token rejection, epoch restart, stop racing
a write, overflow cancellation without partial mutation, registration exhaustion
and generation saturation. Existing partial-lane and owner-only checks remain.

Deliberately disabling deletion logging or the writer-thread check makes the worker
test fail. These controls demonstrate sensitivity to those faults, not proof of all
possible interleavings. TSan compilation succeeds, but both this test and a minimal
pthread-only control exit with SIGSEGV before output. Preserve that as an unresolved
sanitizer/environment gap; do not claim TSan passed or attribute the failure to a
specific runtime cause. No new ASan qualification or application throughput result.

The default owner assembly is unchanged. Worker-enabled builds add bounded registry
storage and lock every registered write; contention and log-overflow rates still need
measurement with actual collector and application work. No allocation or CPU saving
is claimed. Existing HTTP and standalone throughput evidence is unaffected by the
disabled option and unchanged owner assembly; it does not qualify this new path.

## Remaining integration work

1. Capture participant roots while all relevant mutators are quiescent; do not retain
   mutable stack descriptors after resuming their owners. Establish stable root values
   and the rule for references introduced after the initial capture.
2. Bind writer registrations to pool/participant lifetime and collection epochs;
   cover native adapters and every admitted managed mutation, including grouped writes.
3. Coordinate allocation metadata and ownership while marking runs. Keep headers and
   payloads alive until the marker is joined, and handle overflow/cancellation before
   retrying writes or reclaiming objects.
4. Complete remark with a new participant rendezvous, trace current roots and logged
   values, reclaim under exclusive heap ownership, then revoke/release registrations.
5. Test real cross-worker object graphs, suspended tasks, failures and shutdown with
   JIT/AOT before enabling shared-heap admission. Then compare HTTP and standalone
   throughput, CPU, allocations, memory and pauses against the synchronous pool.

## Reproduction

```sh
python3 benchmarks/execution/concurrent-mutation-gate/validate.py \
  --workers --gate-words 64 --output target/gate-workers-qualified
python3 benchmarks/execution/concurrent-mutation-gate/validate.py \
  --workers --output target/gate-workers-capacity-large
```

Use new output directories when reproducing. `workers-validation.json.gz` and
`workers-large-validation.json.gz` retain successful commands, output and input
hashes; `workers-owner-validation.json.gz` is the disabled control. The large-capacity
run precedes only removal of a redundant preprocessor guard and a comment addition.
`workers-controls.json.gz` and its compressed driver retain final assembly comparison
and deliberate-fault checks; `workers-initial-controls.json.gz` records the rejected
assembly-changing refactor. `workers-tsan.json.gz` preserves the unresolved sanitizer
runs including the minimal control. None of these results marks concurrent GC as
default or the shared-worker integration as complete.
