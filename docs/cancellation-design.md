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
