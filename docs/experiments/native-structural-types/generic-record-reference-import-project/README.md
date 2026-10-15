# Imported closed-generic record/reference union

Development qualification, 2026-10-15, macOS ARM64.

The library returns `int | long | bool | Response<int> | Response<long> | HttpError`.
Response<T> is a struct with Number: T and Wide: long. HttpError is a class.
The consumer imports only the native DLL and sends nine inputs through Produce,
Echo and typed patterns. Echo reorders alternatives in its signatures.

Both closed records have two native field lanes, but their nominal constructed
identities must remain distinct. Response<int> carries 42 and 4294967338L;
Response<long> carries 4294967341L and 42L. Their guarded pattern arms return 43
and 45 respectively, while mismatched payloads return errors. HttpError returns
44. Six scalar inputs retain exact Int32/Int64/Boolean tests and guards. After
all nine checks the program returns 42.

```sh
python3 docs/experiments/native-structural-types/verify-raven-scalar-patterns.py \
  --mixed --imported --record --record-reference --generic \
  --compiler /absolute/path/to/rvnc.dll \
  --output target/generic-record-reference-import-new-run
```

Use the retained v3 bundle, a new output directory and a working macOS SDK.
The runner checks a six-alternative native signature, nine imported calls to each
library function, interpreter/AOT agreement, and an exact native import allowlist.
Arena use is 16 bytes for the explicit HttpError object only; the record payloads
and union storage introduce no boxes or carriers.

[Evidence](../raven-generic-record-reference-import-results.json) fingerprints
sources, freshly emitted library/consumer artifacts, compiler/runtime tools and
the retained bundle. No compiler emission or Runtime Contract change was needed.
This qualifies these closed generic value instantiations, not all open generic
execution or reference-valued T substitutions. The bundle was not rebuilt.
Managed JIT/GC, compact layouts, throughput, boxing and full Task application
qualification remain open.
