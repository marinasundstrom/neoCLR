# Native listener lifecycle — work in progress

The HTTP app needs asynchronous sockets and callbacks. This first service slice
implements only Listen, LocalPort and Close through the ordinary Raven `Socket`
wrapper. The opt-in `--bind-socket-listener` requires `--compile-system` and
`--reference-arena`. Exact verified InternalCall contracts receive private bindings;
ordinary managed methods are compiled, not replaced. Accept/read/write, DNS, TLS and
task execution are not enabled by the flag.

## Resource ownership

The host supplies a stack-owned `neoclr_socket_scope`, enters it with the execution
context before entry, and leaves it after every success or fault return. Scope leave
closes outstanding descriptors, including those skipped by a guest fault. It must run
before resetting/reusing the context. Invalid/missing scopes return RuntimeError
without publishing an erased result. LIFO nested distinct contexts are supported on
one thread; same-context reentry, cross-thread scope use and handles surviving entry
are excluded. Scope structs must remain alive and unmodified until leave. Cleanup
reports status 3 if a close syscall fails; it still attempts every remaining close.

A thread-local scope chain holds no guest references. Each scope has 64 slots, matching
the current interpreter socket-resource limit. Process-wide monotonic positive Int64
identities avoid exposing/reusing OS descriptors and prevent stale/foreign handle
aliasing. Closed/foreign LocalPort yields SocketError.Closed; Close is idempotent.
This is private host storage and an OS resource budget, separate from the GC byte
budget. No finalizer or managed callback is registered and native services never collect.
Future pending operations will need explicit callback roots and cancellation/cleanup
rules before async services can be admitted.

## Behavior and costs

The adapter uses strict dotted-decimal IPv4, rejects leading zeros, validates ports
0..65535 and backlog 1..128, uses nonblocking close-on-exec sockets and maps failures
to the existing Byte SocketError codes. Address validation precedes range validation,
which precedes the scope resource limit, matching `src/socket_io.rs`. Success transports
Int64 handles, Int32 ports or Void through the private two-lane erased representation.
Only macOS ARM64 is executable-qualified in this slice. Other OS backends are unqualified.

.NET's [TcpListener.Stop](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.tcplistener.stop?view=net-10.0)
closes the listener and leaves accepted connections for callers to close separately
(primary documentation checked 2026-10-08). neoCLR likewise distinguishes listener
lifetime from future accepted connections. The bounded native host scope additionally
provides deterministic cleanup when guest fault propagation skips explicit Close.
Its benefit is a small ownership boundary with no GC finalization dependency; costs
include host discipline, fixed resource capacity and no resource escape across entry.
This is an experiment, not a claim to improve on .NET disposal/hosting or its full API.

Alternatives: raw file descriptors would allow stale/reused-handle aliasing; GC
finalizers would not give prompt OS resource cleanup; embedding a separate C HTTP
server would bypass the Raven HTTP implementation we want to measure. The selected
scope keeps the actual managed wrapper and existing error contracts in the path.
The interpreter already owns its socket registry and cleanup; no interpreter change
or new public library/bridge API is needed.

## Validation

[Checked-in Raven samples and reproduction](../../../benchmarks/native-web/README.md)
exercise ordinary Result patterns, invalid input, AddressInUse, local ports, repeated
Close, stale handles and rebind in both modes. A deliberate guest fault leaves one
listener for host cleanup and preserves interpreter code/message/stack output.
[Recorded evidence](../../../benchmarks/native-web/listener-validation.json) contains
commands, hashes, dependencies and results. Standalone images link only libSystem.

Sanitized C tests additionally cover missing/null scope arguments, nested/foreign
contexts, resource exhaustion, all 64 descriptors closed at scope exit, nonblocking
and close-on-exec flags. Generated-code tests cover missing host scope, normal close,
forgotten close and fault unwinding with GC frames. Negative binding tests retain
exact-contract and opt-in enforcement. This slice measures correctness, not speed;
server throughput benchmarks must wait for the remaining HTTP path.

## Native accept kernel — 2026-10-08

The GC build now has a private accept-operation adapter ahead of CIL binding. Submission
retains the verified fn<Void> callback with a strong host root and returns an opaque
Int64 operation token. Owner-thread polling performs nonblocking accept and publishes
one retained callback handle without invoking guest code. Result consumption releases
the root. A rotating cursor prevents a fixed slot from always being selected first;
this is not a fairness or latency guarantee for a complete scheduler.

There are 64 operation slots and 64 socket slots per invocation scope. Pending accepts
count toward admission's socket budget, and allocation checks again when adopting a
connection. A second pending accept on one listener yields Busy. Accepted sockets are
nonblocking/close-on-exec and remain independent of listener closure. Cancellation wins
only before an outcome is committed; close settles pending accepts as Closed. Accept
has no connect-style timeout, matching the current interpreter. Once polling delivers
an operation, it cannot deliver it again. Results require delivery and are consumed
once. Scope teardown releases abandoned roots and all sockets, including on faults.

The callback/operation state is private; the host scheduler owns dispatch and may later
replace callback invocation with activation resumption. Polling rejects an active guest
frame or pending fault for the context. Native callers must supply a verified callback;
this kernel is not a public untrusted-pointer ABI. Network errors remain erased Byte
SocketError outcomes; native misuse faults return status 3 without publishing output.

This follows the existing socket/.NET comparison and the
[runtime scheduling review](../../runtime-scheduling-design.md#cross-runtime-reassessment--author-direction-2026-10-08).
A nonblocking kernel plus explicit completion handoff allows the existing Task/Promise
surface to evolve independently of the suspension representation. It adds bounded root
and operation storage, linear slot scans and explicit cleanup; no throughput advantage
is claimed. Compiler bindings, queue pumping, receive/send and native HTTP remain open.

The sanitized `native_accept_defers_delivery_and_releases_completion_roots` test uses
real loopback TCP to check deferred and single delivery, GC retention, result consumption,
Busy/InvalidOperation outcomes, cancellation before/after completion, close with pending
accept, independent accepted-socket lifetime and abandoned-operation cleanup on fault.
The existing native listener lifecycle fixture passes with its GC dependencies linked;
the non-GC listener kernel also passes separately. GC-enabled users of socket-listener.c
must now link native-gc.c and root-probe.c even when only synchronous methods are used.
