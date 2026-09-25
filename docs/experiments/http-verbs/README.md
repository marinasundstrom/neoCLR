# Common HTTP verb checkpoint

Development after Preview 9. Use a matching toolchain bundle:

```sh
python3 docs/experiments/http-verbs/verify.py \
  --toolchain-root /path/to/bundle \
  --runner target/release/examples/measure_async
```

`Sample.rvn` shows PUT with propagation and an explicit successful-status policy.
Client.rvn asserts all twelve new helper overloads and six factories, BaseUri resolution,
pre-cancellation, token forwarding and retained 404 responses through a fake handler.
An independent Python peer verifies PUT UTF-8, PATCH binary content and bodyless DELETE,
computed lengths, content types, 204 completion and client close before peer EOF.
A separate neoCLR server receives three requests from .NET HttpClient, echoes the
method/body and returns 204 for DELETE. Raw DELETE-with-body and unsupported HEAD are
rejected without invoking the handler. This fixture does not implement patch semantics.

Bounds and policies remain: 1,024 content bytes, 2,048 header bytes, fixed deadlines,
connection close, no TLS/chunking/retries/redirects, bodyless GET/DELETE. HEAD needs
request-aware response framing. HttpContext and stream-backed content remain planned.
Run network checks serially. See [design and .NET comparison](../../http-client-design.md#common-verb-checkpoint--2026-09-25).

Validation on 2026-09-25: all contracts and independent exchanges pass. Client:
1,496 allocations, 34 collections, zero live objects. Server: 739 allocations,
19 collections, zero live objects. Signature admission, API snapshot and bootstrap
hash checks pass; all 18 new signatures have XML entries. Website build skipped by
user direction; its example comes from the compiled sample. No broad platform rerun.

HEAD follow-up (2026-09-25): all four client overloads and both request factories
are checked. A raw peer advertises 999,999 representation bytes and keeps the
connection open: HEAD completes with empty content. .NET HttpClient receives a
neoCLR HEAD response with UTF-8 representation length five and no body. The rejected
unknown-method fixture now uses OPTIONS. Client: 1,958 allocations, 44 collections;
server: 914 allocations, 23 collections; both zero live objects. Focused bridge
signatures and API/bootstrap snapshot checks pass; website build remains skipped.
