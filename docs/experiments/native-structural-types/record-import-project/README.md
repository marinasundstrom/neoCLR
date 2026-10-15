# Imported nominal value-record union

Development qualification, 2026-10-15, macOS ARM64.

The separate library declares Response as a struct with stored Number: int and
Wide: long properties. Produce returns `int | long | bool | Response`; Echo spells
its parameter/result alternatives in different orders. The consumer references
only the native DLL. Six scalar inputs exercise guarded/fallback pattern arms;
the record input must preserve both 42 and 4294967338L, producing the distinct
record-arm result 43. After seven checks, interpreter and AOT both return 42.

```sh
python3 docs/experiments/native-structural-types/verify-raven-scalar-patterns.py \
  --mixed --imported --record --compiler /absolute/path/to/rvnc.dll \
  --output target/record-import-new-run
```

The runner uses --mixed to select the four-alternative signature, with --record
selecting a nominal value instead of a reference. Run from neoCLR with the retained
v3 bundle, experimental compiler, native debug tools and a new output directory.
Set SDKROOT to a working macOS SDK if needed. It verifies native signatures, seven
calls to each imported function, scalar tests/extractions and both execution paths.
The native object has no undefined imports; arena usage and faults are zero.
Neither the scalar alternatives nor the struct payload allocate boxes or carriers.

[Evidence](../raven-record-import-results.json) fingerprints the library, consumer,
compiler/runtime tools, runner and retained bundle. No compiler emission or
Runtime Contract change was needed; bundle libraries were not rebuilt. This is
allocation qualification, not a throughput result. The private union layout sums
alternative sizes plus a discriminator, so copy/zero costs and overlapping storage
still need performance work. Direct reference alternatives alongside records are now qualified by the
[record/reference consumer](../record-reference-import-project/README.md). Enums,
boxing, managed JIT/GC and full Task qualification remain open.
