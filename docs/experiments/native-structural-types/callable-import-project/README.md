# Imported callable union consumer

Development qualification, 2026-10-15, macOS ARM64. A separately built library
returns `(() -> int) | int`; the callable binds a Cell instance. The consumer imports
only its native DLL, passes both alternatives through Echo, then uses typed patterns
and invokes the narrowed callable. Exit 42 means both alternatives returned their
expected values.

```sh
python3 docs/experiments/native-structural-types/verify-raven-callable-unions.py \
  --compiler /absolute/path/to/rvnc.dll \
  --output target/callable-union-new-run
```

Use a new output directory and a working Xcode SDK. The runner compiles fresh
library/consumer artifacts against the retained v3 bundle, checks interpreter exit
42, and runs AOT 32 times per eager/paced collector mode with native sanitizers.
Each mode allocates/reclaims exactly 64 objects: one Cell and one callable descriptor
per entry. Union injection/copy adds no carrier allocation. Roots and heap are empty
after every entry. [Evidence](../raven-callable-union-results.json) retains commands
and source/tool/artifact hashes. JIT lifetime and a freshly rebuilt bundle remain
separate qualification steps.
