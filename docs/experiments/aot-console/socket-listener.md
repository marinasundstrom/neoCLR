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

## CIL accept bindings — 2026-10-08

`--bind-socket-accept` now admits the exact reserved SocketAccept(Int64, fn<Void>)
-> Value, SocketConnectResult(Int64) -> Value and SocketCancel(Int64) -> Boolean
InternalCall contracts. It requires --compile-system and --native-gc (and therefore
--reference-arena). Listen/LocalPort/Close still use --bind-socket-listener. Ordinary
managed bodies with these names are rejected, not replaced. Native root exhaustion
propagates NativeMemoryLimit rather than being collapsed to RuntimeError.

The compiled `socket-accept.neoil` consumer creates a receiver, binds its completion,
submits accept and saves the returned operation token in the receiver before returning
the port. The host connects, polls and dispatches; the compiled callback consumes the
result and closes the accepted socket. A second run cancels before polling. Sanitized
checks verify no remaining operations/roots, GC reclamation, retained receiver state,
listener-only ownership after completion and canaries. The focused negative test checks
missing/duplicate options, required GC and managed impostors for all three services.

Full Raven Server admission with the new flag clears these services and next rejects
SocketReceive. This is compiled CIL/native loopback evidence, not a completed Raven
HttpServer or a throughput comparison. TaskQueue integration and transfer services
remain necessary. [Admission evidence](../../../benchmarks/native-web/accept-admission.json).

## Buffered transfer kernel — 2026-10-08

The private GC adapter now also supports nonblocking receive/send and single-use
transfer results. Accept and transfers share 64 operation slots and one rotating poll
cursor. The private poll entry is now `neoclr_socket_poll_v1`; experimental hosts must
rebuild with the matching header. Each pending transfer has a five-second monotonic
deadline, matching the interpreter's ordinary transfer policy. Explicit request-deadline
services are not yet implemented. Empty transfers succeed with zero without I/O; EOF
receives succeed with zero; partial transfers report the actual byte count.

Send snapshots the initialized admitted range into native scratch before returning.
Receive retains its guest array through a strong root, reads into native scratch and
copies only successfully received bytes, setting reserved-array initialization markers
only for that range. Cancellation, expiry and close settle once and release scratch
and receive-buffer roots before callback delivery. Completed callbacks remain rooted
until result consumption or scope cleanup. Pending work has no background guest writes.

Scratch allocations use libc malloc/free with a 256 KiB aggregate per-scope budget;
arrays remain limited to 65,536 bytes. This is separate from the managed GC heap budget.
A native allocation failure is a recoverable SocketError.LimitExceeded; failure to retain
a GC root is a NativeMemoryLimit fault. One pending operation per socket/direction is
allowed, while receive and send can coexist. No raw guest pointer is passed to an
asynchronous OS operation. This follows the interpreter's snapshot and copy-back policy,
with explicit bounded costs rather than a claim of zero-copy or improved throughput.
The existing .NET socket comparison remains relevant; no public API changes occur here.

All seven native_gc tests pass with sanitizers. The new loopback transfer fixture checks
send snapshot stability after mutation/GC, partial receive data and initialization,
Busy/range/uninitialized errors, cancellation, forced deadline expiry, empty send,
EOF and fault teardown with pending receive. Compiled accept and existing listener
lifecycle tests also pass. These are kernel tests; CIL transfer binding and the real
HTTP application remain unfinished. Socket GC builds now also link text-arena.c for
initialized-byte validation.

## Compiled echo consumer — 2026-10-08

`--bind-socket-transfer` binds exact reserved Receive/Send/TransferResult InternalCalls
with native GC required. It retains arrayref<Byte>, offset/count and fn<Void> signature
checks; it does not admit arbitrary native methods with a matching name. The compiled
`socket-echo.neoil` stateful receiver submits accept, then receive, then send, consuming
each prior result during callback execution. Each successor operation roots its new
callback before the submitting guest call returns. The host only polls and dispatches.

`compiled_socket_echo_chains_accept_receive_send_and_reclaims_receivers` passes a
sanitized real one-byte loopback echo with collection between completions, exactly
three callback stages, guest socket close, cleared operations/scratch and an empty heap.
The focused binding test rejects missing/duplicate/GC-less options and managed impostors
for all three services. This is an executable CIL consumer, not yet the Raven Socket or
HttpServer public API. Full Server admission next reaches SocketDeadlineAfter;
[recorded admission](../../../benchmarks/native-web/transfer-admission.json).
