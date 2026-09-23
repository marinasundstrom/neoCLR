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
