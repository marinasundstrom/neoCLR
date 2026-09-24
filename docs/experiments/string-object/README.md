# String through Object

The development sample converts text to Object, compares exact contents, checks
matching hashes and unchanged display, and uses text keys with explicit Object
callbacks in HashMap under GC pressure. Strings are reference types, not boxed values;
the current runtime uses temporary wrappers when adapting intrinsic text to Object.

```sh
cargo build --example measure_async
python3 docs/experiments/string-object/verify.py \
  --toolchain-root /path/to/matching-development-bundle \
  --runner target/debug/examples/measure_async
dotnet run --project docs/experiments/string-object/dotnet/Baseline.csproj
```

Validation on 2026-09-24 reclaimed all 421 allocations across nine collections
(peak 64, zero live). The verifier requires multiple collections and zero live objects. The .NET 10
baseline checks content equality, matching hashes and display, not identical hash
numbers. String ReferenceEquals and identity/base hash calls remain unsupported:
wrapping copied intrinsic text does not establish stable string allocation identity.
Cast/type-test wrapper allocations now participate in pre-allocation GC; retained
wrappers survive small-heap pressure. No interning, Unicode normalization, culture-sensitive comparison or nullable string
storage is introduced. See the [Object review](../../object-model-review.md).
