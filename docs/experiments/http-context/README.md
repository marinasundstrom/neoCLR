# HTTP context lifecycle checkpoint

Development after Preview 9. Use the matching compiler/reference/runtime bundle:

```sh
python3 docs/experiments/http-context/verify.py \
  --toolchain-root /path/to/bundle --runner target/release/examples/measure_async
python3 docs/experiments/http-context/verify-cancellation.py \
  --toolchain-root /path/to/bundle --runner target/release/examples/measure_async
```

`Sample.rvn` is compiled and executed by the lifecycle fixture. It propagates Accept
failure, configures a text response and returns the result of asynchronous Complete.
`HttpResponse.Respond` sets status/content; the matching context methods forward to
that message. Configuration never sends. Close/Dispose abandons unsent content;
Complete snapshots/sends and closes on success, failure or acknowledged cancellation.

The independent Python peers check two simultaneously owned requests completed in
reverse order, UTF-8 text and Content-Type, the sample's visible response, a status-only
204, abandonment, validation failure, duplicate completion, direct response configuration,
context forwards, Disposable dispatch, cancelled/error handlers, cancellation during a
handler wait, pre-cancellation, close during sending, pending-accept cancellation and
server shutdown with an owned context. A separate counter-based probe checks the
16-scope admission bound and cancellation cleanup without requiring a collection of
nested Task/Result types to cross the current importer profile.

The stress fixture uses a 1,024-object heap because it deliberately retains contexts
in an async state machine and opens 16 pending scopes. The earlier 256-object fixture
hit the declared heap quota; that was not an HTTP parse/transport error. Final lifecycle
validation on 2026-09-25: 3,110 allocations, 554 peak live objects, 27 collections and
zero live objects after completion. No full-platform suite or website build is run.

The cancellation fixture sends an incomplete POST and uses a separate control socket
to request cancellation or server shutdown. It checks task acknowledgement and peer
closure without using a sleep to decide when to cancel. Depending on scheduling,
cancellation can win while accept is still pending or while reading the request;
this does not claim to prove a particular native receive offset.

The assertion fixture consumes unit-valued Results explicitly. The frozen compiler's
discarded-unit propagation limitation is recorded in the
[integration notes](../../raven-backend-integration-map.md#http-context-and-configured-responses--2026-09-25).
The user-facing sample still uses propagation for Accept; Complete's Result is returned
directly. [Design, .NET comparison and limits](../../http-server-design.md#context-ownership-and-response-configuration--2026-09-25).

Final cancellation validation: both caller cancellation and server shutdown pass,
each with 194 allocations, peak 94, four collections and zero live objects. The final
probe uses explicit operation fields/method-group callbacks; captured-callback attempts
and their unresolved frozen-compiler/bridge failures are recorded in the integration
notes, rather than presented as fixed.

The existing `http-verbs` independent .NET/raw-peer regression also passes over the
shared context callback path: client 1,958 allocations / 44 collections / zero live;
server 1,175 allocations / peak 118 / 25 collections / zero live. Public lifecycle
signature/private-helper rejection checks, API snapshot and bootstrap fingerprints
pass. Website source/example content is updated; its build is skipped as directed.
