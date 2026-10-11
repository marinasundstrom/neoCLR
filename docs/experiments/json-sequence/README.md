# JsonArray sequence contract probe

Development, 2026-10-11. JsonArray now exposes a read-only Sequence<JsonValue> view.
The [20-case JSON DOM suite](../../../runtime/raven/tests/json-sequence-validation.json)
passes native macOS ARM64 and interpreted execution, including four new cases for
indexing, node identity, explicit null, empty snapshots, shared child mutation,
append-after-capture, for iteration and serializer precedence.

`Bounds.rvn` is a separate executable because terminal Faults cannot be caught by the
test framework. It probes negative and past-end indices through both concrete and
Sequence access. [All eight executions pass](validation.json). Windows is not run.

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/json-sequence/Bounds.rvnproj \
  --bundle /path/to/matching/bundle --output target/json-sequence-bounds
python3 docs/experiments/json-sequence/verify.py \
  --bundle /path/to/matching/bundle --build target/json-sequence-bounds \
  --runtime target/release/neoclr --output target/json-sequence-faults.json
python3 scripts/test-runtime-library.py --suite json-dom \
  --bundle /path/to/matching/bundle --output target/json-sequence-tests
```

The local development libraries were rebuilt with the native-enabled net10.0 Raven
compiler copied from a checkout at 8b917fbcb4182f639deb830f32c84f810e727898. That checkout
also contained unrelated async working changes; the evidence records compiler and
bundle hashes, not a claim of clean-commit compiler reproducibility. No compiler
source change or new Runtime Contract setting is required for this library API.
The CLI documentation facade uses ordinary interface/getter metadata and a
post-compilation Item method rename, matching the existing JsonObject technique.
Native library metadata directly represents the methods and property.

See the [design tradeoffs](../../json-dom-design.md#read-only-array-sequence-view-development-2026-10-11)
and [converter assessment](../../json-converter-design.md).

The [documentation audit](documentation-validation.json) records the current renderer
binaries and both Item pages plus GetIterator. The aggregate snapshot check and
native snapshot integrity tests pass. A full website build was not needed or run.
