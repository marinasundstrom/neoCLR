# Delayed Copy through the VM and a Raven await

**2026-09-23 · Experimental M1/S0 checkpoint.** A controlled isolated worker
returns owned text. The real interpreter retains a completion receiver, posts it
to the default TaskQueue when the result is available, and resumes a Raven consumer
that encodes and copies `Hi!` into its captured managed array. This extends the
[real-heap fixture](../external-io-progress/GC-OWNERSHIP.md) into actual execution.
It is not a stream API or an OS I/O backend.

## The small product

[Copy.rvn](Copy.rvn), called by [Main.rvn](Main.rvn), allocates the three-byte
destination before awaiting. Its async state machine retains the array while Main creates 1,200 disposable arrays. The
pending worker notification retains the Promise and its continuation graph after
the initiating call returns. Main also posts unrelated ready work.

The [expected output](expected.txt) shows entry return, unrelated ready work, the
copied first byte (`72`, H), and the awaited byte count (`3`). The checked run reports
37 collections, 1,257 allocations and reclamations, and zero final live objects.
Those counts are observations of this toolchain, not a stable API or benchmark;
the verifier requires exact output and at least one real collection.

The producer may finish immediately. **Delivery** is deferred to the default queue,
including between callbacks. This sample does not use timing sleeps or claim to
simulate a slow socket. Direct-IL tests independently force each VM collection path and check
that pending receiver graphs survive before notification dispatch.

## Completion while the queue stays busy

[Busy.rvn](Busy.rvn) uses the same copy consumer and posts a small background
callback repeatedly until the consumer finishes. It prints the copied byte and
count, then `Background callbacks stopped`. Without polling between callbacks the
queue never empties, so completion cannot be delivered and the instruction budget
is exhausted. The verifier checks [the complete expected output](busy.expected.txt)
with no timing sleeps. The existing allocation-pressure sample remains separate.

This is cooperative progress, not preemptive scheduling or a latency guarantee.
At most one notification is appended per completed callback. A finite current batch
runs before newly posted work; the normal queue ordering and recursion checks remain.
The hook recognizes the System library's default Drain receiver and Func<Void>
invocation boundary, not merely an application method with a matching name.

## Selected layers and ownership

- The runtime adds experimental `neoCLR.Runtime.NotifyWorker(Int32, Func<Void>)`.
  Registration requires an existing default queue, a live unjoined handle, an exact
  callback signature and one notification per handle. Registered handles cannot be
  joined before notification dispatch. Existing submission limits remain 64 per
  invocation. Ordinary unregistered JoinWorker behavior is preserved.
- The invocation traces registered callbacks at both heap-pressure and array-budget
  collection points. Workers receive owned strings only, never a guest reference or
  native pointer into an array. A ready outcome remains cached until JoinWorker.
- After a callback returns to the real default TaskQueue.Drain, the interpreter
  polls registrations without blocking and appends one ready notification through
  TaskQueue.Post. Current-batch callbacks keep their order; notification joins the
  pending batch. Polling does not occur inside a callback or collection mutation.
  At queue quiescence, the interpreter scans all registrations and waits in bounded
  10 ms intervals when none are ready. A later ready worker can bypass an earlier
  pending worker. It transfers the callback directly into a traced TaskQueue.Post
  frame before the next collection point, then drains the default queue again.
- [Workers.rvn](Workers.rvn) is an **isolated experimental library adapter** using
  notification instead of posting a potentially blocking join. The verifier builds
  it with bootstrap service access and substitutes only its generated fragments in
  a temporary System library. The normal runtime library and public Thread.Start /
  ThreadPool.Queue implementations still use their original queued joins.
- The application uses ordinary Thread/Task/Promise contracts and normal core
  references. It never gains bootstrap RuntimeServices access. Its resumed Raven
  code performs UTF-8 encoding, checks capacity and writes the destination.

Invocation cancellation, Fault and instruction exhaustion use the existing worker
teardown: request cooperative cancellation and join producers before destroying the
registry. The cancellation test cancels after registration with a looping producer;
it does not establish a bounded shutdown guarantee for noncooperative native calls.
Worker failure becomes an invocation Fault when the callback joins it, as before.
There is no new Task error or per-operation cancellation protocol here.

## .NET comparison and provisional decision

Reuse the [primary-source .NET/Tokio comparison](../../http-poc-roadmap.md#evidence-comparisons-and-costs)
and [owned-buffer alternatives](../external-io-progress/README.md#comparison-and-provisional-result).
The .NET baseline is asynchronous memory-based read with completion and cancellation.
This experiment addresses neoCLR's missing invocation roots and completion delivery,
not a defect in CLR arrays or a replacement for .NET's I/O implementation.

Keeping queued joins is simpler but can block unrelated ready callbacks. Posting
only after a result is available demonstrates progress using the existing collector,
state machines and library queue. It adds retained receivers, a host result registry
and polling latency. The owned text boundary followed by encoding/copying costs
allocations and copies; no throughput or latency improvement is claimed. Direct
native writes/pinning would require a stronger lifetime and aliasing contract.
A new array representation or runtime suspension mechanism is not needed by this case.

The low-level notification service is provisional. Normal application APIs are
unchanged. Generalizing this adapter requires queue affinity and fairness decisions:
notifications dispatch only on the default queue. Reposting callbacks no longer
requires the queue to become empty for notification to progress. A callback that
never returns, a blocking host call or an exhausted instruction/allocation budget
can still prevent progress. Explicit TaskQueues are not supported by the adapter.
An unresolved Promise alone still does not keep an invocation alive. Result payloads have no new host-memory budget.

## Reproduce

Use a matching development bundle containing this revision's runtime, bridge and
System library, plus its Raven SDK. Published preview bundles lack this service.
The verifier creates its bootstrap core and experimental library in a temporary
directory; it does not modify the bundle or checked-in generated runtime snapshots.

```sh
python3 docs/experiments/delayed-copy/verify.py --toolchain-root /path/to/development-bundle
cargo test --lib --test workers --test tasks
```

Validated with the local Raven SDK at revision
`e56fc1ddf25d570d6aaa2bddfdf34138198f9941`. Rebuild the bridge and core together.
No Raven compiler or Runtime Contract configuration changed. See the
[integration note](../raven-target/README.md#experimental-worker-notification-2026-09-23).
The `.rvnproj` can build independently, but reproducing this notification experiment
requires the verifier's temporary library replacement; the normal worker library
still runs queued joins.

Ten worker integration tests cover both GC paths, ready callback order,
invalid/duplicate registration, invocation failure/budget exhaustion, producer failure
and cancellation after registration, alongside existing worker behavior. The busy
queue test runs a self-reposting callback until worker delivery stops it; an explicit
queue negative case exhausts its budget without running default notifications. Two
registry tests cover out-of-order readiness, one-shot join, pending-join rejection,
cancelled waiting and producer disconnection. All seven existing Task tests pass.

## Next bounded work

Keep S0 partial. Extend the real invocation evidence to cancellation/completion races,
explicit queue affinity and bounded result storage. Cooperative progress under
self-reposting ready work now has direct-IL and Raven evidence; broader scheduling
policy and preemption remain unselected.
Compare the application cancellation contract before generalizing the adapter or
introducing a file producer. The older controlled ownership fixture already tests
some terminal races, but those are not yet end-to-end VM/Task guarantees. Native
byte-buffer delivery, aliasing rules and noncooperative OS cancellation remain open.
