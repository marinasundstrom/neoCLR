# Imported reference-valued generic record union

Development qualification, 2026-10-15, macOS ARM64.

The library returns a union of Int32, Int64, Boolean, Response<int>, Response<long>,
Response<Cell> and HttpError. Response<T> is a value record with Number: T and
Wide: long; Cell and HttpError are nominal classes. The consumer imports only the
native library DLL. Ten inputs flow through Produce, Echo with reordered union
signatures, and typed patterns.

The Response<Cell> arm checks both fields, copies the record, then changes
copy.Number.Number from 41 to 46. Reading value.Number.Number must observe 46:
the record copy preserves the embedded object's identity. The two numeric generic
records retain their distinct closed identities and scalar field checks. The
program returns 42 only after all ten inputs pass.

```sh
python3 docs/experiments/native-structural-types/verify-raven-scalar-patterns.py \
  --mixed --imported --record --record-reference --generic --reference-generic \
  --compiler /absolute/path/to/rvnc.dll \
  --output target/reference-generic-record-import-new-run
```

Use a new output directory, the retained v3 bundle, and a working macOS SDK.
The runner checks the seven-alternative native signature, ten imported calls to
each library function, scalar tests/extractions, interpreter exit 42 and AOT
result 42. The native import allowlist contains only the object allocator.
Exactly 32 arena bytes cover the explicit HttpError and Cell instances; neither
the generic value records nor structural unions add boxes or carriers.

[Evidence](../raven-reference-generic-record-import-results.json) fingerprints the
sources, freshly compiled library/consumer, tools and retained bundle. No compiler,
Runtime Contract or native layout change was needed. This is reference identity
and allocation qualification, not managed collector stress: the arena host does
not collect or move objects. Open generic execution, broader substitutions,
JIT/GC, boxing, throughput and full Task qualification remain open.


## Native collection run

Add `--native-gc` to the command above to enable collection. This compiles with
native root publication and runs the same ten-case entry 32 times in each of eager
and paced collector modes, with a 2048-byte heap and native bounds/undefined-behavior
sanitizers. Every invocation must return 42, leave no root frames and permit full
heap reclamation. Exactly 64 explicit allocations per mode must be reclaimed.
The fresh run recorded 1792 eager collections and 64 paced collections. The
[GC evidence](../raven-reference-generic-gc-results.json) includes the explicit
runtime-hook import allowlist, both host results and collector source hashes.

This adds nonmoving AOT collection qualification for this fresh Raven consumer.
It does not establish JIT lifetime, async retention, moving collection or all
structural combinations. The retained class-library bundle was not rebuilt.
