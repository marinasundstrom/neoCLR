# UTF-8 builder and joins (development)

This author-selected side quest provides ordinary Raven `System.Text.StringBuilder`
and `String.Join(separator, values)` before returning to the native HTTP driver.
It is not a completed HTTP benchmark or a stable native ABI. See the
[design comparison and limits](../../design/string-building.md).

`Build.rvn` checks fluent chaining, UTF-8 byte counts, combining fragments, cached
snapshots, append after materialization, Clear/reuse, exact and zero quotas, Unicode
separators, empty elements, immutable joins and empty/singleton arrays. The three
fault consumers check Append overflow, atomic line quota preflight and an invalid
constructor limit. Interpreter and native fault diagnostics match byte-for-byte.

`verify.py` compiles the same Raven source to native neoCLR metadata/CIL, executes
that artifact with the interpreter, AOT-compiles it for ARM64, and executes with
UBSan/bounds and the 64 KiB native GC heap. Unsanitized artifacts are checked separately:
the measured standalone binaries depend only on `/usr/lib/libSystem.B.dylib`, not
.NET or a shared neoCLR runtime. macOS ARM64 and the matching compiler/library bundle
are required for this workbench. Limits remain experimental.

```sh
SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk \
python3 docs/experiments/string-building/verify.py \
  --compiler /path/to/Raven.Compiler/rvnc.dll \
  --runtime target/release/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --bundle /path/to/rebuilt-native-class-library \
  --output target/string-building --benchmark

# After all builds and other tests finish, repeat only timing on validated binaries.
python3 docs/experiments/string-building/benchmark.py \
  target/string-building/validation.json target/string-building/benchmark.json
```

The bundle is the flat output of `scripts/build-native-class-library.py`, rebuilt
with these Raven sources and private service declarations. Validation used Raven
`codex/source-object-metadata-resolution` at
`2acfd40ecc88f5ae45ec4178e2f310c12cee8113`, neoCLR main plus this slice, and an Apple M1.
[validation.json](validation.json) records input/artifact hashes, commands, results,
private binding inventory and fault stacks. It includes the earlier measured pass
while other validation was still active; use the separate
[benchmark.json](benchmark.json) for the uncontended rerun after tests completed.
Its validation checksum links the timing to those exact artifacts.

## Measurement

Each program constructs 16 strings from 1024 fragments of 16 UTF-8 bytes and verifies
each result's byte count. Concat builds incrementally; StringBuilder appends then
materializes once; Join first populates an array and joins with an empty separator.
All include their construction/container costs. Five runs alternate order after a
warmup; measurements include process startup, loading/verification, execution and GC.
They are not isolated method timings or matched HTTP/platform comparisons.

| Whole-process median | Interpreter | Native ARM64 |
| --- | ---: | ---: |
| Repeated String.Concat | 1242 ms | 21.0 ms |
| StringBuilder | 2793 ms | 60.1 ms |
| Array plus String.Join | 1494 ms | 58.9 ms |

The current builder and join workloads are **slower than Concat** here. This is
correctness/API progress, not evidence for replacing HTTP header concatenation.
The new service reduces repeated text copies during materialization, but managed
calls, array operations and native root/GC scanning still cost work. Their relative
contributions have not been profiled. No throughput, allocation-reduction or .NET
performance claim follows from these process timings. Small header sets, long
fragments, intermittent snapshots and sustained HTTP requests need separate evidence.

## Focused checks

- `cargo test --test string_construction`: six interpreter construction/indexing checks.
- AOT `console`, `inspection`, `linking`, and `native_gc`: 96 passing tests including
  separator byte preservation, malformed join inputs, sealed-call null parity and
  unsealed virtual-dispatch rejection. C kernels run with and without native GC.
- Legacy String/StringBuilder implementation regeneration and six opaque bridge
  admission checks pass. Native Runtime/Data/Networking/Web builds from Raven source.
- API reference includes both APIs; the reference bridge and website remain development
  artifacts. Public signatures do not imply general native framework coverage.

`Public.rvn` also compiles against the ordinary public CLI reference without
bootstrap RuntimeServices access. `verify_cli.py` generates that reference, compiles
and imports the consumer, builds the checked-in legacy library projection and runs
it in the interpreter. Its [validation](cli-validation.json) passes. The bridge's
signature checks additionally reject changed Join element types and changed fluent
return types. The API snapshot check and website build pass (1825 rendered pages).
