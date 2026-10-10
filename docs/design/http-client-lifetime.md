# HttpClient lifetime and timeouts — development, 2026-10-10

The author requested configurable request timeouts and Disposable support immediately
after the encoding migration. Lifetime support is implemented in this slice; configurable timeout remains next.

## Current gaps

HttpClient has no Timeout member. HttpExchange creates a single 15000 ms
NetworkDeadline covering DNS, connection and transfers. Request cancellation is
supported; built-in exchanges close connections and owned upload streams on completion.
Custom handlers receive a cancellation token. CancellationTokenSource has no timer-based
CancelAfter operation. NetworkDeadline currently accepts only 1–60000 milliseconds.
The native HTTP correctness host also has a 15-second watchdog independent of requests.

## .NET baseline and proposed sequence

Primary contracts checked 2026-10-10:

- [.NET HttpClient.Timeout](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient.timeout?view=net-10.0)
  uses TimeSpan, defaults to 100 seconds, accepts the infinite sentinel, and cannot be
  changed after requests start or disposal. Request cancellation may shorten its budget.
- [.NET HttpClient constructors](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient.-ctor?view=net-10.0)
  own the supplied handler by default; the handler-plus-Boolean overload allows a
  borrowed handler. Clients are intended to be reused.
- [.NET HttpClient.Dispose](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient.dispose?view=net-10.0)
  establishes the familiar IDisposable lifetime surface. Examine the linked runtime
  implementation before defining pending-request cleanup and cancellation races.

First make client/handler ownership explicit, implement Disposable and exercise use,
repeated disposal, borrowed handlers, use after disposal and pending requests. Avoid
adding an inert Dispose method that leaves handler ownership or cancellation undefined.

Then add a Duration-shaped Timeout following the existing neoCLR time model, with a
single monotonic budget per request rather than renewing it between network phases.
Determine cancellation/timeout result distinctions and custom-handler behavior before
implementing the property. The existing 60-second network bound and host watchdog are
constraints to resolve, not reasons to claim a permanent divergence from .NET. A
transport-only setting must not be presented as timing out arbitrary handler pipelines.

## Required qualification

Use deterministic fake handlers for ownership and terminal-state cases, followed by a
controlled local peer for DNS/connect/transfer cancellation and deadline cleanup.
Verify request-timeout, caller-cancellation and client-disposal races finish once,
release registrations/connections and respect borrowed ownership. Run the relevant
interpreted/native consumers on macOS and in Windows CI; record unexecuted platforms
explicitly. Add API reference entries and update showcase authoring only after the
contracts are implemented and tested.

## Lifetime implementation

HttpClient and the abstract HttpHandler base implement Disposable. Default and handler constructors
own their handler; `(handler, disposeHandler: false)` borrows it. Dispose is idempotent,
marks the owner closed before callbacks, requests cancellation of its sends and disposes
an owned handler once. HttpSocketHandler independently cancels its pending exchanges.
Disposed sends fault through the current platform terminal-fault mechanism, including
precancelled sends. Configuration getters remain readable; configuration freezing is
not added in this lifetime slice.

Each send links caller and owner cancellation privately. A completion adapter releases
registrations before publishing its outcome and preserves response/request association.
It waits for handler acknowledgement rather than reporting cancellation before transport
cleanup. Custom handlers which ignore cancellation can remain pending. Completed
responses are caller-owned and survive client disposal. DelegatingHandler supplies forwarding and idempotent inner-handler ownership.
Its nullable InnerHandler property permits construction before assignment, but rejects
null assignments and freezes on first send or disposal. Middleware overrides Send
and normally awaits base.Send. HttpHandler is now an abstract class; existing custom
handlers migrate from interface implementation to override Send. Stateless handlers
inherit Dispose. This experimental API change requires recompilation.

This follows the ownership and cooperative cancellation structure in
[HttpClient.cs](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Net.Http/src/System/Net/Http/HttpClient.cs)
and [HttpMessageInvoker.cs](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Net.Http/src/System/Net/Http/HttpMessageInvoker.cs).
neoCLR currently signals use after disposal with a terminal runtime fault rather than
ObjectDisposedException. Cancellation remains invocation-local, not thread-safe. An
explicit continuation observes cancelled tasks because await cannot currently recover
that outcome. No inert disposal or runtime friend grant is introduced.

The author explicitly selected .NET's delegating-handler architecture during this
implementation. Compared with [DelegatingHandler.cs](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Net.Http/src/System/Net/Http/DelegatingHandler.cs),
the same owned chain and configuration freeze apply. Existing neoCLR names retain
HttpHandler and asynchronous Send; .NET names the base HttpMessageHandler and its
asynchronous method SendAsync. Dispose is directly virtual here because neoCLR has
no finalizer path requiring a disposing Boolean. This simplifies current resource
ownership but is not the complete .NET finalization pattern. Thread-safe disposal
and recoverable object-disposed exceptions remain gaps, not claimed improvements.

## Qualified foundation (2026-10-11)

The ten lifetime/middleware cases and eight guard cases pass in the interpreter and
macOS ARM64 AOT (`target/http-delegating-validation-v8/report.json` and
`target/http-delegating-faults-v3/validation.json`). The pipeline exposed and now uses
[external storage and class dispatch support](native-class-dispatch.md). General
async-base and overload-lookup fixes are on Raven main, with native-specific metadata
and emission on the integration branch. Windows CI is configured but unexecuted.
Timeout remains unimplemented; these results do not qualify a Timeout property.
