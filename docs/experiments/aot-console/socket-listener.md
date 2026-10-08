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
