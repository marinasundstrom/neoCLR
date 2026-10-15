# Imported scalar-union qualification

Development experiment, 2026-10-15, on macOS ARM64.

The separate [library](../scalar-import-library/Main.rvn) produces all three
alternatives of `int | long | bool`, then echoes the union through a parameter
and result whose source alternatives are reordered. The consumer references only
its emitted native DLL, with no project or source reference. Six imported calls
exercise negative and positive Int32, wide and small Int64, and both Boolean
values before typed guarded/fallback matching. Both execution paths return 42.

Run from the neoCLR checkout with the experimental Raven compiler and retained
`target/structural-integrated-libraries-v3` bundle available:

```sh
python3 docs/experiments/native-structural-types/verify-raven-scalar-patterns.py \
  --imported --compiler /absolute/path/to/rvnc.dll \
  --output target/scalar-import-new-run
```

The output directory must be new. Set `SDKROOT` to a working macOS SDK if needed.
The runner builds both artifacts, verifies six calls to each imported function,
native union signatures and exact scalar tests/extractions, executes the
interpreter, compiles closed-world AOT and links a context-v4 host. The native
object has no undefined imports; execution reports no fault and zero arena use.
[Saved evidence](../raven-scalar-import-results.json) fingerprints the compiler,
runtime, library, consumer and retained bundle. Libraries in that bundle were not
rebuilt. This is allocation evidence, not a throughput measurement.

No compiler or Runtime Contract change was required. The native tag/payload
representation is private; this does not establish a stable external C union ABI.
Named guards preserve the source-local consumer's documented parser workaround.
Mixed reference/value alternatives, nominal value alternatives, JIT/collector
stress and full Task applications remain unqualified.
