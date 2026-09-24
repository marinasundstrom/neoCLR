# TCP completion through the interpreter

**2026-09-24 · Test-only socket adapter; real VM execution and TaskQueue dispatch.**

[vm_probe.rs](vm_probe.rs) supplies an already-connected loopback stream to a private
unit-test adapter. A guest byte-array reference and completion delegate register one
pending receive. Native I/O reads into owned host bytes. The invocation copies the
received range, transfers the delegate into a real TaskQueue.Post frame, and executes
the callback through the generated System library. The callback checks the received
byte and untouched prefix before printing `copied`.

This advances beyond the earlier [collector fixture](GC-OWNERSHIP.md): guest instructions
now trigger both real VM collection paths, and callbacks execute rather than merely
sitting in a modeled ready queue. The entry point returns while work remains pending.

## Implementation boundary

The socket adapter, injected stream and exact InternalCall binding exist only in
`cfg(test)` library builds. Normal runtime builds cannot bind TestSocketReceive and
contain no socket polling branch. Injection is thread-local so independent tests do
not share streams. No Raven reference metadata, application Socket API, production
socket dependency or SDK artifact is added.

The [private scheduler](../../runtime-scheduling-design.md#initial-native-host-driver--implemented-2026-09-24)
now owns source arbitration and waiting. Idle and callback-return polling use the same
rotating source policy. Workers signal a durable wake latch after publishing their
outcome; cancellation and the socket probe retain a 10 ms polling fallback. Existing
worker notification behavior is preserved. This is not a public scheduler abstraction.

The test adapter traces destination and callback at both array-budget recovery and
heap-allocation-pressure collection. It polls at queue quiescence and at the existing
safe callback-return boundary. There is no collection point between removing its
pending owner and creating the TaskQueue.Post frame. Pending socket ownership drops
on VM exit, including guest Fault and host cancellation.

The adapter admits one receive of at most eight bytes, requires the default dispatcher,
and validates signed ranges and heap-backed byte storage before taking the injected
socket. Its delegate must have Func<Void> type; ordinary VM delegate binding resolves
the target. Native errors become invocation Faults in this test seam; that is **not**
the proposed public SocketError/Result contract. Completion receives no byte count and
releases the injected connection. General socket handles, per-operation cancellation,
reusable connections and byte-count results remain with the production bridge work.

## Validation

```sh
cargo test --lib socket_vm_probe
cargo test --lib workers::tests
cargo test --test workers
```

Checked on Darwin arm64: **5 VM socket tests, 10 worker-registry tests and 16 worker integration tests pass**.
The combined website build validates all 523 pages.

The fixture checks actual TCP input with unrelated ready work, collection of 100
discarded arrays under each VM budget, exact range copy, callback execution and zero
final live objects. A separate case keeps the queue busy by reposting a callback
until data is visible: completion must be polled between callbacks. Other cases check
Fault/cancellation resource release and invalid-range/missing-dispatcher rejection
before socket injection is consumed. A mixed-source case keeps a worker running
until a real socket callback signals completion, then cancels the invocation.

A console signal releases the peer after guest work has run; no sleep chooses that
ordering. Socket read/write timeouts are test watchdogs. The peer starts its I/O watchdog after the guest
signal, so compiler/library loading time is not mistaken for stalled I/O. The test-only
socket source uses the scheduler’s 10 ms polling fallback; it has no OS readiness
registration or selected production transport backend. The fixture does not prove that every receive is still
pending at the first empty-queue poll, nor a bounded scheduling latency.

## Comparison and remaining work

Reuse the [.NET receive and ownership research](README.md#comparison-and-alternatives).
The selected copy boundary and callback retention follow the existing delayed-copy
integration; the new evidence is real TCP data crossing that boundary during guest
execution. Copying the full destination for checked replacement remains inefficient.
Direct native writes require a stronger aliasing/pinning contract and are not selected.

The next production bridge must provide addressing/socket/error metadata, a reusable
socket registry, bounded pending and ready outcomes, Task/Result completion, and
cancellation semantics. It must also test broader mixed-source fairness and resource teardown under
instruction exhaustion, stalled peers and native errors. A Raven echo
application and API-reference refresh are required before presenting Socket as usable.
