# TCP receive ownership in the managed heap

**2026-09-24 · Test-only integration checkpoint.**

[gc_probe.rs](gc_probe.rs) replaces the first probe's Rc-backed byte destination with
neoCLR's actual ManagedHeap, checked array slots and tracing collector. It combines
real loopback TCP reads with destination/callback roots. It reuses the existing
[managed-heap fixture](../external-io-progress/GC-OWNERSHIP.md) for construction of
arrays and delegate-shaped receivers; it does not execute those delegates.

Socket ownership now lives in a separate invocation-local registry. Pending reads
refer to a socket ID, so cancelling a read does not dispose the connection. Reads
use nonblocking sockets and host-owned buffers on the owner thread. Cancellation
before polling retires the operation without consuming socket bytes. A later read
can receive those bytes. Completion before cancellation wins and is delivered only
once. Close retires pending reads with Closed and releases the connection.

This refines the previous probe's cancellation behavior rather than establishing a
public cancellation contract. There is no concurrent native read requiring a stop
acknowledgement: polling, copy, cancellation and close are serialized. A threaded or
OS-submitted backend would need a different quiescence protocol.

## Checked boundaries

Pending entries trace destination and callback independently. Delivery removes the
pending entry and publishes the callback to a traced ready queue without a collection
point in between. Ready callbacks continue to retain their captured graphs until
released. A callback does not necessarily capture the destination, so neither pending
root can substitute for the other. Native I/O never writes directly into guest memory.

Admission validates heap ownership, byte-array shape, signed ranges, and the fixture's
heap-backed callback receiver. One read per socket is allowed. Pending plus ready
entries share a two-operation cap; finishing native I/O does not permit an unlimited
backlog of undispatched callbacks. Requested host receive storage has an eight-byte
aggregate cap, released on terminal completion. IDs do not recycle within this fixture.
Sockets have a separate two-owner cap. These tiny limits are test settings, not public
runtime defaults or total allocator budgets.

The adapter clones the destination array for checked replacement, as in the earlier
heap fixture. This has a copy/allocation cost and is not an optimized buffer writer.
Aliased application writes remain possible; the future API must document whether they
are permitted while a read is pending. The socket registry issues local IDs only:
production handles need protection against stale references across invocations, as the
existing file registry already provides. Callback signature/target binding also remains
part of the real VM bridge.

## Validation

```sh
cargo test --lib external_io_gc_probe
```

The socket cases cover:

- Real short TCP receive into the middle of a managed byte array after collection
  reclaims 100 unrelated arrays; ready callback roots survive a second collection.
- Cancellation before delivery preserves the socket and bytes for a subsequent read;
  stale cancellation cannot affect the new operation.
- Independent destination and callback roots, idempotent close, and one terminal result.
- Foreign-heap rejection, invalid ranges/callbacks, duplicate socket reads, and admission
  limits that include ready callbacks.
- A ready socket progressing while another remains idle, followed by teardown of both
  pending and ready roots and peer-observed EOF.

Checked locally on Darwin arm64: **13 passed, 0 failed** (the existing eight
managed-heap ownership cases and these five new cases). Test
watchdogs bound peer waits; the 1 ms retry is only a test driver. This is not TaskQueue
integration, a Raven sample, VM Fault/cancellation evidence, or production readiness
registration. Collection is explicitly invoked by the fixture rather than triggered
by executing guest instructions.

## Design comparison and next step

Reuse the [.NET receive and ownership comparison](README.md#comparison-and-alternatives)
and the prior heap research. Keeping a socket alive across a cancelled receive gives
callers a reusable connection, at the cost of separate socket and operation lifetimes.
Owner-thread polling makes the tested cancellation boundary simple; it does not select
a production polling scheduler or reproduce all .NET cancellation races.

The next bridge must trace these roots at both actual VM collection paths and transfer
completed callbacks into TaskQueue/Post frames, using the existing delayed-copy path
as the baseline. It must also distinguish stale operation IDs from Pending and preserve
connection ownership on admission failure. Public Socket contracts and a runnable Raven
echo remain unfinished.
