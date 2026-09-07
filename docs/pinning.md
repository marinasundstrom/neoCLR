# Managed heap pinning for native interop

Status: planned contract, not an implemented native-pointer bridge. The current
collector is nonmoving, but its managed value cells are not a native object ABI.

A pin should provide two guarantees until released: the complete heap allocation
remains reachable, and its native data address does not change. Pinning an interior
field must retain and immobilize the whole owner. Multiple pins compose: releasing
one cannot cancel the guarantees of the others. Releasing the last pin ends the
pinning guarantee; it does not free the object. Other managed references can keep
it alive, and GC determines later reclamation.

Prefer a scoped pin capability for synchronous native calls, with runtime cleanup
on every supported scope-exit path. Native code that retains the address beyond a
call requires an explicit longer-lived registration and release contract. Passing a
raw pointer alone must not silently register a root. Unpinning must occur only after
native users have finished; future async work requires a separate ownership protocol.

Only values with a supported, stable native representation can expose a direct
pointer. Managed reference fields cannot become arbitrary pointer-sized native slots;
strings and other managed representations may require marshalling or copied buffers.
A pin must not expose Rust enum/Vec/String internals as the guest ABI. Native writes
that could change managed references need an explicit barrier/validation contract.
An initial direct-access subset should exclude such writes.

Define how in-place value replacement behaves while pinned: it must preserve the
promised native address and layout or be rejected. Future inheritance must pin the
complete derived allocation even when requested through a base or field view.
Virtual dispatch and base projection must not change the registered native address.

Before exposing this feature, define registration, nested pins, failed acquisition,
Fault/cancellation cleanup, library lifetime, layout eligibility and native retention
rules. Monitoring should count active pins and pinned roots, show retained objects
and eventually report pin durations/bytes once those measurements are defined.
Pinning can constrain a future moving collector; callers should keep scopes as short
as their native-use contract permits.

This follows the current [managed-reference milestone](heap-references.md), with
integration during [object-model work](object-hierarchy.md). Existing heap.alloc/free
remains the independent unmanaged-storage option.

## Two-way interop remains an open design

Plan for calls from NeoCLR into native functions and native consumers of data or
objects supplied by NeoCLR. Direct pinned layouts, marshalled copies and opaque
managed-object handles are distinct possible contracts, not selected universal
representations. A native function might use data only during a call, retain it, or
call back into NeoCLR; each requires explicit rooting, thread/context and release
rules before being supported. Native ownership transfer must not be inferred merely
from receiving a pointer. Callback behavior and terminal Fault propagation also need
a boundary contract. These preview choices may evolve as real interop workloads
clarify requirements; the current memory milestone does not commit a bidirectional
object ABI or expose managed object addresses.
