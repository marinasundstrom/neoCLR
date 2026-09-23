# Pending read: request cancellation, then observe completion

Development experiment, 2026-09-23. This bounded M1 contract probe connects an
operation's cancellation request to existing Task/Promise, Raven await, TaskQueue
and managed GC. It adds no public API, native service, scheduler or filesystem I/O.

A read owns a four-byte array filled with 9 and accepts at most two UTF-8 bytes into
positions 1–2. The destination stays private while pending. A queued callback supplies
a controlled terminal producer event after the initiating function returns. The
consumer awaits Task<Result<int, string>>. Main creates 1,200 disposable arrays
before dispatch, exercising real GC while callbacks and suspended consumers retain
the pending graph. No sleeps choose the outcomes.

```sh
python3 docs/experiments/pending-read/verify.py --toolchain-root /path/to/development/bundle
```

## Observable contract

| Event | State and result |
| --- | --- |
| RequestCancel, including repeated requests | Records intent once; Task stays pending, resource stand-in stays retained |
| RequestClose while pending | Requests cancellation; it does not claim synchronous quiescence |
| Complete("Hi") after a request | Completion may win: destination becomes `[9,72,105,9]`, Task completes with Ok(2) |
| AcknowledgeCancel after a request | Destination unchanged; resource stand-in released, then Promise.Cancel propagates through await |
| Oversized payload or producer failure | Destination unchanged; Task completes with Error, rather than cancellation |
| Short or empty completion | Ok(1) or Ok(0); unused bytes remain unchanged |
| Duplicate or late terminal event | Rejected without a second write, release or consumer notification |

Release happens before terminal Task publication. Continuations are queued, not
invoked inside the producer callback. The actual Task and generated async state
machine provide cancellation propagation; the sample does not implement a second
Task scheduler. An unsolicited cancellation acknowledgement is rejected.

The verifier checks exact output from six scenarios, assertions inside the guest,
at least one actual collection, and zero final live guest objects. This proves
retention/reclamation for these reachable graphs, not every possible pending-root
path. The resource is a release counter, not an OS handle. Terminal events are
single-invocation callbacks, not concurrent producer threads. The existing
[host ownership probe](../external-io-progress/GC-OWNERSHIP.md) and
[VM worker notification sample](../delayed-copy/README.md) exercise different layers.
None of the three alone establishes end-to-end cancellable file I/O.

## .NET comparison and alternatives

Primary contracts reviewed 2026-09-23:

- [.NET cooperative cancellation](https://learn.microsoft.com/en-us/dotnet/standard/threading/cancellation-in-managed-threads)
  separates the request from how an operation responds. A request does not force a
  listener to stop. This distinction is retained here.
- [.NET 10 Stream.ReadAsync](https://learn.microsoft.com/en-us/dotnet/api/system.io.stream.readasync?view=net-10.0)
  offers Memory<byte>/CancellationToken and a ValueTask<int> result. Caller-owned
  memory is a useful existing API baseline; it requires a lifetime/aliasing contract.
  This probe instead hides its destination while pending, and uses neoCLR's existing
  Task cancellation state plus Result for expected errors. It does not establish
  .NET-equivalent token or stream behavior.

These are library/operation policies on existing state machines and managed objects;
no new CLI metadata, GC algorithm or runtime suspension mechanism is selected.
Canceling a Promise at request time would resume the consumer before the modeled
producer acknowledges release. That is unsafe as a general buffer-reuse signal.
The probe therefore keeps the Task pending until one terminal event, including when
closure is requested. An unresponsive producer can leave it pending indefinitely;
timeouts do not prove quiescence and no forced cancellation is provided.

| Buffer alternative | Benefit | Cost / open contract |
| --- | --- | --- |
| Owned result/private pending buffer (this probe) | No caller mutation while pending; simple publication point | Allocations/copies and a result ownership API; fixed private fixture is not a selected API |
| Caller-supplied byte array | Reuses storage, resembles existing InputStream and .NET read | Caller must not reuse/mutate the active range before terminal completion; ordinary aliases are not prevented by current types |
| Direct native access or pinning | May avoid a delivery copy | GC stability, lifetime, cross-thread access and acknowledged release need stronger guarantees; not implemented |

Keeping the synchronous Storage POC unchanged is preferable to hiding blocking I/O
inside a completed Task or prematurely choosing one buffer family. Future lookup,
listing and opening could return Task<Result<...>>, but operation cancellation must
have a defined terminal boundary. For pending stream reads, decide owned results
versus borrowed ranges before choosing signatures. A real close operation also needs
a policy for multiple pending operations, admission after closure and who observes
completion. The sample's RequestClose only models one operation.

## Next integration evidence

Carry this request/terminal distinction into a controlled host producer on the real
invocation/default-queue path. Retain roots until producer acknowledgement and
queued consumer ownership, including host cancellation/Fault teardown. Check late
completion, explicit queue affinity, bounded host payloads and disconnection. Use
owned host payloads first to avoid native pointers into guest arrays. Keep this
separate from choosing a production OS backend or public cancellation-token API.
