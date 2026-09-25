# HTTP cancellation and text helpers

Development release slice 4, 2026-09-25. Requires matching compiler bridge, reference
assembly, generated System library and native runtime. Compare the
[client contract and .NET research](../../http-client-design.md#http-token-forwarding-and-getstring--2026-09-25).

```sh
python3 docs/experiments/http-cancellation/verify.py \
  --toolchain-root /path/to/matching/neoclr-bundle \
  --runner target/release/examples/measure_async
```

`--case headers` or `--case body` selects a single network case. The script compiles
the guest once in a temporary directory. The peer waits until three HTTP requests
have arrived, then sends a byte on a separate control connection. The guest requests
cancellation of two exchanges using one source. The peer observes closure of those
connections before completing the third, uncancelled exchange on the same HttpClient.
No sleep chooses the cancellation instant. The body case sends incomplete response
bodies before the control signal; this does not assert the exact decoder instruction
at which cancellation arrives. It verifies cleanup with incomplete peer data.

The guest checks pre-cancelled Send/Get/GetString, string and Uri helpers, BaseUri,
nested token forwarding, custom-handler acknowledgement, cancellation propagation
through text conversion, and a completed response surviving later cancellation
before its text continuation runs. Strict UTF-8 rejects invalid bytes as Protocol;
GetString currently rejects a custom non-200 status as Unsupported. Callback stages
isolate transport ownership; the normal client example retains Raven async/await.

Both network cases passed on Darwin arm64 with a 256-object heap budget. Header
cancellation: 893 allocations, 16 collections, peak 228 live objects, zero live at
teardown. Incomplete-body cancellation: 957 allocations, 16 collections, the same
peak and zero live at teardown. The .NET 10 custom-handler baseline separately
checks cooperative cancellation, UTF-8 text and non-success status rejection.

The migrated [client fixture](../http-client/README.md) uses GetString through async
forwarding handlers. Selected fragmented UTF-8, invalid UTF-8 and independent Python
HTTP-server cases passed, with 9–10 collections and zero live objects. Exact HTTP
signatures and rejection of non-token arguments passed in the bridge signature suite.
The existing BaseUri fixture also passed after handler migration: 27 collections
and zero live objects. Library/API snapshots are refreshed and checked. No website build, cross-platform
matrix or general framing matrix was run for this slice.

The .NET baseline accepts all 2xx statuses; neoCLR remains limited to 200 until the
status slice. This is an explicit POC limit, not a proposed improvement. The test does
not add HTTPS, server cancellation, configurable timeouts or general status support.
