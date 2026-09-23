# Pending reads and cancellation

**Development experiment, not a public Storage API.** A future asynchronous read
needs a clear boundary between requesting cancellation and knowing the producer
has stopped using its resources.

The current experiment uses ordinary Task/Promise and Raven await. A read owns its
buffer while pending. Queued guest callbacks represent producer events; they do
not perform native I/O or run on another thread.

| Ordered events | Consumer outcome |
| --- | --- |
| Request cancellation, then acknowledge it | Awaiting consumer is cancelled; buffer stays unchanged |
| Request cancellation, then deliver bytes | Completion wins; consumer receives the byte count |
| Deliver an oversized payload or report failure | Expected error; buffer stays unchanged |
| Deliver a short or empty payload | Actual byte count; remaining buffer range stays unchanged |
| Deliver another event after completion | No second write or notification |

A cancellation request leaves the Task pending. The modeled producer releases its
resource before completing or cancelling the Promise. Requesting closure follows
the same rule; it does not claim that a resource is already safe to release. An
unresponsive producer could leave the operation pending indefinitely.

The sample forces allocation pressure before terminal callbacks run and verifies
that actual managed collection retains the pending graph. After dispatch, the check
requires zero final live guest objects. Resource release is represented by a counter;
OS-handle lifetime and concurrent native cancellation are not tested here.

Download the [source and verifier](/samples/pending-read.zip). With matching development
artifacts, extract it and run:

```sh
python3 pending-read/verify.py --toolchain-root /path/to/development/bundle
```

Unlike [.NET Stream.ReadAsync](https://learn.microsoft.com/en-us/dotnet/api/system.io.stream.readasync?view=net-10.0),
this fixture keeps its destination private until completion. Caller-supplied buffers
remain an alternative, but their active range must not be reused while a producer
can still access it. The current type system does not prevent such aliases.

[Storage](storage-provider.md) and [streams](streams.md) remain synchronous.
Task-returning provider operations are future direction. This experiment does not
select public cancellation tokens, owned-buffer results or an async close method.

## Host-backed follow-up

A second development sample connects the private-buffer consumer to a real isolated
worker using the experimental notification adapter. The worker owns its text input
and result. The runtime transfers a retained callback into the default TaskQueue;
the adapter joins the finished producer before resuming the Raven consumer.

Its cancellation policy is deliberately narrow: a request before invocation-side
delivery leaves the Task pending, then discards the producer's result and cancels
the Task after acknowledgement. A request after delivery cannot replace completion.
It does not interrupt the worker; the producer may already have finished when the
request arrives. This tests a safe terminal boundary, not cancellation latency.

Five cases cover cancellation, completion, oversized, empty and short payloads.
An unrelated callback runs before consumer outcomes; GC runs while the pending
graph is retained, and the verifier checks zero final live guest objects. Producers
may complete in any order. Native producer Faults still terminate the invocation.

Download the [consumer sources](/samples/host-pending-read.zip). Reproducing the
notification path requires the shared experimental adapter harness in a matching
checkout, rather than an ordinary published SDK build:

```sh
python3 docs/experiments/delayed-copy/verify.py --toolchain-root /path/to/development/bundle --consumer-root docs/experiments/host-pending-read
```

No public API changes. Per-operation producer cancellation, explicit queues, native
byte payloads and async filesystem operations remain future work.

## Experimental per-worker cancellation services

Development runtime services now allow a low-level adapter to request cancellation
of one isolated job without cancelling its siblings. These are raw runtime services;
ordinary Raven Thread, Task and Storage signatures have not changed. They are not
included in the normal reference-core API selection.

| Runtime service | Contract |
| --- | --- |
| `neoCLR.Runtime.RequestWorkerCancellation(Int32) → Boolean` | First request for a known unjoined job returns true. Repeated requests and requests after join return false. Unknown/negative handles fault. True means a request was recorded, not that cancellation won. |
| `neoCLR.Runtime.JoinWorkerResult(Int32) → Value` | One terminal join returns an erased String for success or erased Void for acknowledged operation cancellation. Other producer failures remain Faults. A registered notification must have dispatched before joining. Repeated joins fault. |

Each job has its own cooperative stop token. The producer observes it at interpreter
instruction boundaries. A native call cannot be interrupted. Invocation teardown
still requests cancellation of every retained job and waits for producers to finish.
No shutdown deadline is promised.

A request retains the receiver and callback roots. `JoinWorkerResult` waits for the
producer result and dedicated-thread cleanup; pooled jobs release their per-job
producer state before publishing the result. The worker thread itself is reusable.
An already-produced success or unrelated Fault is preserved even if a request is
recorded before joining. Cancellation recognition checks the runtime Fault code,
not diagnostic text; guest code cannot manufacture that code.

Host invocation cancellation still terminates the invocation and takes precedence
at existing cancellation checks. The original `JoinWorker` keeps its String-or-Fault
contract, including cancellation Faults. Both new services require the existing
isolated-workers capability; JoinWorkerResult also requires value storage for its
erased result. The cancellation-aware join can block unless called
through the notification path after producer completion.

Direct-IL checks exercise dedicated and pooled jobs, sibling isolation, cancelled
notification delivery and retained managed state through GC. A Raven Task adapter
that maps the Void outcome into Promise.Cancel remains the next integration step.
There is still no public per-operation cancellation-token or async Storage API.
