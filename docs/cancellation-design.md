# Cooperative cancellation — 2026-09-25 development slice

Networking needs a way to request that an operation stop without confusing that
request with completion. `System.Concurrency.CancellationTokenSource` owns the
authority to request cancellation. Its copyable `CancellationToken` exposes the
request state and callback registration. `CancellationRegistration.Dispose()`
removes a callback that has not started. These are development APIs, not Preview 9.

The initial implementation belongs to one invocation. It does not share guest
objects with isolated workers and makes no cross-thread safety guarantee. Register,
Cancel and Dispose must execute on that invocation. No scheduler object is exposed.

## Contract

- A default token and `CancellationToken.None` never request cancellation. Their
  registrations do not retain callbacks.
- Copies of a source token observe the same state. `CanBeCancelled` reports source
  association, including after source disposal; it does not promise future work.
- `Cancel()` sets the request flag before synchronously invoking callbacks in reverse
  registration order. Repeated requests do nothing. Registration after the request
  invokes the callback inline before returning an inert registration.
- Callbacks may register callbacks, remove pending callbacks or repeat Cancel.
  Disposing the source during a callback suppresses remaining unstarted callbacks.
- Registration disposal releases captured state and is idempotent. Source disposal
  releases registrations without requesting cancellation and is also idempotent.
- Token acquisition, registration and Cancel after source disposal fault as contract
  misuse. Existing tokens can still read the request state.
- Callback Faults follow invocation Fault semantics; there is no exception aggregation.

Cancellation does not itself transition a Task. The operation owns resource cleanup
and acknowledges cancellation through its Promise only when it has stopped using
resources. A completion already committed remains completed. Native operations must
retain buffers and handles until backend acknowledgement, even if the token is already
requested. HTTP/socket token overloads and native cancellation wiring are subsequent
work; the source alone does not cancel an existing socket operation.

## .NET comparison and choices

