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
