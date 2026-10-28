# Gzip native interoperability

This source consumer writes two caller ranges totaling 79,986 bytes through
`GZipStream`, crossing the encoder's internal block boundary. It awaits Finish,
keeps its MemoryStream destination open and emits the resulting bytes for independent
verification. This tests compression beyond the bounded decoder's per-stream quota.

```sh
python3 docs/experiments/gzip/verify.py \
  --bundle /absolute/path/to/matching-native-bundle \
  --output /tmp/fresh-gzip-interop
python3 scripts/test-runtime-library.py \
  --bundle /absolute/path/to/matching-native-bundle \
  --output /tmp/fresh-gzip-tests --suite gzip \
  --mode native --mode jit
```

The interoperability script targets the source macOS ARM64 AOT host, then decodes
its exact gzip bytes with Python/zlib and .NET 10 GZipStream. Its 32 MiB host heap
is a test budget, not a platform compression quota. The contract suite also supplies
independent zlib fixtures and checks stored/fixed/dynamic blocks, headers, member
concatenation, malformed data, both quotas, finalization, pending I/O, cancellation
and StreamReader composition. The fixture generator lives beside the suite.

See [validation.json](validation.json), the
[contract evidence](../../../runtime/System.Runtime/test/GZipTests-validation.json) and
[design](../../gzip-compression.md). Results are local development evidence; Windows
and packaging qualification are not inferred. No HTTP content negotiation, automatic
decompression or response-compression middleware is implemented by this experiment.

Local qualification (2026-10-12): six groups pass in both AOT and JIT. The selected
interpreter test `--filter "pending fragmented"` passes; the full suite exceeds its
600-second allowance. `--mode interpreter` is available for continued qualification,
but full interpreter coverage is not claimed. See the retained contract evidence.


2026-10-19 rerun: the unchanged Interop.rvn consumer currently fails native
emission with NEOMETA003 (an Int32 return where TaskOutcome<Int32> is expected).
No new external-output interoperability result is claimed. The separate updated
gzip contract suite passes seven groups in AOT/JIT and two selected interpreter
cases after the internal memory-view migration. See
[retained evidence](../../validation/memory-views-20261019/gzip-integration-validation.json).
