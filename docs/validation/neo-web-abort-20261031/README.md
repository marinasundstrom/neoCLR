# Request-abort cancellation qualification — 2026-10-31

Development work on macOS ARM64 with unified Raven main `813dad20c`, over neoCLR
`c151a229a` plus the source changes identified in this directory's evidence.
Neo.Web remains an experimental web framework built on top of
System.Web.Http.HttpServer. This is not Windows or TLS cancellation qualification.

## Contract

- Socket.WaitForReadClosure observes TCP receive EOF/error without consuming bytes.
  One observer per socket shares the existing 64-operation budget. Receives/sends
  may coexist; cancelling the observer preserves the socket. There is no implicit
  timeout. Unread bytes delay EOF detection, including queued TLS records.
- System.Web.Http.HttpContext.RequestAborted starts monitoring lazily. Observed
  closure/error, explicit Close and server shutdown cancel the request. Successful
  completion stops monitoring without cancellation. Admission failure closes the
  exchange instead of silently leaving it unmonitored. The token is read-only.
- Neo.Web uses the transport context callback, exposes the same RequestAborted
  signal, injects CancellationToken and supplies it to JSON body reads. It
  now starts with the transport’s empty response and no implicit Content-Type
  (previously an empty text/plain response). Explicit writers keep their types.
  Shutdown still joins cooperative handlers; it cannot force an ignoring handler to finish.
- Dynamic invocation accepts exact boxes of owned value-record arguments. This
  closes the CancellationToken marshalling gap without coercion, ref/out copyback
  or borrowed-view support. Raven compiler semantics/configuration are unchanged.
- WebDemo adds /wait-for-abort. Its HTTP consumer waits for entry, closes a real
  client socket and observes handler completion. Its logic test supplies an already
  cancelled token directly. The endpoint's result cannot reach a disconnected peer.

## Validation

The final execution report and mode logs are retained alongside this document.
All nine component/mode combinations pass in validation7:

| Consumer | Interpreter | AOT | JIT |
| --- | --- | --- | --- |
| Framework | 6/6 | 6/6 | 6/6 |
| WebDemo logic | 4/4 | 4/4 | 4/4 |
| WebDemo external HTTP | 30 responses + disconnect | 30 responses + disconnect | 30 responses + disconnect |

Framework checks cover peer closure while a handler awaits its token, normal
completion leaving the token uncancelled even through later server shutdown,
shared context/injected cancellation state, and joined shutdown of another active
handler. Existing routing, middleware, scopes and typed-result checks also run.

The socket unit suite has 39 passing tests, including nonconsumption, queued data
before EOF, concurrent receive, duplicate observation rejection, observer
cancellation and subsequent socket reuse. Dynamic invocation has 8 passing tests,
including exact record boxing, preserved payload and wrong-box rejection. The
host-I/O admission regression test passes. API snapshot checks pass (4 tests).

During development, a new handler type needed an explicit custom-attribute
retention entry. Initial interpreter runs also caught borrowed UTF-8/send-window
temporaries in the test spanning an await. Encoding and send submission now live
in synchronous helpers; Socket.Send snapshots the window before returning its
task. Those test corrections do not relax the borrowed-view lifetime contract.
The runtime invocation gap initially returned UnsupportedType (HTTP 500); the
owned-record argument change fixes it. Initial failed runs are superseded by the
final evidence, not counted as successful checks.

## Reproduction and provenance

Start with a matching qualified development bundle (this run used the preceding
binding slice's validation7 bundle, documented in
[its qualification](../neo-web-bindings-20261031/README.md)). Put the matching dotnet
host on PATH. The checked-in composer rebuilds Networking, System.Web and Neo.Web;
existing core, DI and other library binaries retain their prior provenance.

```sh
cargo build --release -p neoclr -p neoclr-codegen --bin neoclr --bin neoclr-codegen
python3 docs/validation/neo-web-abort-20261031/compose-input-bundle.py \
  --base-bundle /path/to/matching-bundle --output target/abort-inputs
python3 scripts/prepare-jit-bundle.py --bundle target/abort-inputs/bundle \
  --runtime target/release/neoclr --jit-compiler target/release/neoclr-codegen \
  --host-recipe /path/to/passing-native-http-build.json --output target/abort-jit
python3 scripts/validate-neo-web.py --bundle target/abort-jit/bundle \
  --output target/abort-check --codegen target/release/neoclr-codegen \
  --mode interpreter --mode native --mode jit
```

The native host must be rebuilt because the new reserved socket callback service
has a new C entry point. The runtime/codegen and Networking library must be upgraded
together. See [bridge encoding](../../raven-cli-bridge.md#request-read-closure-observation-2026-10-31-development)
and [design comparison](../../design/neo-web.md#request-abort-cancellation-2026-10-31-development).

The API audit uses matching compiled Networking/Web/Neo.Web assemblies and records
component hashes. System.Runtime's declaration/binary is reused; only the
FunctionReflection XML/Markdown remarks are refreshed from its updated source.
That documentation-only projection is separately identified in build evidence.
No new renderer exclusion is introduced. Website publication remains manual.

The final website build checks 6942 pages with 131 retained legacy pages. Existing
missing-description notices remain. It includes the Networking/HTTP/Neo.Web
boundary-review direction; publication was not performed.
