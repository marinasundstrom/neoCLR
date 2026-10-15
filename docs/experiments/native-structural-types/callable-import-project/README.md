# Imported callable union consumer

Development qualification, 2026-10-15, macOS ARM64. A separately built library
returns `(() -> int) | int`; the callable binds a Cell instance. The consumer imports
only its native DLL, passes both alternatives through Echo, then uses typed patterns
and invokes the narrowed callable. Exit 42 means both alternatives returned their
expected values.

```sh
python3 docs/experiments/native-structural-types/verify-raven-callable-unions.py \
  --compiler /absolute/path/to/rvnc.dll \
  --bundle target/structural-integration-libraries-v4 --jit \
  --output target/callable-union-new-run
```

Use a new output directory and a working Xcode SDK. The runner copies the fixtures,
compiles against the selected bundle, checks interpreter exit 42, and runs AOT 32
times per eager/paced collector mode with native sanitizers. `--jit` adds the same
checks in a synchronous managed JIT host, without sanitizer instrumentation.
Each mode allocates/reclaims exactly 64 objects: one Cell and one callable descriptor
per entry. Union injection/copy adds no carrier allocation. Roots and heap are empty
after every entry, before JIT code release.

[Fresh-bundle AOT/JIT evidence](../raven-callable-union-fresh-jit-results.json)
records runtime bd1540362, compiler 0e319052b and the freshly built v4 libraries,
with source/tool/artifact hashes and commands. The earlier
[retained-v3 AOT evidence](../raven-callable-union-results.json) remains historical.
Escaped callbacks and async retention remain separate qualification steps.
