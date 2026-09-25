# Buffered HTTP POST checkpoint

Development after Preview 9. Build a matching SDK/reference/library bundle, then run:

```sh
python3 docs/experiments/http-post/verify.py \
  --toolchain-root /path/to/bundle \
  --runner target/release/examples/measure_async
```

`Sample.rvn` is the application example: configure BaseUri, send UTF-8 text, propagate
transport errors, require a successful status and read the response. ReadText currently
returns a string error, adapted explicitly at this boundary. For a larger application's
shared error type, use deliberate nearby conversion declarations as described in
[the Raven conventions](../../raven-conventions.md#propagating-errors-through-conversions).
`SampleChecks.rvn` keeps explicit success/error/cancellation assertions out of that example.
The verifier substitutes ports into Client/Server sources; those templates are not
standalone applications until prepared by the verifier.

The client checks fake-handler body/header/token forwarding, pre-cancellation before
parsing, oversized-body rejection and Content-Type injection rejection before I/O.
An independent Python peer validates POST byte framing, UTF-8, binary and empty bodies,
plus GET regression and connection closure. A neoCLR server then echoes those requests,
accepts .NET HttpClient POSTs and handles a fragmented binary request. Eight malformed
or unsupported requests are rejected without invoking the application callback:
GET body, duplicate/negative/oversized lengths, transfer encoding, premature EOF,
Expect and content encoding. Run timing-sensitive exchanges serially.

Validation on 2026-09-25: the full interoperability/framing matrix passed, with zero
live managed objects after each run. The propagation example's normal/error/invalid
UTF-8/cancelled checks also pass. `--client-only` reruns these sample assertions and
the independent client peer without repeating the server matrix.

The provider buffers at most 1,024 request bytes and 2,048 header bytes. Missing request
Content-Length means empty; outgoing POST always supplies it. No streaming, chunking,
TLS, compression, Expect/continue, general mutable request headers or additional verbs.
No new native runtime or compiler policy. See [design and comparison](../../http-client-design.md#buffered-post-checkpoint--2026-09-25).