Primary sources reviewed 2026-09-25:
[cooperative cancellation](https://learn.microsoft.com/en-us/dotnet/standard/threading/cancellation-in-managed-threads),
[CancellationTokenSource](https://learn.microsoft.com/dotnet/api/system.threading.cancellationtokensource),
[Cancel documentation source](https://github.com/dotnet/dotnet-api-docs/blob/main/xml/System.Threading/CancellationTokenSource.xml),
and the [.NET cancellation framework design](https://devblogs.microsoft.com/dotnet/net-4-cancellation-framework/).

The library-level source/token split, synchronous LIFO callbacks, inline registration
after cancellation and disposable registrations follow .NET. The executable .NET 10
probe compares those observable rules. This is not a CLI primitive or a requirement
to reproduce .NET's scheduler. Raven emits ordinary classes, a struct and delegates;
the bridge validates their selected reference contracts and imports their bodies.

.NET supports concurrent cancellation/registration and stronger registration disposal
coordination. This first implementation deliberately does less: invocation ownership
avoids locks and blocking callback joins, but prevents sharing tokens between threads.
The registration is a reference handle rather than .NET's value registration; this
simplifies identity and idempotent removal, at an allocation cost. A linked callback
list avoids retaining removed captures; cancellation detaches the list before invoking
user code. Optional delegate storage uses `Option<Func<Void>>` because this runtime
does not currently provide nullable delegate values.

An alternative was to cancel a Task directly from every token callback. That would
report completion before native resource ownership ends. Another alternative was to
introduce a public scheduler first. Neither is needed to express request/acknowledgement;
future runtime suspension can change resumption while preserving this distinction.
Timers, linked sources, execution-context capture, cross-invocation tokens and callback
Fault recovery remain unimplemented. Revisit this ownership restriction before offering
any shared-thread execution API.

## Integration and evidence

The [focused fixture](experiments/cancellation/README.md) covers callback lifecycle,
token copies and arrays under GC, and request versus acknowledged Task cancellation.
The bridge admits exactly one private source reference in the token layout and admits
captured/array token addresses only as non-constructor token receivers. Internal source
and registration helpers remain inaccessible to application code. Runtime Contract
configuration and Raven compiler emission are unchanged.

The source, generated library fragments and API reference must be refreshed together.
The website build is skipped for this slice by author direction.


## Native operation acknowledgement — implemented 2026-09-25

The runtime-library bridge now admits private `SocketCancel(operation)` and
`DnsCancel(operation)` hooks returning Boolean. This extends the existing .NET
comparison above: the source/token API requests cooperation; the provider must
still finish operation ownership before completing a Task. These hooks are an
internal implementation mechanism, not a public handle-based cancellation API.
The managed DNS/socket adapters now call these hooks, including the development HTTP client.

Socket cancellation now covers pending connect, accept, send and receive. The
invocation owner performs nonblocking socket calls, so cancellation can release a
pending connection attempt and transfer buffers without waiting for another native
thread to stop touching guest memory. Cancelling accept preserves the listening
socket; cancelling a transfer preserves its connection and unread peer bytes.
Remaining connection attempts are discarded when cancellation wins.

`true` means cancellation committed the operation outcome. `false` means an outcome
was already committed, including a previous cancellation. Unknown, consumed or
other-invocation operation IDs fault as provider misuse. IDs are backend-specific;
providers must pair them with the corresponding DNS or socket hook. These private
integers are not cross-backend capability tokens. A cancelled outcome still retains
its callback and operation slot until the normal completion callback is delivered
and its result consumed. Cancellation neither invokes guest callbacks inline nor
returns a reusable admission slot prematurely. A later cancellation cannot replace
an owner-observed success/failure or close the successful connection it produced.

DNS is different: blocking host resolution owns only host data and a global capacity
permit. Cancellation commits a guest cancellation outcome and discards late delivery;
it does not stop the host resolver call. Its permit remains charged until that call
returns, even after guest result consumption or invocation teardown. This existing
bounded-detachment policy is preserved; cancellation does not mean all host work ended.

The managed integration checks pre-cancellation before admission, registers
only a valid admitted operation, preserves the native winning outcome, and disposes
the token registration before consuming the operation ID. It consumes cancelled results
through the normal provider callback before cancelling the Promise. HTTP must close
its owned connection before exposing terminal cancellation. No public scheduler or
runtime suspension is needed for these rules.

Validation uses owner-controlled states for pending connects so OS-dependent
immediate-versus-pending connect behavior does not decide the assertion. Tests cover
stream closure, abandoned address attempts, accept/listener reuse, callback/result
ordering, slot retention, transfer buffer preservation, DNS host-capacity retention,
completion races, foreign/stale IDs and exact native signatures/services. The
cancellation-name Rust selection passed 18 tests; the new native signature/service
check passed separately. The name filter also selected existing worker/string tests;
future runs should use the narrower socket/resolver module filters. No cross-platform
matrix or website build is part of this checkpoint.

## Managed networking tokens — 2026-09-25

Development DNS lookup and all connect, accept, receive and send overloads now accept
CancellationToken. Tokenless overloads forward None, preserving existing behavior.
The internal shared-deadline paths also accept tokens and now carry HTTP cancellation.
A pre-cancelled token takes precedence over argument validation. After admission,
only a native cancellation win leads to Promise.Cancel; a completed operation is
not relabelled because the token flag later changed. Registrations are removed before
consuming the native ID. Source disposal removes callbacks without cancelling I/O.

This uses the existing .NET cooperative-cancellation comparison and the
[ReceiveAsync ownership baseline](experiments/socket-api/README.md). The naming and
Result-based error channel remain neoCLR-specific; cancellation is a Task outcome.
No public scheduler or suspension protocol is added. Future suspension can replace
callbacks while retaining the request/acknowledgement boundary. Invocation-local
tokens still cannot coordinate across threads. The [focused managed fixture](experiments/network-cancellation/README.md)
checks the contract with loopback I/O and the matching compiler bridge.

The [HTTP integration](http-client-design.md#http-token-forwarding-and-getstring--2026-09-25)
now forwards tokens through handlers and all transport phases. It closes its connection
after child acknowledgement and before terminal cancellation. Custom handlers retain
responsibility for their own cancellation/cleanup; the client does not force completion.
