# HTTP content types — 2026-10-19

HttpContent is abstract, with HttpMemoryContent, HttpStreamContent and
HttpStringContent as concrete creation APIs. The author explicitly rejected
compatibility aliases and directed allocation-conscious design. Old base byte
constructors, FromText/FromStream and public Bytes/TryGetBytes/ReadText are removed.
See the [public contracts](../../../api-docs/http-body-streams.md) and
[design comparison](../../design/http-content-streaming.md).

Memory content retains its selected window without a byte copy or Sequence wrapper.
Wire and JSON consumers borrow that window. Generic text reading uses StreamReader's
incremental decoder with 256-byte read staging; a small stream adapter enforces the
declared body length and content lifetime. No whole-body byte accumulator remains.
Text fragments and concatenation still allocate, and text creation still encodes
owned bytes eagerly. This is not an allocation-free or HTTP-throughput claim.

## Qualification

All **93 focused test executions** (31 tests in interpreter/JIT/AOT) and **ten
loopback checks** pass. [Retained evidence](evidence/summary.json) separates the
successful executions from earlier failures. Both API snapshot checks pass.

The matching five-library cohort is `target/http-content-types-libraries-v8`, built
with Raven compiler `dcf7ac41f` and the neoCLR constructor-admission fix `987956c36`.
The refreshed bundle is `target/http-content-types-bundle-v4/bundle`. The retained
reports record exact compiler/backend/runtime and library fingerprints.

Focused cases cover selected-window aliases and GC, bounded copies and backpressure,
ownership, cancellation, early EOF and invalid reads. Text checks cover split UTF-8,
malformed input, exact lengths without over-read, preserved I/O errors and quota;
buffered text above the one-shot codec limit remains supported. JSON checks include
selected memory windows with malformed sentinels outside the window. The independent
loopback consumer checks fixed-length, chunked and close-delimited responses,
truncation and malformed framing. Native and legacy API snapshots are refreshed.

Reproduction (use the matching Xcode clang and SDK on this host):

```sh
python3 scripts/test-runtime-library.py --bundle target/http-content-types-bundle-v4/bundle \
  --output target/http-content-contracts --runtime target/release/neoclr \
  --aot target/debug/neoclr-codegen \
  --jit-host target/http-content-types-bundle-v4/bundle/bin/neoclr-jit-host.dylib \
  --suite http-content-copy --suite http-text --suite json-streams \
  --mode native --mode interpreter --mode jit
python3 docs/experiments/http-content-copy/validate.py \
  --bundle target/http-content-types-bundle-v4/bundle \
  --build target/http-content-types-network-v2 --output target/http-content-network.json
```

Initial failures are retained rather than labeled as successful runs: native base
argument admission, a missing memory copy member, fixture compilation, and a stale
manual-runner test label. The final JSON window test replaces an inapplicable
Sequence-construction test. The original six successful mode runs are reused when
only the JSON fixture changed; no unaffected suite rerun is implied.

The maintained API samples migrate to the new types. Dated experiment snapshots
retain their original source/bundle evidence. Broad release-fixture migration and
response construction directly from HttpContent are subsequent work. Unknown-length
output framing, custom producers, HttpJsonContent specialization and the coordinated
Memory-based general stream interface remain open.
