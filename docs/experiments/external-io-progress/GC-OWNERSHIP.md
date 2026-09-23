# Delayed Byte Copy: real managed-heap ownership

**2026-09-23 · Partial M1/S0 evidence.** This follow-up replaces the earlier probe's
Rc-backed destination with neoCLR's actual ManagedHeap, array objects, delegate
receiver graphs, slot writes and tracing collector. It is a test-only research
harness, **not the interpreter event loop**, a Task implementation or a public
I/O API. It does not complete the delayed guest-consumer checkpoint.

## The comprehensible case

A consumer requests a two-byte copy into the middle of a four-byte managed array.
The host producer will eventually provide `[4, 5]`. While it is pending, the caller
releases its local handles and unrelated allocation pressure triggers collection.
The pending operation must keep both the destination and completion consumer alive.

| Point in the checked case | Destination bytes | Heap evidence |
| --- | --- | --- |
| Register offset 1, count 2 | `[9, 9, 9, 9]` | Destination plus consumer receiver are rooted by the pending registry |
| Collect while host producer waits | `[9, 9, 9, 9]` | 100 unrelated allocations are reclaimed; the two required objects survive |
| Apply the host-owned payload on the invocation thread | `[9, 4, 5, 9]` | Short-range copy preserves array identity and untouched bytes |
| Collect before queued consumer dispatch | `[9, 4, 5, 9]` | The ready queue now retains the receiver and its captured array |
| Release the queued consumer and collect | No live destination | Both objects are reclaimed |

This is the lifetime problem behind a future Byte Copy stream read, independently
of sockets, HTTP or the JSON document model. A second case gives the receiver an
unrelated captured marker: the destination and callback are **independent pending
roots**, and only the receiver graph survives after terminal delivery.

## What is real, and what is modeled

[gc_probe.rs](gc_probe.rs) uses the runtime's real `ManagedHeap::allocate/collect`,
`gc::trace`, array ObjectReference representation, receiver traversal and checked
slot replacement. A negative control deliberately omits the roots and verifies that
heap lookup and a stale reference both fail. Rust handles do not secretly keep a
collected guest object accessible.

The `Invocation` registry, ready queue and producer protocol are local to the test.
The fixture internally constructs a delegate-shaped value to test the collector's
existing receiver traversal; it does not bind or execute that target. The consumer
receiver is a heap object with a captured array, not a generated Raven async state
machine or Task/Promise. Guest instruction execution, TaskQueue dispatch, callback
signature admission and VM safepoints are **not** exercised by these tests.

The fixture is included from `src/lib.rs` only under `cfg(test)` so it can access
private collector operations without adding public heap mutation APIs. Normal builds
have no new registry, service, behavior, flags or runtime dependency.

## Ownership decisions tested

- Producers own their byte payloads and a resource stand-in. Their thread captures
  contain no guest Values or references. Completion copies into guest storage only
  on the fixture's invocation thread, after validating the whole payload length.
- Registration checks actual heap membership, byte element type, null/type/range
  errors, a 16-byte requested-transfer limit and an eight-pending-operation cap.
  Equal allocation numbers from another heap are not valid ownership evidence.
- Cancellation is a request. Roots remain until a terminal acknowledgement after
  the producer has released its resource. A queued completion may win; both ordered
  outcomes are tested. Duplicate/late terminal events cannot publish twice or write.
- After delivery, the callback must belong to a traced ready owner before the next
  collection. Removing the pending record alone would create a root gap. In this
  fixture, delivery has no GC safepoint between removing it and publishing ready work.
- An oversized payload or explicit backend failure produces a failed outcome without
  changing the destination. A short or zero-byte payload is allowed. Zero here means
  an empty transfer, not a selected stream EOF contract.
- Shutdown asks all controlled producers to stop, joins them, then clears pending
  and ready roots. A bounded eight-event channel can hold the one terminal event
  from each of the at most eight producers. The one-command channel gives a queued
  completion precedence over a later cancellation request.

The fixture does not prevent caller aliases from writing the array while pending.
An ownership/aliasing contract for a public read API is still needed. It imposes no
production ready-queue budget; a long-running registry would need limits on queued
results as well as pending work. Host Vec allocation is outside guest array accounting.
The 16-byte application check does not establish a native allocator budget.

## .NET comparison and alternatives

Reuse [the host probe's primary-source comparison](README.md#comparison-and-provisional-result)
and [M1's .NET/Tokio research](../../http-poc-roadmap.md#evidence-comparisons-and-costs).
The application baseline remains an asynchronous read into memory with completion
and cancellation. This probe addresses the lifetime layer underneath that operation;
it does not reproduce Socket.ReceiveAsync, select an OS backend or find a CLR defect.

The owned host-payload boundary avoids a native pointer into guest storage and makes
terminal mutation happen in one place. Its costs include an extra copy, two sets of
buffer storage, retained receivers and explicit queue roots. This fixture additionally
clones the whole destination for checked replacement; that is not an efficient copy
implementation. Pinning/direct native writes remain an alternative requiring a
stronger GC, aliasing and cancellation-quiescence contract. No speedup is claimed.

The provisional result is that the existing collector can trace the required graphs
when given the correct roots. There is no evidence here requiring a new array type,
new GC algorithm, metadata feature or suspension mechanism. The critical integration
work is supplying those roots at every actual VM collection point and transferring
them into the real dispatch path without a gap.

## Reproduce and evidence

```sh
cargo test --lib external_io_gc_probe
```

Eight cases cover pending/ready ownership and reclamation, a missing-root negative
control, independently retained destination/receiver graphs, completion/cancellation
ordering and duplicates, oversize/error atomicity, range/type/cross-heap admission,
out-of-order terminal delivery, and eight-producer teardown. The original seven
host-event-loop checks remain separately reproducible as described in [README](README.md).
No sleeps control the outcomes; commands and acknowledgements set their ordering.
The five-second receive timeout is a test watchdog, not an I/O deadline.

## Integration follow-up

The [VM Delayed Copy checkpoint](../delayed-copy/README.md) now connects a reduced
worker producer, actual TaskQueue and Raven consumer. The original acceptance scope
below remains broader than that checkpoint; cancellation races and native payload
limits still need end-to-end evidence.

Integrate the full ownership protocol into the **real invocation**, including both allocation-pressure
and array-budget collection paths. Keep pending roots through completion application,
then retain the actual Promise/continuation in TaskQueue until dispatch. Demonstrate
empty-queue wakeup, unrelated guest progress, a runnable Raven delayed-copy consumer,
and direct-IL rejection of malformed buffers/handles before calling S0 complete.

Also test invocation Fault/cancellation and instruction-budget teardown, lost producers,
channel disconnection, host allocation/ready-queue budgets and noncooperative backends.
Here producers always cooperate, and shutdown cannot establish bounded OS cancellation.
The fixture's explicit failure command is not a producer-disconnection test. A real
callback frame must take ownership after dequeue; this harness stops at ready retention.
