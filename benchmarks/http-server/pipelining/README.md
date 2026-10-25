# HTTP connection reuse and pipelining investigation — 2026-10-25

The author requested this investigation after the bitmap-seeding GC screen.
This is a protocol/ownership baseline, not implemented persistent HTTP support.

## Executed baseline

`probe.py` runs retained single-owner sync and concurrent HTTP hosts. For each it
sends a keep-alive GET, then opens another connection and sends two GET requests
in one write. Both hosts return exactly one complete `Hello, World!` response with
Connection: close and EOF in each case; all four served requests validate and both
hosts shut down successfully. The second pipelined request is not served. The
initial invocation selected a missing sync artifact and is retained separately;
the completed report records actual executable hashes and all received bytes.

```sh
python3 benchmarks/http-server/pipelining/probe.py \
  --server target/concurrent-http-owners-plain/sync-1 \
  --server target/gc-index-only-owners/live-index-only-1 \
  --output target/pipelining-observation.json
```

The probe asserts today's close behavior. It must be replaced with positive reuse
and ordering qualification when support is implemented; passing it does not mean
pipelining is implemented. The retained sync executable is a protocol control, not
a fresh comparative performance build.

## .NET and protocol reference

Primary sources retrieved 2026-10-25:

- [.NET 10 Kestrel HttpProtocol](https://raw.githubusercontent.com/dotnet/aspnetcore/v10.0.0/src/Servers/Kestrel/Core/src/Internal/Http/HttpProtocol.cs): HTTP/1 state belongs to a connection and is reset between requests; keep-alive begins enabled and parsing/error handling may disable it. This is an implementation reference, not a requirement to copy its classes or pipelines.
- [RFC 9112 §9.3 and §9.3.2](https://www.rfc-editor.org/rfc/rfc9112.html#section-9.3): persistence requires unambiguous message framing and consuming the request body or closing. Pipelined responses must preserve request order; parallel execution is optional. Sequential processing is therefore a viable first implementation.
- [TechEmpower comparison boundary](../../gc-suite/techempower-comparison.md): keep these new workloads separate from the current close-per-request GC series.

## Current ownership seams

In `runtime/raven/src/System/Web/Http/HttpServer.rvn`:

- `HttpRequestDecoder.PushCore` records `Consumed`, so the parser already exposes a
  message boundary. It need not discard or reinterpret the next request's bytes.
- `HttpServerExchange.OnReceived` transfers read-ahead to `HttpBodyInput` for streamed
  bodies, but otherwise completes the first context and releases receive storage.
- `HttpContext.Close` disposes both messages and closes the transport. `Finish`
  always calls it, including after successful response transmission.
- The response encoder unconditionally emits Connection: close.

`HttpBodyInput` has private position/extent/remaining state, and its public `Close`
closes the connection even when `ownsConnection` is false. The flag currently only
controls automatic closure when body consumption completes. Persistence needs a
separate completed-body handoff; changing this flag alone would be insufficient.

## Proposed first implementation

A private session owns the connection, unread input window and terminal state.
Each exchange borrows that transport. After response completion and handler/body
completion, retire request-local content and callback roots, then create the next
exchange from the unread window. Keep at most one active handler/response per
connection initially; ordered completion avoids a response reorder queue and its
retained object graphs. Other connections remain independently scheduled.

Do not merely remove the close header. Reuse must be conditional on valid framing,
complete body consumption and absence of close/error/cancellation. An unconsumed
body may safely force connection closure initially; bounded draining can be a
separate measured policy. Explicit server/context abort and transport disposal
must remain idempotent. Preserve HTTP/1.0 limitations until explicitly qualified.

Suspend an idle session without retaining the previous request/response graph.
The session/read buffer must remain rooted while receives are pending under both
collectors. Do not move live managed sessions between isolated owner heaps.
Connection and request admission limits need distinct accounting; a quiet client
must not monopolize all Serve lanes indefinitely.

## Qualification before throughput

1. Two sequential requests on one connection and two coalesced pipelined requests;
   split every header boundary and preserve a partial next request.
2. Ordered distinguishable responses, including delayed handlers and errors.
3. Content-Length/chunked body followed immediately by the next request; unread
   body, malformed framing, explicit close and incomplete EOF cases.
4. Request cancellation, idle timeout, server shutdown, late handler completion and
   repeated disposal, with no callbacks accessing retired exchange state.
5. Force GC between messages and during suspended reads; verify old graph cleanup
   and live session survival in JIT/AOT, sync/concurrent modes.
6. Measure close, sequential keep-alive and pipelined traffic separately with matched
   Kestrel peers, checked response framing/order and client saturation reporting.

No public API signature, compiler bridge or wire protocol changes are made here.

## Implemented read-ahead foundation

`HttpConnection` can now privately retain an unread input window, consume it before
transport reads, and release it when drained or closed. Transfer retains the original
array without a copy; the caller must relinquish writing/pooling it. The helper is
allocated only for nonempty read-ahead and reused after draining. Invalid ranges,
replacement of unread bytes and destination aliasing fail without consuming input.
Buffered reads preserve cancellation and transport deadlines. Connection close is
idempotent and clears retained input.

The server exchange does not yet call this path. Body/exchange/session ownership
and response completion still need integration before keep-alive or pipelining can
be enabled. This is an internal prerequisite, with no throughput improvement claim.

### Validation

`ReadAhead.rvn` compiles the exact private sources without friend grants. It checks
45 input windows, partial and zero-length reads, invalid ranges, alias rejection,
replacement rejection, reuse and repeated close. A helper returns before collection
so the input wrapper is the sole holder of its source array. A real connection test
checks buffered completion, cancellation without consumption, and close behavior.
Explicit task observation in that test asserts immediate buffered completion.

All five runs in `read-ahead-results.json.gz` pass: interpreter, synchronous AOT/JIT,
and concurrent AOT/JIT. Interpreter and synchronous tests force collection between
reads; concurrent tests use automatic 16-allocation/64-publication pacing and each
complete 40 epochs with zero invalid-state reports. Explicit GC.Collect is outside
this concurrent test compiler's admission boundary, so its generated fixture omits
only those calls. This does not establish every possible concurrent interleaving.

Reproduce with the retained toolchain and native recipes documented by the harness:

```sh
python3 benchmarks/http-server/pipelining/validate-read-ahead.py \
  --output target/http-read-ahead-qualification-new
```

This developer harness requires existing `target/gc-patterns-standard-build`,
`target/gc-suite-standard-v1`, `target/gc-index-only-paired`, the retained bundle
and qualified native test runner. It is not a clean-checkout bootstrap command.
Next qualify framing/session ownership before changing public completion behavior.


## Ownership split after author clarification

The author explicitly permits architectural changes to server/client APIs and the
Rust runtime during experimental preview (2026-10-25). Compatibility is not a
reason to preserve the current exchange/connection coupling. The following is the
implementation direction, not an already supported public API:

- Transport owns the socket/TLS state, deadlines, pending I/O and unread bytes.
  Keep it shared between HTTP client and server, on one managed owner heap.
- A protocol session owns connection admission and HTTP/1 framing across messages.
  It runs one exchange at a time initially and decides reuse only after both input
  framing and response completion succeed. EOF-delimited bodies cannot be reused.
- An exchange owns request/response content and handler completion. Successful
  completion retires its graph; abort terminates its session. Message disposal
  must not implicitly terminate a successfully detached transport.

This follows the Kestrel connection/protocol/request separation described above,
without importing framework middleware or requiring parallel handlers on one
connection. The current `Accept` creates an exchange by accepting a new socket;
continuing to use that operation for each message would repeat admission and TLS
work. Split socket acceptance from reading the next request. The current fixed 16
Serve lanes also conflate active exchanges and connections: merely looping within
a lane would let 16 idle persistent clients prevent all new acceptance. Connection
limits, active-handler limits, idle timeouts and readiness scheduling must be
separate, with bounded pending work. A Rust readiness/continuation change is
justified if the existing socket/task primitives cannot represent that separation;
it must not migrate managed references between isolated heaps.

For the client, distinguish disposing response content from returning a verified
fully consumed connection to a pool. Do not share a connection across overlapping
requests initially. Pool key, TLS identity and expiry policy remain separate design
work; the body handoff alone does not establish client pooling.

The first implemented seam is `HttpBodyInput.TryReleaseForReuse`: only complete,
self-delimited, nonbusy, non-owning bodies may transfer trailing bytes to the
connection. Failed transfers preserve body ownership; subsequent ordinary Close
still aborts it. Successful transfer makes later body disposal harmless to the
transport. No server/client session calls this method yet.


The expanded `body-handoff-results.json.gz` fixture passes all five execution
configurations with Content-Length and chunked input followed by `NEXT`, premature
handoff rejection, owning-body rejection, retained-input collision/retry, repeated
handoff rejection and disposal after transfer. Native concurrent runs each report
45 candidate starts, 44 completions, one cancellation and zero invalid states;
these are safety checks, not timing comparisons. Source/compiler hashes and exact
commands are archived. The earlier v2 development run overlapped fixture edits;
v3 is the authoritative fixed-source result. Initial compilation used an unsupported
Encoding.UTF8 spelling, corrected to explicit wire bytes before qualification.


## Input session implementation

`HttpRequestSession` now separates socket acceptance from reading successive HTTP/1
messages on an existing transport. It owns one active request/body, rejects overlap,
and retires that graph only after complete body framing. An unread body closes the
session instead of becoming the next request. The existing decoder was moved,
without changing its body, into `HttpRequestDecoder.rvn`.

A session takes the entire retained window directly, including its cursor, before
parsing. This avoids copying a prefix into a 4 KiB receive buffer while leaving a
second unread tail in the transport. Subsequent network reads use bounded storage.
No global input caching or managed-owner migration is introduced.

Task completion observers are queued in neoCLR. The first test exposed a completed
read still marked busy; the session now observes completed tasks synchronously when
needed and checks observer identity before clearing state. An old queued observer
must not clear a newer read. The fixture compiles System.Web sources without its
binary reference to avoid mixing source and imported identities; the separate
System.Web library build also validates the extracted decoder and session.

This remains an **input** session. Its driver must apply Connection: close policy,
write responses in order, and call Retire only after successful response completion.
It is not yet wired into HttpServer/HttpClient. EOF currently returns the existing
incomplete-request error; distinguish idle orderly EOF when integrating driver
accounting. Acceptance/readiness scheduling and independent admission limits remain
required before enabling persistent connections. No HTTP throughput gain is claimed.


Qualification: `session-input-results.json.gz` passes interpreter and sync/concurrent
AOT/JIT. In addition to the preceding buffer/body checks, each run decodes 128
alternating `/one` and `/two` requests from a retained window over 4 KiB, rejects
reads before retirement, checks repeated retirement/close, and exercises consumed
and unread POST bodies followed immediately by another request. Forced collections
occur between messages in interpreter/sync; concurrent automatic pacing completes
1,335 epochs in each native mode, with no invalid-state reports. These immediate
buffered tests do not qualify suspended network reads, transport cancellation,
response ordering, fairness or throughput. Existing source-hash/recipe requirements
apply. The decoder body is unchanged; trailing blank lines were removed after qualification to satisfy the staged whitespace check. The archive retains the measured source hash.
