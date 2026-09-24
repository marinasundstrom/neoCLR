# Socket receive completion ownership

**2026-09-24 · Reduced host-side integration checkpoint, not a guest API.**

The echo case needs a pending receive to keep its destination alive, allow unrelated
work to progress, and deliver its result once. [probe.rs](probe.rs) combines real
loopback TCP with a bounded invocation-owned receive registry. It builds independently
of the initial transport experiment and changes no runtime or compiler contracts.

A pending entry owns a nonblocking stream, a host receive buffer and an invocation-local
`Rc<RefCell<Vec<u8>>>` destination. Only the owner thread reads and copies; native I/O
never receives a pointer into destination storage. Successful reads copy exactly the
reported count into the requested range and remove the operation before returning its
completion. Short reads and EOF remain distinct. An empty receive completes immediately.
Admission checks range overflow, operation count and total reserved receive bytes before
insertion. Retired IDs are never reused. A rejected admission consumes its supplied
stream owner; a future handle-based adapter must preserve its caller's socket instead.

Cancellation/close are serialized owner-thread actions in this model. If termination
wins before polling, no data is copied, even if the peer has already sent data. If
completion wins, a later cancellation has no effect. Removing a pending entry releases
its socket and buffer budget. Dropping the registry releases every pending owner.
This selects **no public cancellation policy**: cancellation here discards the owned
connection, whereas a future shared socket registry may preserve it. Peer transmission
is not undone by cancellation. There is no foreign-thread producer requiring a separate
quiescence acknowledgement.

## Comparison and alternatives

Reuse the .NET 10 ReceiveAsync and owned-buffer comparisons in the
[external I/O research](../external-io-progress/README.md#comparison-and-provisional-result).
The .NET cancellation/partial-read contract is the existing research baseline.
This probe adapts completion
ownership, not the public .NET API: Result-like terminal errors replace exceptions,
and an empty receive completes immediately rather than offering a readiness probe.

Owner-thread nonblocking polling avoids producer access to guest memory and makes the
completion/cancellation ordering explicit. It costs a host buffer and a delivery copy.
A worker/completion-channel backend would require acknowledgement before releasing
in-flight resources; readiness registration would avoid repeated polling but introduces
OS wakeup and teardown integration. Neither backend is selected here. There are no
performance claims or new scheduling abstractions.

## Reproduce and checked outcome

```sh
mkdir -p target/experiments/socket-completion
rustc --edition 2024 --test docs/experiments/socket-completion/probe.rs -o target/experiments/socket-completion/probe
target/experiments/socket-completion/probe
```

Six tests passed locally on Darwin arm64:

- Pending destination retention, exact short-read range and one terminal result.
- Cancellation and close before delivery leave destination contents unchanged.
- A later ready receive progresses while an earlier receive remains idle.
- Transactional admission limits, overflow checks and stale operation IDs.
- Empty receive and peer EOF complete without modifying the destination.
- Registry teardown releases the destination and produces peer EOF.

The five-second deadlines are test watchdogs. A 1 ms retry is only the test driver,
not a proposed event loop. This fixture does not test real guest GC, TaskQueue delivery,
managed fixed-length arrays, host cancellation of a running VM, accept/connect/send
completion, socket identity, concurrent receive exclusion across aliases, or portable
readiness registration. Destination alias rejection is deliberately conservative and
local to this model. `None` from poll means either pending or already retired; the
future Task adapter must distinguish those states using its operation registry.

Next integrate socket handles and delivery with the existing real-heap / delayed-copy
fixtures before exposing a usable Raven Socket API. The web-app and TCP echo milestones
remain open.

## Managed-heap follow-up

The [real-heap TCP fixture](GC-OWNERSHIP.md) now checks actual collector roots and
separates socket ownership from pending reads. Cancellation preserves the connection
in that fixture. The original reduced probe above remains evidence of the earlier
model; neither fixture exposes a public Socket API.
