# Imported mixed union calls

Development qualification, 2026-10-15, macOS ARM64.

The [library](../mixed-import-library/Main.rvn) publicly declares Response and
returns `int | long | bool | Response`. Its Echo parameter and return spell the
same alternatives in different orders. The consumer references only the emitted
native DLL, imports Response for its typed pattern and reads the Code property.
Seven inputs cover negative/positive Int32, wide/small Int64, both Boolean values
and a reference. Both the interpreter and native AOT return 42.

```sh
python3 docs/experiments/native-structural-types/verify-raven-scalar-patterns.py \
  --mixed --imported --compiler /absolute/path/to/rvnc.dll \
  --output target/mixed-import-new-run
```

Run from the neoCLR checkout with a new output directory, the experimental Raven
compiler, native debug tools and the retained v3 class-library bundle. Set SDKROOT
to a working macOS SDK if needed. The runner builds the library before the
consumer, verifies seven calls to each imported function and the native mixed
signature, executes both backends and checks native imports and arena usage.
Only `neoclr_allocate_object_v1` is imported. The explicit Response uses 16 bytes;
union carriers and scalar boxes add no allocation.

[Evidence](../raven-mixed-import-results.json) records both emitted artifacts,
compiler/runtime hashes and retained bundle labels. No compiler or Runtime
Contract change was needed; the bundle libraries were not rebuilt. This qualifies
one artifact-only library boundary, not general payload layouts, managed JIT/GC,
whole-union boxing, full Task applications or throughput.
