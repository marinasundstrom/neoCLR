# HttpClient lifetime and timeouts — next work, 2026-10-10

The author requested configurable request timeouts and Disposable support immediately
after the encoding migration. This is the next bounded work, not implemented behavior.

## Current gaps

HttpClient has no Timeout or Dispose member. HttpExchange creates a single 15000 ms
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
