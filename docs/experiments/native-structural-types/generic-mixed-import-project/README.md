# Imported closed-generic mixed union

Development qualification, 2026-10-15, macOS ARM64.

The separate library declares `Response<T>` with a stored Number property of T.
Produce returns `int | long | bool | Response<int>`; Echo uses reordered alternatives
in its parameter and result. The consumer references only the native DLL and
matches the imported Response<int>, reading the constructor's stored value.
Its reference arm adds one, so Response<int>(42) must produce 43 rather than the
scalar arm's 42. After seven input checks, both interpreter and AOT return 42.

```sh
python3 docs/experiments/native-structural-types/verify-raven-scalar-patterns.py \
  --mixed --imported --generic --compiler /absolute/path/to/rvnc.dll \
  --output target/generic-mixed-import-new-run
```

Run from neoCLR with the retained v3 bundle, experimental compiler, native debug
tools and a new output directory. The runner verifies a native union containing a
constructed generic alternative and seven calls to each imported function. The
only native import allocates the explicit Response<int>; exact arena use is
16 bytes, with no scalar boxes or union carriers.

[Evidence](../raven-generic-mixed-import-results.json) fingerprints both artifacts,
compiler/runtime tools and the retained bundle. Compiler emission and Runtime
Contract configuration are unchanged. This qualifies an open generic class
declaration used as a closed reference alternative; it does not establish bare T
alternatives, all open generic methods/constraints, nominal-value payloads, boxed
views, managed JIT/GC, full Task execution or throughput. Bundle libraries were
not rebuilt.
