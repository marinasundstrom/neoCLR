# Imported record/reference union

Development qualification, 2026-10-15, macOS ARM64.

The separate library returns `int | long | bool | Response | HttpError`.
Response is a struct with Int32 and Int64 fields; HttpError is a class with an
Int32 code. Echo reorders alternatives in its parameter and result signatures.
The consumer imports only the native DLL, then checks eight inputs using typed
patterns, guards and payload properties. Record and reference arms must return
distinct results (43 and 44); scalar arms retain their existing exact-type checks.
The final successful result is 42.

```sh
python3 docs/experiments/native-structural-types/verify-raven-scalar-patterns.py \
  --mixed --imported --record --record-reference \
  --compiler /absolute/path/to/rvnc.dll \
  --output target/record-reference-import-new-run
```

Use a new output directory and the retained v3 class-library bundle. Set SDKROOT
to a working macOS SDK if needed. The runner verifies a five-alternative native
signature, eight calls to each imported function, two tests/extractions per scalar,
interpreter exit 42 and native result 42. The only native import is the object
allocator; exactly 16 arena bytes are allowed for the explicitly constructed
HttpError. The struct/scalar alternatives introduce no boxes or union carriers.

[Evidence](../raven-record-reference-import-results.json) fingerprints sources,
artifacts, tools and retained bundle. This is a fresh library and consumer build,
not a fresh bundle build or throughput measurement. Runtime Contract configuration
and compiler emission are unchanged. Compact layouts, enums, boxing, managed
JIT/GC and full Task application qualification remain open.
